//! Receipt signing and offline verification.
//!
//! A receipt is only worth something if a stranger can check it without trusting the rig. So the
//! merchant signs the canonical bytes of its receipt with its own Orchard spend-authorizing key —
//! the RedPallas key that authorizes its shielded spends — and the receipt carries the
//! corresponding public key. Anyone holding the receipt can then verify it with no node, no
//! indexer and no rig running, and, given the merchant's unified full viewing key, check that the
//! signer really is the key that controls the address the invoice named.
//!
//! What this does **not** claim: a valid signature proves the receipt came from the holder of that
//! key. It says nothing about whether the merchant's own accounting is honest — that is what the
//! chain evidence in the report is for.

use anyhow::{anyhow, Context, Result};
use canary_observation::Receipt;
use reddsa::orchard::SpendAuth;
use serde::Serialize;

/// The bytes a receipt signature covers: the receipt's own fields, in a fixed order, as JSON.
///
/// Signing a canonical projection of the fields — not the whole `Receipt`, which carries the
/// signature itself — is what lets a verifier rebuild exactly the bytes the signer signed.
#[derive(Serialize)]
struct SigningPayload<'a> {
    format: &'a str,
    body: &'a str,
    claims: &'a [String],
    discloses: &'a [String],
}

/// The exact bytes covered by a receipt's signature.
pub fn signing_payload(receipt: &Receipt) -> Result<Vec<u8>> {
    serde_json::to_vec(&SigningPayload {
        format: &receipt.format,
        body: &receipt.body,
        claims: &receipt.claims,
        discloses: &receipt.discloses,
    })
    .context("serializing the receipt's signing payload")
}

/// The result of checking a receipt's signature.
#[derive(Debug, Clone)]
pub struct SignatureCheck {
    /// The public key the signature was checked against, hex.
    pub signer: String,
    /// True when the receipt also named this signer in its own fields.
    pub signer_matches: bool,
}

/// Checks a receipt's signature against the public key it names.
///
/// Fails when the receipt is unsigned, when the key or signature is malformed, or when the
/// signature does not verify over [`signing_payload`].
pub fn verify_signature(receipt: &Receipt) -> Result<SignatureCheck> {
    let signer_hex = receipt
        .signer
        .as_deref()
        .ok_or_else(|| anyhow!("this receipt is unsigned (no `signer` field)"))?;
    let signature_hex = receipt
        .signature
        .as_deref()
        .ok_or_else(|| anyhow!("this receipt has no `signature` field"))?;

    let signer_bytes = hex_decode(signer_hex).context("decoding the receipt's signer key")?;
    let signer_array: [u8; 32] = signer_bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("the signer key is {} bytes, expected 32", signer_bytes.len()))?;
    let signature_bytes = hex_decode(signature_hex).context("decoding the receipt signature")?;
    let signature_array: [u8; 64] = signature_bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("the signature is {} bytes, expected 64", signature_bytes.len()))?;

    let verifying_key = reddsa::VerificationKey::<SpendAuth>::try_from(
        reddsa::VerificationKeyBytes::<SpendAuth>::from(signer_array),
    )
    .map_err(|e| anyhow!("the signer key is not a valid Orchard spend-authorizing key: {e:?}"))?;
    let signature = reddsa::Signature::<SpendAuth>::from(signature_array);
    let payload = signing_payload(receipt)?;
    verifying_key
        .verify(&payload, &signature)
        .map_err(|e| anyhow!("the receipt's signature does not verify over its own fields: {e}"))?;

    Ok(SignatureCheck {
        signer: hex_encode(&signer_array),
        signer_matches: true,
    })
}

/// The Orchard spend-authorizing public key of a unified full viewing key, hex — the key a receipt
/// signed by the holder of that wallet must name.
///
/// This is the identity binding: an invoice names the merchant's receiver, so a stranger who has
/// the merchant's UFVK can check that the receipt's signer is the key behind the address that was
/// actually paid. The key published is only `ak` (32 bytes) — not a viewing key — so it reveals
/// no note contents or history.
/// The regtest parameters, for decoding regtest unified keys.
///
/// `zcash_protocol` has no `Network::Regtest` variant; a local chain is described by
/// [`LocalNetwork`], whose activation heights are the caller's to set. The values here mirror the
/// harness node (all seven upgrades active from height 1, no NU6.1+).
pub fn regtest_params() -> zcash_protocol::local_consensus::LocalNetwork {
    use zcash_protocol::consensus::BlockHeight;
    let one = Some(BlockHeight::from_u32(1));
    zcash_protocol::local_consensus::LocalNetwork {
        overwinter: one,
        sapling: one,
        blossom: one,
        heartwood: one,
        canopy: one,
        nu5: one,
        nu6: one,
        nu6_1: None,
        nu6_2: None,
        nu6_3: None,
    }
}

pub fn signer_of_ufvk<P: zcash_protocol::consensus::Parameters>(
    params: &P,
    ufvk: &str,
) -> Result<String> {
    let fvk = zcash_keys::keys::UnifiedFullViewingKey::decode(params, ufvk)
        .map_err(|e| anyhow!("decoding the unified full viewing key: {e}"))?;
    let orchard = fvk
        .orchard()
        .ok_or_else(|| anyhow!("that UFVK carries no Orchard component"))?;
    Ok(hex_encode(&ak_of_orchard_fvk(orchard)))
}

/// The 32-byte `ak` of an Orchard full viewing key: the public half of the key that authorizes
/// that wallet's shielded spends, which is what a receipt signed by that wallet must name.
///
/// Read straight out of the FVK's own 96-byte encoding (`ak || nk || rivk`, Zcash Protocol Spec
/// § 5.6.4.4), so this depends on nothing but the published format. Note what is *not* here: an
/// FVK would let its holder read the wallet's notes and history, so only `ak` is ever published.
pub fn ak_of_orchard_fvk(fvk: &orchard::keys::FullViewingKey) -> [u8; 32] {
    let bytes = fvk.to_bytes();
    bytes[0..32]
        .try_into()
        .expect("ak is the first 32 bytes of an Orchard full viewing key")
}

/// Checks a receipt's `body` against the shape the format promises, and its `claims` against the
/// body — the machine-readable half of "a stranger can check it".
///
/// The rules are the ones in `schemas/canary-receipt-v1.json` (shipped for third-party tools);
/// this is the same check, implemented where the verifier can run it with no JSON-Schema library.
/// A claim that contradicts the body is worse than no claim at all, so a receipt whose words and
/// figures disagree fails here rather than being printed as if it were fine.
pub fn validate_claim(receipt: &Receipt) -> Result<Vec<String>> {
    let mut notes = Vec::new();
    if receipt.format != "canary-receipt-v1" {
        return Err(anyhow!(
            "unknown receipt format {:?} — this verifier knows canary-receipt-v1",
            receipt.format
        ));
    }
    let body: serde_json::Value = serde_json::from_str(&receipt.body)
        .context("the receipt body is not JSON (canary-receipt-v1 bodies are JSON text)")?;
    let object = body
        .as_object()
        .ok_or_else(|| anyhow!("the receipt body is not a JSON object"))?;

    for key in ["invoice_id", "amount_zat", "txid", "memo_sha256"] {
        if !object.contains_key(key) {
            return Err(anyhow!("the receipt body has no `{key}`"));
        }
    }
    for key in object.keys() {
        if !["invoice_id", "amount_zat", "txid", "memo_sha256"].contains(&key.as_str()) {
            return Err(anyhow!(
                "the receipt body carries `{key}`, which canary-receipt-v1 does not define \
                 (a body field beyond the format is a disclosure nobody claimed)"
            ));
        }
    }

    let invoice_id = object["invoice_id"]
        .as_str()
        .ok_or_else(|| anyhow!("`invoice_id` is not a string"))?;
    if invoice_id.is_empty() {
        return Err(anyhow!("`invoice_id` is empty"));
    }
    let amount_zat = object["amount_zat"]
        .as_u64()
        .ok_or_else(|| anyhow!("`amount_zat` is not a non-negative integer"))?;
    let txid = object["txid"]
        .as_str()
        .ok_or_else(|| anyhow!("`txid` is not a string"))?;
    let memo_sha256 = object["memo_sha256"]
        .as_str()
        .ok_or_else(|| anyhow!("`memo_sha256` is not a string"))?;
    for (name, value) in [("txid", txid), ("memo_sha256", memo_sha256)] {
        if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(anyhow!(
                "`{name}` is not 64 hex characters (canary-receipt-v1 says lowercase hex)"
            ));
        }
    }

    // The words must state the figures. A receipt is a claim, not a data dump: if the claims do
    // not name the transaction, the amount and the invoice, the artifact is asserting something
    // other than what it carries.
    let words = receipt.claims.join(" ");
    for (what, needle) in [
        ("the transaction", txid.to_string()),
        ("the amount in zatoshis", amount_zat.to_string()),
        ("the invoice id", invoice_id.to_string()),
    ] {
        if !words.contains(&needle) {
            return Err(anyhow!(
                "the receipt's claims never state {what} ({needle}) that its body carries"
            ));
        }
    }
    if !receipt.discloses.is_empty() {
        return Err(anyhow!(
            "the receipt declares disclosures ({}) — a receipt carries nothing beyond its claims",
            receipt.discloses.join(", ")
        ));
    }

    notes.push(format!(
        "body: canary-receipt-v1, invoice {invoice_id}, {amount_zat} zatoshis, txid {txid}"
    ));
    notes.push("claims: state the body's invoice id, amount and transaction".to_string());
    Ok(notes)
}

/// The hex wire form of a RedPallas signature: `r || s`, 64 bytes.
///
/// Orchard's signature type converts to those bytes directly (`From<&Signature<T>> for [u8; 64]`),
/// and the verifier rebuilds the signature from the same 64 bytes.
pub fn signature_to_hex(
    signature: &orchard::primitives::redpallas::Signature<SpendAuth>,
) -> String {
    let bytes: [u8; 64] = signature.into();
    hex_encode(&bytes)
}

pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return Err(anyhow!("hex string has an odd length"));
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| anyhow!("invalid hex at byte {}: {e}", i / 2))
        })
        .collect()
}
