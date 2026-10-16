//! `zanary-probe payer` — the bounded payer: a *subject* the rig tests, not part of the rig.
//!
//! An autonomous payer should be given **authority, not a key**: a budget, a per-payment cap, an
//! allowlist of receivers, an expiry, and a kill switch. This is the reference implementation of that
//! idea — small on purpose, because its job in this project is to be *tested*: the rig runs it, reads
//! the ledger it produces, and the `payer-policy` property checks every decision against the grant.
//!
//! It ships in the rig's own workspace and reuses the rig's wallet code, which is worth saying plainly
//! rather than hiding: the payer and the flow runner share `ops::propose_shielded_payment` and
//! `build_sign_broadcast`, so a receipt the payer issues and the evidence the rig verifies agree by
//! construction. What makes it a subject rather than the rig is that it decides under a *grant*: the
//! grant is the only thing standing between an invoice and a payment.
//!
//! What it does, per invoice it is shown:
//!
//! ```text
//! revoked?            → refuse (kill switch)
//! now > expires_at?   → refuse (authority expired)
//! receiver not listed? → refuse (stranger)
//! amount > cap?        → refuse (per-payment cap)
//! spent + amount > budget? → refuse (budget)
//! otherwise            → pay (shielded, ZIP-302 memo from the invoice) and record the txid
//! ```
//!
//! Every decision — paid or refused — is written to `ledger.jsonl` and into the run's `payer` record,
//! with the rule that produced it. A refusal is a result, not a failure: `--expect-refused N` is how a
//! caller says "this run was supposed to refuse exactly N invoices", and the exit code says whether that
//! happened.

use anyhow::{anyhow, bail, Context, Result};
use std::path::{Path, PathBuf};
use zanary_observation::{Payment, PaymentKind, PayerDecision, PayerGrant, PayerRun, Pool};
use zcash_client_backend::{address::Address, wallet::OvkPolicy};
use zcash_protocol::memo::MemoBytes;

use crate::core::{self, WalletProfile};
use crate::flow::Invoice;
use crate::node::NodeRpc;
use crate::ops;

/// Default lifetime of a grant when the grant file does not say: the same 30 days a claim gets.
const DEFAULT_GRANT_DAYS: i64 = 30;

// ---------------------------------------------------------------------------
// the grant: the authority, and the decision function over it
// ---------------------------------------------------------------------------

/// A grant as written by an operator, before it is recorded for a run.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Grant {
    #[serde(default = "default_budget")]
    pub budget_zat: u64,
    #[serde(default = "default_cap")]
    pub per_payment_cap_zat: u64,
    #[serde(default)]
    pub allowlist: Vec<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    /// Operator's kill switch: a revoked grant is still a valid grant, it just permits nothing.
    #[serde(default)]
    pub revoked: bool,
}

fn default_budget() -> u64 {
    500_000_00 // 0.5 ZEC
}

fn default_cap() -> u64 {
    100_000_00 // 0.1 ZEC per payment
}

/// One decision, with the rule that produced it, phrased so a reader of the ledger knows why.
#[derive(Debug, Clone)]
pub struct Decision {
    pub allowed: bool,
    pub reason: String,
}

/// The whole policy, as one pure function.
///
/// Pure on purpose: this is the part of a bounded payer that must never depend on a wallet, a clock or
/// a network, and the tests below pin every rule. `now` is passed in rather than read, so "the grant
/// expired" is testable without waiting for it.
pub fn decide(
    grant: &PayerGrant,
    spent_zat: u64,
    invoice: &Invoice,
    now: &str,
) -> Decision {
    if grant.revoked {
        return Decision {
            allowed: false,
            reason: "refused: the grant was revoked (kill switch)".to_string(),
        };
    }
    if now >= grant.expires_at.as_str() {
        return Decision {
            allowed: false,
            reason: format!(
                "refused: the grant expired at {} (this invoice was shown at {now})",
                grant.expires_at
            ),
        };
    }
    if !grant.allowlist.is_empty() && !grant.allowlist.contains(&invoice.receiver) {
        return Decision {
            allowed: false,
            reason: format!(
                "refused: {} is not on the allowlist ({} entries)",
                invoice.receiver,
                grant.allowlist.len()
            ),
        };
    }
    if invoice.amount_zat > grant.per_payment_cap_zat {
        return Decision {
            allowed: false,
            reason: format!(
                "refused: invoice {} asks {} zat, above the per-payment cap of {}",
                invoice.invoice_id, invoice.amount_zat, grant.per_payment_cap_zat
            ),
        };
    }
    let after = spent_zat.saturating_add(invoice.amount_zat);
    if after > grant.budget_zat {
        return Decision {
            allowed: false,
            reason: format!(
                "refused: paying {} zat would take this run to {} of a {} zat budget",
                invoice.amount_zat, after, grant.budget_zat
            ),
        };
    }
    Decision {
        allowed: true,
        reason: format!(
            "paid: within the cap ({} zat) and the budget ({} of {} zat spent after this)",
            grant.per_payment_cap_zat, after, grant.budget_zat
        ),
    }
}

// ---------------------------------------------------------------------------
// the command
// ---------------------------------------------------------------------------

/// `zanary-probe payer --grant <grant.json> --invoices <dir> [--out <dir>] [--expect-refused N]
/// [--kill-file <path>] [--account N] [--label <text>] [--ttl-days N]`
pub async fn run(raw: &[String]) -> Result<()> {
    const USAGE: &str = "zanary-probe payer --grant <grant.json> --invoices <dir> [--out <dir>] \
                         [--expect-refused N] [--kill-file <path>] [--account N] [--label <text>] \
                         [--ttl-days N] [--now <rfc3339>]";
    let mut args = ops::Args::parse(raw, USAGE)?;
    let grant_path = PathBuf::from(args.require("--grant")?);
    let invoices_path = PathBuf::from(args.require("--invoices")?);
    let out_dir = PathBuf::from(
        args.value("--out")
            .unwrap_or_else(|| "data/runs/payer".to_string()),
    );
    let expect_refused = args.optional_u32("--expect-refused")?;
    let kill_file = args.value("--kill-file").map(PathBuf::from);
    let account = args.optional_u32("--account")?.unwrap_or(0);
    let label = args
        .value("--label")
        .unwrap_or_else(|| "zanary bounded payer (reference subject)".to_string());
    let ttl_days = args.optional_u32("--ttl-days")?.map(u64::from);
    let now_override = args.value("--now");
    args.finish()?;

    let grant_text = std::fs::read_to_string(&grant_path)
        .with_context(|| format!("reading the grant {}", grant_path.display()))?;
    let grant_file: Grant = serde_json::from_str(&grant_text)
        .with_context(|| format!("parsing the grant {}", grant_path.display()))?;
    let now = match now_override {
        Some(ts) => ts,
        None => utc_now_rfc3339()?,
    };
    let expires_at = match grant_file.expires_at.clone() {
        Some(ts) => ts,
        None => {
            use time::{Duration, OffsetDateTime};
            let started = time::format_description::well_known::Rfc3339;
            let parsed = OffsetDateTime::parse(&now, &started)
                .map_err(|e| anyhow!("--now / clock is not RFC3339: {e}"))?;
            (parsed + Duration::days(DEFAULT_GRANT_DAYS))
                .format(&started)
                .map_err(|e| anyhow!("formatting the default expiry: {e}"))?
        }
    };
    // The kill switch is a file, checked at startup: authority that can be withdrawn by touching a
    // path is the point of a kill switch (and it keeps the grant file itself immutable evidence).
    let revoked = grant_file.revoked
        || kill_file
            .as_ref()
            .map(|path| path.exists())
            .unwrap_or(false);

    let grant = PayerGrant {
        budget_zat: grant_file.budget_zat,
        per_payment_cap_zat: grant_file.per_payment_cap_zat,
        allowlist: grant_file.allowlist.clone(),
        expires_at: expires_at.clone(),
        revoked,
    };

    let invoices = load_invoices(&invoices_path)?;
    if invoices.is_empty() {
        bail!(
            "no invoices in {} — the payer has nothing to decide about (write some with \
             `zanary-probe invoice --out <dir>/<id>.json`)",
            invoices_path.display()
        );
    }

    println!(
        "zanary-probe {} payer — a budget instead of a key",
        env!("CARGO_PKG_VERSION")
    );
    println!("  grant             : {}", grant_path.display());
    println!(
        "  budget / cap      : {} · {} per payment",
        core::format_zats_signed(grant.budget_zat as i128),
        core::format_zats_signed(grant.per_payment_cap_zat as i128)
    );
    println!(
        "  allowlist         : {}",
        if grant.allowlist.is_empty() {
            "(empty — every receiver is allowed)".to_string()
        } else {
            grant.allowlist.join(", ")
        }
    );
    println!("  expires at        : {}", grant.expires_at);
    println!(
        "  kill switch       : {}",
        match (&kill_file, revoked) {
            (_, true) => "TRIPPED — every invoice will be refused".to_string(),
            (Some(path), false) => format!("armed ({} does not exist)", path.display()),
            (None, false) => "not configured".to_string(),
        }
    );
    println!("  invoices          : {} in {}", invoices.len(), invoices_path.display());
    println!();

    // Decide everything first, then pay: the ledger a reviewer reads is complete before any money
    // moves, so a refusal cannot be an artifact of a payment that went wrong earlier.
    let mut decisions: Vec<PayerDecision> = Vec::new();
    let mut spent: u64 = 0;
    let mut to_pay: Vec<(Invoice, String)> = Vec::new();
    for invoice in &invoices {
        let decision = decide(&grant, spent, invoice, &now);
        println!(
            "  {:<8} {} — {}",
            if decision.allowed { "ALLOW" } else { "REFUSE" },
            invoice.invoice_id,
            decision.reason
        );
        if decision.allowed {
            spent = spent.saturating_add(invoice.amount_zat);
            to_pay.push((invoice.clone(), decision.reason.clone()));
        }
        decisions.push(PayerDecision {
            invoice_id: invoice.invoice_id.clone(),
            amount_zat: invoice.amount_zat,
            receiver: invoice.receiver.clone(),
            decision: if decision.allowed { "paid" } else { "refused" }.to_string(),
            reason: decision.reason.clone(),
            txid: None,
            height: None,
        });
    }
    println!();

    // The payer's own wallet: the same dev profile the rig uses, so the chain sees one payer.
    let profile = WalletProfile::dev(account);
    let node = NodeRpc::from_env()?;
    let started_at_height = node.block_count().await? as u32;
    let mut payer = core::bootstrap_with(profile.clone()).await?;
    let funding_shortfall = shortfall(&payer, &to_pay)?;
    if let Some(missing) = funding_shortfall {
        println!(
            "  funding           : the payer is short {} — topping up (coinbase → shield), as the flow \
             runner does",
            core::format_zats_signed(missing as i128)
        );
        drop(payer);
        payer = crate::flow::top_up_for_target(profile, missing).await?;
    }

    let mut payments: Vec<Payment> = Vec::new();
    let mut ledger: Vec<String> = Vec::new();
    for (index, (invoice, allowed_reason)) in to_pay.iter().enumerate() {
        println!(
            "  --- paying invoice {} ({}/{}) ---",
            invoice.invoice_id,
            index + 1,
            to_pay.len()
        );
        let recipient = Address::decode(&payer.network, &invoice.receiver).ok_or_else(|| {
            anyhow!(
                "the invoice's receiver is not a valid address on this chain: {}",
                invoice.receiver
            )
        })?;
        let memo = MemoBytes::from_bytes(invoice.memo.as_bytes())
            .map_err(|e| anyhow!("the invoice's memo was rejected: {e}"))?;
        let proposal = ops::propose_shielded_payment(&mut payer, &recipient, invoice.amount_zat, &memo)?;
        let txid = ops::build_sign_broadcast(&mut payer, &proposal, OvkPolicy::Sender).await?;
        ops::mine_and_resync(&mut payer, ops::CONFIRM_BLOCKS).await?;
        let height = mined_height_of(&mut payer, txid).await?;
        println!(
            "  paid              : {} · {} · height {}",
            txid,
            core::format_zats_signed(invoice.amount_zat as i128),
            height.map(|h| h.to_string()).unwrap_or_else(|| "unmined".into())
        );

        if let Some(entry) = decisions.iter_mut().find(|d| d.invoice_id == invoice.invoice_id) {
            entry.txid = Some(txid.to_string());
            entry.height = height;
        }
        ledger.push(
            serde_json::json!({
                "invoice_id": invoice.invoice_id,
                "amount_zat": invoice.amount_zat,
                "receiver": invoice.receiver,
                "decision": "paid",
                "reason": allowed_reason,
                "txid": txid.to_string(),
                "height": height,
            })
            .to_string(),
        );
        payments.push(Payment {
            label: format!("invoice {}", invoice.invoice_id),
            txid: txid.to_string(),
            height,
            amount_zat: invoice.amount_zat,
            recipient_receivers: vec![invoice.receiver.clone()],
            sender_receivers: Vec::new(),
            memo_hex: Some(hex_encode(invoice.memo.as_bytes())),
            memo_text: Some(invoice.memo.clone()),
            invoice_id: Some(invoice.invoice_id.clone()),
            pool_in: Pool::Orchard,
            pool_out: Pool::Orchard,
            kind: PaymentKind::Shielded,
            policy: Some("FullPrivacy".to_string()),
            observed: std::collections::BTreeMap::from([(
                "payer".to_string(),
                format!("grant {} (cap {}, budget {})", grant_path.display(), grant.per_payment_cap_zat, grant.budget_zat),
            )]),
        });
    }
    for decision in decisions.iter().filter(|d| !d.paid()) {
        println!("  refused           : {} — {}", decision.invoice_id, decision.reason);
        ledger.push(
            serde_json::json!({
                "invoice_id": decision.invoice_id,
                "amount_zat": decision.amount_zat,
                "receiver": decision.receiver,
                "decision": "refused",
                "reason": decision.reason,
            })
            .to_string(),
        );
    }

    // Funding as the rig sees it for this payer: the transparent outputs that backed the payments.
    // Without it the run would claim timing properties it never exercised (they skip), which is the
    // difference between "nothing to check" and "checked and clean".
    let funding = crate::flow::record_funding(&payer)?;
    let ended_at_height = node.block_count().await? as u32;
    let payer_run = PayerRun {
        label,
        grant: grant.clone(),
        decisions: decisions.clone(),
    };
    let run = zanary_observation::FlowRun {
        flow_id: format!("zanary-payer/{}", short_commit(&invoices_path)),
        flow_commit: zanary_report::sha256_hex(
            serde_json::to_string(&decisions)?.as_bytes(),
        ),
        chain: "regtest".to_string(),
        started_at_height,
        ended_at_height,
        funding,
        payments,
        merchant_events: Vec::new(),
        receipts: Vec::new(),
        pool_totals: Vec::new(),
        merchant_is_simulated: true,
        payer: Some(payer_run),
    };

    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("creating the run directory {}", out_dir.display()))?;
    let ledger_path = out_dir.join("ledger.jsonl");
    std::fs::write(
        &ledger_path,
        format!("{}\n", ledger.join("\n")),
    )
    .with_context(|| format!("writing {}", ledger_path.display()))?;
    let artifacts = crate::record::write_run_artifacts(&run, &out_dir, ttl_days)?;
    let report = crate::record::read_report(&out_dir)?;
    let expires_at = zanary_report::expires_at_rfc3339(&report);

    let refused = decisions.iter().filter(|d| !d.paid()).count();
    println!();
    println!("  --- ledger ---");
    for decision in &decisions {
        println!(
            "  {:<8} {} · {} · {}",
            decision.decision,
            decision.invoice_id,
            core::format_zats_signed(decision.amount_zat as i128),
            decision.reason
        );
    }
    println!();
    println!(
        "  {} paid · {} refused · {} spent of {}",
        decisions.len() - refused,
        refused,
        core::format_zats_signed(
            decisions.iter().filter(|d| d.paid()).map(|d| d.amount_zat).sum::<u64>() as i128
        ),
        core::format_zats_signed(grant.budget_zat as i128)
    );
    println!("  ledger            : {}", ledger_path.display());
    println!("  run               : {}", artifacts.run_path.display());
    println!("  findings          : {}", artifacts.findings_path.display());
    println!("  report            : {}", artifacts.report_path.display());
    println!("  report hash       : {}", artifacts.report_hash);
    println!(
        "  claim             : verified at height {} · {} · expires {}",
        artifacts.verified_at_height,
        artifacts.verified_at,
        expires_at.as_deref().unwrap_or("never (no clock)")
    );
    println!();
    println!("  --- properties (the same library the flow runner uses) ---");
    crate::record::print_findings(&out_dir)?;
    println!();
    println!(
        "  properties        : {} pass · {} fail · {} skip",
        artifacts.pass, artifacts.fail, artifacts.skip
    );
    println!();
    println!(
        "  next: zanary-report verify {} --node http://127.0.0.1:18232",
        artifacts.report_path.display()
    );

    if let Some(expected) = expect_refused {
        if refused != expected as usize {
            bail!(
                "the payer refused {} invoice(s), not the {expected} this run expected — a payer that \
                 refuses the wrong set is as broken as one that pays too much",
                refused
            );
        }
        println!("  expectation       : refused exactly {expected} invoice(s) ✓");
    }
    if artifacts.fail > 0 {
        bail!(
            "{} propert{} FAILED on this payer run ({}): see the report above",
            artifacts.fail,
            if artifacts.fail == 1 { "y" } else { "ies" },
            artifacts.failed_properties.join(", ")
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Invoices from a directory (`*.json`) or from a single file, sorted by name so a run is reproducible.
fn load_invoices(path: &Path) -> Result<Vec<Invoice>> {
    let mut files: Vec<PathBuf> = Vec::new();
    if path.is_dir() {
        for entry in std::fs::read_dir(path)
            .with_context(|| format!("reading the invoice directory {}", path.display()))?
        {
            let entry = entry?;
            let candidate = entry.path();
            if candidate.extension().is_some_and(|e| e == "json") {
                files.push(candidate);
            }
        }
    } else {
        files.push(path.to_path_buf());
    }
    files.sort();
    let mut invoices = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file)
            .with_context(|| format!("reading the invoice {}", file.display()))?;
        let invoice: Invoice = serde_json::from_str(&text)
            .with_context(|| format!("parsing the invoice {}", file.display()))?;
        invoices.push(invoice);
    }
    Ok(invoices)
}

/// What the payer needs in its Orchard pool to pay every allowed invoice (plus a fee margin).
fn shortfall(payer: &core::Probe, to_pay: &[(Invoice, String)]) -> Result<Option<u64>> {
    let needed: u64 = to_pay.iter().map(|(invoice, _)| invoice.amount_zat).sum();
    if needed == 0 {
        return Ok(None);
    }
    // The same fee budget the flow runner uses, plus the anchoring margin: one ZIP-317 fee per
    // payment at 2 actions.
    let needed = needed.saturating_add(10_000 * to_pay.len() as u64);
    let available = crate::flow::spendable_orchard(payer)?;
    Ok((available < needed).then(|| needed - available))
}

async fn mined_height_of(
    payer: &mut core::Probe,
    txid: zcash_protocol::TxId,
) -> Result<Option<u32>> {
    use zcash_client_backend::proto::service::TxFilter;
    let raw = payer
        .client
        .get_transaction(TxFilter {
            block: None,
            index: 0,
            hash: txid.as_ref().to_vec(),
        })
        .await
        .context("GetTransaction from the indexer")?
        .into_inner();
    Ok((raw.height != 0).then_some(raw.height as u32))
}

/// A stable, short id for what the payer was asked to pay: the invoices path, hashed.
fn short_commit(path: &Path) -> String {
    let text = path.display().to_string();
    let digest = zanary_report::sha256_hex(text.as_bytes());
    digest.chars().take(8).collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn utc_now_rfc3339() -> Result<String> {
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|e| anyhow!("formatting the current time: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invoice(id: &str, amount_zat: u64, receiver: &str) -> Invoice {
        Invoice {
            invoice_id: id.to_string(),
            amount_zat,
            receiver: receiver.to_string(),
            memo: format!("zanary: invoice {id}"),
            uri: format!("zcash:?address={receiver}&amount=0.1"),
            issued_at_height: 1,
            merchant_account: 0,
        }
    }

    fn grant() -> PayerGrant {
        PayerGrant {
            budget_zat: 1_000,
            per_payment_cap_zat: 400,
            allowlist: vec!["uregtest1merchant".to_string()],
            expires_at: "2026-11-01T00:00:00Z".to_string(),
            revoked: false,
        }
    }

    const NOW: &str = "2026-10-16T00:00:00Z";

    #[test]
    fn allows_a_listed_invoice_inside_cap_and_budget() {
        let decision = decide(&grant(), 0, &invoice("1", 300, "uregtest1merchant"), NOW);
        assert!(decision.allowed, "{}", decision.reason);
    }

    #[test]
    fn refuses_over_the_cap() {
        let decision = decide(&grant(), 0, &invoice("1", 401, "uregtest1merchant"), NOW);
        assert!(!decision.allowed);
        assert!(decision.reason.contains("per-payment cap"), "{}", decision.reason);
    }

    #[test]
    fn refuses_past_the_budget() {
        let decision = decide(&grant(), 900, &invoice("1", 200, "uregtest1merchant"), NOW);
        assert!(!decision.allowed);
        assert!(decision.reason.contains("budget"), "{}", decision.reason);
    }

    #[test]
    fn refuses_a_stranger() {
        let decision = decide(&grant(), 0, &invoice("1", 100, "uregtest1stranger"), NOW);
        assert!(!decision.allowed);
        assert!(decision.reason.contains("allowlist"), "{}", decision.reason);
    }

    #[test]
    fn refuses_after_expiry() {
        let decision = decide(&grant(), 0, &invoice("1", 100, "uregtest1merchant"), "2026-11-01T00:00:01Z");
        assert!(!decision.allowed);
        assert!(decision.reason.contains("expired"), "{}", decision.reason);
    }

    #[test]
    fn refuses_everything_when_revoked() {
        let mut grant = grant();
        grant.revoked = true;
        let decision = decide(&grant, 0, &invoice("1", 1, "uregtest1merchant"), NOW);
        assert!(!decision.allowed);
        assert!(decision.reason.contains("kill switch"), "{}", decision.reason);
    }

    #[test]
    fn an_empty_allowlist_allows_every_receiver() {
        let mut grant = grant();
        grant.allowlist.clear();
        let decision = decide(&grant, 0, &invoice("1", 100, "uregtest1anyone"), NOW);
        assert!(decision.allowed, "{}", decision.reason);
    }
}
