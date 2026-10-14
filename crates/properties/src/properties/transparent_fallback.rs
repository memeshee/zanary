//! `transparent-fallback` (D) — a transparent address is accepted, required, or produced anywhere
//! in the flow.
//!
//! Leak: with a transparent address, amounts, both parties and timing are public and permanent,
//! and the convenience path becomes the default for every user.
//!
//! Detect: scan every address the flow emits or accepts — payment recipient receivers (destinations
//! the flow chose), payment sender receivers (sources it exposed), funding receivers (addresses the
//! flow's wallet handed out or accepted value at), addresses recorded in the free-form `observed`
//! map, and payments whose observed `kind` is `Transparent`.
//!
//! Evidence: the address, the txid, and the side it appeared on.

use crate::{Finding, Property, Severity};
use canary_observation::{FlowRun, PaymentKind};

use super::common::{height_str, is_transparent_address};

pub struct TransparentFallback;

impl Property for TransparentFallback {
    fn id(&self) -> &'static str {
        "transparent-fallback"
    }

    fn severity(&self) -> Severity {
        Severity::Disqualifying
    }

    fn title(&self) -> &'static str {
        "no transparent address anywhere in the flow"
    }

    fn check(&self, run: &FlowRun) -> Finding {
        let mut evidence: Vec<String> = Vec::new();
        let mut observed_addresses = 0usize;

        for p in &run.payments {
            for r in &p.recipient_receivers {
                observed_addresses += 1;
                if is_transparent_address(r) {
                    evidence.push(format!(
                        "txid={} recipient={} (transparent address accepted as a payment destination)",
                        p.txid, r
                    ));
                }
            }
            for r in &p.sender_receivers {
                observed_addresses += 1;
                if is_transparent_address(r) {
                    evidence.push(format!(
                        "txid={} sender_receiver={} (transparent address observed on the funding side)",
                        p.txid, r
                    ));
                }
            }
            if p.kind == PaymentKind::Transparent {
                evidence.push(format!(
                    "txid={} kind=transparent (payment ran on transparent rails end to end)",
                    p.txid
                ));
            }
            for (k, v) in &p.observed {
                if is_transparent_address(v) {
                    observed_addresses += 1;
                    evidence.push(format!(
                        "txid={} observed[{}]={} (transparent address recorded during the payment)",
                        p.txid, k, v
                    ));
                }
            }
        }

        for f in &run.funding {
            observed_addresses += 1;
            if is_transparent_address(&f.receiver) {
                evidence.push(format!(
                    "funding txid={} height={} receiver={} (value entered the flow at a transparent address)",
                    f.txid,
                    height_str(f.height),
                    f.receiver
                ));
            }
        }

        if !evidence.is_empty() {
            return Finding::fail(
                "transparent-fallback",
                Severity::Disqualifying,
                self.title(),
                evidence,
            )
            .with_note(
                "any transparent address in the flow is disqualifying: amounts, both parties and \
                 timing become public and permanent. If the rig's own funding leg is a transparent \
                 coinbase/faucet deposit, the finding is about that leg — re-run with the shielded \
                 wallet receiver recorded in Funding.receiver to isolate the flow's own choices.",
            );
        }

        if observed_addresses == 0 {
            return Finding::skip(
                "transparent-fallback",
                Severity::Disqualifying,
                self.title(),
                "no addresses were recorded on either side of any payment or funding event; \
                 the observation set cannot decide whether a transparent address was used",
            );
        }

        Finding::pass(
            "transparent-fallback",
            Severity::Disqualifying,
            self.title(),
        )
        .with_note(format!(
            "checked {} observed addresses (payment recipients/senders, funding receivers, observed \
             values); none transparent",
            observed_addresses
        ))
    }
}
