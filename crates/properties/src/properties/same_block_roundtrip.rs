//! `same-block-roundtrip` (W) — funding and spending land in the same block.
//!
//! Leak: funding and spending in one block is a hard link for any observer with block-level data.
//!
//! Detect: height comparison across the flow's txids — any payment whose height equals a funding
//! event's height is a finding. Amount equality is reported in the evidence because the spec
//! notes this is D-adjacent when combined with `amount-fingerprint` (reported as W alone only
//! when the amounts differ and the anonymity set is large).
//!
//! Evidence: the two txids and their shared height.

use crate::{Finding, Property, Severity};
use canary_observation::FlowRun;

pub struct SameBlockRoundtrip;

impl Property for SameBlockRoundtrip {
    fn id(&self) -> &'static str {
        "same-block-roundtrip"
    }

    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn title(&self) -> &'static str {
        "funding and spending do not share a block"
    }

    fn check(&self, run: &FlowRun) -> Finding {
        if run.payments.is_empty() {
            return Finding::skip(
                "same-block-roundtrip",
                Severity::Warn,
                self.title(),
                "no payments recorded; no spending side to compare heights with",
            );
        }
        let fundings: Vec<&canary_observation::Funding> =
            run.funding.iter().filter(|f| f.height.is_some()).collect();
        if fundings.is_empty() {
            return Finding::skip(
                "same-block-roundtrip",
                Severity::Warn,
                self.title(),
                "no funding heights recorded; heights cannot be compared",
            );
        }
        let spends: Vec<&canary_observation::Payment> =
            run.payments.iter().filter(|p| p.height.is_some()).collect();
        if spends.is_empty() {
            return Finding::skip(
                "same-block-roundtrip",
                Severity::Warn,
                self.title(),
                "no payment heights recorded; heights cannot be compared",
            );
        }

        let mut evidence: Vec<String> = Vec::new();
        for p in &spends {
            for f in &fundings {
                if p.height == f.height {
                    evidence.push(format!(
                        "payment txid={} height={} funding txid={} height={} (funding and spending in the same block)",
                        p.txid,
                        p.height.unwrap(),
                        f.txid,
                        f.height.unwrap()
                    ));
                    if p.amount_zat == f.amount_zat {
                        evidence.push(format!(
                            "amounts equal ({} zat) — combined with amount-fingerprint this is disqualifying-class, reported here as W alone",
                            p.amount_zat
                        ));
                    } else {
                        evidence.push(format!(
                            "amounts differ (payment={} funding={}) — W alone, D-adjacent only when amounts also fingerprint",
                            p.amount_zat, f.amount_zat
                        ));
                    }
                }
            }
        }

        if !evidence.is_empty() {
            return Finding::fail(
                "same-block-roundtrip",
                Severity::Warn,
                self.title(),
                evidence,
            )
            .with_note(
                "any shared height between a payment and a funding event is a hard link for an \
                 observer with block-level data; the amount-equality line says whether \
                 amount-fingerprint should be read together with this finding.",
            );
        }

        Finding::pass("same-block-roundtrip", Severity::Warn, self.title()).with_note(format!(
            "{} spend(s) checked against {} funding event(s); no shared heights",
            spends.len(),
            fundings.len()
        ))
    }
}
