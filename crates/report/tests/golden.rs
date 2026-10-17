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
//!     --out crates/report/fixtures/index.golden.html --now 2026-10-25T00:00:00Z
//!
//! The golden covers the links as well as the rows: the test builds the same served-path map the
//! CLI builds, so a page that stopped linking its claims fails here and in the test below.

use std::collections::BTreeMap;
use std::path::PathBuf;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use zanary_observation::FlowRun;
use zanary_properties::Finding;
use zanary_report::{
    build_with_claim, hash_report,
    index::{render_index, render_index_full, served_claim_files, ClaimFiles},
    Report,
};

const RUN: &str = include_str!("../fixtures/run.json");
const FINDINGS: &str = include_str!("../fixtures/findings.json");
const REPORT: &str = include_str!("../fixtures/report.json");

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// The reports the index is rendered over, in the same (sorted) order the CLI uses — plus where
/// each one's files are served, computed with the CLI's own helper so the test cannot drift from it.
fn load_reports_with_files() -> (Vec<Report>, BTreeMap<String, ClaimFiles>) {
    let root = fixtures_dir().join("reports");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("reading {}: {e}", root.display()))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let mut reports = Vec::new();
    let mut claim_files = BTreeMap::new();
    for path in &files {
        let report: Report = serde_json::from_str(&std::fs::read_to_string(path).unwrap())
            .unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()));
        claim_files.insert(
            report.report_hash.clone(),
            served_claim_files(&root, path, "claims"),
        );
        reports.push(report);
    }
    (reports, claim_files)
}

/// Just the reports, for the tests that only look at them.
fn load_reports() -> Vec<Report> {
    load_reports_with_files().0
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
    let (reports, claim_files) = load_reports_with_files();
    let now = OffsetDateTime::parse("2026-10-25T00:00:00Z", &Rfc3339).expect("clock parses");
    let html = render_index_full(&reports, &BTreeMap::new(), &claim_files, now)
        .expect("rendering the index");
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

/// A caller that does not say where the claims live gets no link line at all — not a half-written
/// one with nothing after the colon.
#[test]
fn index_without_served_paths_links_nothing() {
    let reports = load_reports();
    let now = OffsetDateTime::parse("2026-10-25T00:00:00Z", &Rfc3339).expect("clock parses");
    let html = render_index(&reports, now).expect("rendering the index");
    assert!(
        !html.contains("<a href=\"claims/"),
        "unexpected claim links"
    );
    assert!(
        html.contains("<h1>ZANARY — privacy claims that expire</h1>"),
        "the page no longer renders"
    );
}

/// The page has to stay usable, not just readable: every card links the claim it was rendered from,
/// and the footer carries the one-command reproduction. A regeneration that drops either is a page a
/// judge can only look at.
#[test]
fn golden_links_the_claims_and_says_how_to_use_them() {
    let golden = std::fs::read_to_string(fixtures_dir().join("index.golden.html"))
        .expect("reading the golden index");
    for needle in [
        "href=\"claims/report-2026-10-10.json\"",
        "href=\"claims/report-2026-10-10-oldA.json\"",
        "href=\"https://github.com/memeshee/zanary\"",
        "href=\"https://github.com/memeshee/zanary#real-vs-mocked\"",
        "href=\"video/\"",
        "git clone https://github.com/memeshee/zanary",
        "harness/up.sh &amp;&amp; scripts/demo-run.sh",
    ] {
        assert!(
            golden.contains(needle),
            "the index no longer offers {needle}"
        );
    }
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
