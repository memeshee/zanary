//! `receiver-reuse` (D) — the same receiver is used for two distinct payments/invoices.
//!
//! Leak: two invoices paid to the same receiver are linkable by the merchant, by the payer's own
//! wallet provider, and by anyone the merchant shares data with — address reuse is the substrate
//! of every customer graph.
//!
//! Detect: group every recipient receiver observed across payments (via the shared model's
//! `receiver_reuse_groups`, which groups by receiver and keeps groups spanning more than one
//! distinct payment label). Any repeat across distinct payments/invoices is a finding, including
//! repeats the flow claims are "diversified". Exact-string repeats only: the model stores
//! addresses as opaque strings, so same-account diversified receivers (which share no substring
//! pattern that is decodable without the bech32 payload) are a documented gap.
//!
//! Evidence: the two payments' txids and the shared receiver.

use std::collections::BTreeSet;

use crate::{Finding, Property, Severity};
use zanary_observation::FlowRun;

pub struct ReceiverReuse;

impl Property for ReceiverReuse {
    fn id(&self) -> &'static str {
        "receiver-reuse"
    }

    fn severity(&self) -> Severity {
        Severity::Disqualifying
    }

    fn title(&self) -> &'static str {
        "no receiver reused across distinct payments"
    }

    fn check(&self, run: &FlowRun) -> Finding {
        if run.payments.len() < 2 {
            return Finding::skip(
                "receiver-reuse",
                Severity::Disqualifying,
                self.title(),
                format!(
                    "only {} payment(s) recorded; reuse needs at least two distinct payments to be \
                     observable (the spec calls for N>=5)",
                    run.payments.len()
                ),
            );
        }

        let groups = run.receiver_reuse_groups();
        let mut evidence: Vec<String> = Vec::new();
        for (receiver, payments) in &groups {
            // The helper keeps groups spanning >1 distinct label, but the underlying vec can hold
            // the same label twice; dedupe by label for the evidence line.
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            let mut distinct: Vec<(&str, &str)> = Vec::new();
            for p in payments {
                if seen.insert(p.label.as_str()) {
                    distinct.push((p.label.as_str(), p.txid.as_str()));
                }
            }
            let used_by: Vec<String> = distinct
                .iter()
                .map(|(label, txid)| format!("txid={txid} label=\"{label}\""))
                .collect();
            evidence.push(format!(
                "receiver={} shared by {} distinct payments: {}",
                receiver,
                distinct.len(),
                used_by.join(", ")
            ));
        }

        if !evidence.is_empty() {
            return Finding::fail(
                "receiver-reuse",
                Severity::Disqualifying,
                self.title(),
                evidence,
            )
            .with_note(
                "grouping is exact-string on recipient receivers; groups span distinct payment \
                 labels, so a retry of one invoice is not counted as two payments. Diversified \
                 unified addresses of one account are not detectable from address strings alone \
                 (documented model gap).",
            );
        }

        let mut finding = Finding::pass("receiver-reuse", Severity::Disqualifying, self.title());
        if run.payments.len() < 5 {
            finding = finding.with_note(format!(
                "no receiver repeats among {} payments; note the spec calls for N>=5 payments \
                 (two payers, two amounts) for a meaningful reuse window",
                run.payments.len()
            ));
        }
        finding
    }
}
