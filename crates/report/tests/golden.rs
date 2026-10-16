//! Golden tests over the bundled fixtures.
//!
//! `crates/report/fixtures/` carries a small **real** run — two shielded payments recorded
//! against the local harness chain (see `docs/REPORT-NOTES.md` for the txids) — plus the
//! reports built from it. These tests pin:
//!
//!   * the report build is deterministic (same run + findings + clock ⇒ same hash), and
//!   * the claim index renders byte-identical HTML for the same inputs and clock.
//!
//! Regenerate the golden HTML with:
//!   cargo run -p zanary-report -- index --reports crates/report/fixtures/reports \
//!     --out crates/report/fixtures/index.golden.html --now 2026-11-01T00:00:00Z

use std::path::PathBuf;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use zanary_observation::FlowRun;
use zanary_properties::Finding;
use zanary_report::{build_with_claim, hash_report, index::render_index, Report};

const RUN: &str = include_str!("../fixtures/run.json");
const FINDINGS: &str = include_str!("../fixtures/findings.json");
const REPORT: &str = include_str!("../fixtures/report.json");

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// Loading the reports the index is rendered over, in the same (sorted) order the CLI uses.
fn load_reports() -> Vec<Report> {
    let dir = fixtures_dir().join("reports");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| {
            serde_json::from_str(&std::fs::read_to_string(path).unwrap())
                .unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()))
        })
        .collect()
}

/// The report build is deterministic: the bundled run + findings + clock reproduce the
/// bundled report exactly, hash and evidence included.
#[test]
fn fixture_report_rebuilds_to_the_same_hash() {
    let run: FlowRun = serde_json::from_str(RUN).expect("fixture run parses");
    let findings: Vec<Finding> = serde_json::from_str(FINDINGS).expect("fixture findings parse");
    let rebuilt = build_with_claim(
        &run,
        findings,
        Some("2026-10-10T20:30:00Z".to_string()),
        None,
    )
    .expect("rebuilding the fixture report");
    let fixture: Report = serde_json::from_str(REPORT).expect("fixture report parses");

    assert_eq!(rebuilt.report_hash, fixture.report_hash, "hash drifted");
    assert_eq!(
        serde_json::to_value(&rebuilt).unwrap(),
        serde_json::to_value(&fixture).unwrap(),
        "rebuilt report differs from the fixture"
    );
}

/// Every report in the index fixture recomputes its own hash (no silently altered claims).
#[test]
fn fixture_reports_hash_correctly() {
    for report in load_reports() {
        let recomputed = hash_report(&report).expect("hashing a fixture report");
        assert_eq!(
            recomputed, report.report_hash,
            "fixture report {} has a stale hash",
            report.flow_id
        );
    }
}

/// The index renders byte-identical HTML for the same reports and clock.
#[test]
fn index_matches_golden_html() {
    let reports = load_reports();
    let now = OffsetDateTime::parse("2026-10-25T00:00:00Z", &Rfc3339).expect("clock parses");
    let html = render_index(&reports, now).expect("rendering the index");
    let golden_path = fixtures_dir().join("index.golden.html");
    let golden = std::fs::read_to_string(&golden_path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", golden_path.display()));
    assert_eq!(
        html,
        golden,
        "index output drifted from {}; regenerate it with `zanary-report index --reports \
         crates/report/fixtures/reports --out crates/report/fixtures/index.golden.html \
         --now 2026-10-25T00:00:00Z`",
        golden_path.display()
    );
}

/// The claim clock: live → stale → dead as the TTL windows pass (per docs/CLAIMS.md).
#[test]
fn fixture_claims_expire_on_schedule() {
    let reports = load_reports();
    let now = OffsetDateTime::parse("2026-10-25T00:00:00Z", &Rfc3339).expect("clock parses");
    let (claims, regressions) = zanary_report::index::index_claims(&reports, now);
    let statuses: Vec<(&str, &str)> = claims
        .iter()
        .map(|c| {
            (
                c.report.verified_at.as_deref().unwrap_or("(none)"),
                c.status.map(|s| s.as_str()).unwrap_or("undated"),
            )
        })
        .collect();
    assert!(
        statuses.contains(&("2026-10-10T18:00:00Z", "dead")),
        "{statuses:?}"
    );
    assert!(
        statuses.contains(&("2026-10-10T18:30:00Z", "stale")),
        "{statuses:?}"
    );
    assert!(
        statuses.contains(&("2026-10-10T20:30:00Z", "live")),
        "{statuses:?}"
    );
    assert!(
        statuses.contains(&("2026-10-10T21:05:00Z", "live")),
        "{statuses:?}"
    );

    // The receiver-reuse regression: passed in the first run of the revision, failed in the
    // second (both real runs; see docs/REPORT-NOTES.md).
    let properties: Vec<&str> = regressions.iter().map(|r| r.property).collect();
    assert_eq!(properties, vec!["receiver-reuse"], "{properties:?}");
}
