//! CANARY report — turning a run into evidence someone else can check, and a claim that expires.
//!
//! A report is:
//!   * the findings from [`canary_properties`] for one flow revision,
//!   * plus an evidence array (txids, receivers, memo bytes, heights) that a verifier re-checks
//!     against the chain without trusting whoever produced the report,
//!   * plus a canonical hash so the run can be anchored in a shielded memo and referenced later.
//!
//! Nothing here may depend on the reporter's honesty: the verifier's job is to re-derive the facts.

use anyhow::Result;
use canary_observation::FlowRun;
use canary_properties::Finding;
use serde::{Deserialize, Serialize};

pub const REPORT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub version: u32,
    pub flow_id: String,
    pub flow_commit: String,
    pub chain: String,
    /// Chain height the run was made against: the clock a claim's TTL starts from.
    pub verified_at_height: u32,
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
    Transaction { txid: String, height: Option<u32> },
    Receiver { address: String, first_seen_payment: String },
    Memo { txid: String, memo_hex: String },
    PoolTotals { height: u32, transparent_zat: u64, sapling_zat: u64, orchard_zat: u64 },
    MerchantField { key: String, value: String, path: String },
}

/// Build a report from a recorded run and its findings.
pub fn build(run: &FlowRun, findings: Vec<Finding>) -> Result<Report> {
    let mut report = Report {
        version: REPORT_VERSION,
        flow_id: run.flow_id.clone(),
        flow_commit: run.flow_commit.clone(),
        chain: run.chain.clone(),
        verified_at_height: run.ended_at_height,
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
        return Ok(sha256_hex(b"canary:empty-evidence"));
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
        out.push(Evidence::Transaction { txid: p.txid.clone(), height: p.height });
        for r in &p.recipient_receivers {
            out.push(Evidence::Receiver { address: r.clone(), first_seen_payment: p.label.clone() });
        }
        if let Some(memo) = &p.memo_hex {
            out.push(Evidence::Memo { txid: p.txid.clone(), memo_hex: memo.clone() });
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
            out.push(Evidence::MerchantField { key: k.clone(), value: v.clone(), path: e.path.clone() });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_root_is_deterministic_and_order_sensitive() {
        let a = vec![
            Evidence::Transaction { txid: "aa".into(), height: Some(1) },
            Evidence::Transaction { txid: "bb".into(), height: Some(2) },
        ];
        let b = vec![
            Evidence::Transaction { txid: "bb".into(), height: Some(2) },
            Evidence::Transaction { txid: "aa".into(), height: Some(1) },
        ];
        assert_eq!(evidence_root(&a).unwrap(), evidence_root(&a).unwrap());
        assert_ne!(evidence_root(&a).unwrap(), evidence_root(&b).unwrap());
    }
}
