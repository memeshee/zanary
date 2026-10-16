//! `zanary-probe flow` — execute a payment flow on the harness and record what happened.
//!
//! The rig's product is not "a wallet that can pay", it is "a flow whose privacy claims can be
//! checked": this command runs a flow definition end to end — N invoices, each paid by the payer
//! wallet to a receiver the *merchant* issued, carrying a ZIP-302 memo — records every
//! observation into the shared `zanary_observation::FlowRun` model, evaluates the property
//! library over that record, and writes a report. One command, real transactions, no mocks.
//!
//! The merchant is a separate wallet: different seed, different database, its own sync and its
//! own trial decryption. That is what makes "the memo is readable at the recipient" a
//! third-party observation instead of the payer's self-report, and it is the same separation the
//! verifier needs later.
//!
//! The insecure control run is the same command with a different definition
//! (`"receiver_strategy": "reused"`, `"memo": false`): identical evidence shape, so
//! "profiler vs control" is one definition apart, not one codebase apart.

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, path::{Path, PathBuf}};

use zanary_observation::{
    FlowRun, Funding, MerchantEvent, Payment, PaymentKind, Pool, PoolTotals, Receipt,
};
use zcash_client_backend::{
    data_api::{
        locking::LockFilter, wallet::ConfirmationsPolicy, wallet::TargetHeight, Account as _,
        CoinbaseFilter, InputSource, WalletRead, WalletWrite,
    },
    decrypt_transaction,
    proto::service::TxFilter,
    wallet::OvkPolicy,
};
use zcash_client_sqlite::AccountUuid;
use zcash_keys::{
    address::Address,
    keys::{UnifiedAddressRequest, UnifiedFullViewingKey},
};
use zip32::DiversifierIndex;
use zcash_primitives::transaction::Transaction;
use zcash_protocol::{consensus::BranchId, memo::MemoBytes};

use crate::{
    core::{self, BalanceSnapshot, Probe, WalletProfile},
    node::NodeRpc,
    ops,
};

// ---------------------------------------------------------------------------
// the flow definition
// ---------------------------------------------------------------------------

/// One invoice the merchant issues and the payer settles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceSpec {
    pub invoice_id: String,
    /// Amount in zatoshis. Integers only: money never goes through a float.
    pub amount_zat: u64,
    /// Memo template. `{invoice_id}` is substituted; absent means "no memo template".
    #[serde(default)]
    pub memo_template: Option<String>,
}

fn default_receiver_strategy() -> String {
    "fresh".to_string()
}

/// The account a dedicated receipt key is derived at when the definition does not name one.
/// Account 1, because the merchant's own account 0 is exactly the key a receipt must not publish.
fn default_receipt_account() -> u32 {
    1
}

fn is_default_receipt_account(value: &u32) -> bool {
    *value == 1
}

fn default_true() -> bool {
    true
}

/// The flow definition — hashed into `flow_commit`, so a claim is about this exact revision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowDefinition {
    pub flow_id: String,
    pub invoices: Vec<InvoiceSpec>,
    /// `fresh` issues a new diversified receiver per invoice (the private flow);
    /// `reused` pays every invoice to one receiver (the control).
    #[serde(default = "default_receiver_strategy")]
    pub receiver_strategy: String,
    /// Whether payments bind a memo. The control sets this false.
    #[serde(default = "default_true")]
    pub memo: bool,
    /// Privacy policy, enforced by our own code before anything is built and recorded
    /// verbatim on every payment.
    #[serde(default)]
    pub policy: Option<String>,
    /// How much the payer's wallet should be funded with before the run, in zatoshis
    /// (the runner funds and shields when the payer cannot cover the invoices).
    #[serde(default)]
    pub funding_zat: u64,
    /// ZIP-32 account of the payer wallet (the dev seed). Omitted means account 0.
    ///
    /// Two definitions with different `payer_account` values are two *different payers* for
    /// the same merchant — the setup the payer-linkability property needs to try to cluster
    /// them.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub payer_account: u32,
    /// ZIP-32 account of the merchant wallet (the merchant seed). Omitted means account 0.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub merchant_account: u32,
    /// ZIP-32 account whose key signs receipts — a *dedicated* key derived from the merchant seed,
    /// never the wallet's own spend authority. Signing with the wallet's key would put a linkable
    /// identity on every receipt; with a dedicated key, a receipt names a receipt key, and the
    /// wallet's attestation is what binds the two — optionally. Defaults to account 1 and is
    /// skipped when it holds that value, so existing definitions' `flow_commit`s are unchanged.
    #[serde(
        default = "default_receipt_account",
        skip_serializing_if = "is_default_receipt_account"
    )]
    pub receipt_account: u32,
    /// Settlement jitter: the inclusive range of blocks the runner mines between payments.
    ///
    /// Omitted means the fixed three blocks the earlier runs used. A wider range spreads
    /// settlement across unpredictable heights, so a payment's distance from its funding stops
    /// being a fixed number — which is exactly what `timing-correlation` measures. Skipped when
    /// absent, so adding this field did not change any existing definition's `flow_commit`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jitter_blocks: Option<[u32; 2]>,
    /// Seed for the jitter, so a jittered run reproduces exactly. Omitted means a random seed per
    /// run; the heights that came out are recorded either way, so the run stays auditable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jitter_seed: Option<u64>,
    /// How invoices are settled: `per_invoice` (the default, one transaction each) or `batch` (one
    /// transaction carrying every invoice, which is a stated disclosure — see
    /// `docs/THREAT-MODEL.md`: a batch links its invoices to each other by construction).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement: Option<String>,
    /// Seed for the batch's output order, so a batched run reproduces exactly. Omitted means a
    /// random seed; either way the order the runner drew is recorded on every payment
    /// (`observed.batch_order`), because an unrecorded random value makes a run unauditable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_seed: Option<u64>,
}

/// The settlement jitter: how many blocks to mine between payments.
///
/// Deterministic given a seed, random otherwise — a run must be reproducible for a claim about it
/// to be checkable, but the default should not be a number someone could predict.
struct Jitter {
    rng: rand::rngs::StdRng,
    min: u32,
    max: u32,
    fixed: Option<u32>,
    drawn: Vec<u32>,
}

impl Jitter {
    fn from_definition(definition: &FlowDefinition) -> Self {
        use rand::SeedableRng;
        let seed = definition
            .jitter_seed
            .unwrap_or_else(|| rand::random::<u64>());
        let (min, max, fixed) = match definition.jitter_blocks {
            Some([min, max]) if min <= max && min >= 1 => (min, max, None),
            Some([min, max]) => {
                eprintln!(
                    "  jitter: ignoring jitter_blocks [{min}, {max}] (want 1 <= min <= max); \
                     falling back to the fixed three blocks"
                );
                (3, 3, Some(3))
            }
            None => (3, 3, Some(3)),
        };
        Jitter {
            rng: rand::rngs::StdRng::seed_from_u64(seed),
            min,
            max,
            fixed,
            drawn: Vec::new(),
        }
    }

    /// Blocks to mine before the next payment.
    fn next(&mut self) -> u64 {
        let blocks = match self.fixed {
            Some(fixed) => fixed,
            None => {
                use rand::Rng;
                self.rng.gen_range(self.min..=self.max)
            }
        };
        self.drawn.push(blocks);
        blocks as u64
    }

    fn describe(&self) -> String {
        match self.fixed {
            Some(fixed) => format!("fixed {fixed} block(s) between payments"),
            None => format!(
                "jittered {}-{} blocks between payments, drawn: {:?}",
                self.min, self.max, self.drawn
            ),
        }
    }
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

impl FlowDefinition {
    /// Canonical bytes of the definition: the hash of these is the run's `flow_commit`.
    fn canonical(&self) -> Result<String> {
        serde_json::to_string(self).context("serializing the flow definition")
    }
}

// ---------------------------------------------------------------------------
// entry point
// ---------------------------------------------------------------------------

/// What one flow run produced. The CLI wants an exit code; the CI gate
/// ([`crate::gate`]) wants the numbers and the paths, so the two share one implementation
/// instead of one shelling out to the other.
#[derive(Debug, Clone)]
pub struct FlowOutcome {
    pub flow_id: String,
    pub flow_commit: String,
    pub out_dir: PathBuf,
    pub pass: usize,
    pub fail: usize,
    pub skip: usize,
    /// The properties that FAILED, in the order the library reported them.
    pub failed_properties: Vec<String>,
    pub report_hash: String,
    pub report_path: PathBuf,
    pub findings_path: PathBuf,
    pub run_path: PathBuf,
    pub verified_at: Option<String>,
    pub verified_at_height: u32,
}

/// `zanary-probe flow` — run one definition and record it.
///
/// A failing property is a *result*, not a crash: the run, its findings and its report are all
/// written, and only then does the exit code say the flow found something.
pub async fn run(raw: &[String]) -> Result<()> {
    let outcome = run_recording(raw).await?;
    println!(
        "\n  next: zanary-report verify {} --node http://127.0.0.1:18232",
        outcome.report_path.display()
    );
    if outcome.fail > 0 {
        bail!(
            "{} propert{} FAILED on this flow — see the report above",
            outcome.fail,
            if outcome.fail == 1 { "y" } else { "ies" }
        );
    }
    Ok(())
}

/// The body of `flow`: runs the definition and returns the outcome instead of printing a verdict
/// and exiting.
pub async fn run_recording(raw: &[String]) -> Result<FlowOutcome> {
    let (definition_path, out_dir, ttl_days) = parse_args(raw)?;
    let text = std::fs::read_to_string(&definition_path)
        .with_context(|| format!("reading the flow definition {}", definition_path.display()))?;
    let definition: FlowDefinition = serde_json::from_str(&text)
        .with_context(|| format!("parsing the flow definition {}", definition_path.display()))?;
    if definition.invoices.is_empty() {
        bail!("the flow definition has no invoices");
    }
    match definition.receiver_strategy.as_str() {
        "fresh" | "reused" => {}
        other => bail!(
            "unknown receiver_strategy `{other}` — expected `fresh` (a new diversified receiver \
             per invoice) or `reused` (the insecure control)"
        ),
    }
    if definition.receipt_account == definition.merchant_account {
        bail!(
            "`receipt_account` and `merchant_account` are both {}: a receipt signed by the \
             merchant's own spend authority publishes the wallet's identity on every receipt, \
             which is the thing the dedicated receipt key exists to avoid",
            definition.merchant_account
        );
    }
    let settlement = definition.settlement.as_deref().unwrap_or("per_invoice");
    match settlement {
        "per_invoice" => {}
        "batch" => {
            // A batch puts every invoice in one transaction, and the merchant tells its outputs
            // apart by amount: `merchant_read` looks for the output whose value equals the invoice's.
            // Two invoices with the same amount in one batch would be indistinguishable, so this is
            // refused here rather than mis-recorded later.
            let mut seen: Vec<u64> = Vec::new();
            for invoice in &definition.invoices {
                if seen.contains(&invoice.amount_zat) {
                    bail!(
                        "settlement `batch` needs a distinct amount per invoice (invoice {} \
                         repeats {} zatoshis): the merchant identifies its outputs by amount, so \
                         duplicates cannot be told apart in one transaction",
                        invoice.invoice_id,
                        invoice.amount_zat
                    );
                }
                seen.push(invoice.amount_zat);
            }
        }
        other => bail!(
            "unknown settlement `{other}` — expected `per_invoice` (one transaction per invoice) \
             or `batch` (one transaction carrying every invoice, at the cost of linking them)"
        ),
    }

    let flow_commit = zanary_report::sha256_hex(definition.canonical()?.as_bytes());
    let policy = match &definition.policy {
        Some(policy) => ops::PrivacyPolicy::parse(policy)?,
        // Default: the strictest policy that still permits a shielded payment.
        None => ops::PrivacyPolicy::FullPrivacy,
    };

    println!(
        "zanary-probe {} flow — ZANARY (ZECATHON, Core & Tooling)",
        env!("CARGO_PKG_VERSION")
    );
    println!("  flow              : {}", definition.flow_id);
    println!("  flow commit       : {flow_commit}");
    // Settlement timing belongs in the header: it is a property of the definition, and printing it
    // before the wallet work means a definition can be checked in seconds rather than after a sync.
    let mut jitter = Jitter::from_definition(&definition);
    // Two different questions, two lines: *how* the invoices are settled (one transaction each, or
    // one for the whole batch) and on what *timing* (fixed spacing, or a jittered draw).
    println!(
        "  settlement        : {}",
        if settlement == "batch" {
            "batch — ONE transaction carries every invoice (a stated disclosure: it links them)".to_string()
        } else {
            "per_invoice — one transaction per invoice".to_string()
        }
    );
    println!("  settlement timing : {}", jitter.describe());
    println!("  chain             : regtest (throwaway harness chain)");
    println!(
        "  invoices          : {} ({} total)",
        definition.invoices.len(),
        core::format_zats_signed(
            definition
                .invoices
                .iter()
                .map(|i| i.amount_zat)
                .sum::<u64>() as i128
        )
    );
    println!(
        "  receiver strategy : {}",
        if definition.receiver_strategy == "fresh" {
            "fresh diversified receiver per invoice"
        } else {
            "ONE receiver reused for every invoice (control)"
        }
    );
    println!(
        "  memo              : {}",
        if definition.memo {
            "one ZIP-302 memo per payment"
        } else {
            "none (control)"
        }
    );
    println!("  policy            : {}", policy.name());

    let node = NodeRpc::from_env()?;
    let mut started_at_height = node.block_count().await? as u32;
    if started_at_height == 0 {
        // A chain at height 0 has no tree state for a wallet to sync from: the sync asks for the
        // state below its scan range and lightwalletd refuses a height that does not exist yet
        // ("Block height out of range"), so the wallet — and therefore the flow — cannot start.
        // Mining block 1 is part of running a flow on a brand-new chain, and doing it here is what
        // makes `harness/up.sh` plus one command sufficient on a fresh checkout (docs/CI.md).
        node.generate(1).await?;
        started_at_height = node.block_count().await? as u32;
        println!("  chain             : mined block 1 (an empty chain has nothing to sync from)");
    }
    // Every wallet this run opens syncs against the indexer, so the indexer has to hold every block
    // the node does — including block 1 just mined, and including whatever the node mined while the
    // indexer was restarting. Waiting for that here is cheaper than debugging a wallet that "cannot
    // see" a payment the chain already has.
    let indexer_tip = core::wait_for_indexer_height(&core::endpoint(), started_at_height).await?;
    println!("  indexer tip       : {indexer_tip} (node height {started_at_height})");
    println!("  start height      : {started_at_height}");

    // The payer: the probe's own wallet. The merchant: its own seed and database.
    let mut payer = core::bootstrap_with(WalletProfile::dev(definition.payer_account)).await?;
    let mut merchant =
        core::bootstrap_with(WalletProfile::merchant(definition.merchant_account)).await?;
    println!("  payer account     : {:?}", payer.account);
    println!("  merchant account  : {:?}", merchant.account);
    // The receipt key: derived from the merchant seed at its own account, used for nothing but
    // signing. A receipt that named the wallet's own spend authority would be a linkable identity
    // for the merchant's money; this key names neither.
    let receipt_key =
        WalletProfile::merchant(definition.receipt_account).usk(&merchant.network)?;
    println!(
        "  receipt account   : {} (a dedicated receipt key — the wallet attests it, and the \
         attestation is optional to publish)",
        definition.receipt_account
    );

    let mut run = FlowRun {
        flow_id: definition.flow_id.clone(),
        flow_commit: flow_commit.clone(),
        chain: "regtest".to_string(),
        started_at_height,
        ended_at_height: started_at_height,
        funding: Vec::new(),
        payments: Vec::new(),
        merchant_events: Vec::new(),
        receipts: Vec::new(),
        pool_totals: Vec::new(),
        // The rig runs the merchant side itself, and says so.
        merchant_is_simulated: true,
    };

    // Pool totals at the start, so pool crossings are checkable after the fact.
    let before_pools = node.pool_totals(started_at_height).await?;
    run.pool_totals.push(PoolTotals {
        height: started_at_height,
        transparent_zat: before_pools.transparent_zat,
        sapling_zat: before_pools.sapling_zat,
        orchard_zat: before_pools.orchard_zat,
    });

    // Funding: make sure the payer's Orchard pool covers the invoices. The top-up goes through
    // the same `shield` path the CLI documents, and it is what makes a flow runnable on a chain
    // that was just created — see `top_up_payer` for the two funding paths and their order.
    if let Some(missing) = funding_shortfall(&payer, &definition)? {
        let target = funding_target(&definition, missing);
        println!(
            "  funding           : spendable Orchard short by {} — topping up to {}",
            core::format_zats_signed(missing as i128),
            core::format_zats_signed(target as i128)
        );
        payer = top_up_payer(payer, &definition, target).await?;
        if let Some(missing) = funding_shortfall(&payer, &definition)? {
            bail!(
                "after funding and shielding the payer is still short {} — a freshly shielded \
                 note needs confirmations before it is spendable; re-run the flow",
                core::format_zats_signed(missing as i128)
            );
        }
    } else {
        println!(
            "  funding           : already covered ({})",
            core::format_zats_signed(spendable_orchard(&payer)? as i128)
        );
    }

    // Funding as the rig saw it, recorded *after* the top-up. The order is the point: on this
    // harness the node mines to the flow's own transparent receiver, so the coinbase the shield
    // consumed *is* the funding — a record taken before the top-up would say "nothing funded this
    // flow" on a fresh chain, and the properties that compare funding heights against payment
    // heights would then have nothing to compare.
    run.funding = record_funding(&payer)?;
    println!(
        "  funding recorded  : {} transparent output(s)",
        run.funding.len()
    );

    // How the invoices are settled: one transaction each, or one transaction carrying all of
    // them. A different shape of evidence, not a different rig — same wallets, same proposal
    // code, same property library; only the settlement step differs.
    match settlement {
        "batch" => {
            settle_batch(
                &definition,
                policy,
                &mut payer,
                &mut merchant,
                &receipt_key,
                &mut jitter,
                &mut run,
            )
                .await?
        }
        _ => {
            settle_per_invoice(
                &definition,
                policy,
                &mut payer,
                &mut merchant,
                &receipt_key,
                &mut jitter,
                &mut run,
            )
                .await?
        }
    }

    // Mine the last payment fully into the window the merchant's policy reads at, then take
    // the closing pool totals.
    ops::mine_and_resync(&mut payer, ops::CONFIRM_BLOCKS).await?;
    let ended_at_height = node.block_count().await? as u32;
    let after_pools = node.pool_totals(ended_at_height).await?;
    run.ended_at_height = ended_at_height;
    run.pool_totals.push(PoolTotals {
        height: ended_at_height,
        transparent_zat: after_pools.transparent_zat,
        sapling_zat: after_pools.sapling_zat,
        orchard_zat: after_pools.orchard_zat,
    });

    // --- record: run.json, findings.json, report.json -------------------------------
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("creating the run directory {}", out_dir.display()))?;
    let run_path = out_dir.join("run.json");
    std::fs::write(&run_path, serde_json::to_string_pretty(&run)?)
        .with_context(|| format!("writing {}", run_path.display()))?;

    let findings = zanary_properties::evaluate(&run);
    let findings_path = out_dir.join("findings.json");
    std::fs::write(&findings_path, serde_json::to_string_pretty(&findings)?)
        .with_context(|| format!("writing {}", findings_path.display()))?;

    let verified_at = utc_now_rfc3339()?;
    let report = zanary_report::build_with_claim(
        &run,
        findings.clone(),
        Some(verified_at.clone()),
        // A claim without a clock cannot expire, and `--ttl-days` is how a CI job says how long
        // it is willing to stand behind this run (docs/CLAIMS.md).
        ttl_days.map(|days| days * 86_400),
    )?;
    let report_path = out_dir.join("report.json");
    std::fs::write(&report_path, zanary_report::canonical_json(&report)?)
        .with_context(|| format!("writing {}", report_path.display()))?;

    // --- summary --------------------------------------------------------------------
    println!();
    println!("  --- run summary ---");
    // The jitter as it actually played out: the header prints the range before anything is drawn.
    println!("  settlement timing : {}", jitter.describe());
    // A batch is visible in the summary because it is one txid on several payments — which is the
    // disclosure the definition asked for, stated where a reader will see it.
    let batch_txids: std::collections::BTreeSet<&str> = run
        .payments
        .iter()
        .filter(|p| p.observed.contains_key("batch_size"))
        .map(|p| p.txid.as_str())
        .collect();
    for txid in &batch_txids {
        let size = run
            .payments
            .iter()
            .find(|p| p.txid == *txid)
            .and_then(|p| p.observed.get("batch_size"))
            .map(String::as_str)
            .unwrap_or("?");
        println!("  batch             : {size} payments in one transaction ({txid})");
    }
    for payment in &run.payments {
        println!(
            "  {} · {} · {} · height {}",
            payment.label,
            payment.txid,
            core::format_zats_signed(payment.amount_zat as i128),
            payment
                .height
                .map(|h| h.to_string())
                .unwrap_or_else(|| "unmined".to_string())
        );
    }
    println!();
    let (mut pass, mut fail, mut skip) = (0usize, 0usize, 0usize);
    for finding in &findings {
        match finding.verdict {
            zanary_properties::Verdict::Pass => pass += 1,
            zanary_properties::Verdict::Fail => fail += 1,
            zanary_properties::Verdict::Skip => skip += 1,
        }
        println!(
            "  {} {} {} — {}",
            finding.severity.as_str(),
            match finding.verdict {
                zanary_properties::Verdict::Pass => "pass",
                zanary_properties::Verdict::Fail => "FAIL",
                zanary_properties::Verdict::Skip => "skip",
            },
            finding.property,
            finding.title
        );
    }
    println!();
    println!("  {pass} pass · {fail} fail · {skip} skip");
    println!("  report            : {}", report_path.display());
    println!("  report hash       : {}", report.report_hash);
    println!("  run               : {}", run_path.display());
    println!("  findings          : {}", findings_path.display());

    Ok(FlowOutcome {
        flow_id: run.flow_id.clone(),
        flow_commit: flow_commit.clone(),
        out_dir: out_dir.clone(),
        pass,
        fail,
        skip,
        failed_properties: findings
            .iter()
            .filter(|finding| matches!(finding.verdict, zanary_properties::Verdict::Fail))
            .map(|finding| finding.property.clone())
            .collect(),
        report_hash: report.report_hash.clone(),
        report_path,
        findings_path,
        run_path,
        verified_at: Some(verified_at),
        verified_at_height: ended_at_height,
    })
}

// ---------------------------------------------------------------------------
// settlement: one transaction per invoice, or one for the whole batch
// ---------------------------------------------------------------------------

/// Settles every invoice in its own transaction — the runner's original path.
///
/// Each payment is proposed, built and broadcast on its own, so every invoice gets its own txid,
/// its own height and its own fee; payments are spaced by the jittered (or fixed) block count.
#[allow(clippy::too_many_arguments)]
async fn settle_per_invoice(
    definition: &FlowDefinition,
    policy: ops::PrivacyPolicy,
    payer: &mut Probe,
    merchant: &mut Probe,
    receipt_key: &zcash_keys::keys::UnifiedSpendingKey,
    jitter: &mut Jitter,
    run: &mut FlowRun,
) -> Result<()> {
    // One receiver for the whole run when the definition asks for the control.
    let mut reused_receiver: Option<String> = None;

    for (index, invoice) in definition.invoices.iter().enumerate() {
        println!();
        println!(
            "  --- invoice {} ({}/{}) ---",
            invoice.invoice_id,
            index + 1,
            definition.invoices.len()
        );

        // 1. The merchant issues the invoice: a receiver and a memo.
        let receiver = match definition.receiver_strategy.as_str() {
            "reused" => match &reused_receiver {
                Some(receiver) => receiver.clone(),
                None => {
                    let receiver = issue_receiver(merchant)?;
                    reused_receiver = Some(receiver.clone());
                    receiver
                }
            },
            _ => issue_receiver(merchant)?,
        };
        let memo_text = if definition.memo {
            invoice
                .memo_template
                .clone()
                .unwrap_or_else(|| "zanary: invoice {invoice_id}".to_string())
                .replace("{invoice_id}", &invoice.invoice_id)
        } else {
            String::new()
        };
        println!("  merchant receiver : {receiver}");
        if definition.memo {
            println!("  invoice memo      : {memo_text:?}");
        }

        // 2. The payer pays it, through the same code path `zanary-probe pay` uses.
        let recipient = Address::decode(&payer.network, &receiver).ok_or_else(|| {
            anyhow!("the merchant issued an address this chain cannot decode: {receiver}")
        })?;
        let (required, revealed) = ops::required_policy_for_recipient(&recipient);
        ops::check_policy(policy, required, revealed, "payment")?;
        let memo = MemoBytes::from_bytes(memo_text.as_bytes())
            .map_err(|e| anyhow!("memo rejected: {e}"))?;
        let proposal =
            ops::propose_shielded_payment(payer, &recipient, invoice.amount_zat, &memo)?;
        let fee_zat = proposal.steps().first().balance().fee_required().into_u64();
        let txid = ops::build_sign_broadcast(payer, &proposal, OvkPolicy::Sender).await?;
        println!("  txid              : {txid}");
        // Enough blocks for the payment to be mined (so the merchant's read is of a mined
        // transaction) and for the payer's change note to reach the wallet's confirmation policy
        // before the next invoice. How many is the definition's business: three by default, or a
        // drawn number when it asks for jitter.
        ops::mine_and_resync(payer, jitter.next()).await?;
        // The payment's height is the block that mined it — not the tip after mining. The
        // indexer reports it, and `zanary-report verify` re-checks exactly this number.
        let height = mined_height_of(&mut payer.client, txid).await?;
        let Some(height) = height else {
            bail!(
                "transaction {txid} is not mined yet, so the run cannot record its height \
                 (the harness mines blocks after broadcasting; this is an indexer lag)"
            );
        };

        // 3. The merchant reads it independently: its own wallet, its own decryption.
        let merchant_view = merchant_read(merchant, txid, invoice.amount_zat).await?;
        if let Some(read) = &merchant_view {
            check_read_binds_invoice(definition.memo, &invoice.invoice_id, read)?;
        }
        println!(
            "  merchant verified : {}",
            match &merchant_view {
                Some(read) => format!(
                    "yes — incoming {:?} at orchard output {}",
                    read.memo_text, read.output_index
                ),
                None => "NO — the merchant's wallet could not decrypt this payment".to_string(),
            }
        );

        run.payments.push(Payment {
            label: format!("invoice {}", invoice.invoice_id),
            txid: txid.to_string(),
            height: Some(height),
            amount_zat: invoice.amount_zat,
            recipient_receivers: vec![receiver.clone()],
            // A shielded payment reveals no sender receiver to anyone, including the rig.
            sender_receivers: Vec::new(),
            memo_hex: Some(hex_encode(memo.as_slice())),
            memo_text: if definition.memo {
                Some(memo_text.clone())
            } else {
                None
            },
            invoice_id: Some(invoice.invoice_id.clone()),
            pool_in: Pool::Orchard,
            pool_out: Pool::Orchard,
            kind: PaymentKind::Shielded,
            policy: Some(policy.name().to_string()),
            observed: std::collections::BTreeMap::from([
                ("built_by".to_string(), "zanary-probe flow".to_string()),
                ("fee_zat".to_string(), fee_zat.to_string()),
                (
                    "merchant_verified".to_string(),
                    merchant_view.is_some().to_string(),
                ),
            ]),
        });

        // 4. What the merchant side received, first-hand.
        let mut fields = std::collections::BTreeMap::new();
        fields.insert("invoice_id".to_string(), invoice.invoice_id.clone());
        fields.insert("txid".to_string(), txid.to_string());
        fields.insert("amount_zat".to_string(), invoice.amount_zat.to_string());
        let mut headers = std::collections::BTreeMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("user-agent".to_string(), "zanary-rig/0.0.1".to_string());
        run.merchant_events.push(MerchantEvent {
            height: Some(height),
            method: "POST".to_string(),
            path: format!("/webhook/invoice/{}", invoice.invoice_id),
            headers,
            fields,
            remote_ip: None,
            settled_payment_label: Some(format!("invoice {}", invoice.invoice_id)),
        });

        // 5. A receipt: the smallest artifact that proves this payment settled this invoice.
        //
        // Its `claims` state exactly what the body carries (amount, txid, invoice id, a hash of
        // the memo) and `discloses` is empty, because an artifact that carries more than its
        // claim is the leak `report-leak` looks for. Nothing about the payer is in here.
        let mut receipt = Receipt {
            label: format!("receipt {}", invoice.invoice_id),
            format: "zanary-receipt-v1".to_string(),
            body: serde_json::json!({
                "invoice_id": invoice.invoice_id,
                "amount_zat": invoice.amount_zat,
                "txid": txid.to_string(),
                "memo_sha256": zanary_report::sha256_hex(memo.as_slice()),
            })
            .to_string(),
            claims: vec![
                format!(
                    "invoice {} was paid {} zatoshis in transaction {}",
                    invoice.invoice_id, invoice.amount_zat, txid
                ),
                "the payment binds the invoice id in a ZIP-302 memo, whose hash is in the body"
                    .to_string(),
            ],
            discloses: Vec::new(),
            signer: None,
            signature: None,
            attestation: None,
        };
        // The merchant signs its own receipt, so the receipt proves *who* issued it and a
        // stranger can check that offline (`zanary-report verify-receipt`).
        sign_receipt(merchant, receipt_key, &mut receipt)?;
        run.receipts.push(receipt);

        // The payer must still be able to cover the remaining invoices.
        if let Some(missing) = shortfall(payer, &definition.invoices[index + 1..], "per_invoice")? {
            bail!(
                "the payer wallet cannot cover the remaining invoices ({missing} zatoshis short) \
                 — raise `funding_zat` in the definition or shield more transparent funds"
            );
        }
    }
    Ok(())
}

/// One invoice the merchant has issued inside a batch: the receiver it was issued at, the memo
/// that binds it, and the payment the payer's request carries for it.
struct IssuedInvoice {
    /// Position in the definition's invoice list — the invoice's identity inside the batch.
    index: usize,
    invoice_id: String,
    amount_zat: u64,
    receiver: String,
    memo_text: String,
    memo: MemoBytes,
    recipient: Address,
}

/// Executes the batch settlement: the merchant issues every invoice, the payer settles all of
/// them in **one** transaction whose output order is shuffled, and the merchant then reads every
/// payment back out of that one transaction with its own wallet and its own keys.
///
/// What this buys and what it gives up is written down in `docs/THREAT-MODEL.md` *before* it was
/// built: one txid, one pool movement, one height and one ZIP-317 fee instead of N, and no
/// positional mapping from output order to invoice — at the cost of linking the batch's invoices
/// to each other by construction. So the run records the batch on every payment (`batch_id`,
/// `batch_size`, `batch_order`) and `payer-linkability` fails on a batched run, which is the
/// correct verdict rather than a bug to hide.
#[allow(clippy::too_many_arguments)]
async fn settle_batch(
    definition: &FlowDefinition,
    policy: ops::PrivacyPolicy,
    payer: &mut Probe,
    merchant: &mut Probe,
    receipt_key: &zcash_keys::keys::UnifiedSpendingKey,
    jitter: &mut Jitter,
    run: &mut FlowRun,
) -> Result<()> {
    let count = definition.invoices.len();

    // 1. The merchant issues every invoice first: a fresh receiver and a memo each, exactly as it
    //    would for a single payment. Nothing about the batch is the payer's choice yet.
    let mut issued: Vec<IssuedInvoice> = Vec::with_capacity(count);
    let mut reused_receiver: Option<String> = None;
    for (index, invoice) in definition.invoices.iter().enumerate() {
        let receiver = match definition.receiver_strategy.as_str() {
            "reused" => match &reused_receiver {
                Some(receiver) => receiver.clone(),
                None => {
                    let receiver = issue_receiver(merchant)?;
                    reused_receiver = Some(receiver.clone());
                    receiver
                }
            },
            _ => issue_receiver(merchant)?,
        };
        let memo_text = if definition.memo {
            invoice
                .memo_template
                .clone()
                .unwrap_or_else(|| "zanary: invoice {invoice_id}".to_string())
                .replace("{invoice_id}", &invoice.invoice_id)
        } else {
            String::new()
        };
        let recipient = Address::decode(&payer.network, &receiver).ok_or_else(|| {
            anyhow!("the merchant issued an address this chain cannot decode: {receiver}")
        })?;
        let (required, revealed) = ops::required_policy_for_recipient(&recipient);
        ops::check_policy(policy, required, revealed, "batch payment")?;
        let memo = MemoBytes::from_bytes(memo_text.as_bytes())
            .map_err(|e| anyhow!("memo rejected: {e}"))?;
        issued.push(IssuedInvoice {
            index,
            invoice_id: invoice.invoice_id.clone(),
            amount_zat: invoice.amount_zat,
            receiver,
            memo_text,
            memo,
            recipient,
        });
    }
    println!("  issued            : {count} invoice(s), to be settled in one transaction");
    for item in &issued {
        println!(
            "    {} · {} → {}",
            item.invoice_id,
            core::format_zats_signed(item.amount_zat as i128),
            item.receiver
        );
    }

    // 2. Shuffle the order the payments enter the request, so the transaction's output order
    //    carries no information about the invoice list's order. The draw is recorded on every
    //    payment (`batch_order`) — an unrecorded random value would make the run unauditable.
    let order = batch_order(count, definition.batch_seed);
    println!(
        "  batch order       : {:?} — invoice ids in the order the request carries them: {}",
        order,
        order
            .iter()
            .map(|&i| definition.invoices[i].invoice_id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let payments: Vec<(Address, u64, MemoBytes)> = order
        .iter()
        .map(|&i| {
            (
                issued[i].recipient.clone(),
                issued[i].amount_zat,
                issued[i].memo.clone(),
            )
        })
        .collect();

    // 3. One proposal, one broadcast, one transaction, one fee.
    let proposal = ops::propose_shielded_payments(payer, &payments)?;
    let fee_zat = proposal.steps().first().balance().fee_required().into_u64();
    println!(
        "  batch fee         : {} for all {count} payments (one ZIP-317 fee, not {count})",
        core::format_zats_signed(fee_zat as i128)
    );
    let txid = ops::build_sign_broadcast(payer, &proposal, OvkPolicy::Sender).await?;
    println!("  batch txid        : {txid}");
    ops::mine_and_resync(payer, jitter.next()).await?;
    let height = mined_height_of(&mut payer.client, txid).await?;
    let Some(height) = height else {
        bail!("the batch transaction {txid} is not mined yet, so the run cannot record its height");
    };

    // 4. The merchant reads that one transaction once, with its own wallet and its own keys, and
    //    finds its payments by amount — the one binding the definition guarantees is unique
    //    inside a batch (duplicates are refused before a payment is ever built).
    let reads = merchant_outputs(merchant, txid).await?;
    println!(
        "  merchant read     : {} output(s) of tx {txid} decrypted with the merchant's own keys",
        reads.len()
    );
    if reads.len() != count {
        bail!(
            "the merchant's own wallet decrypted {} output(s) of the batch transaction, but the \
             flow paid {count} invoices — a batch must carry exactly one output per invoice",
            reads.len()
        );
    }

    // The fee belongs to the *transaction*, so it is recorded on exactly one payment: recording it
    // on all N would count it N times and hide real pool movement from the `unshielding` check.
    // `transaction_fee_zat` carries the same number on every payment, for readability.
    for item in &issued {
        let Some(read) = reads.iter().find(|read| read.value_zat == item.amount_zat) else {
            bail!(
                "the merchant's wallet found no output of {} zatoshis in the batch transaction \
                 (it decrypted {} output(s): {})",
                item.amount_zat,
                reads.len(),
                reads
                    .iter()
                    .map(|r| format!("{} @ output {}", r.value_zat, r.output_index))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        };
        if definition.memo && !read.memo_text.contains(&item.invoice_id) {
            bail!(
                "the merchant read a payment of {} zatoshis at orchard output {} but its memo \
                 ({:?}) does not name invoice {} — in a batch the memo is what separates two \
                 payments once the amount has matched",
                read.value_zat,
                read.output_index,
                read.memo_text,
                item.invoice_id
            );
        }
        let position = order
            .iter()
            .position(|&i| i == item.index)
            .expect("every issued invoice is in the order");
        println!(
            "    {} · {} · orchard output {} · memo {:?}",
            item.invoice_id,
            core::format_zats_signed(item.amount_zat as i128),
            read.output_index,
            read.memo_text
        );

        let mut observed = std::collections::BTreeMap::from([
            (
                "built_by".to_string(),
                "zanary-probe flow (batch)".to_string(),
            ),
            ("batch_id".to_string(), txid.to_string()),
            ("batch_size".to_string(), count.to_string()),
            ("batch_order".to_string(), (position + 1).to_string()),
            (
                "batch_output_index".to_string(),
                read.output_index.to_string(),
            ),
            ("merchant_verified".to_string(), "true".to_string()),
            ("transaction_fee_zat".to_string(), fee_zat.to_string()),
        ]);
        if item.index == 0 {
            // The first invoice in the definition's order carries the transaction's fee.
            observed.insert("fee_zat".to_string(), fee_zat.to_string());
            observed.insert(
                "fee_scope".to_string(),
                format!("transaction (one ZIP-317 fee for all {count} payments, recorded once)"),
            );
        } else {
            observed.insert(
                "fee_scope".to_string(),
                "shared with the batch transaction".to_string(),
            );
        }

        run.payments.push(Payment {
            label: format!("invoice {}", item.invoice_id),
            txid: txid.to_string(),
            height: Some(height),
            amount_zat: item.amount_zat,
            recipient_receivers: vec![item.receiver.clone()],
            sender_receivers: Vec::new(),
            memo_hex: Some(hex_encode(item.memo.as_slice())),
            memo_text: definition.memo.then(|| item.memo_text.clone()),
            invoice_id: Some(item.invoice_id.clone()),
            pool_in: Pool::Orchard,
            pool_out: Pool::Orchard,
            kind: PaymentKind::Shielded,
            policy: Some(policy.name().to_string()),
            observed,
        });

        let mut fields = std::collections::BTreeMap::new();
        fields.insert("invoice_id".to_string(), item.invoice_id.clone());
        fields.insert("txid".to_string(), txid.to_string());
        fields.insert("amount_zat".to_string(), item.amount_zat.to_string());
        fields.insert("batch_size".to_string(), count.to_string());
        let mut headers = std::collections::BTreeMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("user-agent".to_string(), "zanary-rig/0.0.1".to_string());
        run.merchant_events.push(MerchantEvent {
            height: Some(height),
            method: "POST".to_string(),
            path: format!("/webhook/invoice/{}", item.invoice_id),
            headers,
            fields,
            remote_ip: None,
            settled_payment_label: Some(format!("invoice {}", item.invoice_id)),
        });

        // One receipt per invoice, as in the per-invoice flow. The txid it names is the batch's,
        // so two receipts from one batch naming one txid *is* the disclosure batching makes —
        // made by the transaction itself, not hidden by the receipt formatting.
        let mut receipt = Receipt {
            label: format!("receipt {}", item.invoice_id),
            format: "zanary-receipt-v1".to_string(),
            body: serde_json::json!({
                "invoice_id": item.invoice_id,
                "amount_zat": item.amount_zat,
                "txid": txid.to_string(),
                "memo_sha256": zanary_report::sha256_hex(item.memo.as_slice()),
            })
            .to_string(),
            claims: vec![
                format!(
                    "invoice {} was paid {} zatoshis in transaction {}",
                    item.invoice_id, item.amount_zat, txid
                ),
                "the payment binds the invoice id in a ZIP-302 memo, whose hash is in the body"
                    .to_string(),
            ],
            discloses: Vec::new(),
            signer: None,
            signature: None,
            attestation: None,
        };
        sign_receipt(merchant, receipt_key, &mut receipt)?;
        run.receipts.push(receipt);
    }

    println!(
        "  batch settled     : {count} invoice(s) in one transaction ({txid} @ height {height})"
    );
    Ok(())
}

/// Refuses a merchant read whose memo does not name the invoice it is supposed to settle.
///
/// The amount is what the merchant reads with; the memo is what names the invoice. A read that
/// matches on amount but names a different invoice (or no invoice at all) is a mis-attribution,
/// and recording it as settled would be worse than failing the run.
fn check_read_binds_invoice(
    memo_expected: bool,
    invoice_id: &str,
    read: &MerchantRead,
) -> Result<()> {
    if !memo_expected || read.memo_text.contains(invoice_id) {
        return Ok(());
    }
    bail!(
        "the merchant read a payment of {} zatoshis but its memo ({:?}) does not name invoice {} \
         — this payment does not settle this invoice",
        read.value_zat,
        read.memo_text,
        invoice_id
    )
}

/// The order a batch's payments are added to the ZIP-321 request: a shuffle of `0..count`.
///
/// Seeded so a batched run reproduces exactly, random when the definition gives no seed — and
/// recorded on every payment either way, because an unrecorded draw makes the run unauditable.
fn batch_order(count: usize, seed: Option<u64>) -> Vec<usize> {
    use rand::{Rng, SeedableRng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed.unwrap_or_else(rand::random::<u64>));
    let mut order: Vec<usize> = (0..count).collect();
    // Fisher-Yates: every permutation is equally likely, which is the point of shuffling at all.
    for i in (1..order.len()).rev() {
        let j = rng.gen_range(0..=i);
        order.swap(i, j);
    }
    order
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Where a definition's run goes when `--out` is not given: `data/runs/<stem of the definition>`.
///
/// The CI gate resolves the same default, so `flow` and `test` never disagree about where a run
/// landed (and a gate that reads the wrong directory would report a stale report as this run).
pub fn default_out_dir(definition: &Path) -> PathBuf {
    let stem = definition
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "run".to_string());
    PathBuf::from("data/runs").join(stem)
}

fn parse_args(raw: &[String]) -> Result<(PathBuf, PathBuf, Option<u64>)> {
    const USAGE: &str = "zanary-probe flow --definition <path> [--out <dir>] [--ttl-days <n>]";
    let mut definition: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut ttl_days: Option<u64> = None;
    let mut iter = raw.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--definition" => {
                definition = Some(PathBuf::from(
                    iter.next()
                        .ok_or_else(|| anyhow!("--definition needs a path\nusage: {USAGE}"))?,
                ));
            }
            "--out" => {
                out = Some(PathBuf::from(
                    iter.next()
                        .ok_or_else(|| anyhow!("--out needs a path\nusage: {USAGE}"))?,
                ));
            }
            "--ttl-days" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--ttl-days needs a number\nusage: {USAGE}"))?;
                ttl_days = Some(
                    value
                        .parse()
                        .with_context(|| format!("`--ttl-days {value}` is not a number"))?,
                );
            }
            other => bail!("unknown argument `{other}`\nusage: {USAGE}"),
        }
    }
    let definition = definition.ok_or_else(|| anyhow!("--definition is required\nusage: {USAGE}"))?;
    let out = out.unwrap_or_else(|| default_out_dir(&definition));
    Ok((definition, out, ttl_days))
}

/// Issues a fresh diversified Orchard-only receiver from the merchant wallet.
///
/// Orchard-only is deliberate: a receiver that also carries a transparent or Sapling
/// receiver widens what the payer's wallet may do with it, and the merchant has no reason
/// to accept a transparent fallback.
fn issue_receiver(merchant: &mut Probe) -> Result<String> {
    let (address, index) = merchant
        .db
        .get_next_available_address(merchant.account, UnifiedAddressRequest::ORCHARD)
        .context("generating the merchant's next diversified receiver")?
        .ok_or_else(|| anyhow!("the merchant wallet could not generate another Orchard receiver"))?;
    println!("  merchant diversifier: {index:?}");
    Ok(address.encode(&merchant.network))
}

/// The payer's spendable Orchard value (what a new payment can actually draw on).
fn spendable_orchard(probe: &Probe) -> Result<u64> {
    let summary = core::wallet_summary(&probe.db)?;
    Ok(summary
        .as_ref()
        .and_then(|summary| summary.account_balances().get(&probe.account))
        .map(|balance| balance.orchard_balance().spendable_value().into_u64())
        .unwrap_or(0))
}

/// The ZIP-317 fee budget for a plan of `invoice_count` invoices under `settlement`.
///
/// Per-invoice settlement pays one one-output Orchard transfer per invoice: 10_000 zatoshis each.
/// A batch pays **one** fee for one transaction — but that transaction carries N payment outputs
/// plus change, and ZIP-317 charges 5_000 zatoshis per logical action, so N outputs cost at least
/// 5_000 × N. Budgeting 10_000 + 5_000 × N covers the bundle whatever the input selection does;
/// the *recorded* fee is still the proposal's own number, never this estimate.
fn fee_budget_zat(invoice_count: usize, settlement: &str) -> u64 {
    if settlement == "batch" {
        10_000 + 5_000 * invoice_count as u64
    } else {
        10_000 * invoice_count as u64
    }
}

/// How short the payer is of covering `invoices`, if at all.
fn shortfall(probe: &Probe, invoices: &[InvoiceSpec], settlement: &str) -> Result<Option<u64>> {
    let needed: u64 = invoices.iter().map(|invoice| invoice.amount_zat).sum();
    let needed = needed.saturating_add(fee_budget_zat(invoices.len(), settlement));
    let spendable = spendable_orchard(probe)?;
    Ok((spendable < needed).then(|| needed - spendable))
}

/// How short the payer is of covering every invoice in the definition, if at all.
fn funding_shortfall(probe: &Probe, definition: &FlowDefinition) -> Result<Option<u64>> {
    shortfall(probe, &definition.invoices, settlement_of(definition))
}

/// The definition's settlement mode: `per_invoice` unless it says otherwise.
fn settlement_of(definition: &FlowDefinition) -> &str {
    definition.settlement.as_deref().unwrap_or("per_invoice")
}

/// How much the runner should move into the payer's Orchard pool when funding is needed:
/// the definition's own target, never less than what the invoices require.
/// Brings the payer's Orchard pool up to what the flow needs, from whatever this chain offers.
///
/// Two funding paths exist, and the order matters:
///
/// 1. **The payer's own coinbase.** The harness conf mines straight to the flow's transparent
///    receiver (`mineraddress` + `minetolocalwallet=0` in `data/zcashd/.zcash/zcash.conf`), so what
///    funds a flow is coinbase the payer already owns — it has to be mature (100 confirmations on
///    regtest) and then shielded. On a chain that was just created there is nothing else, which is
///    why this path comes first: it is the difference between "a flow a stranger can run" and "a
///    flow that needs the rig pre-seeded by hand".
/// 2. **A transfer from the node wallet** (`ops::fund`), for a chain whose conf mines to the node's
///    own wallet. On the harness conf the node wallet holds no coinbase, so this is the fallback,
///    not the default — and it is checked rather than assumed.
///
/// The wallet handle is closed around `shield`/`fund`: they are the commands a human runs, and they
/// open the wallet themselves, so two connections never contend for the same SQLite file.
async fn top_up_payer(open_payer: Probe, definition: &FlowDefinition, target: u64) -> Result<Probe> {
    /// Coinbase needs 100 confirmations on regtest; the margin leaves room for the blocks the
    /// shield and the flow's own payments mine.
    const COINBASE_MATURITY_BLOCKS: u64 = 105;
    let profile = WalletProfile::dev(definition.payer_account);
    let node = NodeRpc::from_env()?;

    // The handle the caller passed in is closed here and reopened below, so the one wallet file is
    // never open twice while `shield`/`fund` do their own opening.
    drop(open_payer);

    node.generate(COINBASE_MATURITY_BLOCKS).await?;
    let mut payer = core::bootstrap_with(profile.clone()).await?;
    let transparent = BalanceSnapshot::capture(&payer.db, payer.account)?.transparent;
    println!(
        "  funding           : mined {COINBASE_MATURITY_BLOCKS} blocks (coinbase to the payer's own \
         receiver); payer transparent balance now {}",
        core::format_zats_signed(transparent)
    );

    if transparent > 0 {
        drop(payer);
        ops::shield(&[]).await?;
        payer = core::bootstrap_with(profile.clone()).await?;
        if funding_shortfall(&payer, &definition)?.is_none() {
            return Ok(payer);
        }
        println!(
            "  funding           : the payer's own coinbase did not cover the flow — falling back to \
             a transfer from the node wallet"
        );
    } else {
        println!(
            "  funding           : this chain mines to the node's wallet (no coinbase to the flow's \
             address), so the payer has none of its own — using the node wallet"
        );
    }

    drop(payer);
    ops::fund(&["--zec".to_string(), format!("{}", target as f64 / 100_000_000.0)]).await?;
    ops::shield(&[]).await?;
    core::bootstrap_with(profile).await
}

fn funding_target(definition: &FlowDefinition, missing: u64) -> u64 {
    let needed: u64 = definition.invoices.iter().map(|i| i.amount_zat).sum::<u64>()
        + fee_budget_zat(definition.invoices.len(), settlement_of(definition));
    definition.funding_zat.max(needed).max(missing)
}

/// Records the funding the flow's own wallet holds, as the flow sees it.
///
/// On this harness the node mines to the flow's address, so the transparent outputs listed here
/// are the rig's own plumbing: the value is the flow's starting position, and the flow holds it
/// in its shielded pool. A transparent receiver in `Funding.receiver` would be a finding about
/// the rig rather than about the flow (and `transparent-fallback` says as much), so the
/// receiver recorded is the flow's own Orchard receiver and the transparent origin stays in the
/// label. What the properties need from this record is how much arrived, when, and where it
/// landed — the amount-fingerprint and timing-correlation checks.
fn record_funding(payer: &Probe) -> Result<Vec<Funding>> {
    let tip = payer
        .db
        .chain_height()
        .context("reading the payer chain height")?
        .ok_or_else(|| anyhow!("the payer wallet has no chain height yet"))?;
    let target = TargetHeight::from(tip + 1);
    let orchard_receiver = payer
        .ufvk
        .find_address(
            DiversifierIndex::from(0u32),
            UnifiedAddressRequest::ORCHARD,
        )
        .map(|(address, _)| address.encode(&payer.network))
        .map_err(|e| anyhow!("deriving the payer's Orchard receiver: {e}"))?;
    let receivers = payer
        .db
        .get_transparent_receivers(payer.account, true, true)
        .context("listing the payer's transparent receivers")?;
    let mut funding = Vec::new();
    for address in receivers.keys() {
        let outputs = payer
            .db
            .get_spendable_transparent_outputs(
                address,
                target,
                ConfirmationsPolicy::default(),
                CoinbaseFilter::AllTransparentOutputs,
                LockFilter::Unfiltered,
            )
            .context("listing the payer's spendable transparent outputs")?;
        for output in outputs {
            funding.push(Funding {
                label: format!(
                    "transparent output {}:{} (rig mining, shielded into the flow's Orchard receiver)",
                    output.outpoint().txid(),
                    output.outpoint().n()
                ),
                txid: output.outpoint().txid().to_string(),
                height: output.mined_height().map(u32::from),
                amount_zat: output.value().into_u64(),
                receiver: orchard_receiver.clone(),
                pool: Pool::Orchard,
            });
        }
    }
    funding.sort_by_key(|f| (f.height, f.txid.clone()));
    Ok(funding)
}

/// What the merchant's own wallet learned from one payment: the value and memo text of the
/// Orchard output addressed to it, and where that output sits in the transaction's Orchard
/// bundle.
///
/// The output index is recorded because in a batch it is the only positional fact there is —
/// and because the merchant must *not* match on it: two invoices in one transaction are told
/// apart by amount (and bound by memo), never by position. Recording the index makes the
/// distinction visible in the run.
pub struct MerchantRead {
    pub value_zat: u64,
    pub memo_text: String,
    pub output_index: usize,
}

/// Every Orchard output of `txid` the merchant's own wallet can decrypt, in bundle order.
///
/// This is the third-party read: the merchant fetches the transaction from the indexer,
/// parses it, and trial-decrypts it with *its own* UFVK — the payer's wallet is not involved.
async fn merchant_outputs(
    merchant: &mut Probe,
    txid: zcash_protocol::TxId,
) -> Result<Vec<MerchantRead>> {
    let raw = merchant
        .client
        .get_transaction(TxFilter {
            block: None,
            index: 0,
            hash: txid.as_ref().to_vec(),
        })
        .await
        .context("GetTransaction from the indexer")?
        .into_inner();
    if raw.data.is_empty() {
        bail!("the indexer returned an empty transaction for {txid}");
    }
    let height = (raw.height != 0).then(|| zcash_protocol::consensus::BlockHeight::from_u32(raw.height as u32));
    let tx = Transaction::read(
        &raw.data[..],
        BranchId::for_height(&merchant.network, height.unwrap_or(zcash_protocol::consensus::BlockHeight::from_u32(1))),
    )
    .context("parsing the transaction the indexer returned")?;

    let mut ufvks: HashMap<AccountUuid, UnifiedFullViewingKey> = HashMap::new();
    for account_id in merchant
        .db
        .get_account_ids()
        .context("listing the merchant's accounts")?
    {
        if let Some(account) = merchant.db.get_account(account_id).context("reading an account")? {
            if let Some(ufvk) = account.ufvk() {
                ufvks.insert(account.id(), ufvk.clone());
            }
        }
    }
    let tip = merchant
        .db
        .chain_height()
        .context("reading the merchant chain height")?;
    let decrypted = decrypt_transaction(&merchant.network, height, tip, &tx, &ufvks);
    let mut reads = Vec::new();
    for output in decrypted.orchard_outputs() {
        if *output.account() != merchant.account {
            continue;
        }
        reads.push(MerchantRead {
            value_zat: output.note().0.value().inner(),
            memo_text: ops::memo_text(output.memo().as_slice()),
            output_index: output.index(),
        });
    }
    // Bundle order: the chain's own order for those outputs, which is what "no positional
    // mapping from output to invoice" has to be measured against.
    reads.sort_by_key(|read| read.output_index);
    Ok(reads)
}

/// The merchant's read of **one** payment: the output of `txid` addressed to it whose value
/// equals `expected_zat`.
///
/// Amount is the key, which is exactly why a batch must have a distinct amount per invoice
/// (refused at definition time, not mis-recorded later) — and why the caller cross-checks the
/// memo binding on top: an amount match alone says "an output of this value exists", the memo
/// says *which invoice* it settles.
async fn merchant_read(
    merchant: &mut Probe,
    txid: zcash_protocol::TxId,
    expected_zat: u64,
) -> Result<Option<MerchantRead>> {
    Ok(merchant_outputs(merchant, txid)
        .await?
        .into_iter()
        .find(|read| read.value_zat == expected_zat))
}

/// The height the chain has for a transaction, as the indexer reports it (`None` while it is
/// still in the mempool).
async fn mined_height_of(
    client: &mut crate::core::IndexerClient,
    txid: zcash_protocol::TxId,
) -> Result<Option<u32>> {
    let raw = client
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

/// Signs a receipt with the merchant wallet's Orchard spend-authorizing key.
///
/// RedPallas, the same key that authorizes the merchant's shielded spends, so the signature is
/// over the receipt's own fields and the public half is the key behind the address the invoice
/// named. `zanary-report verify-receipt --fvk <the merchant's ufvk>` checks that binding offline,
/// with no node, no indexer and no rig running.
fn sign_receipt(
    merchant: &Probe,
    receipt_key: &zcash_keys::keys::UnifiedSpendingKey,
    receipt: &mut Receipt,
) -> Result<()> {
    use orchard::keys::{FullViewingKey, SpendAuthorizingKey};

    // `randomize` with the identity scalar is the un-randomized key pair: the same derivation
    // `zanary_report::receipt::verify_signature` rebuilds from a published key, so the signer a
    // stranger derives is exactly the key named here.
    let unrandomized = |sk: &orchard::keys::SpendingKey| {
        SpendAuthorizingKey::from(sk).randomize(&pasta_curves::pallas::Scalar::from(0u64))
    };
    let rng = rand::rngs::OsRng;

    // The receipt is signed by the *dedicated* receipt key — never the wallet's spend authority,
    // which is what would make every receipt a linkable identity for the merchant's money.
    let payload = zanary_report::receipt::signing_payload(receipt)?;
    let receipt_ak =
        zanary_report::receipt::ak_of_orchard_fvk(&FullViewingKey::from(receipt_key.orchard()));
    receipt.signer = Some(zanary_report::hex_encode(&receipt_ak));
    receipt.signature = Some(zanary_report::receipt::signature_to_hex(
        &unrandomized(receipt_key.orchard()).sign(rng, &payload),
    ));

    // The wallet's side of the binding: a statement, signed with the wallet's own key, that this
    // receipt key is its own. A verifier holding the merchant's UFVK checks it; a receipt
    // published without it is bound to no wallet, which is the privacy property, not a gap.
    let mut attestation = zanary_observation::ReceiptAttestation {
        format: "zanary-receipt-attestation-v1".to_string(),
        receipt_key: zanary_report::hex_encode(&receipt_ak),
        wallet_ak: zanary_report::hex_encode(&zanary_report::receipt::ak_of_orchard_fvk(
            &FullViewingKey::from(merchant.usk.orchard()),
        )),
        signature: String::new(),
    };
    let attestation_payload = zanary_report::receipt::attestation_payload(&attestation)?;
    attestation.signature = zanary_report::receipt::signature_to_hex(
        &unrandomized(merchant.usk.orchard()).sign(rng, &attestation_payload),
    );
    receipt.attestation = Some(attestation);
    Ok(())
}

// ---------------------------------------------------------------------------
// invoice intake: the merchant issues, the payer pays, the merchant verifies
// ---------------------------------------------------------------------------

use zcash_protocol::value::Zatoshis;
// `to_zcash_address` wants a `NetworkType`, and `LocalNetwork` only offers it through this trait.
use zcash_protocol::consensus::Parameters as _;

/// One invoice, as it travels from the merchant to the payer: a fresh diversified Orchard receiver,
/// the amount, the ZIP-302 memo, and the ZIP-321 URI that carries all three in the format a payer's
/// own software reads.
///
/// The receiver is fresh per invoice on purpose — that is what `receiver-reuse` measures — and the
/// URI is the machine-readable form: a payer can be handed this file, or just the URI in it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub invoice_id: String,
    pub amount_zat: u64,
    pub receiver: String,
    pub memo: String,
    pub uri: String,
    pub issued_at_height: u32,
    pub merchant_account: u32,
}

/// `zanary-probe invoice --out <file> [--id <id>] [--amount-zec <zec>] [--account N]`
///
/// The merchant side of an agent payment: issue an invoice the payer can act on with no channel
/// other than the file.
pub async fn issue_invoice(raw: &[String]) -> Result<()> {
    const USAGE: &str = "zanary-probe invoice --out <file> [--id <invoice id>] \
                         [--amount-zec <amount>] [--account N]";
    let mut out: Option<String> = None;
    let mut invoice_id = format!("{}", 7000 + (rand::random::<u16>() % 999));
    let mut amount_zec = "0.25".to_string();
    let mut account = 0u32;
    let mut i = 0;
    while i < raw.len() {
        let value = |i: usize, flag: &str| -> Result<String> {
            raw.get(i + 1)
                .cloned()
                .ok_or_else(|| anyhow!("{flag} needs a value\nUSAGE: {USAGE}"))
        };
        match raw[i].as_str() {
            "--out" => {
                out = Some(value(i, "--out")?);
                i += 2;
            }
            "--id" => {
                invoice_id = value(i, "--id")?;
                i += 2;
            }
            "--amount-zec" => {
                amount_zec = value(i, "--amount-zec")?;
                i += 2;
            }
            "--account" => {
                account = value(i, "--account")?
                    .parse()
                    .context("--account expects a number")?;
                i += 2;
            }
            other => bail!("unexpected argument for invoice: {other}\nUSAGE: {USAGE}"),
        }
    }
    let out = out.ok_or_else(|| anyhow!("invoice needs --out <file>\nUSAGE: {USAGE}"))?;
    let amount_zat = ops::parse_zec(&amount_zec)?;

    let mut merchant = core::bootstrap_with(WalletProfile::merchant(account)).await?;
    let tip = merchant
        .db
        .chain_height()
        .context("reading the merchant chain height")?
        .ok_or_else(|| anyhow!("the merchant wallet has no chain height yet"))?;
    let (address, _) = merchant
        .db
        .get_next_available_address(merchant.account, UnifiedAddressRequest::ORCHARD)
        .context("issuing a fresh Orchard receiver")?
        .ok_or_else(|| anyhow!("the merchant wallet has no Orchard address left to issue"))?;
    let receiver = address.encode(&merchant.network);
    let memo = format!("zanary: invoice {invoice_id}");
    let memo_bytes = MemoBytes::from_bytes(memo.as_bytes())
        .map_err(|e| anyhow!("building the invoice memo: {e}"))?;
    let payment = zip321::Payment::new(
        address.to_zcash_address(merchant.network.network_type()),
        Some(Zatoshis::from_u64(amount_zat).map_err(|e| anyhow!("invalid amount: {e}"))?),
        Some(memo_bytes),
        None,
        None,
        vec![],
    )
    .map_err(|e| anyhow!("building the ZIP-321 payment: {e}"))?;
    let uri = zip321::TransactionRequest::new(vec![payment])
        .map_err(|e| anyhow!("building the ZIP-321 request: {e}"))?
        .to_uri();

    let invoice = Invoice {
        invoice_id: invoice_id.clone(),
        amount_zat,
        receiver: receiver.clone(),
        memo: memo.clone(),
        uri: uri.clone(),
        issued_at_height: u32::from(tip),
        merchant_account: account,
    };
    std::fs::write(&out, serde_json::to_string_pretty(&invoice)?)
        .with_context(|| format!("writing the invoice to {out}"))?;

    println!("  invoice           : {invoice_id}");
    println!("  amount            : {}", core::format_zats_signed(amount_zat as i128));
    println!("  receiver (fresh)  : {receiver}");
    println!("  memo              : {memo}");
    println!("  ZIP-321 URI       : {uri}");
    println!("  issued at height  : {tip}");
    println!("  written to        : {out}");
    println!(
        "\n  next: zanary-probe pay --invoice {out} --policy AllowRevealedAmounts"
    );
    Ok(())
}

/// `zanary-probe verify-invoice --invoice <file> --txid <txid>`
///
/// The merchant side of settlement, standalone: its own wallet syncs, fetches the transaction, and
/// trial-decrypts it with its own UFVK. Nothing here trusts the payer's word — only the chain and
/// the merchant's own keys.
pub async fn verify_invoice(raw: &[String]) -> Result<()> {
    const USAGE: &str = "zanary-probe verify-invoice --invoice <file> --txid <txid>";
    let mut invoice_path: Option<String> = None;
    let mut txid_hex: Option<String> = None;
    let mut i = 0;
    while i < raw.len() {
        let value = |i: usize, flag: &str| -> Result<String> {
            raw.get(i + 1)
                .cloned()
                .ok_or_else(|| anyhow!("{flag} needs a value\nUSAGE: {USAGE}"))
        };
        match raw[i].as_str() {
            "--invoice" => {
                invoice_path = Some(value(i, "--invoice")?);
                i += 2;
            }
            "--txid" => {
                txid_hex = Some(value(i, "--txid")?);
                i += 2;
            }
            other => bail!("unexpected argument for verify-invoice: {other}\nUSAGE: {USAGE}"),
        }
    }
    let invoice_path =
        invoice_path.ok_or_else(|| anyhow!("verify-invoice needs --invoice <file>\nUSAGE: {USAGE}"))?;
    let txid_hex = txid_hex.ok_or_else(|| anyhow!("verify-invoice needs --txid <txid>\nUSAGE: {USAGE}"))?;
    let invoice: Invoice = serde_json::from_str(
        &std::fs::read_to_string(&invoice_path)
            .with_context(|| format!("reading the invoice {invoice_path}"))?,
    )
    .with_context(|| format!("parsing the invoice {invoice_path}"))?;

    let mut bytes: [u8; 32] = hex_decode(&txid_hex)
        .context("decoding the txid")?
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("a txid is 32 bytes"))?;
    // A displayed txid is the reverse of the transaction's internal byte order (as in Bitcoin), so
    // the printed hex has to be reversed before it becomes a `TxId` — otherwise the node is asked
    // for a transaction whose id is this one backwards, and answers "No such mempool or blockchain
    // transaction".
    bytes.reverse();
    let txid = zcash_protocol::TxId::from_bytes(bytes);

    let mut merchant = core::bootstrap_with(WalletProfile::merchant(invoice.merchant_account)).await?;
    println!("  invoice           : {}", invoice.invoice_id);
    println!("  expecting         : {}", core::format_zats_signed(invoice.amount_zat as i128));
    match merchant_read(&mut merchant, txid, invoice.amount_zat).await? {
        Some(read) => {
            let binds = read.memo_text.contains(&invoice.invoice_id);
            println!(
                "  amount read       : {}",
                core::format_zats_signed(read.value_zat as i128)
            );
            println!("  orchard output    : {}", read.output_index);
            println!("  memo read         : {}", read.memo_text);
            println!(
                "  binds invoice id  : {}",
                if binds { "yes" } else { "NO" }
            );
            if !binds {
                bail!(
                    "the merchant read the payment but its memo does not name invoice {} — \
                     this payment does not settle this invoice",
                    invoice.invoice_id
                );
            }
            println!("\n  PAID — the merchant's own wallet read this invoice's id in the payment's memo");
            Ok(())
        }
        None => bail!(
            "the merchant's wallet could not decrypt {txid} — it is not a payment to this merchant"
        ),
    }
}

impl Invoice {
    /// Reads an invoice from disk and checks it against its own ZIP-321 URI.
    ///
    /// The URI is the form a payer's own software would consume, so it is parsed here rather than
    /// trusted: if the file's fields and the URI disagree, the invoice is refused. This is the
    /// intake path — a payer acts on the standard, not on this rig's JSON.
    pub fn from_path(path: &str) -> Result<Self> {
        let invoice: Invoice = serde_json::from_str(
            &std::fs::read_to_string(path).with_context(|| format!("reading the invoice {path}"))?,
        )
        .with_context(|| format!("parsing the invoice {path}"))?;
        let request = zip321::TransactionRequest::from_uri(&invoice.uri)
            .map_err(|e| anyhow!("the invoice's ZIP-321 URI does not parse: {e}"))?;
        let payment = request
            .payments()
            .values()
            .next()
            .ok_or_else(|| anyhow!("the invoice's ZIP-321 URI carries no payment"))?;
        if payment.recipient_address().encode() != invoice.receiver {
            bail!(
                "the invoice's URI pays {} but the file says {} — refusing an invoice that \
                 disagrees with itself",
                payment.recipient_address().encode(),
                invoice.receiver
            );
        }
        if payment.amount().map(|a| a.into_u64()) != Some(invoice.amount_zat) {
            bail!("the invoice's URI and its `amount_zat` disagree");
        }
        let uri_memo = payment.memo().map(|m| ops::memo_text(m.as_slice()));
        if uri_memo.as_deref() != Some(invoice.memo.as_str()) {
            bail!("the invoice's URI and its `memo` disagree");
        }
        Ok(invoice)
    }
}

/// Lowercase hex, for recording raw memo bytes.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Decodes lowercase hex into bytes, for txids that arrive as text.
fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 {
        bail!("hex string has an odd length");
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| anyhow!("invalid hex at byte {}: {e}", i / 2))
        })
        .collect()
}

/// Current UTC time as RFC3339, for the report's claim clock.
fn utc_now_rfc3339() -> Result<String> {
    let output = std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .context("running `date` for the claim clock")?;
    if !output.status.success() {
        bail!("`date -u` failed while building the claim clock");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
