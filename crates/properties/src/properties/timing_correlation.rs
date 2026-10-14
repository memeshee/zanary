//! `timing-correlation` (W) — a payment follows its funding within a window that identifies the
//! payer.
//!
//! Leak: the interval between funding and spending is a partial identifier when the pool is small
//! or the amounts are unusual.
//!
//! Detect: for each spend, compute the delay to the nearest preceding inflow to the same flow's
//! wallets and flag delays inside a fixed window. The observation model records heights, not
//! timestamps, so the spec's "same block or < 10 minutes" is expressed as
//! [`TIMING_WINDOW_BLOCKS`] blocks (default 1 = the immediately following block). A delay of 0
//! (same block) is deliberately *not* re-flagged here — `same-block-roundtrip` owns that case,
//! and keeping the two checks disjoint means each one reports a distinct observation.
//!
//! Evidence: heights, delays, and the candidate count at the spend height (payments + funding
//! events recorded at that height; a chain-wide count is not part of the observation model).

use crate::{Finding, Property, Severity};
use canary_observation::{FlowRun, Funding, Payment};

use super::common::height_str;

/// Spends within this many blocks after their nearest preceding inflow are flagged.
/// Expressed in blocks because the model has no timestamps; 1 = adjacent block.
pub const TIMING_WINDOW_BLOCKS: u32 = 1;

pub struct TimingCorrelation;

impl Property for TimingCorrelation {
    fn id(&self) -> &'static str {
        "timing-correlation"
    }

    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn title(&self) -> &'static str {
        "spends are not tightly correlated with their funding"
    }

    fn check(&self, run: &FlowRun) -> Finding {
        if run.payments.is_empty() {
            return Finding::skip(
                "timing-correlation",
                Severity::Warn,
                self.title(),
                "no payments recorded; nothing to correlate against funding",
            );
        }

        let inflows: Vec<&Funding> = run.funding.iter().filter(|f| f.height.is_some()).collect();
        if inflows.is_empty() {
            return Finding::skip(
                "timing-correlation",
                Severity::Warn,
                self.title(),
                "no funding heights recorded; there is no inflow to measure a delay against",
            );
        }

        let spends: Vec<&Payment> = run.payments.iter().filter(|p| p.height.is_some()).collect();
        if spends.is_empty() {
            return Finding::skip(
                "timing-correlation",
                Severity::Warn,
                self.title(),
                "no payment heights recorded; delays are not computable",
            );
        }

        let mut evidence: Vec<String> = Vec::new();
        let mut considered = 0usize;
        let mut same_block_only = 0usize;
        let mut had_preceding = false;

        for p in &spends {
            let h = p.height.unwrap();
            let preceding: Vec<&&Funding> =
                inflows.iter().filter(|f| f.height.unwrap() <= h).collect();
            if preceding.is_empty() {
                continue;
            }
            had_preceding = true;
            // Nearest strictly-earlier block; delay 0 belongs to same-block-roundtrip.
            let Some(delay) = preceding
                .iter()
                .map(|f| h - f.height.unwrap())
                .filter(|d| *d >= 1)
                .min()
            else {
                same_block_only += 1;
                continue;
            };
            considered += 1;
            if delay <= TIMING_WINDOW_BLOCKS {
                let nearest = preceding
                    .iter()
                    .filter(|f| h - f.height.unwrap() == delay)
                    .max_by_key(|f| f.height.unwrap())
                    .expect("delay came from this set");
                let candidates = run.payments.iter().filter(|q| q.height == Some(h)).count()
                    + run.funding.iter().filter(|f| f.height == Some(h)).count();
                evidence.push(format!(
                    "payment txid={} height={} funding txid={} height={} delay_blocks={} window_blocks={} (spend follows its funding within the correlation window)",
                    p.txid,
                    h,
                    nearest.txid,
                    height_str(nearest.height),
                    delay,
                    TIMING_WINDOW_BLOCKS
                ));
                evidence.push(format!(
                    "height={} record_candidates={} (payments+fundings recorded at this height; chain-wide candidate count is not in the observation model)",
                    h, candidates
                ));
            }
        }

        if !evidence.is_empty() {
            return Finding::fail("timing-correlation", Severity::Warn, self.title(), evidence)
                .with_note(format!(
                "window={} block(s) — timestamps are not recorded in the observation model, so the \
                 spec's \"< 10 minutes\" is expressed in blocks and configurable via \
                 TIMING_WINDOW_BLOCKS; delay 0 is reported by same-block-roundtrip instead.",
                TIMING_WINDOW_BLOCKS
            ));
        }

        if !had_preceding {
            return Finding::skip(
                "timing-correlation",
                Severity::Warn,
                self.title(),
                "no spend follows a recorded inflow in block order; delays are not computable",
            );
        }

        if considered == 0 {
            return Finding::skip(
                "timing-correlation",
                Severity::Warn,
                self.title(),
                format!(
                    "every qualifying spend is same-block with its only recorded inflow ({} spend(s)); \
                     the adjacent-block window is not applicable — see same-block-roundtrip",
                    same_block_only
                ),
            );
        }

        Finding::pass("timing-correlation", Severity::Warn, self.title()).with_note(format!(
            "{} spend(s) checked against {} inflow(s); none within the {} block window (delay 0 excluded by design)",
            considered,
            inflows.len(),
            TIMING_WINDOW_BLOCKS
        ))
    }
}
