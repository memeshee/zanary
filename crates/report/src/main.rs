//! `zanary-report` — build a report from a run, verify one against the chain, and turn a
//! directory of reports into an expiring claim index.
//!
//! ```text
//! zanary-report verify <report.json> [--ufvk <ufvk>] [--node <url>] [--indexer <url>]
//! zanary-report build --run <run.json> [--findings <findings.json>]
//!                     [--verified-at <rfc3339>] [--ttl-days <n>] [--out <report.json>]
//! zanary-report sample-run [--out <run.json>]
//! zanary-report index --reports <dir> --out <index.html> [--now <rfc3339>]
//! ```

use anyhow::{Context, Result};
use zanary_observation::FlowRun;
use zanary_properties::Finding;
use zanary_report::{
    build_with_claim,
    index::render_index_with_anchors,
    verify::{verify_report, VerifyOptions},
    Report,
};
use std::{fs, path::PathBuf};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

/// The bundled fixture run: a real run recorded against the local harness chain
/// (`crates/report/fixtures/run.json`). `sample-run` emits it so the whole pipeline can be
/// exercised from a clean checkout.
const SAMPLE_RUN: &str = include_str!("../fixtures/run.json");

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("verify") => verify(&args).await,
        Some("verify-anchor") => verify_anchor(&args),
        Some("verify-receipt") => verify_receipt(&args),
        Some("build") => build(&args),
        Some("sample-run") => sample_run(&args),
        Some("index") => index(&args),
        Some("matrix") => matrix(&args),
        Some("help") | Some("--help") | Some("-h") | None => {
            usage();
            Ok(0)
        }
        Some(other) => {
            usage();
            Err(anyhow::anyhow!("unknown subcommand `{other}`"))
        }
    };
    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::exit(2);
        }
    }
}

/// `zanary-report verify-receipt <receipt.json> [--fvk <ufvk>] [--signer <hex>]`
///
/// Offline by construction: it reads the receipt, rebuilds the bytes the merchant signed, and
/// checks the signature. No node, no indexer, no wallet, no rig — so it runs with the engine
/// stopped, which is the point of a receipt.
fn verify_receipt(args: &[String]) -> Result<i32> {
    let path = args
        .get(2)
        .ok_or_else(|| anyhow::anyhow!("verify-receipt needs a receipt.json path"))?;
    let mut fvk: Option<String> = None;
    let mut expected_signer: Option<String> = None;
    let mut network = "regtest";
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--network" => {
                // `zcash_protocol` has no regtest variant in `Network`; regtest keys are decoded
                // with `LocalNetwork`, which `receipt::regtest_params` supplies.
                network = match args.get(i + 1).map(String::as_str) {
                    Some("regtest") => "regtest",
                    Some("test") => "test",
                    Some("main") => "main",
                    other => {
                        return Err(anyhow::anyhow!(
                            "--network expects regtest, test or main (got {other:?})"
                        ))
                    }
                };
                i += 2;
            }
            "--fvk" => {
                fvk = args.get(i + 1).cloned();
                i += 2;
            }
            "--signer" => {
                expected_signer = args.get(i + 1).cloned();
                i += 2;
            }
            other => return Err(anyhow::anyhow!("unexpected argument for verify-receipt: {other}")),
        }
    }

    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading {path}"))?;
    let receipt: zanary_observation::Receipt = serde_json::from_str(&text)
        .with_context(|| format!("parsing {path} as a receipt"))?;

    println!("receipt   : {path}");
    println!("format    : {}", receipt.format);
    for claim in &receipt.claims {
        println!("claim     : {claim}");
    }
    if !receipt.discloses.is_empty() {
        println!("discloses : {}", receipt.discloses.join(", "));
    }

    let signature_check = zanary_report::receipt::verify_signature(&receipt);
    let claim_check = zanary_report::receipt::validate_claim(&receipt);

    match &signature_check {
        Ok(check) => {
            println!("signature : VALID (Orchard RedPallas over the receipt's own fields)");
            println!("signer    : {}", check.signer);
        }
        Err(e) => println!("signature : INVALID — {e}"),
    }
    match &claim_check {
        Ok(notes) => {
            for note in notes {
                println!("claim ok  : {note}");
            }
        }
        Err(e) => println!("claim     : INVALID — {e}"),
    }
    if signature_check.is_err() || claim_check.is_err() {
        println!("\n  FAIL — the receipt does not check out (see the lines above)");
        return Ok(1);
    }
    // Both checks passed above, so this is the verified signer.
    let check = signature_check.expect("the signature check passed");

    if let Some(expected) = expected_signer {
        if !expected.eq_ignore_ascii_case(&check.signer) {
            println!("expected  : {expected}  MISMATCH");
            println!("\n  FAIL — the receipt was not signed by the key you expected");
            return Ok(1);
        }
        println!("expected  : {expected}  matches");
    }

    if let Some(ufvk) = fvk {
        let wallet_ak = match network {
            "main" => zanary_report::receipt::signer_of_ufvk(
                &zcash_protocol::consensus::Network::MainNetwork,
                &ufvk,
            )?,
            "test" => zanary_report::receipt::signer_of_ufvk(
                &zcash_protocol::consensus::Network::TestNetwork,
                &ufvk,
            )?,
            _ => zanary_report::receipt::signer_of_ufvk(
                &zanary_report::receipt::regtest_params(),
                &ufvk,
            )?,
        };
        println!("from --fvk: {wallet_ak}");
        if wallet_ak.eq_ignore_ascii_case(&check.signer) {
            // A receipt signed directly by the wallet's own key binds itself.
            println!("            matches — this receipt is signed by that wallet's own key");
        } else {
            // A dedicated-key receipt binds through the wallet's attestation instead, and a
            // receipt published without one is bound to no wallet at all — which is a failure of
            // the check you asked for, not a failure of the receipt.
            match zanary_report::receipt::verify_attestation(&receipt, &wallet_ak) {
                Ok(()) => println!(
                    "            matches — the receipt key is attested by that wallet (binding verified)"
                ),
                Err(e) => {
                    println!("\n  FAIL — nothing binds this receipt to that wallet: {e}");
                    return Ok(1);
                }
            }
        }
    } else if let Some(att) = receipt.attestation.as_ref() {
        println!(
            "attested  : receipt key {} is attested by wallet key {} — verify the binding with \
             --fvk <that wallet's UFVK>",
            zanary_report::receipt::short(&att.receipt_key),
            zanary_report::receipt::short(&att.wallet_ak)
        );
    } else {
        println!(
            "binding   : none — the receipt names a receipt key and carries no attestation, so it \
             is bound to no wallet. That is the design, not a defect: publishing it reveals which \
             receipt key issued it and nothing about the wallet behind it."
        );
    }

    println!("\n  summary: signature verified offline (no node, no indexer, no rig)");
    Ok(0)
}

/// `zanary-report matrix --runs <dir> --out <file.md>`
///
/// One property × flow table over every run directory under `--runs`. The submission artifact: the
/// private flows and the control side by side, each column pinned to its `flow_commit`.
fn matrix(args: &[String]) -> Result<i32> {
    let runs = arg_value(args, "--runs")
        .ok_or_else(|| anyhow::anyhow!("matrix needs --runs <dir>"))?;
    let out =
        arg_value(args, "--out").ok_or_else(|| anyhow::anyhow!("matrix needs --out <file.md>"))?;
    let columns = zanary_report::matrix::collect(std::path::Path::new(runs))?;
    if columns.is_empty() {
        return Err(anyhow::anyhow!(
            "no run directories with run.json + findings.json under {runs}"
        ));
    }
    let markdown = zanary_report::matrix::render_markdown(&columns);
    fs::write(out, &markdown).with_context(|| format!("writing {out}"))?;

    use zanary_properties::Verdict;
    for column in &columns {
        println!(
            "  {} @ {} — {} pass · {} fail · {} skip",
            column.flow_id,
            &column.commit[..8.min(column.commit.len())],
            column.count(Verdict::Pass),
            column.count(Verdict::Fail),
            column.count(Verdict::Skip),
        );
    }
    let properties = columns
        .iter()
        .map(|c| c.findings.len())
        .max()
        .unwrap_or(0);
    println!(
        "\n  matrix: {out} — {} run(s) × {properties} property(ies)",
        columns.len()
    );
    Ok(0)
}

fn usage() {
    eprintln!(
        "zanary-report — reports a stranger can check, claims that expire\n\
         \n\
         USAGE:\n\
         \x20 zanary-report verify <report.json> [--ufvk <ufvk>] [--node <url>] [--indexer <url>]\n\
         \x20 zanary-report verify-receipt <receipt.json> [--network regtest|test|main]\n\
         \x20                     [--fvk <ufvk>] [--signer <hex>]\n\
         \x20                     offline: checks the receipt's signature over its own fields —\n\
         \x20                     no node, no indexer, no rig. --fvk binds the receipt's key to\n\
         \x20                     that wallet: directly, or through the wallet's attestation when\n\
         \x20                     the receipt was signed by a dedicated receipt key.\n\
         \x20 zanary-report build --run <run.json> [--findings <findings.json>]\n\
         \x20                     [--verified-at <rfc3339>] [--ttl-days <n>] [--out <report.json>]\n\
         \x20 zanary-report sample-run [--out <run.json>]\n\
         \x20 zanary-report index --reports <dir> --out <index.html> [--now <rfc3339>]\n\
         \x20 zanary-report matrix --runs <dir> --out <file.md>\n\
         \x20 zanary-report verify-anchor --report <report.json> [--root <hex>]\n\
         \x20                     --beacon-ufvk <ufvk> [--since <height>] [--chain regtest]\n\
         \x20                     [--node <url>]\n\
         \x20                     anchored evidence: scans the chain for a shielded memo carrying\n\
         \x20                     this evidence root and reports the EARLIEST one — the timestamp\n\
         \x20                     nobody has to be trusted for. Needs a node and the beacon's\n\
         \x20                     published viewing key, never ZANARY. Exits 0 when the anchor\n\
         \x20                     agrees with the report, 1 otherwise.\n\
         \n\
         verify exits 0 when every check passes, 1 on any mismatch.\n\
         Environment: ZANARY_NODE_URL (default http://127.0.0.1:18232),\n\
         \x20 ZANARY_NODE_USER/PASS (default zanary/zanary), ZANARY_LWD_URL (default\n\
         \x20 http://127.0.0.1:9067)."
    );
}

fn arg_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

/// `abcdef12…34567890`: a hash shown with both ends, for comparisons where the difference lives in
/// the tail. A root that differs only in its last digit must not print identically to the one it
/// disagrees with — that would read as "these agree" while the check says FAIL.
fn short_tail(value: &str) -> String {
    if value.len() <= 20 {
        return value.to_string();
    }
    format!("{}…{}", &value[..8], &value[value.len() - 8..])
}

fn parse_ts(s: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(s, &Rfc3339)
        .with_context(|| format!("`{s}` is not an RFC3339 timestamp (e.g. 2026-10-11T20:30:00Z)"))
}

/// `zanary-report verify-anchor --report <report.json> [--root <hex>] --beacon-ufvk <ufvk>`
///                      `[--since <height>] [--chain regtest] [--node <url>]`
///
/// The stranger's check on an anchor: with a node, a chain and the beacon's *published viewing key*
/// — nothing from ZANARY — scan the chain for a shielded memo carrying this evidence root, and
/// report the earliest one. The height of that memo is the only timestamp in this system that
/// nobody has to be trusted for.
///
/// Anchoring is permissionless (anyone can pay the beacon), so the accepted hit is the *earliest*
/// one and its fields are checked against the report: a later copy of the same root is harmless,
/// and a root anchored before the report it claims to be about is refused outright.
fn verify_anchor(args: &[String]) -> Result<i32> {
    let report_path = arg_value(args, "--report");
    let mut expected_root = arg_value(args, "--root").map(str::to_string);
    let beacon_ufvk = arg_value(args, "--beacon-ufvk")
        .ok_or_else(|| anyhow::anyhow!("verify-anchor needs --beacon-ufvk <ufvk>"))?
        .to_string();
    let mut chain = "regtest".to_string();
    let mut since = 1u32;
    if let Some(value) = arg_value(args, "--chain") {
        chain = value.to_string();
    }
    if let Some(value) = arg_value(args, "--since") {
        since = value
            .parse()
            .with_context(|| format!("--since expects a height, got `{value}`"))?;
    }
    let node = match arg_value(args, "--node") {
        Some(url) => zanary_report::node::NodeRpc::new(
            url.strip_prefix("http://").unwrap_or(url).trim_end_matches('/'),
            std::env::var("ZANARY_NODE_USER").unwrap_or_else(|_| "zanary".into()),
            std::env::var("ZANARY_NODE_PASS").unwrap_or_else(|_| "zanary".into()),
        ),
        None => zanary_report::node::NodeRpc::from_env(),
    };

    // The report, when given, is read and re-hashed here: anchoring is about an exact artifact, so
    // the root being searched for is derived from the artifact rather than taken on its word.
    let report: Option<Report> = match report_path {
        Some(path) => Some(
            serde_json::from_str(
                &fs::read_to_string(path).with_context(|| format!("reading {path}"))?,
            )
            .with_context(|| format!("parsing {path} as a report"))?,
        ),
        None => None,
    };
    let mut checks: Vec<(String, &'static str, String)> = Vec::new();
    let mut recomputed_hash: Option<String> = None;
    let mut recomputed_root: Option<String> = None;
    if let Some(report) = &report {
        let hash = zanary_report::hash_report(report)?;
        let root = zanary_report::evidence_root(&report.evidence)?;
        if report.report_hash != hash {
            checks.push((
                "report hash".to_string(),
                "FAIL",
                format!(
                    "the file's `report_hash` is {} but its canonical JSON hashes to {hash} — \
                     refusing to search for the anchor of a report that does not hash to itself",
                    short(&report.report_hash)
                ),
            ));
        } else {
            checks.push((
                "report hash".to_string(),
                "PASS",
                format!("sha256 over canonical JSON recomputes: {}", short(&hash)),
            ));
        }
        if let Some(arg) = &expected_root {
            if arg != &root {
                checks.push((
                    "root argument".to_string(),
                    "FAIL",
                    format!(
                        "--root {} disagrees with the report's evidence root {} — the report's own \
                         root is the one that gets searched",
                        short_tail(arg),
                        short_tail(&root)
                    ),
                ));
            }
        }
        expected_root = Some(root.clone());
        recomputed_hash = Some(hash);
        recomputed_root = Some(root);
    }
    let Some(root) = expected_root.clone() else {
        return Err(anyhow::anyhow!(
            "verify-anchor needs --report <report.json> or --root <evidence root>"
        ));
    };

    println!("zanary-report verify-anchor — evidence root {}", short(&root));
    println!("  chain             : {chain} (scanning from height {since})");
    println!();
    let outcome = zanary_report::anchor::scan(&node, &chain, &beacon_ufvk, since)
        .context("scanning the chain for anchor memos with the beacon's viewing key")?;
    println!(
        "  scan              : {} block(s) {}..={} read · {} transaction(s) trial-decrypted",
        outcome.blocks_read, outcome.since, outcome.tip, outcome.transactions_read
    );
    println!(
        "  anchor memos      : {} found in that range",
        outcome.hits.len()
    );

    let hits = outcome.with_root(&root);
    match hits.first() {
        None => {
            checks.push((
                "anchor".to_string(),
                "FAIL",
                format!(
                    "no anchor memo in blocks {}..={} carries this evidence root ({} anchor \
                     memo(s) were readable, none of them this one)",
                    outcome.since, outcome.tip, outcome.hits.len()
                ),
            ));
        }
        Some(hit) => {
            checks.push((
                "anchor".to_string(),
                "PASS",
                format!(
                    "first anchored at height {} in tx {} (orchard output {})",
                    hit.height, hit.txid, hit.output_index
                ),
            ));
            if hits.len() > 1 {
                checks.push((
                    "anchor copies".to_string(),
                    "PASS",
                    format!(
                        "{} anchors carry this root; the earliest is the timestamp, later copies \
                         (heights {}) prove nothing extra",
                        hits.len(),
                        hits
                            .iter()
                            .skip(1)
                            .map(|h| h.height.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                ));
            }
            if let Some(report) = &report {
                if hit.height < report.verified_at_height {
                    checks.push((
                        "anchor height".to_string(),
                        "FAIL",
                        format!(
                            "the anchor is at height {} but the report claims to have been made at \
                             height {} — a root cannot be anchored before it existed",
                            hit.height, report.verified_at_height
                        ),
                    ));
                } else {
                    checks.push((
                        "anchor height".to_string(),
                        "PASS",
                        format!(
                            "anchored at height {} ≥ the report's height {}",
                            hit.height, report.verified_at_height
                        ),
                    ));
                }
                let hash = recomputed_hash.clone().unwrap_or_default();
                for disagreement in hit.anchor.disagreements(report, &hash, &root) {
                    checks.push(("anchor payload".to_string(), "FAIL", disagreement));
                }
                if hit.anchor.disagreements(report, &hash, &root).is_empty() {
                    checks.push((
                        "anchor payload".to_string(),
                        "PASS",
                        "every field the report can re-derive agrees with the memo".to_string(),
                    ));
                }
            }
            println!("  anchor memo       : {:?}", hit.anchor.memo_text()?);
        }
    }

    println!();
    let (mut pass, mut fail) = (0usize, 0usize);
    for (label, verdict, detail) in &checks {
        match *verdict {
            "FAIL" => fail += 1,
            _ => pass += 1,
        }
        println!("  {label:<17} {verdict:<4} {detail}");
    }
    println!();
    let recheck_root = recomputed_root.clone().unwrap_or_else(|| root.clone());
    println!(
        "  summary: {} pass · {} fail → {}",
        pass,
        fail,
        if fail == 0 {
            "ANCHORED"
        } else {
            "NOT ANCHORED"
        }
    );
    if fail == 0 {
        println!(
            "\n  re-check: zanary-report verify-anchor --report <report.json> --beacon-ufvk <ufvk> \
             --node <node>\n  root     : {recheck_root}"
        );
        Ok(0)
    } else {
        Ok(1)
    }
}

async fn verify(args: &[String]) -> Result<i32> {
    let path = args
        .get(2)
        .ok_or_else(|| anyhow::anyhow!("verify needs a report path"))?;
    let report: Report = serde_json::from_str(
        &fs::read_to_string(path).with_context(|| format!("reading {path}"))?,
    )
    .with_context(|| format!("parsing {path} as a report"))?;

    let mut opts = VerifyOptions::default();
    if let Some(url) = arg_value(args, "--node") {
        opts.node = zanary_report::node::NodeRpc::new(
            url.strip_prefix("http://").unwrap_or(url).trim_end_matches('/'),
            std::env::var("ZANARY_NODE_USER").unwrap_or_else(|_| "zanary".into()),
            std::env::var("ZANARY_NODE_PASS").unwrap_or_else(|_| "zanary".into()),
        );
    }
    if let Some(url) = arg_value(args, "--indexer") {
        opts.indexer_url = url.to_string();
    }
    if let Some(ufvk) = arg_value(args, "--ufvk") {
        opts.extra_ufvk = Some(ufvk.to_string());
    }

    println!(
        "zanary-report verify — {} @ {}",
        report.flow_id,
        short(&report.flow_commit)
    );
    println!(
        "  chain {} · verified at height {} · {} evidence entr{}",
        report.chain,
        report.verified_at_height,
        report.evidence.len(),
        if report.evidence.len() == 1 { "y" } else { "ies" },
    );
    println!();

    let outcome = verify_report(&report, &opts).await?;
    for check in &outcome.checks {
        println!(
            "  {:<46} {:<5} {}",
            check.label,
            check.verdict.as_str(),
            check.detail
        );
    }
    println!();
    println!(
        "  summary: {} pass · {} fail · {} skip · {} warn → {}",
        outcome.passes(),
        outcome.fails(),
        outcome.skips(),
        outcome.warns(),
        if outcome.verified() {
            "VERIFIED"
        } else {
            "NOT VERIFIED"
        },
    );
    Ok(if outcome.verified() { 0 } else { 1 })
}

fn build(args: &[String]) -> Result<i32> {
    let run_path = arg_value(args, "--run").ok_or_else(|| anyhow::anyhow!("build needs --run <run.json>"))?;
    let run: FlowRun = serde_json::from_str(
        &fs::read_to_string(run_path).with_context(|| format!("reading {run_path}"))?,
    )
    .with_context(|| format!("parsing {run_path} as a flow run"))?;

    let findings: Vec<Finding> = match arg_value(args, "--findings") {
        Some(path) => serde_json::from_str(
            &fs::read_to_string(path).with_context(|| format!("reading {path}"))?,
        )
        .with_context(|| format!("parsing {path} as findings"))?,
        // The property registry is filled in as each property lands; until then a run with no
        // findings is honest — the evidence is still checkable.
        None => zanary_properties::evaluate(&run),
    };

    let verified_at = match arg_value(args, "--verified-at") {
        Some(ts) => ts.to_string(),
        None => OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .context("formatting the current time")?,
    };
    let ttl_seconds = match arg_value(args, "--ttl-days") {
        Some(days) => Some(
            days.parse::<u64>()
                .context("parsing --ttl-days")?
                .checked_mul(86_400)
                .ok_or_else(|| anyhow::anyhow!("--ttl-days overflows"))?,
        ),
        None => None,
    };

    let report = build_with_claim(&run, findings, Some(verified_at), ttl_seconds)?;
    let out = PathBuf::from(arg_value(args, "--out").unwrap_or("report.json"));
    fs::write(&out, serde_json::to_string_pretty(&report)?)
        .with_context(|| format!("writing {}", out.display()))?;

    let root = zanary_report::evidence_root(&report.evidence)?;
    println!("built {}", out.display());
    println!("  flow        : {} @ {}", report.flow_id, short(&report.flow_commit));
    println!("  chain       : {} (verified at height {})", report.chain, report.verified_at_height);
    println!("  verified_at : {}", report.verified_at.as_deref().unwrap_or("(none)"));
    println!("  ttl         : {}d", zanary_report::claim_ttl_seconds(&report) / 86_400);
    println!("  findings    : {}", report.findings.len());
    println!("  evidence    : {} entries", report.evidence.len());
    println!("  report_hash : {}", report.report_hash);
    println!("  evidence_root: {root}");
    Ok(0)
}

fn sample_run(args: &[String]) -> Result<i32> {
    let out = PathBuf::from(arg_value(args, "--out").unwrap_or("run.json"));
    fs::write(&out, SAMPLE_RUN).with_context(|| format!("writing {}", out.display()))?;
    println!("wrote {}", out.display());
    println!(
        "  a real run recorded against the local harness chain (see crates/report/fixtures/);\n\
         \x20 build it with:\n\
         \x20   zanary-report build --run {} --findings crates/report/fixtures/findings.json \\\n\
         \x20     --verified-at 2026-10-10T20:30:00Z --out report.json",
        out.display()
    );
    Ok(0)
}

fn index(args: &[String]) -> Result<i32> {
    let dir = arg_value(args, "--reports").ok_or_else(|| anyhow::anyhow!("index needs --reports <dir>"))?;
    let out = arg_value(args, "--out").ok_or_else(|| anyhow::anyhow!("index needs --out <index.html>"))?;
    let now = match arg_value(args, "--now") {
        Some(ts) => parse_ts(ts)?,
        None => OffsetDateTime::now_utc(),
    };

    // Two layouts are accepted, because both are natural ways to keep runs on disk: a flat
    // directory of report files, and a directory of run directories each holding its own
    // `report.json` (which is what `zanary-probe flow` writes under `data/runs/`). Reading only
    // one of them would make the documented command silently index nothing.
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("reading reports dir {dir}"))? {
        let path = entry?.path();
        if path.is_dir() {
            let nested = path.join("report.json");
            if nested.is_file() {
                files.push(nested);
            }
            continue;
        }
        if path.extension().is_some_and(|e| e == "json") {
            files.push(path);
        }
    }
    files.sort();

    let mut reports = Vec::new();
    for path in &files {
        let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        match serde_json::from_str::<Report>(&text) {
            Ok(report) => reports.push(report),
            Err(e) => eprintln!("warning: skipping {}: {e}", path.display()),
        }
    }

    // Anchors, when the runs carry them: an anchor record lives beside the report it anchors
    // (`<run>/anchor.json` for the run-directory layout, `<report.json>.anchor.json` for a flat one).
    // A report whose hash recomputes and whose anchor record agrees is a claim with a public
    // timestamp; the page says which of those it has.
    let mut anchors: std::collections::BTreeMap<String, zanary_report::anchor::AnchorRef> =
        std::collections::BTreeMap::new();
    for path in &files {
        let candidate = match path.file_name().map(|n| n == "report.json") {
            Some(true) => path.parent().map(|dir| dir.join("anchor.json")),
            _ => Some(PathBuf::from(format!("{}.anchor.json", path.display()))),
        };
        let Some(candidate) = candidate else { continue };
        if !candidate.is_file() {
            continue;
        }
        match zanary_report::anchor::AnchorRecord::from_path(&candidate.to_string_lossy()) {
            Ok(record) => {
                anchors.insert(
                    record.anchor.report_hash.clone(),
                    zanary_report::anchor::AnchorRef {
                        txid: record.txid.clone(),
                        height: record.height,
                    },
                );
            }
            Err(e) => eprintln!("warning: ignoring {}: {e}", candidate.display()),
        }
    }

    let html = render_index_with_anchors(&reports, &anchors, now)?;
    fs::write(out, html).with_context(|| format!("writing {out}"))?;
    let (claims, regressions) = zanary_report::index::index_claims(&reports, now);
    let live = claims.iter().filter(|c| c.status == Some(zanary_report::ClaimStatus::Live)).count();
    let stale = claims.iter().filter(|c| c.status == Some(zanary_report::ClaimStatus::Stale)).count();
    let dead = claims.iter().filter(|c| c.status == Some(zanary_report::ClaimStatus::Dead)).count();
    println!("indexed {} report(s) → {out}", reports.len());
    println!("  live {live} · stale {stale} · dead {dead} · regressions {}", regressions.len());
    println!(
        "  anchors: {} report(s) carry an anchor record (the rest are unanchored evidence)",
        anchors.len()
    );
    Ok(0)
}

fn short(hash: &str) -> String {
    if hash.len() > 12 {
        format!("{}…", &hash[..12])
    } else {
        hash.to_string()
    }
}
