//! The leak matrix: one property × flow table across a directory of runs.
//!
//! Every run directory holds a `run.json` (what happened) and a `findings.json` (what the property
//! library made of it). The matrix puts them side by side, so the claim "the private flow leaks
//! nothing this library can see, and the control leaks four things" is one table instead of a
//! reader's memory of several runs. Each column names its `flow_commit` — a claim is about an exact
//! revision of a flow, never about a flow name — and the re-check command for that run's report.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use zanary_observation::FlowRun;
use zanary_properties::{Finding, Verdict};

/// One run's column in the matrix.
pub struct RunColumn {
    pub flow_id: String,
    pub commit: String,
    pub dir: PathBuf,
    pub findings: Vec<Finding>,
    pub report_hash: Option<String>,
}

impl RunColumn {
    pub fn count(&self, verdict: Verdict) -> usize {
        self.findings
            .iter()
            .filter(|f| f.verdict == verdict)
            .count()
    }

    pub fn verdict(&self, property: &str) -> Option<&Finding> {
        self.findings.iter().find(|f| f.property == property)
    }
}

/// Reads every run directory under `runs_dir` that has both files, in name order.
pub fn collect(runs_dir: &Path) -> Result<Vec<RunColumn>> {
    let mut columns = Vec::new();
    let entries = std::fs::read_dir(runs_dir)
        .with_context(|| format!("reading the runs directory {}", runs_dir.display()))?;
    for entry in entries {
        let dir = entry?.path();
        let run_path = dir.join("run.json");
        let findings_path = dir.join("findings.json");
        if !run_path.is_file() || !findings_path.is_file() {
            continue;
        }
        let run: FlowRun = serde_json::from_str(
            &std::fs::read_to_string(&run_path)
                .with_context(|| format!("reading {}", run_path.display()))?,
        )
        .with_context(|| format!("parsing {}", run_path.display()))?;
        let findings: Vec<Finding> = serde_json::from_str(
            &std::fs::read_to_string(&findings_path)
                .with_context(|| format!("reading {}", findings_path.display()))?,
        )
        .with_context(|| format!("parsing {}", findings_path.display()))?;
        let report_hash = std::fs::read_to_string(dir.join("report.json"))
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .and_then(|value| {
                value
                    .get("report_hash")
                    .and_then(|h| h.as_str())
                    .map(str::to_string)
            });
        columns.push(RunColumn {
            flow_id: run.flow_id,
            commit: run.flow_commit,
            dir,
            findings,
            report_hash,
        });
    }
    columns.sort_by(|a, b| a.flow_id.cmp(&b.flow_id));
    Ok(columns)
}

/// Renders the matrix as markdown: a table a reader can scan, then the per-run re-check lines.
pub fn render_markdown(columns: &[RunColumn]) -> String {
    let mut out = String::new();
    out.push_str("# ZANARY leak matrix\n\n");
    out.push_str(
        "Every cell is one property of the leak library applied to one run of one flow definition.\n\
         `pass` means the property held; **FAIL** means it did not; `skip` means the run did not\n\
         exercise it. Each column names the `flow_commit` it is about, so a claim is pinned to an\n\
         exact revision of a definition, and the re-check line under the table re-derives that run's\n\
         report from the chain.\n\n",
    );

    // Property order: the order the first run listed them in (the library's own D, W, N order).
    let mut properties: Vec<(String, String)> = Vec::new();
    for column in columns {
        for finding in &column.findings {
            if !properties.iter().any(|(p, _)| p == &finding.property) {
                properties.push((
                    finding.property.clone(),
                    format!("{:?}", finding.severity).to_lowercase(),
                ));
            }
        }
    }

    out.push_str("| property |");
    for column in columns {
        out.push_str(&format!(
            " {} `{}` |",
            column.flow_id,
            crate::head_chars(&column.commit, 8)
        ));
    }
    out.push('\n');
    out.push_str("|---|");
    for _ in columns {
        out.push_str("---|");
    }
    out.push('\n');
    for (property, severity) in &properties {
        out.push_str(&format!("| `{property}` ({severity}) |"));
        for column in columns {
            let cell = match column.verdict(property) {
                Some(finding) => match finding.verdict {
                    Verdict::Pass => "pass".to_string(),
                    Verdict::Fail => "**FAIL**".to_string(),
                    Verdict::Skip => "skip".to_string(),
                },
                None => "—".to_string(),
            };
            out.push_str(&format!(" {cell} |"));
        }
        out.push('\n');
    }
    out.push_str("| **verdict** |");
    for column in columns {
        out.push_str(&format!(
            " **{} pass · {} fail · {} skip** |",
            column.count(Verdict::Pass),
            column.count(Verdict::Fail),
            column.count(Verdict::Skip)
        ));
    }
    out.push_str("\n\n");

    for column in columns {
        out.push_str(&format!(
            "- `{}` @ `{}` — run in `{}`",
            column.flow_id,
            crate::head_chars(&column.commit, 8),
            column.dir.display()
        ));
        match &column.report_hash {
            Some(hash) => out.push_str(&format!(
                "; report `{hash}`\n  ```\n  zanary-report verify {}/report.json --node <node>\n  ```\n",
                column.dir.display()
            )),
            None => out.push('\n'),
        }
    }
    out.push_str(
        "\nRegenerate: `zanary-report matrix --runs data/runs --out docs/LEAK-MATRIX.md`.\n",
    );
    out
}
