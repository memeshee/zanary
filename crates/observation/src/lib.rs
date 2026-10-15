//! ZANARY shared observation model.
//!
//! A *flow run* is what the rig watches while it executes a payment flow on the throwaway chain:
//! every payment the flow made, every funding event, everything the merchant side received, and
//! every artifact the flow exported. Properties are evaluated against this record, never against
//! the flow's own claims — that separation is the whole point of the rig.
//!
//! Keep this crate dumb: data only, no logic, additive changes only (other crates depend on it).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Which pool value moved through, as far as the rig could tell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pool {
    Transparent,
    Sapling,
    Orchard,
    /// Moved between pools in one transaction (shield, unshield, or a mixed bundle).
    CrossPool,
    Unknown,
}

/// What kind of event a payment is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentKind {
    /// Shielded source and shielded destination, same pool.
    Shielded,
    /// Transparent source or destination.
    Transparent,
    /// Value crossed a pool boundary (shield, unshield, or a route through transparent).
    CrossPool,
    Unknown,
}

/// A single payment the flow made (outgoing or self-directed), with everything the rig observed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Human label from the flow definition, e.g. "invoice 4471".
    pub label: String,
    pub txid: String,
    pub height: Option<u32>,
    pub amount_zat: u64,
    /// Receivers the payment was addressed to (as requested by the flow).
    pub recipient_receivers: Vec<String>,
    /// Receivers observed on the funding side, when the rig can see them.
    pub sender_receivers: Vec<String>,
    /// Raw memo bytes, hex.
    pub memo_hex: Option<String>,
    /// Decoded memo text when printable.
    pub memo_text: Option<String>,
    /// Invoice/order id the flow claims this payment settles.
    pub invoice_id: Option<String>,
    pub pool_in: Pool,
    pub pool_out: Pool,
    pub kind: PaymentKind,
    /// Privacy policy / spend-policy argument the flow passed to its wallet layer, verbatim.
    pub policy: Option<String>,
    /// Extra observations, keyed. Free-form on purpose: the rig records what it saw.
    #[serde(default)]
    pub observed: BTreeMap<String, String>,
}

impl Payment {
    pub fn kind(&self) -> PaymentKind {
        self.kind
    }
}

/// Value entering the flow's wallets (from the miner, a faucet, or a treasury).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Funding {
    pub label: String,
    pub txid: String,
    pub height: Option<u32>,
    pub amount_zat: u64,
    pub receiver: String,
    pub pool: Pool,
}

/// One request the merchant/facilitator side of the flow received. The rig runs that side itself,
/// so this is a first-hand record of exactly what a third party learned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantEvent {
    pub height: Option<u32>,
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub fields: BTreeMap<String, String>,
    pub remote_ip: Option<String>,
    /// Label of the payment this event settled, when the merchant could link them.
    pub settled_payment_label: Option<String>,
}

/// An artifact the flow exported to prove something (receipt, report, audit export).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub label: String,
    pub format: String,
    /// The artifact body as the flow emitted it (text or base64).
    pub body: String,
    /// What the artifact is meant to prove.
    #[serde(default)]
    pub claims: Vec<String>,
    /// What else it carries, as observed by the rig.
    #[serde(default)]
    pub discloses: Vec<String>,
    /// The public key that signed this receipt, hex (Orchard spend-authorizing key, RedPallas) —
    /// present when the flow signed it. See `zanary_report::receipt` for the scheme.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer: Option<String>,
    /// The signature over the receipt's signing payload, hex.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// The issuing wallet's statement that the receipt key is its own, present when the flow
    /// attested it. A receipt signed by a *dedicated* receipt key plus an attestation binds the
    /// signer to the wallet for anyone holding the wallet's UFVK; publishing the receipt without
    /// the attestation links it to no wallet at all. See `docs/THREAT-MODEL.md`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attestation: Option<ReceiptAttestation>,
}

/// A wallet's statement that a receipt key belongs to it: the wallet's own Orchard
/// spend-authorizing key signs a payload naming the receipt key's spend-authorizing key. This is
/// what lets a third party bind a receipt's *dedicated* signer back to the wallet — and it is
/// optional to publish, which is the point: a receipt on its own reveals neither.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptAttestation {
    /// The attestation format, e.g. `zanary-receipt-attestation-v1`.
    pub format: String,
    /// The receipt key being attested, hex — the same key as the receipt's `signer`.
    pub receipt_key: String,
    /// The wallet key making the statement, hex (its Orchard spend-authorizing key).
    pub wallet_ak: String,
    /// The signature by `wallet_ak` over the canonical attestation payload, hex.
    pub signature: String,
}

/// Value-pool totals at a height, so pool crossings and unshielding are checkable after the fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolTotals {
    pub height: u32,
    pub transparent_zat: u64,
    pub sapling_zat: u64,
    pub orchard_zat: u64,
}

/// Everything the rig observed for one execution of one flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowRun {
    pub flow_id: String,
    /// Hash of the flow definition, so a claim is about an exact revision.
    pub flow_commit: String,
    pub chain: String,
    pub started_at_height: u32,
    pub ended_at_height: u32,
    #[serde(default)]
    pub funding: Vec<Funding>,
    #[serde(default)]
    pub payments: Vec<Payment>,
    #[serde(default)]
    pub merchant_events: Vec<MerchantEvent>,
    #[serde(default)]
    pub receipts: Vec<Receipt>,
    #[serde(default)]
    pub pool_totals: Vec<PoolTotals>,
    /// True when the rig itself simulated the merchant side (stated, never implied).
    #[serde(default)]
    pub merchant_is_simulated: bool,
}

impl FlowRun {
    /// Payment labels that share a recipient receiver with another payment.
    pub fn receiver_reuse_groups(&self) -> BTreeMap<String, Vec<&Payment>> {
        let mut groups: BTreeMap<String, Vec<&Payment>> = BTreeMap::new();
        for p in &self.payments {
            for r in &p.recipient_receivers {
                groups.entry(r.clone()).or_default().push(p);
            }
        }
        groups.retain(|_, v| {
            let mut labels: Vec<&str> = v.iter().map(|p| p.label.as_str()).collect();
            labels.sort_unstable();
            labels.dedup();
            labels.len() > 1
        });
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pay(label: &str, receiver: &str) -> Payment {
        Payment {
            label: label.into(),
            txid: format!("tx-{label}"),
            height: Some(10),
            amount_zat: 1_000_000,
            recipient_receivers: vec![receiver.into()],
            sender_receivers: vec![],
            memo_hex: None,
            memo_text: None,
            invoice_id: None,
            pool_in: Pool::Orchard,
            pool_out: Pool::Orchard,
            kind: PaymentKind::Shielded,
            policy: None,
            observed: BTreeMap::new(),
        }
    }

    #[test]
    fn receiver_reuse_groups_only_report_shared_receivers() {
        let run = FlowRun {
            flow_id: "f".into(),
            flow_commit: "c".into(),
            chain: "regtest".into(),
            started_at_height: 1,
            ended_at_height: 20,
            funding: vec![],
            payments: vec![pay("a", "uregtest1same"), pay("b", "uregtest1same"), pay("c", "uregtest1fresh")],
            merchant_events: vec![],
            receipts: vec![],
            pool_totals: vec![],
            merchant_is_simulated: false,
        };
        let groups = run.receiver_reuse_groups();
        assert_eq!(groups.len(), 1);
        assert!(groups.contains_key("uregtest1same"));
    }
}
