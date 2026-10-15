//! `amount-fingerprint` (W) — amounts are round, repeated, or echo a funding amount so deposits
//! and withdrawals link.
//!
//! Leak: Quesnelle (2017) linked 31.5% of coins entering the pool to later exits using amount and
//! timing alone. Round amounts, echoed amounts (pay exactly what you received), and deposit-shaped
//! amounts are the same attack.
//!
//! Detect: compare the multiset of amounts across the flow's funding and spending events:
//!  * a payment amount divisible by [`ROUND_GRANULARITY_ZAT`] (0.1 ZEC) is a round amount;
//!  * two payments with exactly equal amounts are a repeat;
//!  * a payment exactly equal to a funding amount is an echo;
//!  * a payment within [`AMOUNT_EPSILON_ZAT`] of a funding amount is flagged only when exactly
//!    one funding candidate exists (the "how few candidates" weighting); more candidates make the
//!    match ambiguous and it is reported on the pass note instead.
//!
//! Evidence: the two events, the match type, and the candidate count.

use crate::{Finding, Property, Severity};
use zanary_observation::FlowRun;

/// Near-echo tolerance in zatoshis (0.00005 ZEC — covers a conventional fee's drift).
pub const AMOUNT_EPSILON_ZAT: u64 = 5_000;
/// Amounts divisible by this are "round" (0.1 ZEC granularity).
pub const ROUND_GRANULARITY_ZAT: u64 = 10_000_000;

pub struct AmountFingerprint;

impl Property for AmountFingerprint {
    fn id(&self) -> &'static str {
        "amount-fingerprint"
    }

    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn title(&self) -> &'static str {
        "amounts do not fingerprint deposits or repeat"
    }

    fn check(&self, run: &FlowRun) -> Finding {
        if run.payments.is_empty() {
            return Finding::skip(
                "amount-fingerprint",
                Severity::Warn,
                self.title(),
                "no payments recorded; there is no spending side to compare against funding",
            );
        }

        let mut evidence: Vec<String> = Vec::new();
        let mut near_misses: Vec<String> = Vec::new();

        // Round amounts.
        for p in &run.payments {
            if p.amount_zat > 0 && p.amount_zat % ROUND_GRANULARITY_ZAT == 0 {
                evidence.push(format!(
                    "txid={} amount_zat={} (round amount: divisible by {} zat = 0.1 ZEC)",
                    p.txid, p.amount_zat, ROUND_GRANULARITY_ZAT
                ));
            }
        }

        // Exact repeats across payments.
        for i in 0..run.payments.len() {
            for j in (i + 1)..run.payments.len() {
                let (a, b) = (&run.payments[i], &run.payments[j]);
                if a.amount_zat > 0 && a.amount_zat == b.amount_zat {
                    evidence.push(format!(
                        "txid={} txid={} amount_zat={} (exact repeat across payments)",
                        a.txid, b.txid, a.amount_zat
                    ));
                }
            }
        }

        // Funding echoes.
        for p in &run.payments {
            let candidates: Vec<&zanary_observation::Funding> = run
                .funding
                .iter()
                .filter(|f| f.amount_zat.abs_diff(p.amount_zat) <= AMOUNT_EPSILON_ZAT)
                .collect();
            if candidates.is_empty() {
                continue;
            }
            let exact = candidates.iter().find(|f| f.amount_zat == p.amount_zat);
            if let Some(f) = exact {
                evidence.push(format!(
                    "payment txid={} amount_zat={} == funding txid={} amount_zat={} (exact echo of a funding amount — deposit and withdrawal link; candidates_within_epsilon={})",
                    p.txid,
                    p.amount_zat,
                    f.txid,
                    f.amount_zat,
                    candidates.len()
                ));
            } else if candidates.len() == 1 {
                let f = candidates[0];
                evidence.push(format!(
                    "payment txid={} amount_zat={} within epsilon={} of funding txid={} amount_zat={} diff={} (unique candidate — deposit-shaped amount links)",
                    p.txid,
                    p.amount_zat,
                    AMOUNT_EPSILON_ZAT,
                    f.txid,
                    f.amount_zat,
                    f.amount_zat.abs_diff(p.amount_zat)
                ));
            } else {
                near_misses.push(format!(
                    "txid={} amount_zat={} is within epsilon={} of {} funding amounts (ambiguous, not flagged)",
                    p.txid,
                    p.amount_zat,
                    AMOUNT_EPSILON_ZAT,
                    candidates.len()
                ));
            }
        }

        if !evidence.is_empty() {
            return Finding::fail("amount-fingerprint", Severity::Warn, self.title(), evidence)
                .with_note(format!(
                    "thresholds: round granularity={} zat (0.1 ZEC), echo epsilon={} zat; \
                     near-echoes are flagged only when exactly one funding candidate exists \
                     (candidate count is reported per match).",
                    ROUND_GRANULARITY_ZAT, AMOUNT_EPSILON_ZAT
                ));
        }

        let mut finding = Finding::pass("amount-fingerprint", Severity::Warn, self.title());
        let note = if run.funding.is_empty() {
            "no round or repeated payment amounts; no funding events recorded, so echo checks were not applicable".to_string()
        } else if near_misses.is_empty() {
            format!(
                "{} payments checked against {} funding events; no round, repeated or echoed amounts (epsilon={} zat)",
                run.payments.len(),
                run.funding.len(),
                AMOUNT_EPSILON_ZAT
            )
        } else {
            format!(
                "no flagged amounts; near-misses: {}",
                near_misses.join("; ")
            )
        };
        finding = finding.with_note(note);
        finding
    }
}
