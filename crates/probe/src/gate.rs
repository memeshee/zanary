//! `zanary-probe test` — the CI gate: one flow, one command, an exit code a build can read.
//!
//! Every piece already existed separately: `flow` runs a definition and records it, `verify`
//! re-derives the report's facts from the chain, `anchor` publishes the evidence root in a
//! shielded memo. The gate composes them the way a pipeline needs them, in the order a reviewer
//! would ask for them — run, state the verdict, verify it against the chain, publish the
//! timestamp — in **one process with one exit code**, because a build step cannot stitch four
//! commands together and still be honest about which one failed.
//!
//! The second job of the gate is the one a test suite usually forgets: **a control flow that must
//! bite**. `--expect fail` passes only when the property library actually caught the planted leaks,
//! so a property that has quietly stopped detecting anything fails the build instead of looking
//! like a clean run. A privacy gate that has gone silent is a failure, not a pass.
//!
//! Exit codes:
//! ```text
//! 0  the run matched its expectation and the chain agrees with the report
//! 1  the run did not match its expectation: a private flow leaked, or a control flow that must
//!    fail passed — either way the verdict a reviewer would have drawn is not the one on record
//! 2  the gate itself is broken (chain unreachable, the report no longer hashes to itself, the
//!    anchor is missing or disagrees): an unverifiable run is never reported as a privacy result
//! ```
//!
//! Exit 2 exists because "the rig could not check" and "the flow leaks" are different news, and a
//! CI job that conflates them teaches people to ignore it.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use zanary_report::{
    anchor::{AnchorRecord, ScanOutcome},
    verify::{verify_report, VerifyOptions},
    Report,
};

use crate::flow;
use crate::ops::{self, Args};

/// What the flow definition is expected to do. A control flow is not a broken flow: it is a flow
/// whose leaks are the test, and it is only useful if the gate insists on finding them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expect {
    Pass,
    Fail,
}

impl Expect {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "pass" => Ok(Expect::Pass),
            "fail" => Ok(Expect::Fail),
            other => bail!(
                "--expect expects `pass` (a flow that must not leak) or `fail` (a deliberately \
                 leaky control whose failures are the test), got `{other}`"
            ),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Expect::Pass => "pass",
            Expect::Fail => "fail",
        }
    }

    /// Printed before the run, so a reader of the log knows what a green build means here.
    fn describe(self) -> &'static str {
        match self {
            Expect::Pass => "every property must hold (a leak fails this build)",
            Expect::Fail => {
                "the property library must CATCH this flow's leaks (a control that stops biting \
                 fails this build)"
            }
        }
    }
}

/// What the anchor step found, for the summary and the claim block.
struct AnchorInfo {
    height: u32,
    txid: String,
    memo: String,
    url: String,
}

/// `zanary-probe test --definition <flow.json> [--out <dir>] [--expect pass|fail] [--anchor]
/// [--ttl-days <n>] [--summary <path>] [--node <url>] [--indexer <url>]`
pub async fn test(raw: &[String]) -> Result<i32> {
    const USAGE: &str = "zanary-probe test --definition <flow.json> [--out <dir>] \
                         [--expect pass|fail] [--anchor] [--ttl-days <n>] [--summary <path>] \
                         [--node <url>] [--indexer <url>]";
    let mut args = Args::parse(raw, USAGE)?;
    let definition = PathBuf::from(args.require("--definition")?);
    let expect = match args.value("--expect") {
        Some(value) => Expect::parse(value.as_str())?,
        None => Expect::Pass,
    };
    let anchor = args.flag("--anchor");
    let ttl_days = args.optional_u32("--ttl-days")?.map(u64::from);
    let summary_path = args.value("--summary").map(PathBuf::from);
    let node_url = args.value("--node");
    let indexer_url = args.value("--indexer");
    let out_arg = args.value("--out").map(PathBuf::from);
    args.finish()?;

    let out_dir = match &out_arg {
        Some(dir) => dir.clone(),
        None => flow::default_out_dir(&definition),
    };

    println!(
        "zanary-probe {} test — a privacy claim is a test result, so let it fail a build",
        env!("CARGO_PKG_VERSION")
    );
    println!("  flow definition   : {}", definition.display());
    println!("  run directory     : {}", out_dir.display());
    println!("  expectation       : {}", expect.describe());
    if let Some(days) = ttl_days {
        println!("  claim TTL         : {days} day(s)");
    }
    println!();

    // ---- 1. run the flow (same code path as `zanary-probe flow`, in this process) -------------
    let mut flow_args = vec![
        "--definition".to_string(),
        definition.display().to_string(),
        "--out".to_string(),
        out_dir.display().to_string(),
    ];
    if let Some(days) = ttl_days {
        flow_args.push("--ttl-days".to_string());
        flow_args.push(days.to_string());
    }
    let outcome = flow::run_recording(&flow_args).await?;

    // ---- 2. re-derive the report's facts from the chain --------------------------------------
    let report: Report = serde_json::from_str(
        &std::fs::read_to_string(&outcome.report_path)
            .with_context(|| format!("reading {}", outcome.report_path.display()))?,
    )
    .with_context(|| format!("parsing {} as a report", outcome.report_path.display()))?;

    let mut opts = VerifyOptions::default();
    if let Some(url) = &node_url {
        opts.node = zanary_report::node::NodeRpc::new(
            hostport(url),
            std::env::var("ZANARY_NODE_USER").unwrap_or_else(|_| "zanary".into()),
            std::env::var("ZANARY_NODE_PASS").unwrap_or_else(|_| "zanary".into()),
        );
    }
    if let Some(url) = &indexer_url {
        opts.indexer_url = url.clone();
    }

    println!();
    println!("  --- chain verification (nothing here trusts the rig) ---");
    let verification = verify_report(&report, &opts).await?;
    for check in &verification.checks {
        println!(
            "  {:<46} {:<5} {}",
            check.label,
            check.verdict.as_str(),
            check.detail
        );
    }
    let chain_verified = verification.verified();
    println!(
        "  {:<46} {:<5} {} pass · {} fail · {} skip · {} warn",
        "verification summary",
        if chain_verified { "PASS" } else { "FAIL" },
        verification.passes(),
        verification.fails(),
        verification.skips(),
        verification.warns()
    );

    // ---- 3. publish the evidence root (optional) ---------------------------------------------
    let mut anchor_info: Option<AnchorInfo> = None;
    if anchor {
        println!();
        println!("  --- anchor: a shielded memo carries the evidence root ---");
        ops::anchor(&[
            "--report".to_string(),
            outcome.report_path.display().to_string(),
        ])
        .await?;

        let anchor_path = outcome.report_path.with_file_name("anchor.json");
        let anchor_path_text = anchor_path.display().to_string();
        let record = AnchorRecord::from_path(&anchor_path_text)
            .with_context(|| format!("reading the anchor record {anchor_path_text}"))?;
        // The record is the rig's own claim about what it published; it has to agree with the
        // report before a stranger's scan is even worth running.
        record.check_against(&report)?;

        let evidence_root = zanary_report::evidence_root(&report.evidence)?;
        let scan_node = match &node_url {
            Some(url) => zanary_report::node::NodeRpc::new(
                hostport(url),
                std::env::var("ZANARY_NODE_USER").unwrap_or_else(|_| "zanary".into()),
                std::env::var("ZANARY_NODE_PASS").unwrap_or_else(|_| "zanary".into()),
            ),
            None => zanary_report::node::NodeRpc::from_env(),
        };
        let scan: ScanOutcome =
            zanary_report::anchor::scan(&scan_node, &report.chain, &record.beacon_ufvk, 1)
                .context("scanning the chain for anchor memos with the beacon's published key")?;
        println!(
            "  scan              : {} block(s) read · {} transaction(s) trial-decrypted · {} anchor memo(s)",
            scan.blocks_read, scan.transactions_read, scan.hits.len()
        );
        let hits = scan.with_root(&evidence_root);
        let Some(hit) = hits.first() else {
            // The rig just wrote an anchor it cannot find again. That is a broken gate, not a leak.
            bail!(
                "the anchor for root {} was written to {} but a scan of blocks {}..={} with the \
                 beacon's published viewing key does not find it — the gate cannot certify a \
                 timestamp it cannot reproduce",
                &evidence_root[..8.min(evidence_root.len())],
                anchor_path_text,
                scan.since,
                scan.tip
            );
        };
        println!(
            "  anchor            : height {} · tx {} · earliest of {} copy/copies",
            hit.height,
            hit.txid,
            hits.len()
        );
        anchor_info = Some(AnchorInfo {
            height: hit.height,
            txid: hit.txid.clone(),
            memo: hit.anchor.memo_text()?,
            url: record.beacon_ufvk.clone(),
        });
    }

    // ---- 4. the claim block -------------------------------------------------------------------
    let verdict_matches = match expect {
        Expect::Pass => outcome.fail == 0,
        Expect::Fail => outcome.fail > 0,
    };
    let expires_at = zanary_report::expires_at_rfc3339(&report);
    let exit_code = if !chain_verified {
        2
    } else if verdict_matches {
        0
    } else {
        1
    };

    println!();
    println!("  --- claim ---");
    println!(
        "  flow              : {} @ {}",
        outcome.flow_id,
        &outcome.flow_commit[..8.min(outcome.flow_commit.len())]
    );
    println!(
        "  verified at       : height {} · {}",
        report.verified_at_height,
        report
            .verified_at
            .as_deref()
            .unwrap_or("(no wall clock recorded)")
    );
    println!(
        "  properties        : {} pass · {} fail · {} skip",
        outcome.pass, outcome.fail, outcome.skip
    );
    if !outcome.failed_properties.is_empty() {
        println!(
            "  caught            : {}",
            outcome.failed_properties.join(", ")
        );
    }
    println!(
        "  report hash       : {} — recomputed over the report's own bytes",
        &outcome.report_hash[..16.min(outcome.report_hash.len())]
    );
    match &anchor_info {
        Some(info) => println!(
            "  anchor            : height {} · tx {}",
            info.height,
            &info.txid[..16.min(info.txid.len())]
        ),
        None => println!("  anchor            : none (no chain timestamp on this run)"),
    }
    match &expires_at {
        Some(when) => println!(
            "  expires           : {when} — re-probe to re-issue the claim, or it goes stale then dead"
        ),
        None => println!("  expires           : never — a claim without a clock cannot expire"),
    }

    println!();
    let verdict = if !chain_verified {
        println!(
            "  verdict           : RIG ERROR — the report does not match the chain, so the run says \
             nothing about the flow either way"
        );
        "rig-error"
    } else if verdict_matches {
        println!(
            "  verdict           : {}",
            match expect {
                Expect::Pass => "PASS — every property held, and the chain agrees with the report",
                Expect::Fail =>
                    "CAUGHT — the property library found the leaks this flow plants, \
                                 which is what a control flow is for",
            }
        );
        "as-expected"
    } else if expect == Expect::Pass {
        println!(
            "  verdict           : FAIL — {} propert{} leaked; this is what a failing build looks like",
            outcome.fail,
            if outcome.fail == 1 { "y" } else { "ies" }
        );
        "leaked"
    } else {
        println!(
            "  verdict           : FAIL — this control flow was supposed to be caught by the \
             property library and was not: a detector that stopped biting is a broken gate"
        );
        "control-not-caught"
    };
    println!("  exit code         : {exit_code}");

    // ---- 5. the machine-readable summary ------------------------------------------------------
    if let Some(path) = &summary_path {
        let summary = serde_json::json!({
            "version": 1,
            "flow_id": outcome.flow_id,
            "flow_commit": outcome.flow_commit,
            "flow_definition": definition.display().to_string(),
            "run_dir": outcome.out_dir.display().to_string(),
            "pass": outcome.pass,
            "fail": outcome.fail,
            "skip": outcome.skip,
            "failed_properties": outcome.failed_properties,
            "report_hash": outcome.report_hash,
            "report_file": outcome.report_path.display().to_string(),
            "chain": report.chain,
            "verified_at_height": report.verified_at_height,
            "verified_at": report.verified_at,
            "claim_expires_at": expires_at,
            "chain_verified": chain_verified,
            "chain_checks": {
                "pass": verification.passes(),
                "fail": verification.fails(),
                "skip": verification.skips(),
                "warn": verification.warns(),
            },
            "anchor": anchor_info.as_ref().map(|info| serde_json::json!({
                "height": info.height,
                "txid": info.txid,
                "memo": info.memo,
                "beacon_ufvk": info.url,
            })),
            "expectation": expect.as_str(),
            "verdict": verdict,
            "exit_code": exit_code,
        });
        std::fs::write(path, serde_json::to_string_pretty(&summary)?)
            .with_context(|| format!("writing the test summary {}", path.display()))?;
        println!("  summary           : {}", path.display());
    }

    Ok(exit_code)
}

/// `NodeRpc::new` wants `host:port`; the CLI takes a URL because that is what people paste.
fn hostport(url: &str) -> String {
    url.strip_prefix("http://")
        .unwrap_or(url)
        .trim_end_matches('/')
        .to_string()
}
