//! ZANARY property library — the 13 leak classes from `docs/PROPERTIES.md`, each a pure function
//! over a recorded [`FlowRun`].
//!
//! Contract for every property:
//!  * it is *pure*: same `FlowRun` in, same `Finding` out, no network, no clock, no randomness;
//!  * it never trusts the flow's own narration — it decides from observed txids, receivers, memos,
//!    heights and merchant events;
//!  * it emits `Verdict::Skip` with a reason when the observation set cannot decide the question
//!    (a skip is honest; a guessed pass is not);
//!  * every `Fail` carries evidence a stranger could re-check (txids, receivers, heights).

pub mod properties;

pub use properties::{AMOUNT_EPSILON_ZAT, PAYER_LINKABILITY_THRESHOLD, TIMING_WINDOW_BLOCKS};

use serde::{Deserialize, Serialize};
use zanary_observation::FlowRun;

/// How bad a failure of this property is. `D` failures disqualify a flow's privacy claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Note,
    Warn,
    Disqualifying,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Disqualifying => "D",
            Severity::Warn => "W",
            Severity::Note => "N",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Pass,
    Fail,
    Skip,
}

/// One property's result on one run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub property: String,
    pub severity: Severity,
    pub verdict: Verdict,
    pub title: String,
    /// Human-readable observations that produced the verdict; each one re-checkable.
    #[serde(default)]
    pub evidence: Vec<String>,
    /// Present on `Skip`, and on `Fail` when the fix is not obvious.
    pub note: Option<String>,
}

impl Finding {
    pub fn pass(property: &'static str, severity: Severity, title: impl Into<String>) -> Self {
        Finding {
            property: property.into(),
            severity,
            verdict: Verdict::Pass,
            title: title.into(),
            evidence: vec![],
            note: None,
        }
    }

    pub fn fail(
        property: &'static str,
        severity: Severity,
        title: impl Into<String>,
        evidence: Vec<String>,
    ) -> Self {
        Finding {
            property: property.into(),
            severity,
            verdict: Verdict::Fail,
            title: title.into(),
            evidence,
            note: None,
        }
    }

    pub fn skip(
        property: &'static str,
        severity: Severity,
        title: impl Into<String>,
        why: impl Into<String>,
    ) -> Self {
        Finding {
            property: property.into(),
            severity,
            verdict: Verdict::Skip,
            title: title.into(),
            evidence: vec![],
            note: Some(why.into()),
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

/// A property is a named check over a recorded run.
pub trait Property: Send + Sync {
    fn id(&self) -> &'static str;
    fn severity(&self) -> Severity;
    fn title(&self) -> &'static str;
    fn check(&self, run: &FlowRun) -> Finding;
}

/// Every property the rig implements, in report order (disqualifying first).
pub fn all_properties() -> Vec<Box<dyn Property>> {
    // Implemented in `properties/*.rs`, one module per leak class.
    crate::registry()
}

/// Evaluate every property over one run.
pub fn evaluate(run: &FlowRun) -> Vec<Finding> {
    let mut out: Vec<Finding> = all_properties().into_iter().map(|p| p.check(run)).collect();
    out.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| verdict_rank(a.verdict).cmp(&verdict_rank(b.verdict)))
            .then_with(|| a.property.cmp(&b.property))
    });
    out
}

fn verdict_rank(v: Verdict) -> u8 {
    match v {
        Verdict::Fail => 0,
        Verdict::Skip => 1,
        Verdict::Pass => 2,
    }
}

/// A run passes only when no disqualifying property failed.
pub fn passes(findings: &[Finding]) -> bool {
    !findings
        .iter()
        .any(|f| f.verdict == Verdict::Fail && f.severity == Severity::Disqualifying)
}

fn registry() -> Vec<Box<dyn Property>> {
    crate::properties::registry()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_exposes_every_property_exactly_once() {
        let props = all_properties();
        assert_eq!(props.len(), 13);
        let mut ids: Vec<&str> = props.iter().map(|p| p.id()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 13, "property ids must be unique");
    }

    #[test]
    fn ranking_puts_failures_first() {
        assert!(verdict_rank(Verdict::Fail) < verdict_rank(Verdict::Skip));
        assert!(verdict_rank(Verdict::Skip) < verdict_rank(Verdict::Pass));
    }
}
