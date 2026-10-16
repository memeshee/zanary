//! Recording a run: `run.json`, `findings.json`, `report.json`, in that order.
//!
//! Both the flow runner and the bounded payer end their run the same way, and the order matters: the
//! *run* is the observation (what the rig saw), the *findings* are the property library's verdict over
//! it, and the *report* is the canonical, hashable artifact a stranger verifies. Writing them from one
//! place keeps a report's provenance identical whichever subject produced it — the alternative is two
//! slightly different definitions of "the evidence", which is the failure this project exists to catch.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use zanary_observation::FlowRun;
use zanary_report::Report;

/// Where a run's artifacts landed, plus the verdict summary a CLI wants to print and exit on.
#[derive(Debug, Clone)]
pub struct RunArtifacts {
    /// Which flow this was, and which revision of it: the pair a claim is about.
    pub flow_id: String,
    pub flow_commit: String,
    /// The directory the artifacts were written into.
    pub out_dir: PathBuf,
    pub run_path: PathBuf,
    pub findings_path: PathBuf,
    pub report_path: PathBuf,
    pub report_hash: String,
    pub pass: usize,
    pub fail: usize,
    pub skip: usize,
    /// The properties that FAILED, in the library's order.
    pub failed_properties: Vec<String>,
    pub verified_at: String,
    pub verified_at_height: u32,
}

/// Writes `run.json` + `findings.json` + `report.json` for `run` into `out_dir`.
pub fn write_run_artifacts(
    run: &FlowRun,
    out_dir: &Path,
    ttl_days: Option<u64>,
) -> Result<RunArtifacts> {
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("creating the run directory {}", out_dir.display()))?;

    let run_path = out_dir.join("run.json");
    std::fs::write(&run_path, serde_json::to_string_pretty(run)?)
        .with_context(|| format!("writing {}", run_path.display()))?;

    let findings = zanary_properties::evaluate(run);
    let findings_path = out_dir.join("findings.json");
    std::fs::write(&findings_path, serde_json::to_string_pretty(&findings)?)
        .with_context(|| format!("writing {}", findings_path.display()))?;

    let verified_at = utc_now_rfc3339()?;
    let report = zanary_report::build_with_claim(
        run,
        findings.clone(),
        Some(verified_at.clone()),
        ttl_days.map(|days| days * 86_400),
    )?;
    let report_path = out_dir.join("report.json");
    std::fs::write(&report_path, zanary_report::canonical_json(&report)?)
        .with_context(|| format!("writing {}", report_path.display()))?;

    let (mut pass, mut fail, mut skip) = (0usize, 0usize, 0usize);
    for finding in &findings {
        match finding.verdict {
            zanary_properties::Verdict::Pass => pass += 1,
            zanary_properties::Verdict::Fail => fail += 1,
            zanary_properties::Verdict::Skip => skip += 1,
        }
    }

    Ok(RunArtifacts {
        flow_id: run.flow_id.clone(),
        flow_commit: run.flow_commit.clone(),
        out_dir: out_dir.to_path_buf(),
        run_path,
        findings_path,
        report_path,
        report_hash: report.report_hash.clone(),
        pass,
        fail,
        skip,
        failed_properties: findings
            .iter()
            .filter(|f| matches!(f.verdict, zanary_properties::Verdict::Fail))
            .map(|f| f.property.clone())
            .collect(),
        verified_at,
        verified_at_height: run.ended_at_height,
    })
}

/// Prints the per-property verdict lines a CLI run ends with, in the library's order.
pub fn print_findings(out_dir: &Path) -> Result<usize> {
    let findings_path = out_dir.join("findings.json");
    let findings: Vec<zanary_properties::Finding> = serde_json::from_str(
        &std::fs::read_to_string(&findings_path)
            .with_context(|| format!("reading {}", findings_path.display()))?,
    )?;
    let mut fails = 0usize;
    for finding in &findings {
        if matches!(finding.verdict, zanary_properties::Verdict::Fail) {
            fails += 1;
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
    Ok(fails)
}

/// The `Report` a run's directory holds, parsed (used by callers that want the claim's clock).
pub fn read_report(out_dir: &Path) -> Result<Report> {
    let path = out_dir.join("report.json");
    Ok(serde_json::from_str(
        &std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?,
    )?)
}

/// The one clock in this workspace's probe: RFC3339 UTC, from the same `time` the report crate uses
/// for claim TTLs, so "expired" means one thing everywhere (flow.rs used to shell out to `date` and
/// payer.rs had its own copy).
pub(crate) fn utc_now_rfc3339() -> Result<String> {
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|e| anyhow::anyhow!("formatting the current time: {e}"))
}
