//! `memo-content` (W) — the memo carries PII in cleartext, or anything that must not be permanent.
//!
//! Leak: memos are permanently encrypted to a recipient's key, but they are also read by anyone
//! who gets the viewing key — a support desk, an accountant, a subpoena, a future employee — and
//! Zcash's memos cannot be rotated or deleted. PII that must not be permanent belongs nowhere
//! near a memo.
//!
//! Detect: decode every payment's memo bytes (hex or pre-decoded text) and flag:
//!  * email-shaped tokens,
//!  * phone-shaped runs and government-id-shaped runs (`###-##-####`),
//!  * runs of [`LONG_DIGIT_RUN_LEN`] or more consecutive digits (account-number-shaped),
//!  * key-like labels ("email:", "ssn", "passport", …),
//!  * unstructured free text: [`FREE_TEXT_WORD_THRESHOLD`] or more purely alphabetic words.
//!
//! Evidence: the txid, the memo hex, the flagged span, and the flag kind.

use crate::{Finding, Property, Severity};
use zanary_observation::FlowRun;

use super::common::{contains_email, contains_phone_or_id, has_long_digit_run, memo_plaintext};

/// A memo with this many or more purely alphabetic words is treated as unstructured prose.
pub const FREE_TEXT_WORD_THRESHOLD: usize = 4;
/// Consecutive digits at or above this length are flagged as account-number-shaped.
pub const LONG_DIGIT_RUN_LEN: usize = 10;

const KEY_LIKE_LABELS: &[&str] = &[
    "name:",
    "full name",
    "email:",
    "phone:",
    "address:",
    "ssn",
    "passport",
    "date of birth",
    "dob",
    "tax id",
];

pub struct MemoContent;

impl Property for MemoContent {
    fn id(&self) -> &'static str {
        "memo-content"
    }

    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn title(&self) -> &'static str {
        "memos carry no PII or free text"
    }

    fn check(&self, run: &FlowRun) -> Finding {
        if run.payments.is_empty() {
            return Finding::skip(
                "memo-content",
                Severity::Warn,
                self.title(),
                "no payments recorded; no memo bytes to inspect",
            );
        }

        let mut evidence: Vec<String> = Vec::new();
        let mut inspected = 0usize;

        for p in &run.payments {
            let Some(text) = memo_plaintext(p) else {
                continue;
            };
            inspected += 1;
            let lower = text.to_lowercase();
            let hex_note = p
                .memo_hex
                .as_deref()
                .map(|h| {
                    let shown: String = h.chars().take(64).collect();
                    format!("memo_hex={}", shown)
                })
                .unwrap_or_else(|| "memo_hex=-".to_string());

            if let Some(span) = contains_email(&text) {
                evidence.push(format!(
                    "txid={} {} memo flag=email span=\"{}\" (PII in a permanent memo)",
                    p.txid, hex_note, span
                ));
            }
            if let Some((span, kind)) = contains_phone_or_id(&text) {
                evidence.push(format!(
                    "txid={} {} memo flag={} span=\"{}\" (identity-shaped value in a permanent memo)",
                    p.txid, hex_note, kind, span
                ));
            } else if let Some(run_digits) = has_long_digit_run(&text, LONG_DIGIT_RUN_LEN) {
                evidence.push(format!(
                    "txid={} {} memo flag=long-digit-run span=\"{}\" (account-number-shaped value in a permanent memo)",
                    p.txid, hex_note, run_digits
                ));
            }
            for label in KEY_LIKE_LABELS {
                if lower.contains(label) {
                    evidence.push(format!(
                        "txid={} {} memo flag=key-like span=\"{}\" (labelled identity field in a permanent memo)",
                        p.txid, hex_note, label
                    ));
                }
            }
            let words = text
                .split_whitespace()
                .filter(|w| w.len() >= 2 && w.chars().all(|c| c.is_ascii_alphabetic()))
                .count();
            if words >= FREE_TEXT_WORD_THRESHOLD {
                evidence.push(format!(
                    "txid={} {} memo flag=free-text words={} (unstructured prose not needed to make the payment's claim verifiable)",
                    p.txid, hex_note, words
                ));
            }
        }

        if !evidence.is_empty() {
            return Finding::fail("memo-content", Severity::Warn, self.title(), evidence)
                .with_note(
                    "memos cannot be rotated or deleted and are readable by anyone holding the \
                     viewing key. The invoice binding alone (see memo-absent) supports the \
                     payment's claim; PII and prose belong in a channel that can be erased.",
                );
        }

        if inspected == 0 {
            return Finding::skip(
                "memo-content",
                Severity::Warn,
                self.title(),
                "no memo bytes were recorded on any payment; there is nothing to inspect",
            );
        }

        Finding::pass("memo-content", Severity::Warn, self.title()).with_note(format!(
            "{} memo(s) decoded and scanned for emails, phone/gov-id shapes, long digit runs, \
             key-like labels and free text; none flagged",
            inspected
        ))
    }
}
