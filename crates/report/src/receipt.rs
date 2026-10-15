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
use zanary_observation::{Receipt, ReceiptAttestation};
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

/// The bytes an attestation signature covers: the statement's own fields, in a fixed order.
///
/// Like [`signing_payload`], this is a canonical projection — the attestation's signature field is
/// not part of what it signs — so a verifier rebuilds the signed bytes from the artifact alone.
#[derive(Serialize)]
struct AttestationPayload<'a> {
    format: &'a str,
    receipt_key: &'a str,
    wallet_ak: &'a str,
}

/// The exact bytes covered by an attestation's signature.
pub fn attestation_payload(attestation: &ReceiptAttestation) -> Result<Vec<u8>> {
    serde_json::to_vec(&AttestationPayload {
        format: &attestation.format,
        receipt_key: &attestation.receipt_key,
        wallet_ak: &attestation.wallet_ak,
    })
    .context("serializing the attestation's signing payload")
}

/// Checks that a receipt's attestation binds its **receipt key** to `wallet_ak` — the wallet key a
/// stranger derived from the merchant's UFVK.
///
/// This is what keeps the binding property while the receipt itself stops carrying the wallet's
/// identity: the receipt is signed by a dedicated key, and this attestation is the wallet's signed
/// statement that the key is its own. Fails when the receipt carries no attestation (nothing binds
/// it to any wallet), when the attestation names a different receipt key or wallet, or when its
/// signature does not verify.
pub fn verify_attestation(receipt: &Receipt, wallet_ak_hex: &str) -> Result<()> {
    let attestation = receipt.attestation.as_ref().ok_or_else(|| {
        anyhow!(
            "this receipt carries no attestation, so nothing binds its receipt key to a wallet — \
             a dedicated-key receipt proves only which receipt key issued it"
        )
    })?;
    if attestation.format != "zanary-receipt-attestation-v1" {
        return Err(anyhow!(
            "unknown attestation format `{}` (expected `zanary-receipt-attestation-v1`)",
            attestation.format
        ));
    }
    let signer = receipt
        .signer
        .as_deref()
        .ok_or_else(|| anyhow!("this receipt is unsigned (no `signer` field)"))?;
    if !attestation.receipt_key.eq_ignore_ascii_case(signer) {
        return Err(anyhow!(
            "the attestation names receipt key {} but the receipt is signed by {}",
            short(attestation.receipt_key.as_str()),
            short(signer)
        ));
    }
    if !attestation.wallet_ak.eq_ignore_ascii_case(wallet_ak_hex) {
        return Err(anyhow!(
            "the attestation was made by wallet key {} and not by {}",
            short(attestation.wallet_ak.as_str()),
            short(wallet_ak_hex)
        ));
    }

    let wallet_bytes = hex_decode(&attestation.wallet_ak).context("decoding the wallet key")?;
    let wallet_array: [u8; 32] = wallet_bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("the wallet key is {} bytes, expected 32", wallet_bytes.len()))?;
    let signature_bytes =
        hex_decode(&attestation.signature).context("decoding the attestation signature")?;
    let signature_array: [u8; 64] = signature_bytes
        .as_slice()
        .try_into()
        .map_err(|_| {
            anyhow!(
                "the attestation signature is {} bytes, expected 64",
                signature_bytes.len()
            )
        })?;

    let verifying_key = reddsa::VerificationKey::<SpendAuth>::try_from(
        reddsa::VerificationKeyBytes::<SpendAuth>::from(wallet_array),
    )
    .map_err(|e| {
        anyhow!("the attesting wallet key is not a valid Orchard spend-authorizing key: {e:?}")
    })?;
    let signature = reddsa::Signature::<SpendAuth>::from(signature_array);
    let payload = attestation_payload(attestation)?;
    verifying_key
        .verify(&payload, &signature)
        .map_err(|e| anyhow!("the attestation's signature does not verify: {e}"))?;
    Ok(())
}

/// `first8…last8` for a comparison message, so two different keys can never look alike in it.
pub fn short(hex: &str) -> String {
    if hex.len() > 20 {
        format!("{}…{}", &hex[..8], &hex[hex.len() - 8..])
    } else {
        hex.to_string()
    }
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
/// The rules are the ones in `schemas/zanary-receipt-v1.json` (shipped for third-party tools);
/// this is the same check, implemented where the verifier can run it with no JSON-Schema library.
/// A claim that contradicts the body is worse than no claim at all, so a receipt whose words and
/// figures disagree fails here rather than being printed as if it were fine.
pub fn validate_claim(receipt: &Receipt) -> Result<Vec<String>> {
    let mut notes = Vec::new();
    if receipt.format != "zanary-receipt-v1" {
        return Err(anyhow!(
            "unknown receipt format {:?} — this verifier knows zanary-receipt-v1",
            receipt.format
        ));
    }
    let body: serde_json::Value = serde_json::from_str(&receipt.body)
        .context("the receipt body is not JSON (zanary-receipt-v1 bodies are JSON text)")?;
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
                "the receipt body carries `{key}`, which zanary-receipt-v1 does not define \
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
                "`{name}` is not 64 hex characters (zanary-receipt-v1 says lowercase hex)"
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
        "body: zanary-receipt-v1, invoice {invoice_id}, {amount_zat} zatoshis, txid {txid}"
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

#[cfg(test)]
mod tests {
    use super::*;
    use orchard::keys::{FullViewingKey, SpendAuthorizingKey, SpendingKey};

    /// A deterministic Orchard keypair (spend key + its `ak` hex), the same derivation the probe
    /// signs with: `SpendAuthorizingKey::from(&sk).randomize(identity)` is the un-randomized pair.
    fn keypair(seed_byte: u8) -> (SpendingKey, String) {
        let sk = SpendingKey::from_bytes([seed_byte; 32])
            .into_option()
            .expect("a valid spending key");
        let ak = ak_of_orchard_fvk(&FullViewingKey::from(&sk));
        (sk, hex_encode(&ak))
    }

    fn sign(sk: &SpendingKey, payload: &[u8]) -> String {
        let key =
            SpendAuthorizingKey::from(sk).randomize(&pasta_curves::pallas::Scalar::from(0u64));
        signature_to_hex(&key.sign(rand::rngs::OsRng, payload))
    }

    /// A receipt signed by a *dedicated* receipt key, naming that key as its signer.
    fn receipt_signed_by(receipt_key: &SpendingKey, receipt_ak_hex: &str) -> Receipt {
        let mut receipt = Receipt {
            label: "receipt test".to_string(),
            format: "zanary-receipt-v1".to_string(),
            body: "{\"amount_zat\":1,\"invoice_id\":\"1\",\"memo_sha256\":\"00\",\"txid\":\"11\"}"
                .to_string(),
            claims: vec!["invoice 1 was paid 1 zatoshi".to_string()],
            discloses: Vec::new(),
            signer: Some(receipt_ak_hex.to_string()),
            signature: None,
            attestation: None,
        };
        let payload = signing_payload(&receipt).expect("payload");
        receipt.signature = Some(sign(receipt_key, &payload));
        receipt
    }

    /// The wallet's signed statement that a receipt key is its own.
    fn attestation(
        receipt_key_hex: &str,
        wallet_sk: &SpendingKey,
        wallet_ak_hex: &str,
    ) -> ReceiptAttestation {
        let mut att = ReceiptAttestation {
            format: "zanary-receipt-attestation-v1".to_string(),
            receipt_key: receipt_key_hex.to_string(),
            wallet_ak: wallet_ak_hex.to_string(),
            signature: String::new(),
        };
        let payload = attestation_payload(&att).expect("payload");
        att.signature = sign(wallet_sk, &payload);
        att
    }

    #[test]
    fn an_attestation_binds_the_receipt_key_to_the_wallet() {
        let (wallet_sk, wallet_ak) = keypair(7);
        let (receipt_sk, receipt_ak) = keypair(9);
        assert_ne!(wallet_ak, receipt_ak, "two keypairs must differ");
        let mut receipt = receipt_signed_by(&receipt_sk, &receipt_ak);
        receipt.attestation = Some(attestation(&receipt_ak, &wallet_sk, &wallet_ak));

        verify_signature(&receipt).expect("the receipt signature is intact");
        verify_attestation(&receipt, &wallet_ak).expect("the wallet attests that receipt key");
    }

    #[test]
    fn a_receipt_without_an_attestation_is_not_bound_to_any_wallet() {
        let (_, wallet_ak) = keypair(7);
        let (receipt_sk, receipt_ak) = keypair(9);
        let receipt = receipt_signed_by(&receipt_sk, &receipt_ak);

        // The signature stands on its own — that is the privacy property: the receipt proves which
        // receipt key issued it and says nothing about the wallet behind it.
        verify_signature(&receipt).expect("the receipt signature is intact");
        let err = verify_attestation(&receipt, &wallet_ak).expect_err("no attestation, no binding");
        assert!(format!("{err}").contains("carries no attestation"), "{err}");
    }

    #[test]
    fn an_attestation_that_names_another_receipt_key_is_refused() {
        let (wallet_sk, wallet_ak) = keypair(7);
        let (receipt_sk, receipt_ak) = keypair(9);
        let (_, other_receipt_ak) = keypair(11);
        let mut receipt = receipt_signed_by(&receipt_sk, &receipt_ak);
        // A genuine wallet attestation — for a *different* receipt key, grafted onto this receipt.
        receipt.attestation = Some(attestation(&other_receipt_ak, &wallet_sk, &wallet_ak));

        let err = verify_attestation(&receipt, &wallet_ak).expect_err("must not bind");
        assert!(format!("{err}").contains("names receipt key"), "{err}");
    }

    #[test]
    fn an_attestation_by_another_wallet_is_refused() {
        let (wallet_sk, wallet_ak) = keypair(7);
        let (receipt_sk, receipt_ak) = keypair(9);
        let (_, other_wallet_ak) = keypair(11);
        let mut receipt = receipt_signed_by(&receipt_sk, &receipt_ak);
        receipt.attestation = Some(attestation(&receipt_ak, &wallet_sk, &wallet_ak));

        let err = verify_attestation(&receipt, &other_wallet_ak).expect_err("must not bind");
        assert!(format!("{err}").contains("was made by wallet key"), "{err}");
    }

    #[test]
    fn a_flipped_attestation_signature_fails() {
        let (wallet_sk, wallet_ak) = keypair(7);
        let (receipt_sk, receipt_ak) = keypair(9);
        let mut receipt = receipt_signed_by(&receipt_sk, &receipt_ak);
        let mut att = attestation(&receipt_ak, &wallet_sk, &wallet_ak);
        // Flip one hex digit of the signature: the cheapest tamper there is.
        let mut bytes = att.signature.into_bytes();
        bytes[0] = if bytes[0] == b'0' { b'1' } else { b'0' };
        att.signature = String::from_utf8(bytes).expect("hex stays hex");
        receipt.attestation = Some(att);

        let err = verify_attestation(&receipt, &wallet_ak).expect_err("tampered must fail");
        assert!(format!("{err}").contains("does not verify"), "{err}");
    }

    #[test]
    fn the_attestation_is_not_part_of_the_receipt_signature() {
        // Attaching an attestation must not invalidate the receipt's own signature, or the field
        // could not have been added to a format that already has receipts in the wild.
        let (wallet_sk, wallet_ak) = keypair(7);
        let (receipt_sk, receipt_ak) = keypair(9);
        let mut receipt = receipt_signed_by(&receipt_sk, &receipt_ak);
        let before = signing_payload(&receipt).expect("payload");

        receipt.attestation = Some(attestation(&receipt_ak, &wallet_sk, &wallet_ak));

        assert_eq!(
            before,
            signing_payload(&receipt).expect("payload"),
            "the signed bytes must not move when an attestation is attached"
        );
        verify_signature(&receipt).expect("the receipt signature still verifies");
    }

    #[test]
    fn an_unknown_attestation_format_is_refused() {
        let (wallet_sk, wallet_ak) = keypair(7);
        let (receipt_sk, receipt_ak) = keypair(9);
        let mut receipt = receipt_signed_by(&receipt_sk, &receipt_ak);
        let mut att = attestation(&receipt_ak, &wallet_sk, &wallet_ak);
        att.format = "zanary-receipt-attestation-v2".to_string();
        receipt.attestation = Some(att);

        let err = verify_attestation(&receipt, &wallet_ak).expect_err("unknown format");
        assert!(format!("{err}").contains("unknown attestation format"), "{err}");
    }
}
