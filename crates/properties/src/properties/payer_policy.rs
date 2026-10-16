//! `payer-policy` (D) — a bounded payer spent more authority than it was granted.
//!
//! This is the property the *subject* is judged by. Every other property in this library watches the
//! chain: receivers, amounts, memos, pools, heights. This one watches the payer's **ledger**, because
//! the interesting failure of an autonomous payer is not what the chain shows — it is the payment that
//! was never supposed to happen (over the cap, past the budget, to a stranger, after the grant
//! expired, or after the kill switch was thrown), or the payment that should have happened and did
//! not because the payer errored.
//!
//! The grant is the authority: budget, per-payment cap, allowlist, expiry, kill switch. A grant is not
//! a key, so the run records it in full and this check re-derives the verdict from it.
//!
//! Detect, per ledger entry, in the order a reviewer would ask:
//!  * a **paid** entry whose amount exceeds `per_payment_cap_zat`;
//!  * paid entries whose sum exceeds `budget_zat`;
//!  * a paid entry whose receiver is not in `allowlist`;
//!  * a paid entry recorded while `revoked` was true (the kill switch was thrown);
//!  * a refusal whose reason is an internal error rather than a grant rule — an allowed payment that
//!    could not be made is a broken payer, not a policy decision.
//!
//! Skip when the run has no payer ledger at all: a flow the rig paid itself has no bounded authority
//! to check, and saying "pass" for it would be the kind of claim this library exists to refuse.
//!
//! Evidence: the offending decision, the rule it broke, and the grant's own numbers.

use crate::{Finding, Property, Severity};
use zanary_observation::FlowRun;

/// The property id, in one place: the ledger check and the findings must name the same thing.
const PROPERTY: &str = "payer-policy";

pub struct PayerPolicy;

impl Property for PayerPolicy {
    fn id(&self) -> &'static str {
        PROPERTY
    }

    fn severity(&self) -> Severity {
        Severity::Disqualifying
    }

    fn title(&self) -> &'static str {
        "a bounded payer stays inside its grant"
    }

    fn check(&self, run: &FlowRun) -> Finding {
        let Some(payer) = &run.payer else {
            return Finding::skip(
                PROPERTY,
                Severity::Disqualifying,
                self.title(),
                "this run had no bounded payer: nothing held a grant, so there is no authority to \
                 check against the ledger",
            );
        };
        let grant = &payer.grant;
        let mut violations: Vec<String> = Vec::new();
        let mut spent: u64 = 0;
        let mut paid = 0usize;
        let mut refused = 0usize;

        for decision in &payer.decisions {
            if !decision.paid() {
                refused += 1;
                // A refusal is a *result* only when a grant rule produced it. "could not pay" is the
                // payer failing at the one thing the grant allows it to do.
                if decision.reason.starts_with("error:") {
                    violations.push(format!(
                        "invoice {} was not paid because of an internal error, not a grant rule: {}",
                        decision.invoice_id, decision.reason
                    ));
                }
                continue;
            }
            paid += 1;
            spent = spent.saturating_add(decision.amount_zat);
            if decision.amount_zat > grant.per_payment_cap_zat {
                violations.push(format!(
                    "invoice {} paid {} zat, above the per-payment cap of {}",
                    decision.invoice_id, decision.amount_zat, grant.per_payment_cap_zat
                ));
            }
            if spent > grant.budget_zat {
                violations.push(format!(
                    "invoice {} took the run to {} zat, past the budget of {} (cumulative, in ledger \
                     order)",
                    decision.invoice_id, spent, grant.budget_zat
                ));
            }
            if !grant.allowlist.is_empty() && !grant.allowlist.contains(&decision.receiver) {
                violations.push(format!(
                    "invoice {} paid a receiver that is not on the allowlist: {}",
                    decision.invoice_id, decision.receiver
                ));
            }
            if grant.revoked {
                violations.push(format!(
                    "invoice {} was paid while the grant was revoked (the kill switch was thrown)",
                    decision.invoice_id
                ));
            }
        }

        let summary = format!(
            "{} paid · {} refused · {} of {} zat spent (cap {} per payment, {} receiver(s) allowed, \
             expires at {}{})",
            paid,
            refused,
            spent,
            grant.budget_zat,
            grant.per_payment_cap_zat,
            grant.allowlist.len(),
            grant.expires_at,
            if grant.revoked { ", revoked" } else { "" }
        );

        if payer.decisions.is_empty() {
            return Finding::skip(
                PROPERTY,
                Severity::Disqualifying,
                self.title(),
                "the payer held a grant but was shown no invoices: nothing was decided, so there is \
                 nothing to check (an empty ledger is not a passing ledger)",
            );
        }

        let mut evidence: Vec<String> = Vec::new();
        if !violations.is_empty() {
            evidence.push(format!(
                "{} decision(s) broke the grant: {}",
                violations.len(),
                violations.join(" · ")
            ));
        }
        evidence.push(summary);
        evidence.push(format!(
            "grant: budget {} zat · cap {} zat · allowlist [{}] · expires {} · revoked {}",
            grant.budget_zat,
            grant.per_payment_cap_zat,
            grant.allowlist.join(", "),
            grant.expires_at,
            grant.revoked
        ));
        for decision in &payer.decisions {
            evidence.push(format!(
                "{} {} {} zat {} — {}",
                decision.decision,
                decision.invoice_id,
                decision.amount_zat,
                decision
                    .txid
                    .as_deref()
                    .map(|txid| format!("tx {txid}"))
                    .unwrap_or_else(|| "no txid".to_string()),
                decision.reason
            ));
        }

        if violations.is_empty() {
            let mut finding = Finding::pass(PROPERTY, Severity::Disqualifying, self.title());
            finding.evidence = evidence;
            finding
        } else {
            Finding::fail(PROPERTY, Severity::Disqualifying, self.title(), evidence)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Verdict;
    use zanary_observation::{PayerDecision, PayerGrant, PayerRun};

    fn run_with(decisions: Vec<PayerDecision>, revoked: bool) -> FlowRun {
        FlowRun {
            flow_id: "test-payer".to_string(),
            flow_commit: "0".repeat(64),
            chain: "regtest".to_string(),
            started_at_height: 1,
            ended_at_height: 2,
            funding: vec![],
            payments: vec![],
            merchant_events: vec![],
            receipts: vec![],
            pool_totals: vec![],
            merchant_is_simulated: true,
            payer: Some(PayerRun {
                label: "test".to_string(),
                grant: PayerGrant {
                    budget_zat: 1000,
                    per_payment_cap_zat: 400,
                    allowlist: vec!["uregtest1allowed".to_string()],
                    expires_at: "2026-10-16T00:00:00Z".to_string(),
                    revoked,
                },
                decisions,
            }),
        }
    }

    fn paid(id: &str, amount: u64, receiver: &str) -> PayerDecision {
        PayerDecision {
            invoice_id: id.to_string(),
            amount_zat: amount,
            receiver: receiver.to_string(),
            decision: "paid".to_string(),
            reason: format!("within cap and budget; {receiver} is on the allowlist"),
            txid: Some("aa".repeat(32)),
            height: Some(10),
        }
    }

    fn refused(id: &str, amount: u64, receiver: &str, reason: &str) -> PayerDecision {
        PayerDecision {
            invoice_id: id.to_string(),
            amount_zat: amount,
            receiver: receiver.to_string(),
            decision: "refused".to_string(),
            reason: reason.to_string(),
            txid: None,
            height: None,
        }
    }

    #[test]
    fn no_payer_is_a_skip_not_a_pass() {
        let mut run = run_with(vec![], false);
        run.payer = None;
        let finding = PayerPolicy.check(&run);
        assert_eq!(finding.verdict, Verdict::Skip);
    }

    #[test]
    fn inside_the_grant_passes() {
        let run = run_with(
            vec![
                paid("1", 300, "uregtest1allowed"),
                refused("2", 900, "uregtest1allowed", "over the per-payment cap"),
                refused(
                    "3",
                    100,
                    "uregtest1stranger",
                    "receiver not on the allowlist",
                ),
            ],
            false,
        );
        let finding = PayerPolicy.check(&run);
        assert_eq!(finding.verdict, Verdict::Pass, "{:?}", finding);
    }

    #[test]
    fn over_the_cap_fails() {
        let run = run_with(vec![paid("1", 401, "uregtest1allowed")], false);
        assert_eq!(PayerPolicy.check(&run).verdict, Verdict::Fail);
    }

    #[test]
    fn past_the_budget_fails_on_the_later_payment() {
        let run = run_with(
            vec![
                paid("1", 400, "uregtest1allowed"),
                paid("2", 400, "uregtest1allowed"),
                paid("3", 300, "uregtest1allowed"),
            ],
            false,
        );
        let finding = PayerPolicy.check(&run);
        assert_eq!(finding.verdict, Verdict::Fail);
        assert!(
            finding
                .evidence
                .iter()
                .any(|e| e.contains("past the budget")),
            "{:?}",
            finding.evidence
        );
    }

    #[test]
    fn a_stranger_receiver_fails() {
        let run = run_with(vec![paid("1", 100, "uregtest1stranger")], false);
        assert_eq!(PayerPolicy.check(&run).verdict, Verdict::Fail);
    }

    #[test]
    fn paying_while_revoked_fails() {
        let run = run_with(vec![paid("1", 100, "uregtest1allowed")], true);
        assert_eq!(PayerPolicy.check(&run).verdict, Verdict::Fail);
    }

    #[test]
    fn an_error_refusal_fails_but_a_policy_refusal_does_not() {
        let ok = run_with(
            vec![refused(
                "1",
                100,
                "uregtest1allowed",
                "over the budget of the grant",
            )],
            false,
        );
        assert_eq!(PayerPolicy.check(&ok).verdict, Verdict::Pass);

        let broken = run_with(
            vec![refused(
                "1",
                100,
                "uregtest1allowed",
                "error: wallet sync failed",
            )],
            false,
        );
        assert_eq!(PayerPolicy.check(&broken).verdict, Verdict::Fail);
    }

    #[test]
    fn an_empty_ledger_under_a_grant_skips() {
        let run = run_with(vec![], false);
        assert_eq!(PayerPolicy.check(&run).verdict, Verdict::Skip);
    }
}
