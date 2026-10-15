//! `server-metadata` (W) — the flow hands a third party (facilitator, API, merchant) information
//! about the payer.
//!
//! Leak: a payment rail that learns who pays, from where, and for what has rebuilt the
//! card-network surveillance model on a privacy chain. The rig runs the merchant side itself and
//! logs exactly what it receives: the flow fails if the merchant can link two payments from the
//! same payer, or receives any identity field it does not need to serve the request.
//!
//! Detect:
//!  * identity-shaped keys in request fields and headers (token match: email, name, user,
//!    customer, payer, wallet, address, kyc, …; `user-agent` exempted as a browser artifact),
//!  * identity-shaped values (email / address / long digit run) under non-allow-listed keys,
//!  * linkage: one `remote_ip` on requests settling two or more distinct payments, or one
//!    identity-shaped (key, value) pair repeated across distinct payments.
//!
//! Evidence: the recorded requests (values redacted), and the field that made the link possible.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Finding, Property, Severity};
use zanary_observation::{FlowRun, MerchantEvent};

use super::common::{
    contains_email, has_long_digit_run, height_str, is_transparent_address, mask_ip, redact,
};

/// Key tokens that mark a field/header as identity information a settle request does not need.
const IDENTITY_KEY_TOKENS: &[&str] = &[
    "email",
    "phone",
    "name",
    "user",
    "username",
    "account",
    "payer",
    "customer",
    "wallet",
    "address",
    "addr",
    "kyc",
    "ssn",
    "passport",
    "dob",
    "birth",
    "tax",
    "device",
    "fingerprint",
    "cookie",
    "authorization",
    "auth",
    "ip",
    "remote",
    "location",
    "geo",
    "browser",
    "mac",
];

/// Field keys that are legitimately needed to serve a settlement request.
const ALLOWED_FIELD_KEYS: &[&str] = &[
    "invoice_id",
    "invoiceid",
    "invoice",
    "amount",
    "amount_zat",
    "method",
    "path",
    "currency",
    "asset",
    "memo",
    "memo_hex",
    "txid",
    "network",
    "chain",
    "version",
    "format",
    "content_type",
    "nonce",
    "timestamp",
    "request_id",
    "accept",
];

fn split_key_tokens(k: &str) -> Vec<String> {
    // Split on non-alphanumerics and camelCase boundaries ("userEmail" -> user, email).
    let mut spaced = String::new();
    let mut prev_lower = false;
    for c in k.chars() {
        if c.is_ascii_uppercase() && prev_lower {
            spaced.push('_');
        }
        spaced.push(c.to_ascii_lowercase());
        prev_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
    }
    spaced
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

fn identity_token(k: &str) -> Option<String> {
    split_key_tokens(k)
        .into_iter()
        .find(|t| IDENTITY_KEY_TOKENS.contains(&t.as_str()))
}

fn is_browser_ua_key(k: &str) -> bool {
    split_key_tokens(k) == ["user", "agent"]
}

fn is_allowed_field_key(k: &str) -> bool {
    ALLOWED_FIELD_KEYS.contains(&k.to_ascii_lowercase().as_str())
}

fn value_identity_shaped(v: &str) -> bool {
    contains_email(v).is_some() || is_transparent_address(v) || has_long_digit_run(v, 10).is_some()
}

/// The payment a merchant event can be tied to, for linkage counting.
fn payment_key(e: &MerchantEvent) -> Option<String> {
    if let Some(l) = &e.settled_payment_label {
        if !l.trim().is_empty() {
            return Some(l.clone());
        }
    }
    e.fields
        .get("invoice_id")
        .filter(|v| !v.trim().is_empty())
        .cloned()
}

pub struct ServerMetadata;

impl Property for ServerMetadata {
    fn id(&self) -> &'static str {
        "server-metadata"
    }

    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn title(&self) -> &'static str {
        "merchant side receives no payer identity or linkage"
    }

    fn check(&self, run: &FlowRun) -> Finding {
        if run.merchant_events.is_empty() {
            return Finding::skip(
                "server-metadata",
                Severity::Warn,
                self.title(),
                "no merchant-side requests recorded; there is nothing the third party is known to \
                 have received",
            );
        }

        let mut evidence: Vec<String> = Vec::new();

        for e in &run.merchant_events {
            let ctx = format!("path={} height={}", e.path, height_str(e.height));
            for (k, v) in &e.fields {
                if is_browser_ua_key(k) {
                    continue;
                }
                if let Some(tok) = identity_token(k) {
                    evidence.push(format!(
                        "field key=\"{}\" matched=\"{}\" value=\"{}\" {} (identity field received; value redacted)",
                        k, tok, redact(v), ctx
                    ));
                } else if !is_allowed_field_key(k) && value_identity_shaped(v) {
                    evidence.push(format!(
                        "field key=\"{}\" value=\"{}\" {} (value shaped like payer identity; redacted)",
                        k, redact(v), ctx
                    ));
                }
            }
            for (k, v) in &e.headers {
                if is_browser_ua_key(k) {
                    continue;
                }
                if let Some(tok) = identity_token(k) {
                    evidence.push(format!(
                        "header key=\"{}\" matched=\"{}\" value=\"{}\" {} (identity-bearing header received; value redacted)",
                        k, tok, redact(v), ctx
                    ));
                }
            }
        }

        // Linkage 1: one remote_ip settling two or more distinct payments.
        let mut by_ip: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
        for e in &run.merchant_events {
            if let Some(ip) = &e.remote_ip {
                if let Some(pk) = payment_key(e) {
                    by_ip.entry(ip.as_str()).or_default().insert(pk);
                }
            }
        }
        for (ip, pks) in &by_ip {
            if pks.len() >= 2 {
                evidence.push(format!(
                    "remote_ip={} seen on requests settling {} distinct payments [{}] (the merchant can link them)",
                    mask_ip(ip),
                    pks.len(),
                    pks.iter().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
        }

        // Linkage 2: one identity-shaped (key, value) pair repeated across distinct payments.
        let mut by_fv: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
        for e in &run.merchant_events {
            for (k, v) in &e.fields {
                if is_allowed_field_key(k) || is_browser_ua_key(k) || !value_identity_shaped(v) {
                    continue;
                }
                if let Some(pk) = payment_key(e) {
                    by_fv.entry((k.clone(), v.clone())).or_default().insert(pk);
                }
            }
        }
        for ((k, v), pks) in &by_fv {
            if pks.len() >= 2 {
                evidence.push(format!(
                    "field key={} value=\"{}\" repeated across {} distinct payments [{}] (payer link)",
                    k,
                    redact(v),
                    pks.len(),
                    pks.iter().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
        }

        if !evidence.is_empty() {
            return Finding::fail("server-metadata", Severity::Warn, self.title(), evidence)
                .with_note(
                    "the merchant side is the rig's own first-hand record. Identity keys are \
                     matched token-wise (email, name, user, customer, payer, wallet, address, kyc, \
                     …); user-agent is exempted as a browser artifact; values in evidence are \
                     redacted to their first character. Linkage = same remote_ip or same \
                     identity-shaped field value across >=2 distinct settled payments.",
                );
        }

        Finding::pass("server-metadata", Severity::Warn, self.title()).with_note(format!(
            "{} merchant-side request(s) checked for identity fields, identity-shaped values, and \
             cross-payment linkage; none found",
            run.merchant_events.len()
        ))
    }
}
