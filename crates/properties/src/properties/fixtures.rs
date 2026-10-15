//! Test fixtures: a `clean` run where every property passes, and a `leaky` run where every
//! property fails. Both are hand-built `FlowRun`s so a reader can re-check every observation by
//! eye; txids are 64-hex-like strings produced from a byte seed.

use std::collections::BTreeMap;

use zanary_observation::{
    FlowRun, Funding, MerchantEvent, Payment, PaymentKind, Pool, PoolTotals, Receipt,
};

use super::common::hex_of;

/// A 64-char hex-looking txid seed (e.g. `0x11` -> `"1111…"`).
pub(crate) fn tid(seed: u8) -> String {
    format!("{:02x}", seed).repeat(32)
}

fn merchant_event(height: u32, invoice: &str, settled_label: &str) -> MerchantEvent {
    let mut fields = BTreeMap::new();
    fields.insert("invoice_id".to_string(), invoice.to_string());
    let mut headers = BTreeMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    MerchantEvent {
        height: Some(height),
        method: "POST".to_string(),
        path: "/v1/settle".to_string(),
        headers,
        fields,
        remote_ip: None,
        settled_payment_label: Some(settled_label.to_string()),
    }
}

fn clean_payment(
    label: &str,
    txid: String,
    height: u32,
    amount: u64,
    recipient: &str,
    memo: &str,
) -> Payment {
    Payment {
        label: label.to_string(),
        txid,
        height: Some(height),
        amount_zat: amount,
        recipient_receivers: vec![recipient.to_string()],
        sender_receivers: vec![],
        memo_hex: Some(hex_of(memo)),
        memo_text: Some(memo.to_string()),
        invoice_id: Some(memo.to_string()),
        pool_in: Pool::Orchard,
        pool_out: Pool::Orchard,
        kind: PaymentKind::Shielded,
        policy: Some("FullPrivacy".to_string()),
        observed: BTreeMap::new(),
    }
}

/// A run that is clean on all 12 properties: shielded everywhere, fresh receivers, distinct
/// non-round amounts, invoice-bound memos with no PII, spends well after funding, a merchant side
/// that receives only what it needs, and a receipt that discloses nothing extra.
pub(crate) fn clean_run() -> FlowRun {
    FlowRun {
        flow_id: "clean-flow".to_string(),
        flow_commit: "clean-commit".to_string(),
        chain: "regtest".to_string(),
        started_at_height: 90,
        ended_at_height: 120,
        funding: vec![Funding {
            label: "shielded deposit".to_string(),
            txid: tid(0x11),
            height: Some(100),
            amount_zat: 500_000_000,
            receiver: "uregtest1cleanfundwallet000000000000000000000000000000000000000000"
                .to_string(),
            pool: Pool::Orchard,
        }],
        payments: vec![
            clean_payment(
                "invoice A",
                tid(0x21),
                110,
                1_234_567,
                "uregtest1payeealpha0000000000000000000000000000000000000000000000",
                "INV-1001",
            ),
            clean_payment(
                "invoice B",
                tid(0x22),
                113,
                2_345_678,
                "uregtest1payeebeta00000000000000000000000000000000000000000000000",
                "ORDER-2002",
            ),
            clean_payment(
                "invoice C",
                tid(0x23),
                116,
                3_456_789,
                "uregtest1payeegamma000000000000000000000000000000000000000000000",
                "TICKET-3003",
            ),
        ],
        merchant_events: vec![
            merchant_event(110, "INV-1001", "invoice A"),
            merchant_event(113, "ORDER-2002", "invoice B"),
            merchant_event(116, "TICKET-3003", "invoice C"),
        ],
        receipts: vec![Receipt {
            label: "settlement-proof".to_string(),
            format: "text".to_string(),
            body: format!("invoice INV-1001 settled in tx {}", tid(0x21)),
            claims: vec!["invoice INV-1001 settled".to_string()],
            discloses: vec![],
            // An unsigned fixture: the signature scheme has its own fixtures in
            // `zanary_report::receipt` (a well-formed receipt verifies, one flipped byte does not).
            signer: None,
            signature: None,
            attestation: None,
        }],
        pool_totals: vec![
            PoolTotals {
                height: 95,
                transparent_zat: 0,
                sapling_zat: 0,
                orchard_zat: 0,
            },
            PoolTotals {
                height: 100,
                transparent_zat: 0,
                sapling_zat: 0,
                orchard_zat: 500_000_000,
            },
            PoolTotals {
                height: 110,
                transparent_zat: 0,
                sapling_zat: 0,
                orchard_zat: 500_000_000,
            },
            PoolTotals {
                height: 116,
                transparent_zat: 0,
                sapling_zat: 0,
                orchard_zat: 500_000_000,
            },
        ],
        merchant_is_simulated: true,
    }
}

/// A run that leaks on all 12 properties: transparent funding and recipient, an unshielded
/// payment, a reused receiver, clusterable payments, an explicit policy downgrade, missing and
/// PII-carrying memos, a funding echo, same-block and adjacent-block spends, identity fields on
/// the merchant side, and a report that carries a viewing key.
pub(crate) fn leaky_run() -> FlowRun {
    let leaky_fund_receiver = "tmLeakyTransparentFundingWallet1111111111111111";
    let shared_recipient = "tmLeakySharedRecipient111111111111111111111";

    let mut e1_fields = BTreeMap::new();
    e1_fields.insert("invoice_id".to_string(), "4472".to_string());
    e1_fields.insert("email".to_string(), "alice@example.com".to_string());
    e1_fields.insert("customer_ref".to_string(), "cust-77".to_string());
    let mut e2_fields = BTreeMap::new();
    e2_fields.insert("invoice_id".to_string(), "4473".to_string());
    e2_fields.insert("customer_ref".to_string(), "cust-77".to_string());
    let mut headers = BTreeMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());

    let pii_memo = "email alice@example.com phone 555-123-4567";

    FlowRun {
        flow_id: "leaky-flow".to_string(),
        flow_commit: "leaky-commit".to_string(),
        chain: "regtest".to_string(),
        started_at_height: 45,
        ended_at_height: 65,
        funding: vec![Funding {
            label: "transparent faucet drop".to_string(),
            txid: tid(0x31),
            height: Some(50),
            amount_zat: 500_000_000,
            receiver: leaky_fund_receiver.to_string(),
            pool: Pool::Transparent,
        }],
        payments: vec![
            Payment {
                label: "invoice 4471".to_string(),
                txid: tid(0x41),
                height: Some(50),
                amount_zat: 500_000_000,
                recipient_receivers: vec![shared_recipient.to_string()],
                sender_receivers: vec![leaky_fund_receiver.to_string()],
                memo_hex: None,
                memo_text: None,
                invoice_id: Some("4471".to_string()),
                pool_in: Pool::Orchard,
                pool_out: Pool::Transparent,
                kind: PaymentKind::CrossPool,
                policy: Some("AllowRevealedAmounts".to_string()),
                observed: BTreeMap::new(),
            },
            Payment {
                label: "invoice 4472".to_string(),
                txid: tid(0x42),
                height: Some(51),
                amount_zat: 1_234_567,
                recipient_receivers: vec![shared_recipient.to_string()],
                sender_receivers: vec![leaky_fund_receiver.to_string()],
                memo_hex: None,
                memo_text: None,
                invoice_id: None,
                pool_in: Pool::Transparent,
                pool_out: Pool::Orchard,
                kind: PaymentKind::CrossPool,
                policy: None,
                observed: BTreeMap::new(),
            },
            Payment {
                label: "invoice 4473".to_string(),
                txid: tid(0x43),
                height: Some(60),
                amount_zat: 1_234_567,
                recipient_receivers: vec![
                    "uregtest1leakfresh0000000000000000000000000000000000000000000000000"
                        .to_string(),
                ],
                sender_receivers: vec![],
                memo_hex: Some(hex_of(pii_memo)),
                memo_text: Some(pii_memo.to_string()),
                invoice_id: Some("4473".to_string()),
                pool_in: Pool::Orchard,
                pool_out: Pool::Orchard,
                kind: PaymentKind::Shielded,
                policy: None,
                observed: BTreeMap::new(),
            },
        ],
        merchant_events: vec![
            MerchantEvent {
                height: Some(51),
                method: "POST".to_string(),
                path: "/v1/settle".to_string(),
                headers: headers.clone(),
                fields: e1_fields,
                remote_ip: Some("203.0.113.9".to_string()),
                settled_payment_label: Some("invoice 4472".to_string()),
            },
            MerchantEvent {
                height: Some(60),
                method: "POST".to_string(),
                path: "/v1/settle".to_string(),
                headers,
                fields: e2_fields,
                remote_ip: Some("203.0.113.9".to_string()),
                settled_payment_label: Some("invoice 4473".to_string()),
            },
        ],
        receipts: vec![Receipt {
            label: "leak-report".to_string(),
            format: "json".to_string(),
            body: "{\"claim\":\"payment made\",\"ivk\":\"uvk1examplekeymaterial\"}".to_string(),
            claims: vec!["payment made".to_string()],
            discloses: vec!["viewing_key".to_string(), "other_payments".to_string()],
            signer: None,
            signature: None,
            attestation: None,
        }],
        pool_totals: vec![
            PoolTotals {
                height: 50,
                transparent_zat: 0,
                sapling_zat: 0,
                orchard_zat: 500_000_000,
            },
            PoolTotals {
                height: 51,
                transparent_zat: 100_000_000,
                sapling_zat: 0,
                orchard_zat: 450_000_000,
            },
            PoolTotals {
                height: 60,
                transparent_zat: 100_000_000,
                sapling_zat: 0,
                orchard_zat: 440_000_000,
            },
        ],
        merchant_is_simulated: true,
    }
}
