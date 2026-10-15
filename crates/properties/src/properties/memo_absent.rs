//! `memo-absent` (W) — the flow claims invoice reconciliation but binds no memo to the payment.
//!
//! Leak: not directly — but it breaks the only reconciliation channel a shielded payment has,
//! which in practice pushes the merchant back to transparent rails to reconcile.
//!
//! Detect: a flow is "reconciliation-shaped" when at least one payment carries an `invoice_id`.
//! For every such payment, assert (a) a memo exists (bytes or decoded text), and (b) a fresh
//! verifier can recompute the binding from the chain alone: the invoice id appears in the memo's
//! plaintext (normalized to lowercased alphanumerics, containment checked in either direction so
//! `4471` binds `INV-4471`).
//!
//! Evidence: txid, invoice id, and the memo bytes (or their absence).

use crate::{Finding, Property, Severity};
use zanary_observation::FlowRun;

use super::common::{memo_plaintext, normalize_alnum};

pub struct MemoAbsent;

impl Property for MemoAbsent {
    fn id(&self) -> &'static str {
        "memo-absent"
    }

    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn title(&self) -> &'static str {
        "invoice-bearing payments bind their invoice in a memo"
    }

    fn check(&self, run: &FlowRun) -> Finding {
        let reconciliation: Vec<&zanary_observation::Payment> = run
            .payments
            .iter()
            .filter(|p| {
                p.invoice_id
                    .as_deref()
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false)
            })
            .collect();

        if reconciliation.is_empty() {
            return Finding::skip(
                "memo-absent",
                Severity::Warn,
                self.title(),
                "no payment claims an invoice (invoice_id is unset everywhere); the flow is not \
                 reconciliation-shaped, so there is nothing to bind",
            );
        }

        let mut evidence: Vec<String> = Vec::new();
        for p in &reconciliation {
            let invoice = p.invoice_id.as_deref().unwrap_or("").trim();
            match memo_plaintext(p) {
                None => evidence.push(format!(
                    "txid={} invoice={} no memo bytes bound (a verifier cannot recompute the invoice→payment binding from the chain)",
                    p.txid, invoice
                )),
                Some(text) => {
                    let ni = normalize_alnum(invoice);
                    let nm = normalize_alnum(&text);
                    let bound = !nm.is_empty()
                        && (!ni.is_empty()
                            && (nm.contains(&ni) || (ni.contains(&nm) && nm.len() >= 2)));
                    if !bound {
                        evidence.push(format!(
                            "txid={} invoice={} memo=\"{}\" does not contain the invoice id (binding not recomputable)",
                            p.txid, invoice, text
                        ));
                    }
                }
            }
        }

        if !evidence.is_empty() {
            return Finding::fail("memo-absent", Severity::Warn, self.title(), evidence).with_note(
                "binding = alphanumeric-lowercase containment between invoice id and memo \
                     plaintext, either direction. Memos are the only reconciliation channel a \
                     shielded payment has; without the binding the merchant falls back to \
                     transparent rails.",
            );
        }

        Finding::pass("memo-absent", Severity::Warn, self.title()).with_note(format!(
            "{} invoice-bearing payments checked; every one carries a memo that contains its invoice id",
            reconciliation.len()
        ))
    }
}
