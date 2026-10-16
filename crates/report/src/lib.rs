//! ZANARY report — turning a run into evidence someone else can check, and a claim that expires.
//!
//! A report is:
//!   * the findings from [`zanary_properties`] for one flow revision,
//!   * plus an evidence array (txids, receivers, memo bytes, heights) that a verifier re-checks
//!     against the chain without trusting whoever produced the report,
//!   * plus a canonical hash so the run can be anchored in a shielded memo and referenced later,
//!   * plus the wall-clock `verified_at` and `ttl_seconds` that turn it into an *expiring claim*.
//!
//! Nothing here may depend on the reporter's honesty: the verifier's job is to re-derive the
//! facts (`zanary-report verify`), and the index's job is to show which claims are still current
//! (`zanary-report index`).

use anyhow::Result;
use zanary_observation::FlowRun;
use zanary_properties::Finding;
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub mod anchor;
pub mod index;
pub mod matrix;
pub mod node;
pub mod receipt;
pub mod verify;

pub const REPORT_VERSION: u32 = 1;

/// Default claim lifetime: 30 days (docs/CLAIMS.md).
pub const DEFAULT_TTL_SECONDS: u64 = 30 * 24 * 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub version: u32,
    pub flow_id: String,
    pub flow_commit: String,
    pub chain: String,
    /// Chain height the run was made against: the clock a claim's TTL starts from.
    pub verified_at_height: u32,
    /// RFC3339 UTC wall-clock time the run finished. The claim's TTL clock starts here; a
    /// report without it is still checkable evidence, but it cannot expire (and the index
    /// says so rather than guessing a date).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<String>,
    /// Claim lifetime in seconds. `None` means [`DEFAULT_TTL_SECONDS`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u64>,
    pub findings: Vec<Finding>,
    /// Facts a verifier re-checks against the chain.
    pub evidence: Vec<Evidence>,
    /// True when the merchant side was simulated by the rig (stated, never implied).
    pub merchant_is_simulated: bool,
    /// sha256 over the canonical JSON of everything above.
    pub report_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Evidence {
    Transaction {
        txid: String,
        height: Option<u32>,
        /// When this payment was one of several settled in a single transaction: how many payments
        /// that transaction carried. The verifier re-checks it against the chain — a batch claim is
        /// only worth making if the transaction's own Orchard outputs back it up. Skipped when
        /// absent, so reports written before batching existed still hash and verify unchanged.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        batch_size: Option<u32>,
    },
    Receiver {
        address: String,
        first_seen_payment: String,
        /// The txid of the payment this receiver was first seen in — the binding a verifier
        /// needs to check "this receiver belongs to that transaction". Absent in reports
        /// written before this field existed, which the verifier reports rather than guesses.
        #[serde(default)]
        first_seen_txid: String,
    },
    Memo {
        txid: String,
        memo_hex: String,
    },
    PoolTotals {
        height: u32,
        transparent_zat: u64,
        sapling_zat: u64,
        orchard_zat: u64,
    },
    MerchantField {
        key: String,
        value: String,
        path: String,
    },
}

impl Evidence {
    /// Short label for one-line reports.
    pub fn label(&self) -> String {
        match self {
            Evidence::Transaction { txid, height, .. } => match height {
                Some(h) => format!("tx {txid}@{h}"),
                None => format!("tx {txid}"),
            },
            Evidence::Receiver { address, first_seen_payment, .. } => {
                format!("receiver {address} ({first_seen_payment})")
            }
            Evidence::Memo { txid, .. } => format!("memo {txid}"),
            Evidence::PoolTotals { height, .. } => format!("pool totals @{height}"),
            Evidence::MerchantField { key, path, .. } => format!("merchant field {path}:{key}"),
        }
    }
}

/// Build a report from a recorded run and its findings (no claim clock attached).
pub fn build(run: &FlowRun, findings: Vec<Finding>) -> Result<Report> {
    build_with_claim(run, findings, None, None)
}

/// Build a report, attaching the claim clock (`verified_at`, `ttl_seconds`).
pub fn build_with_claim(
    run: &FlowRun,
    findings: Vec<Finding>,
    verified_at: Option<String>,
    ttl_seconds: Option<u64>,
) -> Result<Report> {
    if let Some(ts) = &verified_at {
        // Fail fast on an unparseable clock: a claim whose date cannot be read cannot expire.
        OffsetDateTime::parse(ts, &Rfc3339)
            .map_err(|e| anyhow::anyhow!("verified_at is not RFC3339 UTC: {e}"))?;
    }
    let mut report = Report {
        version: REPORT_VERSION,
        flow_id: run.flow_id.clone(),
        flow_commit: run.flow_commit.clone(),
        chain: run.chain.clone(),
        verified_at_height: run.ended_at_height,
        verified_at,
        ttl_seconds,
        findings,
        evidence: collect_evidence(run),
        merchant_is_simulated: run.merchant_is_simulated,
        report_hash: String::new(),
    };
    report.report_hash = hash_report(&report)?;
    Ok(report)
}

/// Canonical JSON: field order is the struct order, so a hash is reproducible across machines.
pub fn canonical_json(report: &Report) -> Result<String> {
    Ok(serde_json::to_string(report)?)
}

/// sha256 of the report with `report_hash` cleared.
pub fn hash_report(report: &Report) -> Result<String> {
    let mut cleared = report.clone();
    cleared.report_hash = String::new();
    Ok(sha256_hex(canonical_json(&cleared)?.as_bytes()))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    hex_encode(&h.finalize())
}

pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Merkle root over the evidence leaves — what gets anchored in a shielded memo.
pub fn evidence_root(evidence: &[Evidence]) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut leaves: Vec<[u8; 32]> = evidence
        .iter()
        .map(|e| {
            let mut h = Sha256::new();
            h.update(serde_json::to_vec(e).unwrap_or_default());
            h.finalize().into()
        })
        .collect();

    if leaves.is_empty() {
        return Ok(sha256_hex(b"zanary:empty-evidence"));
    }
    while leaves.len() > 1 {
        if leaves.len() % 2 == 1 {
            let last = *leaves.last().expect("non-empty");
            leaves.push(last);
        }
        leaves = leaves
            .chunks(2)
            .map(|pair| {
                let mut h = Sha256::new();
                h.update(pair[0]);
                h.update(pair[1]);
                let out: [u8; 32] = h.finalize().into();
                out
            })
            .collect();
    }
    Ok(hex_encode(&leaves[0]))
}

pub fn collect_evidence(run: &FlowRun) -> Vec<Evidence> {
    let mut out = Vec::new();
    for p in &run.payments {
        out.push(Evidence::Transaction {
            txid: p.txid.clone(),
            height: p.height,
            // A batch is recorded on the payment by the runner (`observed.batch_size`), and this is
            // where the claim leaves the run and enters the evidence a stranger re-checks.
            batch_size: p
                .observed
                .get("batch_size")
                .and_then(|size| size.parse::<u32>().ok()),
        });
        for r in &p.recipient_receivers {
            out.push(Evidence::Receiver {
                address: r.clone(),
                first_seen_payment: p.label.clone(),
                first_seen_txid: p.txid.clone(),
            });
        }
        if let Some(memo) = &p.memo_hex {
            out.push(Evidence::Memo {
                txid: p.txid.clone(),
                memo_hex: memo.clone(),
            });
        }
    }
    for t in &run.pool_totals {
        out.push(Evidence::PoolTotals {
            height: t.height,
            transparent_zat: t.transparent_zat,
            sapling_zat: t.sapling_zat,
            orchard_zat: t.orchard_zat,
        });
    }
    for e in &run.merchant_events {
        for (k, v) in &e.fields {
            out.push(Evidence::MerchantField {
                key: k.clone(),
                value: v.clone(),
                path: e.path.clone(),
            });
        }
    }
    out
}

/// The claim's lifetime in seconds (the report's TTL, or the 30-day default).
pub fn claim_ttl_seconds(report: &Report) -> u64 {
    report.ttl_seconds.unwrap_or(DEFAULT_TTL_SECONDS)
}

/// When the claim expires, if it carries a clock.
pub fn expires_at(report: &Report) -> Option<OffsetDateTime> {
    let verified_at = OffsetDateTime::parse(report.verified_at.as_deref()?, &Rfc3339).ok()?;
    verified_at.checked_add(time::Duration::seconds(
        i64::try_from(claim_ttl_seconds(report)).unwrap_or(i64::MAX),
    ))
}

/// [`expires_at`] as RFC3339 — for tools that print a claim's clock (the CI gate does, so the
/// date a build stands behind is on the build's own log line). `None` when the report carries no
/// wall clock, which is also the reason it cannot expire.
pub fn expires_at_rfc3339(report: &Report) -> Option<String> {
    expires_at(report)?.format(&Rfc3339).ok()
}

/// A claim's status against the wall clock, per docs/CLAIMS.md:
/// `live` until `expires_at`, then `stale` (one missed re-probe) for one more TTL, then `dead`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimStatus {
    Live,
    Stale,
    Dead,
}

impl ClaimStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ClaimStatus::Live => "live",
            ClaimStatus::Stale => "stale",
            ClaimStatus::Dead => "dead",
        }
    }
}

/// `None` when the report has no parseable `verified_at` — an undated claim cannot expire.
pub fn claim_status(report: &Report, now: OffsetDateTime) -> Option<ClaimStatus> {
    let expires = expires_at(report)?;
    if now < expires {
        Some(ClaimStatus::Live)
    } else {
        let ttl = time::Duration::seconds(i64::try_from(claim_ttl_seconds(report)).unwrap_or(i64::MAX));
        if now < expires + ttl {
            Some(ClaimStatus::Stale)
        } else {
            Some(ClaimStatus::Dead)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_root_is_deterministic_and_order_sensitive() {
        let a = vec![
            Evidence::Transaction { txid: "aa".into(), height: Some(1), batch_size: None },
            Evidence::Transaction { txid: "bb".into(), height: Some(2), batch_size: None },
        ];
        let b = vec![
            Evidence::Transaction { txid: "bb".into(), height: Some(2), batch_size: None },
            Evidence::Transaction { txid: "aa".into(), height: Some(1), batch_size: None },
        ];
        assert_eq!(evidence_root(&a).unwrap(), evidence_root(&a).unwrap());
        assert_ne!(evidence_root(&a).unwrap(), evidence_root(&b).unwrap());
    }

    #[test]
    fn claim_status_follows_ttl_arithmetic() {
        let mut report = Report {
            version: REPORT_VERSION,
            flow_id: "f".into(),
            flow_commit: "c".into(),
            chain: "regtest".into(),
            verified_at_height: 10,
            verified_at: Some("2026-01-01T00:00:00Z".into()),
            ttl_seconds: Some(86_400),
            findings: vec![],
            evidence: vec![],
            merchant_is_simulated: true,
            report_hash: String::new(),
        };
        let t = |s: &str| OffsetDateTime::parse(s, &Rfc3339).unwrap();
        assert_eq!(claim_status(&report, t("2026-01-01T12:00:00Z")), Some(ClaimStatus::Live));
        assert_eq!(claim_status(&report, t("2026-01-02T00:00:00Z")), Some(ClaimStatus::Stale));
        assert_eq!(claim_status(&report, t("2026-01-03T00:00:00Z")), Some(ClaimStatus::Dead));
        // No clock → no status.
        report.verified_at = None;
        assert_eq!(claim_status(&report, t("2026-01-01T12:00:00Z")), None);
    }
}
