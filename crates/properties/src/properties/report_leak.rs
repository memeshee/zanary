//! `report-leak` (N) — an exported receipt/report reveals more than the claim it supports.
//!
//! Leak: the receipt or report a flow emits to *prove* something (payment made, invoice settled,
//! totals for an auditor) can leak more than the claim: payee identity, other payments, wallet
//! history, or a viewing key that unlocks everything.
//!
//! Detect: inspect every artifact the flow exports —
//!  * any non-empty `discloses` list is excess by construction (it is what else the artifact
//!    carries beyond its claims);
//!  * key-material tokens in the body (IVK/OVK/UFVK/FVK/viewing key/spending key/seed/mnemonic);
//!  * address-shaped tokens in the body that appear nowhere in the run's records (possible payee
//!    or third-party address);
//!  * email-shaped values in the body.
//!
//! Evidence: the artifact, the claim it supports, and the excess it carries.

use std::collections::BTreeSet;

use crate::{Finding, Property, Severity};
use zanary_observation::FlowRun;

use super::common::{contains_email, looks_like_address};

/// Key material that would unlock more than any single claim. Tokens of length >= 4 are matched
/// as substrings; shorter ones require a token boundary.
const KEY_MATERIAL_TOKENS: &[&str] = &[
    "ivk",
    "ovk",
    "ufvk",
    "fvk",
    "viewing key",
    "viewingkey",
    "incoming viewing key",
    "outgoing viewing key",
    "spending key",
    "spendingkey",
    "seed phrase",
    "mnemonic",
];

fn key_material_hit(body: &str) -> Option<String> {
    let lower = body.to_lowercase();
    for tok in KEY_MATERIAL_TOKENS {
        if tok.contains(' ') || tok.len() >= 4 {
            if lower.contains(tok) {
                return Some((*tok).to_string());
            }
        } else {
            for t in lower.split(|c: char| !c.is_ascii_alphanumeric()) {
                if t == *tok || t.starts_with(tok) {
                    return Some((*tok).to_string());
                }
            }
        }
    }
    None
}

fn body_addresses(body: &str) -> Vec<String> {
    body.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| looks_like_address(t))
        .map(str::to_string)
        .collect()
}

pub struct ReportLeak;

impl Property for ReportLeak {
    fn id(&self) -> &'static str {
        "report-leak"
    }

    fn severity(&self) -> Severity {
        Severity::Note
    }

    fn title(&self) -> &'static str {
        "exported artifacts reveal no more than their claim"
    }

    fn check(&self, run: &FlowRun) -> Finding {
        if run.receipts.is_empty() {
            return Finding::skip(
                "report-leak",
                Severity::Note,
                self.title(),
                "no exported artifacts recorded; there is nothing to inspect",
            );
        }

        // Addresses the run already knows about: anything else in a receipt body is excess.
        let mut known: BTreeSet<&str> = BTreeSet::new();
        for p in &run.payments {
            for r in &p.recipient_receivers {
                known.insert(r.as_str());
            }
            for r in &p.sender_receivers {
                known.insert(r.as_str());
            }
        }
        for f in &run.funding {
            known.insert(f.receiver.as_str());
        }

        let mut evidence: Vec<String> = Vec::new();

        for r in &run.receipts {
            let claim = if r.claims.is_empty() {
                "no stated claim".to_string()
            } else {
                format!("claim=\"{}\"", r.claims.join("; "))
            };
            for d in &r.discloses {
                evidence.push(format!(
                    "receipt=\"{}\" {} discloses=\"{}\" (artifact carries information beyond its claim)",
                    r.label, claim, d
                ));
            }
            if let Some(tok) = key_material_hit(&r.body) {
                evidence.push(format!(
                    "receipt=\"{}\" {} body contains key-material token \"{}\" (viewing/spending key material would unlock everything)",
                    r.label, claim, tok
                ));
            }
            for addr in body_addresses(&r.body) {
                if !known.contains(addr.as_str()) {
                    evidence.push(format!(
                        "receipt=\"{}\" {} body address={} not recorded anywhere in this run (possible payee/third-party address)",
                        r.label, claim, addr
                    ));
                }
            }
            if let Some(e) = contains_email(&r.body) {
                evidence.push(format!(
                    "receipt=\"{}\" {} body carries email-shaped value \"{}\" (PII beyond the claim)",
                    r.label, claim, e
                ));
            }
        }

        if !evidence.is_empty() {
            return Finding::fail("report-leak", Severity::Note, self.title(), evidence).with_note(
                "an exported artifact should be verifiable for its stated claim and useless for \
                 anything else: no IVK/OVK, no address list beyond the claim, no amounts or PII \
                 not required. Verifiability of the body itself is out of scope here — this check \
                 flags excess disclosure.",
            );
        }

        Finding::pass("report-leak", Severity::Note, self.title()).with_note(format!(
            "{} exported artifact(s) checked for excess disclosures, key material, unknown \
             addresses and PII; none found",
            run.receipts.len()
        ))
    }
}
