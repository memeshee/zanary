//! `payer-linkability` (D) — repeat payments by the same payer are clusterable by an observer.
//!
//! Leak: even with fresh receivers, a payer whose payments share a distinguishable pattern is
//! fingerprintable: the same "agent" buying twice is a profile.
//!
//! Detect: the rig acts as an adversary with full chain visibility and the merchant's logs, then
//! tries to cluster. Every payment pair is scored on six signals (see the weight constants
//! below); pairs at or above [`CONFIDENCE_THRESHOLD`] (default 0.70) union into a cluster; any
//! cluster of two or more payments is a finding. Confidence is the sum of integer weights / 100,
//! so comparisons are exact.
//!
//! Five of the six signals are *heuristics* the adversary weighs; the sixth is a fact. Two payments
//! that share a txid are in one transaction — public proof that the outputs were created together
//! by one payer, with no inference required. That is what batched settlement discloses
//! (`docs/THREAT-MODEL.md`), so its weight is 1.00 on its own and a batched run fails this
//! property by design rather than by accident.
//!
//! Evidence: the cluster (txids), the signals that produced it with their weights, and the
//! threshold. The spec's "control" (re-running the same analysis on a deliberately clean flow)
//! is not expressible inside a pure function over one run — documented gap; the unit tests use a
//! clean fixture as the control.

use std::collections::BTreeSet;

use crate::{Finding, Property, Severity};
use canary_observation::{FlowRun, Payment};

use super::common::{memo_plaintext, memo_structure, Dsu};

/// Clustering confidence threshold, in weight points (0.70 expressed exactly).
pub const CONFIDENCE_THRESHOLD: f64 = 0.70;
pub(crate) const THRESHOLD_POINTS: u32 = 70;

/// Pairwise signal weights, in points of confidence. Sum is capped at 100.
pub const W_SHARED_TRANSACTION_POINTS: u32 = 100;
pub const W_SHARED_SENDER_RECEIVER_POINTS: u32 = 40;
pub const W_SENDER_TRACES_TO_FUNDING_POINTS: u32 = 35;
pub const W_EXACT_AMOUNT_POINTS: u32 = 25;
pub const W_MEMO_STRUCTURE_POINTS: u32 = 20;
pub const W_SAME_HEIGHT_POINTS: u32 = 15;
pub const W_ADJACENT_HEIGHT_POINTS: u32 = 10;

fn pts(p: u32) -> f64 {
    p as f64 / 100.0
}

pub struct PayerLinkability;

fn score_pair(a: &Payment, b: &Payment, funding_receivers: &BTreeSet<&str>) -> (u32, Vec<String>) {
    let mut points: u32 = 0;
    let mut signals: Vec<String> = Vec::new();

    // One transaction: not a heuristic. Two payments inside one txid were created together, which
    // is the disclosure a batch makes whether or not anyone infers it.
    if a.txid == b.txid {
        points += W_SHARED_TRANSACTION_POINTS;
        signals.push(format!(
            "shared_transaction({:.2}): txid={} carried both",
            pts(W_SHARED_TRANSACTION_POINTS),
            a.txid
        ));
    }

    // Shared sender receiver: a direct on-chain link between two payments.
    let shared: Vec<&String> = a
        .sender_receivers
        .iter()
        .filter(|r| !r.is_empty() && b.sender_receivers.contains(r))
        .collect();
    if !shared.is_empty() {
        points += W_SHARED_SENDER_RECEIVER_POINTS;
        signals.push(format!(
            "shared_sender_receiver({:.2}): {}",
            pts(W_SHARED_SENDER_RECEIVER_POINTS),
            shared
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // The payer's own funding transaction exposed: a sender receiver equal to a funding receiver.
    let traced: Vec<&String> = a
        .sender_receivers
        .iter()
        .chain(b.sender_receivers.iter())
        .filter(|r| funding_receivers.contains(r.as_str()))
        .collect();
    if !traced.is_empty() {
        points += W_SENDER_TRACES_TO_FUNDING_POINTS;
        signals.push(format!(
            "sender_receiver_traces_to_funding({:.2}): {}",
            pts(W_SENDER_TRACES_TO_FUNDING_POINTS),
            traced
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if a.amount_zat > 0 && a.amount_zat == b.amount_zat {
        points += W_EXACT_AMOUNT_POINTS;
        signals.push(format!(
            "exact_amount({:.2}): amount_zat={}",
            pts(W_EXACT_AMOUNT_POINTS),
            a.amount_zat
        ));
    }

    if let (Some(ma), Some(mb)) = (memo_plaintext(a), memo_plaintext(b)) {
        let sa = memo_structure(&ma);
        let sb = memo_structure(&mb);
        if !sa.is_empty() && sa == sb && (sa.contains('#') || sa.len() >= 3) {
            points += W_MEMO_STRUCTURE_POINTS;
            signals.push(format!(
                "memo_structure({:.2}): \"{}\"",
                pts(W_MEMO_STRUCTURE_POINTS),
                sa
            ));
        }
    }

    if let (Some(ha), Some(hb)) = (a.height, b.height) {
        if ha == hb {
            points += W_SAME_HEIGHT_POINTS;
            signals.push(format!(
                "same_height({:.2}): height={}",
                pts(W_SAME_HEIGHT_POINTS),
                ha
            ));
        } else if ha.abs_diff(hb) == 1 {
            points += W_ADJACENT_HEIGHT_POINTS;
            signals.push(format!(
                "adjacent_height({:.2}): heights {} and {}",
                pts(W_ADJACENT_HEIGHT_POINTS),
                ha,
                hb
            ));
        }
    }

    (points.min(100), signals)
}

impl Property for PayerLinkability {
    fn id(&self) -> &'static str {
        "payer-linkability"
    }

    fn severity(&self) -> Severity {
        Severity::Disqualifying
    }

    fn title(&self) -> &'static str {
        "payments are not clusterable by an observer"
    }

    fn check(&self, run: &FlowRun) -> Finding {
        if run.payments.len() < 2 {
            return Finding::skip(
                "payer-linkability",
                Severity::Disqualifying,
                self.title(),
                format!(
                    "only {} payment(s) recorded; clustering needs at least two payments",
                    run.payments.len()
                ),
            );
        }

        let funding_receivers: BTreeSet<&str> =
            run.funding.iter().map(|f| f.receiver.as_str()).collect();
        let n = run.payments.len();

        let mut dsu = Dsu::new(n);
        let mut pair_lines: Vec<(usize, usize, u32, Vec<String>)> = Vec::new();
        let mut best_pair: u32 = 0;

        for i in 0..n {
            for j in (i + 1)..n {
                let (points, signals) =
                    score_pair(&run.payments[i], &run.payments[j], &funding_receivers);
                if points > best_pair {
                    best_pair = points;
                }
                if points >= THRESHOLD_POINTS {
                    dsu.union(i, j);
                    pair_lines.push((i, j, points, signals));
                }
            }
        }

        // Collect clusters of size >= 2.
        let mut clusters: std::collections::BTreeMap<usize, Vec<usize>> =
            std::collections::BTreeMap::new();
        for i in 0..n {
            let root = dsu.find(i);
            clusters.entry(root).or_default().push(i);
        }
        let linked: Vec<Vec<usize>> = clusters.into_values().filter(|c| c.len() >= 2).collect();

        if !linked.is_empty() {
            let mut evidence: Vec<String> = Vec::new();
            for (ci, cluster) in linked.iter().enumerate() {
                let txids: Vec<String> = cluster
                    .iter()
                    .map(|&i| run.payments[i].txid.clone())
                    .collect();
                evidence.push(format!(
                    "cluster payer#{}: txids=[{}] ({} payments linked)",
                    ci + 1,
                    txids.join(", "),
                    cluster.len()
                ));
                for (i, j, points, signals) in &pair_lines {
                    if cluster.contains(i) && cluster.contains(j) {
                        evidence.push(format!(
                            "pair txid={} txid={} confidence={:.2} signals=[{}]",
                            run.payments[*i].txid,
                            run.payments[*j].txid,
                            pts(*points),
                            signals.join("; ")
                        ));
                    }
                }
            }
            evidence.push(format!(
                "threshold={:.2} weights: shared_transaction={:.2} shared_sender_receiver={:.2} sender_receiver_traces_to_funding={:.2} exact_amount={:.2} memo_structure={:.2} same_height={:.2} adjacent_height={:.2}",
                CONFIDENCE_THRESHOLD,
                pts(W_SHARED_TRANSACTION_POINTS),
                pts(W_SHARED_SENDER_RECEIVER_POINTS),
                pts(W_SENDER_TRACES_TO_FUNDING_POINTS),
                pts(W_EXACT_AMOUNT_POINTS),
                pts(W_MEMO_STRUCTURE_POINTS),
                pts(W_SAME_HEIGHT_POINTS),
                pts(W_ADJACENT_HEIGHT_POINTS),
            ));
            return Finding::fail(
                "payer-linkability",
                Severity::Disqualifying,
                self.title(),
                evidence,
            )
            .with_note(
                "heuristic except for `shared_transaction`, which is a fact rather than an \
                 inference: payments inside one txid were created together, which is exactly what \
                 batched settlement discloses (docs/THREAT-MODEL.md). Confidence = \
                 min(1.0, sum of fixed weights); pairs at or above the documented threshold 0.70 \
                 union into clusters. Timestamps are not recorded in the observation model, so \
                 heights stand in for time buckets; change outputs are not recorded, so observable \
                 sender receivers stand in for change behaviour.",
            );
        }

        Finding::pass("payer-linkability", Severity::Disqualifying, self.title()).with_note(
            format!(
                "no pair reached threshold={:.2}; highest pair confidence={:.2} across {} payments",
                CONFIDENCE_THRESHOLD,
                pts(best_pair),
                n
            ),
        )
    }
}
