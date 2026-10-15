//! The claim index — a static page that says which privacy claims are still current.
//!
//! Every report is a claim: *this revision of this flow passed these properties at this height,
//! and here is the evidence*. The index re-derives each claim's status from its own clock
//! (`verified_at` + `ttl_seconds`, per docs/CLAIMS.md):
//!
//!   * `live`  — inside the TTL;
//!   * `stale` — one re-probe missed (inside the second TTL window);
//!   * `dead`  — two missed: the claim was true then and has not been re-proved since.
//!
//! A future `fail` on a property that passed earlier is shown as a **regression**, with both
//! runs linked. The page is deliberately deterministic for the same inputs: stable ordering,
//! one explicit `--now`, no ambient clock reads.

use anyhow::Result;
use zanary_properties::{Severity, Verdict};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::{claim_status, claim_ttl_seconds, expires_at, hash_report, ClaimStatus, Report};

/// One report plus what the index derived about it.
pub struct IndexedClaim<'a> {
    pub report: &'a Report,
    /// The report's own hash recomputes over its canonical JSON.
    pub hash_valid: bool,
    /// `None` when the claim carries no parseable clock.
    pub status: Option<ClaimStatus>,
    /// The next re-issue of the same flow revision, when one exists (history is append-only).
    pub superseded_by: Option<&'a Report>,
}

pub struct Regression<'a> {
    pub property: &'a str,
    pub severity: Severity,
    pub from: &'a Report,
    pub to: &'a Report,
}

/// Sort key: stable, clock-first ordering. Unparseable clocks sort first, by string.
fn order_key(report: &Report) -> (String, String, i128, u32, String) {
    let ts = report
        .verified_at
        .as_deref()
        .and_then(|s| OffsetDateTime::parse(s, &Rfc3339).ok())
        .map(|t| t.unix_timestamp_nanos())
        .unwrap_or(i128::MIN);
    (
        report.flow_id.clone(),
        report.flow_commit.clone(),
        ts,
        report.verified_at_height,
        report.report_hash.clone(),
    )
}

/// Index a set of reports: stable ordering, per-claim status, supersede links, regressions.
pub fn index_claims<'a>(reports: &'a [Report], now: OffsetDateTime) -> (Vec<IndexedClaim<'a>>, Vec<Regression<'a>>) {
    let mut ordered: Vec<&Report> = reports.iter().collect();
    ordered.sort_by(|a, b| order_key(a).cmp(&order_key(b)));

    let mut claims: Vec<IndexedClaim<'a>> = Vec::with_capacity(ordered.len());
    for (i, report) in ordered.iter().enumerate() {
        let hash_valid = hash_report(report)
            .map(|h| h == report.report_hash)
            .unwrap_or(false);
        let status = claim_status(report, now);
        // The successor is the earliest later report for the same flow revision.
        let superseded_by = ordered
            .iter()
            .skip(i + 1)
            .find(|candidate| {
                candidate.flow_id == report.flow_id && candidate.flow_commit == report.flow_commit
            })
            .copied();
        claims.push(IndexedClaim {
            report,
            hash_valid,
            status,
            superseded_by,
        });
    }

    // Regressions: a property that passed (the last non-skip verdict) and now fails, within
    // the same flow revision.
    let mut regressions = Vec::new();
    let mut groups: Vec<(String, String)> = Vec::new();
    for report in &ordered {
        let key = (report.flow_id.clone(), report.flow_commit.clone());
        if !groups.contains(&key) {
            groups.push(key);
        }
    }
    for (flow_id, flow_commit) in groups {
        let group: Vec<&Report> = ordered
            .iter()
            .filter(|r| r.flow_id == flow_id && r.flow_commit == flow_commit)
            .copied()
            .collect();
        let mut last_verdict: std::collections::BTreeMap<&str, (&Report, Verdict)> =
            std::collections::BTreeMap::new();
        for report in group {
            for finding in &report.findings {
                if finding.verdict == Verdict::Skip {
                    continue;
                }
                match last_verdict.get(finding.property.as_str()) {
                    Some((_, Verdict::Pass)) if finding.verdict == Verdict::Fail => {
                        let (from, _) = last_verdict[finding.property.as_str()];
                        regressions.push(Regression {
                            property: finding.property.as_str(),
                            severity: finding.severity,
                            from,
                            to: report,
                        });
                    }
                    _ => {}
                }
                last_verdict.insert(finding.property.as_str(), (report, finding.verdict));
            }
        }
    }

    (claims, regressions)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn short(hash: &str) -> String {
    if hash.len() > 12 {
        format!("{}…", &hash[..12])
    } else {
        hash.to_string()
    }
}

fn severity_str(s: Severity) -> &'static str {
    s.as_str()
}

fn verdict_class(v: Verdict) -> &'static str {
    match v {
        Verdict::Pass => "pass",
        Verdict::Fail => "fail",
        Verdict::Skip => "skip",
    }
}

fn status_class(s: Option<ClaimStatus>) -> &'static str {
    match s {
        Some(ClaimStatus::Live) => "live",
        Some(ClaimStatus::Stale) => "stale",
        Some(ClaimStatus::Dead) => "dead",
        None => "undated",
    }
}

fn status_label(s: Option<ClaimStatus>) -> &'static str {
    match s {
        Some(ClaimStatus::Live) => "live",
        Some(ClaimStatus::Stale) => "stale (one re-probe missed)",
        Some(ClaimStatus::Dead) => "dead (two re-probes missed)",
        None => "undated",
    }
}

/// Human countdown, e.g. "expires in 21d" or "expired 7d ago".
fn countdown(report: &Report, now: OffsetDateTime) -> String {
    let Some(expires) = expires_at(report) else {
        return "no clock".to_string();
    };
    let delta = expires - now;
    let days = delta.whole_days().abs();
    let hours = (delta - time::Duration::days(delta.whole_days())).whole_hours().abs();
    if delta.is_positive() {
        format!("expires in {days}d {hours}h")
    } else {
        format!("expired {days}d {hours}h ago")
    }
}

/// Render the whole index as one self-contained HTML page.
pub fn render_index(reports: &[Report], now: OffsetDateTime) -> Result<String> {
    render_index_with_anchors(reports, &std::collections::BTreeMap::new(), now)
}

/// [`render_index`], with the anchors each report has on chain.
///
/// The anchor is what makes a claim a *timestamped* claim rather than a file someone can edit, so
/// the page shows it next to the report hash: the transaction and the height that mined it. A
/// report with no anchor record renders exactly as before, which is why passing an empty map
/// reproduces the earlier pages byte for byte.
pub fn render_index_with_anchors(
    reports: &[Report],
    anchors: &std::collections::BTreeMap<String, crate::anchor::AnchorRef>,
    now: OffsetDateTime,
) -> Result<String> {
    let (claims, regressions) = index_claims(reports, now);
    let live = claims.iter().filter(|c| c.status == Some(ClaimStatus::Live)).count();
    let stale = claims.iter().filter(|c| c.status == Some(ClaimStatus::Stale)).count();
    let dead = claims.iter().filter(|c| c.status == Some(ClaimStatus::Dead)).count();
    let undated = claims.iter().filter(|c| c.status.is_none()).count();
    let invalid = claims.iter().filter(|c| !c.hash_valid).count();

    let mut html = String::new();
    html.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("<title>ZANARY — privacy claims that expire</title>\n");
    html.push_str("<style>\n");
    html.push_str(
        ":root { color-scheme: dark; }\n\
         body { margin: 0 auto; max-width: 62rem; padding: 2rem 1.25rem 4rem; \
         font: 15px/1.55 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; \
         background: #0d1117; color: #d7dde5; }\n\
         h1 { font-size: 1.3rem; margin: 0 0 .25rem; }\n\
         h2 { font-size: 1.05rem; margin: 2.2rem 0 .4rem; }\n\
         .sub { color: #8b96a5; margin: 0 0 1.4rem; }\n\
         .summary span { margin-right: 1rem; }\n\
         .badge { display: inline-block; padding: .05rem .45rem; border-radius: .3rem; \
         font-weight: 700; font-size: .82rem; }\n\
         .live { background: #14351f; color: #58d68d; }\n\
         .stale { background: #3a3113; color: #f4d03f; }\n\
         .dead { background: #3b1b1b; color: #ec7063; }\n\
         .undated { background: #262b33; color: #9aa4b2; }\n\
         .pass { color: #58d68d; }\n\
         .fail { color: #ec7063; font-weight: 700; }\n\
         .skip { color: #9aa4b2; }\n\
         .card { border: 1px solid #262b33; border-radius: .5rem; padding: 1rem 1.1rem; \
         margin: 1rem 0; background: #10151c; }\n\
         table { border-collapse: collapse; width: 100%; margin-top: .6rem; }\n\
         th, td { text-align: left; padding: .25rem .6rem .25rem 0; vertical-align: top; }\n\
         th { color: #8b96a5; font-weight: 400; }\n\
         code { color: #c8d3e0; }\n\
         .meta { color: #8b96a5; font-size: .9rem; }\n\
         .warn { color: #f4d03f; }\n\
         .reg { border-left: 3px solid #ec7063; padding-left: .8rem; margin: .8rem 0; }\n\
         footer { margin-top: 3rem; color: #8b96a5; font-size: .9rem; }\n\
         a { color: #7fb3ff; }\n",
    );
    html.push_str("</style>\n</head>\n<body>\n");
    html.push_str("<h1>ZANARY — privacy claims that expire</h1>\n");
    html.push_str(
        "<p class=\"sub\">Every claim is “this exact revision of this flow passed these \
         properties at this height, and here is the evidence”. Evidence, not scores: re-run \
         <code>zanary-report verify &lt;report.json&gt;</code> to falsify any row.</p>\n",
    );
    html.push_str(&format!(
        "<p class=\"sub summary\">generated {} · <span class=\"badge live\">{live} live</span> \
         <span class=\"badge stale\">{stale} stale</span> \
         <span class=\"badge dead\">{dead} dead</span>{} \
         <span>{} regression{}</span>{}</p>\n",
        esc(&now.format(&Rfc3339).unwrap_or_else(|_| now.to_string())),
        if undated > 0 { format!(" <span class=\"badge undated\">{undated} undated</span>") } else { String::new() },
        regressions.len(),
        if regressions.len() == 1 { "" } else { "s" },
        if invalid > 0 {
            format!(
                " <span class=\"warn\">⚠ {invalid} report{} whose hash does not recompute</span>",
                if invalid == 1 { "" } else { "s" }
            )
        } else {
            String::new()
        },
    ));

    if claims.is_empty() {
        html.push_str("<p>No reports in this index.</p>\n");
    }

    for claim in &claims {
        let report = claim.report;
        html.push_str("<div class=\"card\">\n");
        html.push_str(&format!(
            "<h2>{} @ <code>{}</code> <span class=\"badge {}\">{}</span></h2>\n",
            esc(&report.flow_id),
            esc(&short(&report.flow_commit)),
            status_class(claim.status),
            esc(status_label(claim.status)),
        ));
        let verified_at = report
            .verified_at
            .as_deref()
            .unwrap_or("(no verified_at)");
        html.push_str(&format!(
            "<p class=\"meta\">chain <code>{}</code> · verified at height {} · verified_at \
             <code>{}</code> · ttl {}d · {}{}</p>\n",
            esc(&report.chain),
            report.verified_at_height,
            esc(verified_at),
            claim_ttl_seconds(report) / 86_400,
            esc(&countdown(report, now)),
            if claim.superseded_by.is_some() {
                let s = claim.superseded_by.unwrap();
                format!(
                    " · <span class=\"meta\">re-issued by <code>{}</code></span>",
                    esc(&short(&s.report_hash))
                )
            } else {
                String::new()
            },
        ));
        html.push_str(&format!(
            "<p class=\"meta\">evidence root <code title=\"{}\">{}</code> · report hash \
             <code title=\"{}\">{}</code>{}</p>\n",
            esc(&crate::evidence_root(&report.evidence).unwrap_or_default()),
            esc(&short(&crate::evidence_root(&report.evidence).unwrap_or_default())),
            esc(&report.report_hash),
            esc(&short(&report.report_hash)),
            if claim.hash_valid {
                String::new()
            } else {
                " <span class=\"warn\">— hash does not recompute; this report has been altered</span>".to_string()
            },
        ));
        // Anchor status: only rendered when the caller actually looked for anchors beside the
        // reports. An index built without that lookup says nothing rather than guessing.
        if !anchors.is_empty() {
            match anchors.get(&report.report_hash) {
                Some(anchor) => html.push_str(&format!(
                    "<p class=\"meta\">anchor memo <code title=\"{}\">{}</code> @ height {} — \
                     anchored evidence: the chain says this root existed by then</p>\n",
                    esc(&anchor.txid),
                    esc(&short(&anchor.txid)),
                    anchor.height,
                )),
                None => html.push_str(
                    "<p class=\"meta\">no anchor record beside this report — unanchored evidence is \
                     only as old as the file that carries it</p>\n",
                ),
            }
        }
        html.push_str(&format!(
            "<p class=\"meta\">{} evidence entr{} · merchant side {}</p>\n",
            report.evidence.len(),
            if report.evidence.len() == 1 { "y" } else { "ies" },
            if report.merchant_is_simulated { "simulated (stated)" } else { "real" },
        ));

        if report.findings.is_empty() {
            html.push_str("<p class=\"meta\">no findings recorded in this report</p>\n");
        } else {
            html.push_str(
                "<table>\n<tr><th>property</th><th>sev</th><th>verdict</th><th>what</th></tr>\n",
            );
            for finding in &report.findings {
                html.push_str(&format!(
                    "<tr><td><code>{}</code></td><td>{}</td><td class=\"{}\">{}</td>\
                     <td>{} {}</td></tr>\n",
                    esc(&finding.property),
                    severity_str(finding.severity),
                    verdict_class(finding.verdict),
                    match finding.verdict {
                        Verdict::Pass => "PASS",
                        Verdict::Fail => "FAIL",
                        Verdict::Skip => "SKIP",
                    },
                    esc(&finding.title),
                    finding
                        .note
                        .as_deref()
                        .map(|n| format!("<span class=\"meta\">({})</span>", esc(n)))
                        .unwrap_or_default(),
                ));
            }
            html.push_str("</table>\n");
        }
        html.push_str("</div>\n");
    }

    if !regressions.is_empty() {
        html.push_str("<h2>Regressions</h2>\n");
        html.push_str(
            "<p class=\"meta\">A property that passed earlier now fails — both runs linked, \
             because the failure is the finding.</p>\n",
        );
        for regression in &regressions {
            html.push_str(&format!(
                "<div class=\"reg\"><code>{}</code> <span class=\"badge\">{}</span> passed at \
                 <code>{}</code> ({}) and now <span class=\"fail\">fails</span> at \
                 <code>{}</code> ({})</div>\n",
                esc(regression.property),
                severity_str(regression.severity),
                esc(&short(&regression.from.report_hash)),
                esc(regression.from.verified_at.as_deref().unwrap_or("undated")),
                esc(&short(&regression.to.report_hash)),
                esc(regression.to.verified_at.as_deref().unwrap_or("undated")),
            ));
        }
    }

    html.push_str(
        "<footer>An unverified privacy claim decays to a liability. This page is the \
         convenience, not the authority — every row can be re-derived from the chain by anyone \
         with the report and a node.</footer>\n",
    );
    html.push_str("</body>\n</html>\n");
    Ok(html)
}
