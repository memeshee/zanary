//! `policy-downgrade` (D) — the wallet is instructed to relax its privacy policy
//! (`AllowRevealedAmounts`, weaker), or a node performs an implicit downgrade on the flow's behalf.
//!
//! Leak: a wallet told to reveal amounts reveals amounts — the UI can still say "shielded".
//!
//! Detect: two signals —
//!  1. explicit: every privacy-policy argument recorded on a payment (`Payment.policy`, verbatim)
//!     is tokenized and normalized; a token matching a known downgrade fragment (with negation
//!     prefixes excluded) is a finding;
//!  2. implicit: a payment that spends shielded value across a pool boundary
//!     (`pool_in` shielded, `pool_out` transparent or a mixed bundle) reveals amounts; zcashd
//!     refuses such a spend by default with "would reveal transaction amounts", so the flow either
//!     fixed the flow or turned the guard off.
//!
//! Evidence: the call/argument (or the pool legs), and the resulting txid.

use crate::{Finding, Property, Severity};
use canary_observation::{FlowRun, Pool};

use super::common::{height_str, normalize_alnum, pool_str};

/// Downgrade fragments, matched against the normalized (lowercased, alphanumerics-only) token.
/// Covers the zcash_client_backend `PrivacyPolicy` variants and their equivalents.
pub const DOWNGRADE_FRAGMENTS: &[&str] = &[
    "allowrevealedamounts",
    "allowrevealedsenders",
    "allowrevealedrecipients",
    "allowlinkingaccountaddresses",
    "allowfullytransparent",
    "allowtransparent",
    "revealedamounts",
    "revealedsenders",
    "revealedrecipients",
    "linkingaccountaddresses",
    "fullytransparent",
];

/// Tokens beginning with these are negated instructions ("NoAllowRevealedAmounts") and are not
/// counted as downgrades.
const NEGATION_PREFIXES: &[&str] = &["no", "not", "disallow", "deny", "without", "block", "never"];

fn token_is_downgrade(tok_norm: &str) -> bool {
    DOWNGRADE_FRAGMENTS.iter().any(|f| tok_norm.contains(f))
        && !NEGATION_PREFIXES.iter().any(|p| tok_norm.starts_with(p))
}

pub struct PolicyDowngrade;

impl Property for PolicyDowngrade {
    fn id(&self) -> &'static str {
        "policy-downgrade"
    }

    fn severity(&self) -> Severity {
        Severity::Disqualifying
    }

    fn title(&self) -> &'static str {
        "no privacy-policy downgrade instructed or implied"
    }

    fn check(&self, run: &FlowRun) -> Finding {
        if run.payments.is_empty() {
            return Finding::skip(
                "policy-downgrade",
                Severity::Disqualifying,
                self.title(),
                "no payments recorded; no wallet-layer calls to inspect",
            );
        }

        let mut evidence: Vec<String> = Vec::new();
        let mut any_captured = false;

        for p in &run.payments {
            if let Some(policy) = &p.policy {
                any_captured = true;
                for tok in policy.split(|c: char| matches!(c, '|' | ',' | ';' | '+' | ' ')) {
                    let norm = normalize_alnum(tok);
                    if token_is_downgrade(&norm) {
                        evidence.push(format!(
                            "txid={} policy=\"{}\" matched={} (wallet instructed to relax its privacy policy)",
                            p.txid,
                            policy,
                            tok.trim()
                        ));
                    }
                }
            }

            // Implicit downgrade: shielded value crossing a pool boundary reveals amounts.
            let shielded_in = matches!(p.pool_in, Pool::Sapling | Pool::Orchard);
            let revealing_out = matches!(p.pool_out, Pool::Transparent | Pool::CrossPool);
            if shielded_in && revealing_out {
                evidence.push(format!(
                    "txid={} height={} pool_in={} pool_out={} (cross-pool spend of shielded value reveals amounts — implicit downgrade)",
                    p.txid,
                    height_str(p.height),
                    pool_str(p.pool_in),
                    pool_str(p.pool_out)
                ));
                let policy_clean = p.policy.as_deref().map(|pol| {
                    !pol.split(|c: char| matches!(c, '|' | ',' | ';' | '+' | ' '))
                        .any(|tok| token_is_downgrade(&normalize_alnum(tok)))
                });
                if policy_clean == Some(true) {
                    evidence.push(format!(
                        "txid={} recorded policy=\"{}\" contradicts the observed reveal",
                        p.txid,
                        p.policy.as_deref().unwrap_or("")
                    ));
                }
            }
        }

        if !evidence.is_empty() {
            return Finding::fail(
                "policy-downgrade",
                Severity::Disqualifying,
                self.title(),
                evidence,
            )
            .with_note(
                "explicit fragments checked: AllowRevealedAmounts, AllowRevealedSenders, \
                 AllowRevealedRecipients, AllowLinkingAccountAddresses, AllowFullyTransparent and \
                 equivalents; negation prefixes (no/not/disallow/deny/without/block/never) are \
                 excluded. Implicit: shielded→transparent or shielded→mixed-bundle spends reveal \
                 amounts; zcashd refuses those by default with \"…would reveal transaction \
                 amounts. THIS MAY AFFECT YOUR PRIVACY…\".",
            );
        }

        if !any_captured {
            return Finding::skip(
                "policy-downgrade",
                Severity::Disqualifying,
                self.title(),
                "no privacy-policy argument was captured for any payment and no cross-pool reveal \
                 was observed; the observation set cannot tell whether a downgrade was requested \
                 (record Payment.policy verbatim, e.g. \"FullPrivacy\", to make this decidable)",
            );
        }

        Finding::pass(
            "policy-downgrade",
            Severity::Disqualifying,
            self.title(),
        )
        .with_note("every recorded policy token was checked against the downgrade fragments; none matched, and no shielded→transparent/cross-pool spend was observed")
    }
}
