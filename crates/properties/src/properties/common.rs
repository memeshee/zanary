//! Shared, dependency-free helpers for the leak-class checks.
//!
//! Everything here is a pure string/byte utility: address-shape classification, memo decoding,
//! small pattern scanners. No clock, no randomness, no I/O. The checks are deliberately built on
//! *observations* (address strings, memo bytes, heights) and never on the flow's own labels.

use zanary_observation::{Payment, Pool};

/// Zcash transparent address prefixes: mainnet `t1`/`t3`, testnet `t2`, regtest `tm`.
const TRANSPARENT_PREFIXES: &[&str] = &["t1", "t2", "t3", "tm"];
/// Shielded address prefixes: legacy Sapling `zs`/`zc`, unified `u1`/`utest`/`uregtest`.
const SHIELDED_PREFIXES: &[&str] = &["zs", "zc", "u1", "utest", "uregtest"];

/// A transparent address: known prefix + alphanumeric body + plausible length.
/// This is a documented heuristic — the observation model stores addresses as plain strings and
/// does not tag their type, so classification is by prefix and shape.
pub(crate) fn is_transparent_address(s: &str) -> bool {
    let s = s.trim();
    (26..=64).contains(&s.len())
        && TRANSPARENT_PREFIXES.iter().any(|p| s.starts_with(p))
        && s.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Any Zcash address shape (transparent or shielded). Used to find addresses inside free text
/// such as receipt bodies, where they are not structurally tagged.
pub(crate) fn looks_like_address(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() || s.chars().any(|c| !c.is_ascii_alphanumeric()) {
        return false;
    }
    if is_transparent_address(s) {
        return true;
    }
    (20..=250).contains(&s.len()) && SHIELDED_PREFIXES.iter().any(|p| s.starts_with(p))
}

pub(crate) fn pool_str(p: Pool) -> &'static str {
    match p {
        Pool::Transparent => "transparent",
        Pool::Sapling => "sapling",
        Pool::Orchard => "orchard",
        Pool::CrossPool => "cross_pool",
        Pool::Unknown => "unknown",
    }
}

pub(crate) fn height_str(h: Option<u32>) -> String {
    h.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string())
}

pub(crate) fn decode_hex(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if s.is_empty() || !s.len().is_multiple_of(2) {
        return None;
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks(2) {
        let hi = (pair[0] as char).to_digit(16)?;
        let lo = (pair[1] as char).to_digit(16)?;
        out.push(((hi << 4) | lo) as u8);
    }
    Some(out)
}

/// Hex encoding of a string's bytes (test fixtures and evidence helpers).
#[cfg(test)]
pub(crate) fn hex_of(s: &str) -> String {
    s.bytes().map(|b| format!("{b:02x}")).collect()
}

/// The memo's plaintext, preferring the decoded `memo_text` and falling back to decoding
/// `memo_hex` (Zcash memos are NUL-padded; trailing NULs and whitespace are stripped).
pub(crate) fn memo_plaintext(p: &Payment) -> Option<String> {
    if let Some(t) = &p.memo_text {
        let t = t.trim_end_matches('\0').trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    let hex = p.memo_hex.as_deref()?;
    let bytes = decode_hex(hex)?;
    let text = String::from_utf8_lossy(&bytes);
    let text = text.trim_end_matches('\0').trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

/// Lowercased alphanumerics only — the normalization used for invoice↔memo binding checks.
pub(crate) fn normalize_alnum(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// The *structure* of a memo: lowercased with digit runs collapsed to `#`.
/// `INV-1001` and `INV-1002` share the structure `inv-#`; content is deliberately discarded.
pub(crate) fn memo_structure(s: &str) -> String {
    let mut out = String::new();
    let mut in_digits = false;
    for c in s.trim().to_lowercase().chars() {
        if c.is_ascii_digit() {
            if !in_digits {
                out.push('#');
                in_digits = true;
            }
        } else {
            in_digits = false;
            out.push(c);
        }
    }
    out.trim().to_string()
}

/// Returns the first email-shaped token, if any.
pub(crate) fn contains_email(s: &str) -> Option<String> {
    for tok in s.split(|c: char| {
        c.is_whitespace()
            || matches!(
                c,
                ',' | ';' | '"' | '\'' | '<' | '>' | '(' | ')' | '[' | ']'
            )
    }) {
        let Some(at) = tok.find('@') else { continue };
        let (local, rest) = tok.split_at(at);
        let dom = &rest[1..];
        if !local.is_empty() && dom.contains('.') && !dom.ends_with('.') && dom.len() >= 3 {
            return Some(tok.to_string());
        }
    }
    None
}

/// Phone- or government-id-shaped runs. Boundaries: a run starts at `+`, `(`, or a digit that is
/// not glued to a preceding alphanumeric (so `INV-1001` is not a phone number, `555-123-4567` is).
/// Returns `(span, kind)` with kind `"gov-id"` or `"phone"`.
pub(crate) fn contains_phone_or_id(s: &str) -> Option<(String, &'static str)> {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let starts = c == '+' || c == '(' || c.is_ascii_digit();
        let prev_alnum = i > 0 && chars[i - 1].is_ascii_alphanumeric();
        if !(starts && !prev_alnum) {
            i += 1;
            continue;
        }
        let mut j = i;
        let mut run = String::new();
        while j < chars.len()
            && (chars[j].is_ascii_digit()
                || chars[j] == '+'
                || chars[j] == '-'
                || chars[j] == '('
                || chars[j] == ')'
                || chars[j] == ' ')
        {
            run.push(chars[j]);
            j += 1;
        }
        let run = run.trim_end().to_string();
        let digits = run.chars().filter(|c| c.is_ascii_digit()).count();
        if digits >= 7 {
            if is_gov_id_shape(&run) {
                return Some((run, "gov-id"));
            }
            let has_sep = run.contains('-') || run.contains('(') || run.contains(')');
            if run.starts_with('+') || (has_sep && digits >= 9) || digits >= 10 {
                return Some((run, "phone"));
            }
        }
        i = j.max(i + 1);
    }
    None
}

/// `###-##-####` grouped exactly 3-2-4.
fn is_gov_id_shape(run: &str) -> bool {
    let groups: Vec<usize> = run
        .split('-')
        .map(|g| g.chars().filter(|c| c.is_ascii_digit()).count())
        .collect();
    groups == [3, 2, 4]
}

/// A run of `min` or more consecutive digits (account-number-shaped).
pub(crate) fn has_long_digit_run(s: &str, min: usize) -> Option<String> {
    let mut run = String::new();
    for c in s.chars().chain(std::iter::once(' ')) {
        if c.is_ascii_digit() {
            run.push(c);
        } else {
            if run.len() >= min {
                return Some(run);
            }
            run.clear();
        }
    }
    None
}

/// First character kept, rest masked — for values that must appear in evidence without being
/// re-published in the report.
pub(crate) fn redact(v: &str) -> String {
    if v.chars().count() <= 2 {
        "***".to_string()
    } else {
        format!("{}***", v.chars().next().unwrap())
    }
}

/// Keep the first two octets of an IPv4 address; everything else masked.
pub(crate) fn mask_ip(ip: &str) -> String {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() == 4 {
        format!("{}.{}.*.*", parts[0], parts[1])
    } else {
        "***".to_string()
    }
}

/// Minimal disjoint-set for clustering payments by pair confidence.
pub(crate) struct Dsu {
    parent: Vec<usize>,
}

impl Dsu {
    pub(crate) fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }

    pub(crate) fn find(&mut self, x: usize) -> usize {
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut cur = x;
        while self.parent[cur] != cur {
            let next = self.parent[cur];
            self.parent[cur] = root;
            cur = next;
        }
        root
    }

    pub(crate) fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.parent[ra] = rb;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_classification_is_shape_based() {
        assert!(is_transparent_address(
            "tmLeakyTransparentFundingWallet1111111111111111"
        ));
        assert!(is_transparent_address(
            "t1exampleTransparentAddressForTests000000000"
        ));
        assert!(!is_transparent_address("uregtest1shielded000000000000000"));
        assert!(!is_transparent_address("tmux")); // too short
        assert!(!is_transparent_address("tm has spaces 1111111111111111111"));
    }

    #[test]
    fn memo_decode_trims_nul_padding() {
        let hex = format!("{}{}", hex_of("INV-1001"), "00".repeat(8));
        let mut p = Payment {
            label: "x".into(),
            txid: "t".into(),
            height: None,
            amount_zat: 1,
            recipient_receivers: vec![],
            sender_receivers: vec![],
            memo_hex: Some(hex),
            memo_text: None,
            invoice_id: None,
            pool_in: Pool::Orchard,
            pool_out: Pool::Orchard,
            kind: zanary_observation::PaymentKind::Shielded,
            policy: None,
            observed: Default::default(),
        };
        assert_eq!(memo_plaintext(&p).as_deref(), Some("INV-1001"));
        p.memo_text = Some("INV-1002".into());
        assert_eq!(memo_plaintext(&p).as_deref(), Some("INV-1002"));
    }

    #[test]
    fn phone_and_id_scanners_do_not_fire_on_invoice_ids() {
        assert!(contains_phone_or_id("INV-1001").is_none());
        assert!(contains_phone_or_id("555-123-4567").is_some());
        assert_eq!(
            contains_phone_or_id("123-45-6789").map(|(_, k)| k),
            Some("gov-id")
        );
        assert!(contains_phone_or_id("call 555 123 4567 now").is_some());
    }

    #[test]
    fn structure_collapses_digits() {
        assert_eq!(memo_structure("INV-1001"), "inv-#");
        assert_eq!(memo_structure("INV-1002"), "inv-#");
        assert_ne!(memo_structure("ORDER-1002"), "inv-#");
    }

    #[test]
    fn email_scanner_finds_tokens() {
        assert_eq!(
            contains_email("ping alice@example.com now").as_deref(),
            Some("alice@example.com")
        );
        assert!(contains_email("no email here").is_none());
        assert!(contains_email("INV-1001").is_none());
    }
}
