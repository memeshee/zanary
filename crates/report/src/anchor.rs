//! Anchoring: turning a report into a claim with a public, chain-enforced timestamp.
//!
//! A run's evidence is hashed into an `evidence_root` and a `report_hash`, and the payload below is
//! written into a **shielded memo** paid to a public beacon address. The chain then says, for
//! everyone and without any authority, that *evidence with this root existed by height H*: nobody
//! can backdate a passing run, and nobody can remove an anchor once mined.
//!
//! What the anchor reveals, stated the way the rest of this rig states things:
//!  * it reveals the **hashes** (`evidence_root`, `report_hash`, `flow_commit`) and the height the
//!    report was made at — nothing else. No identity, no payer, no amount, no finding text;
//!  * the **memo is public to whoever holds the beacon's viewing key**, which is published on
//!    purpose: that key can only read this log of hashes, so publishing it is what makes the log
//!    checkable by a stranger, and it is not the payer's key;
//!  * the anchor payment's own spending side is shielded, so an observer learns that *some* wallet
//!    anchored a root, not which one — which is the same unlinkability the rig asserts elsewhere;
//!  * the log is **permissionless**: anyone can pay the beacon and add an anchor (it is a payment to
//!    a published address). That cannot forge a timestamp — a root's earliest anchor is still the
//!    moment it first existed on chain — but it does mean the log can be spammed, and a verifier
//!    must therefore report *which* hit it accepted (the earliest) rather than "an anchor exists".
//!
//! The payload is a **closed** object: a field this version does not define is refused, because a
//! field beyond the format is a disclosure nobody claimed — the same rule `report-leak` applies to
//! receipts.

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use zcash_client_backend::decrypt_transaction;
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_primitives::transaction::Transaction;
use zcash_protocol::consensus::{BlockHeight, BranchId};
use zip32::AccountId;

use crate::node::NodeRpc;
use crate::verify::ChainParams;
use crate::Report;

/// The memo prefix: a scanner can filter on it before parsing anything.
pub const MEMO_PREFIX: &str = "zanary-anchor:v1:";
/// Payload version, carried in the payload as well as in the prefix.
pub const ANCHOR_VERSION: u32 = 1;
/// What an anchor payment carries, in zatoshis. An anchor needs a note to hang the memo on, so the
/// amount is the smallest thing that makes one: it is not a fee and not a payment for anything.
pub const ANCHOR_AMOUNT_ZAT: u64 = 1_000;

/// The anchored payload, as it travels in the memo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anchor {
    /// Payload version. A version this code does not know is refused, never best-effort parsed.
    pub v: u32,
    /// Chain the report was made against (`regtest`, `main`, `test`).
    pub chain: String,
    /// Hash of the flow definition the report is about.
    pub flow_commit: String,
    /// Hash of the report itself (`report_hash`, recomputed by the verifier).
    pub report_hash: String,
    /// Merkle root over the report's evidence array.
    pub evidence_root: String,
    /// Height the report was made at — the anchor must be mined at or after it.
    pub verified_at_height: u32,
}

impl Anchor {
    /// The memo text carried on chain: prefix, then the payload as canonical JSON.
    pub fn memo_text(&self) -> Result<String> {
        let json = serde_json::to_string(self).context("serializing the anchor payload")?;
        let text = format!("{MEMO_PREFIX}{json}");
        // A ZIP-302 memo holds 512 bytes; an anchor that does not fit is a bug, not a truncation.
        if text.len() > 512 {
            bail!(
                "the anchor memo is {} bytes, more than the 512 a ZIP-302 memo holds",
                text.len()
            );
        }
        Ok(text)
    }

    /// Parses a memo body: `Ok(None)` for anything that is not an anchor at all, an error for
    /// something that claims to be one and is malformed. A prefix match that does not parse must
    /// never be silently skipped — that is how a corrupted anchor becomes "no anchor".
    pub fn from_memo_text(text: &str) -> Result<Option<Self>> {
        let Some(json) = text.strip_prefix(MEMO_PREFIX) else {
            return Ok(None);
        };
        let value: serde_json::Value = serde_json::from_str(json)
            .with_context(|| format!("an anchor memo carries invalid JSON: {json:?}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| anyhow!("an anchor memo does not carry a JSON object"))?;
        for key in object.keys() {
            if !KNOWN_FIELDS.contains(&key.as_str()) {
                bail!(
                    "a memo carries an anchor-shaped payload with field `{key}`, which \
                     zanary-anchor-v{ANCHOR_VERSION} does not define"
                );
            }
        }
        let anchor: Anchor = serde_json::from_value(value).with_context(|| {
            format!("an anchor memo does not parse as v{ANCHOR_VERSION}: {json:?}")
        })?;
        if anchor.v != ANCHOR_VERSION {
            bail!(
                "an anchor memo carries payload version {} — this code knows version {ANCHOR_VERSION}",
                anchor.v
            );
        }
        Ok(Some(anchor))
    }

    /// Everything this payload says that the report it claims to anchor contradicts.
    ///
    /// An anchor is only worth anything if it is about a specific report, so the verifier checks
    /// every field it can re-derive rather than trusting the match on the root alone.
    pub fn disagreements(
        &self,
        report: &Report,
        report_hash: &str,
        evidence_root: &str,
    ) -> Vec<String> {
        let mut out = Vec::new();
        let expect = |what: &str, got: &str, want: &str| {
            format!("{what}: anchor says {got}, the report says {want}")
        };
        if self.chain != report.chain {
            out.push(expect("chain", &self.chain, &report.chain));
        }
        if self.flow_commit != report.flow_commit {
            out.push(expect(
                "flow_commit",
                &self.flow_commit,
                &report.flow_commit,
            ));
        }
        if self.report_hash != report_hash {
            out.push(expect("report_hash", &self.report_hash, report_hash));
        }
        if self.evidence_root != evidence_root {
            out.push(expect("evidence_root", &self.evidence_root, evidence_root));
        }
        if self.verified_at_height != report.verified_at_height {
            out.push(format!(
                "verified_at_height: anchor says {}, the report says {}",
                self.verified_at_height, report.verified_at_height
            ));
        }
        out
    }
}

/// What an anchor run writes next to the report it anchors: the payload, the memo as it went on
/// chain, and the chain facts that make it findable again.
///
/// This file is a convenience, not an authority: everything in it is re-derivable from the chain by
/// `zanary-report verify-anchor`, which is why it may carry the beacon's viewing key — that key only
/// ever reads this public log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorRecord {
    pub anchor: Anchor,
    /// The memo text as broadcast (prefix + payload).
    pub memo: String,
    pub txid: String,
    /// Height that mined the anchoring transaction.
    pub height: u32,
    /// The beacon receiver the memo was paid to, and its diversifier index — the anchor number
    /// (CAₙ) is the index, so anchors are countable without linking them by destination.
    pub beacon_receiver: String,
    pub beacon_receiver_index: String,
    /// The beacon's unified full viewing key. Published on purpose: it decrypts only anchor memos.
    pub beacon_ufvk: String,
    pub chain: String,
    /// Which wallet paid for the anchor (a label, never an identity — see the module docs).
    pub payer_label: String,
}

impl AnchorRecord {
    pub fn from_path(path: &str) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading the anchor record {path}"))?;
        serde_json::from_str(&text).with_context(|| format!("parsing the anchor record {path}"))
    }

    pub fn write(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)
            .with_context(|| format!("writing {}", path.display()))
    }

    /// Re-checks that this record describes the same report: every hash in the payload must be the
    /// one the report itself hashes to. Used by the probe before it broadcasts, so a stale or
    /// hand-edited record cannot become an on-chain claim.
    pub fn check_against(&self, report: &Report) -> Result<()> {
        let hash = crate::hash_report(report)?;
        let root = crate::evidence_root(&report.evidence)?;
        let disagreements = self.anchor.disagreements(report, &hash, &root);
        if !disagreements.is_empty() {
            bail!(
                "the anchor record does not describe this report:\n  {}",
                disagreements.join("\n  ")
            );
        }
        Ok(())
    }
}

/// A pointer to an anchor, as the claim page shows it: the transaction that carried the memo and
/// the height that mined it — the only timestamp in this system nobody has to be trusted for.
#[derive(Debug, Clone)]
pub struct AnchorRef {
    pub txid: String,
    pub height: u32,
}

/// The fields of an anchor payload. Used to refuse anything else in a memo that claims the prefix.
const KNOWN_FIELDS: &[&str] = &[
    "v",
    "chain",
    "flow_commit",
    "report_hash",
    "evidence_root",
    "verified_at_height",
];

/// One anchor found on chain.
#[derive(Debug, Clone)]
pub struct AnchorHit {
    pub anchor: Anchor,
    pub txid: String,
    /// The height that mined the anchoring transaction — the timestamp, and the only one that counts.
    pub height: u32,
    /// Which Orchard output of that transaction carried the memo.
    pub output_index: usize,
}

/// Finds every anchor memo in `[since, tip]` that the beacon's viewing key can decrypt, earliest
/// first.
///
/// This is the stranger's path: a node, a chain, the published beacon viewing key, and nothing from
/// ZANARY. `getblock <hash> 2` returns each block's transactions with their raw bytes, so the whole
/// scan is two RPC calls per block and no per-transaction calls at all.
///
/// A scan that finds nothing must never look like a scan that did not run: the caller reports how
/// many blocks and transactions were read, and how many anchor memos they carried.
pub fn scan(node: &NodeRpc, chain: &str, beacon_ufvk: &str, since: u32) -> Result<ScanOutcome> {
    let params = ChainParams::for_chain(chain)?;
    let ufvk = UnifiedFullViewingKey::decode(&params, beacon_ufvk)
        .map_err(|e| anyhow!("the beacon's viewing key is not a UFVK for {chain}: {e}"))?;
    let mut keys: HashMap<AccountId, UnifiedFullViewingKey> = HashMap::new();
    keys.insert(AccountId::ZERO, ufvk);

    let tip = node.tip_height()?;
    let mut hits: Vec<AnchorHit> = Vec::new();
    let mut blocks_read = 0usize;
    let mut transactions_read = 0usize;
    for height in since..=tip {
        let transactions = node
            .block_transactions(height)
            .with_context(|| format!("reading block {height}"))?;
        blocks_read += 1;
        for (txid, raw) in transactions {
            transactions_read += 1;
            let tx = match Transaction::read(
                &raw[..],
                BranchId::for_height(&params, BlockHeight::from_u32(height)),
            ) {
                Ok(tx) => tx,
                // A transaction this code cannot parse is not silently an absent anchor: say so.
                Err(e) => bail!("cannot parse transaction {txid} at height {height}: {e}"),
            };
            let decrypted = decrypt_transaction(
                &params,
                Some(BlockHeight::from_u32(height)),
                Some(BlockHeight::from_u32(tip)),
                &tx,
                &keys,
            );
            for output in decrypted.orchard_outputs() {
                let text = crate::anchor::memo_text_of(output.memo().as_slice());
                if let Some(anchor) = Anchor::from_memo_text(&text)? {
                    hits.push(AnchorHit {
                        anchor,
                        txid: txid.clone(),
                        height,
                        output_index: output.index(),
                    });
                }
            }
        }
    }
    hits.sort_by_key(|hit| (hit.height, hit.output_index));
    Ok(ScanOutcome {
        hits,
        blocks_read,
        transactions_read,
        tip,
        since,
    })
}

/// What a scan read and what it found.
pub struct ScanOutcome {
    pub hits: Vec<AnchorHit>,
    pub blocks_read: usize,
    pub transactions_read: usize,
    pub tip: u32,
    pub since: u32,
}

impl ScanOutcome {
    /// Anchors carrying this evidence root, earliest first.
    pub fn with_root<'a>(&'a self, evidence_root: &str) -> Vec<&'a AnchorHit> {
        self.hits
            .iter()
            .filter(|hit| hit.anchor.evidence_root == evidence_root)
            .collect()
    }
}

/// Renders decrypted memo bytes: trailing NUL padding removed, lossy UTF-8.
///
/// The memo field is zero-padded to 512 bytes; an anchor that came back with padding would fail to
/// parse for a reason that has nothing to do with the anchor.
pub fn memo_text_of(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .rposition(|b| *b != 0)
        .map(|i| i + 1)
        .unwrap_or(0);
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Anchor {
        Anchor {
            v: ANCHOR_VERSION,
            chain: "regtest".into(),
            flow_commit: "aa".repeat(32),
            report_hash: "bb".repeat(32),
            evidence_root: "cc".repeat(32),
            verified_at_height: 616,
        }
    }

    #[test]
    fn memo_text_round_trips() {
        let anchor = sample();
        let text = anchor.memo_text().unwrap();
        assert!(text.starts_with(MEMO_PREFIX));
        assert_eq!(Anchor::from_memo_text(&text).unwrap(), Some(anchor));
    }

    #[test]
    fn a_non_anchor_memo_is_none_and_a_broken_one_is_an_error() {
        assert_eq!(
            Anchor::from_memo_text("zanary: invoice 4471").unwrap(),
            None
        );
        assert_eq!(Anchor::from_memo_text("").unwrap(), None);
        // Prefix matches, so it claims to be an anchor: it must fail loudly, never parse as absent.
        assert!(Anchor::from_memo_text(&format!("{MEMO_PREFIX}not json")).is_err());
        assert!(Anchor::from_memo_text(&format!("{MEMO_PREFIX}{{}}")).is_err());
    }

    #[test]
    fn an_undefined_field_is_refused() {
        let mut value = serde_json::to_value(sample()).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("payer".to_string(), serde_json::json!("kiter"));
        let err = Anchor::from_memo_text(&format!("{MEMO_PREFIX}{value}"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("payer"), "{err}");
    }

    #[test]
    fn an_unknown_version_is_refused() {
        let mut anchor = sample();
        anchor.v = 99;
        // The prefix says v1 and the payload says 99: parse the payload first, then refuse it.
        let json = serde_json::to_string(&anchor).unwrap();
        let err = Anchor::from_memo_text(&format!("{MEMO_PREFIX}{json}"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("version 99"), "{err}");
    }
}
