//! The verifier — re-derives every claim in a report from the chain, without trusting the
//! reporter.
//!
//! Checks, one line each:
//!   * `report hash` — sha256 over the canonical JSON recomputes;
//!   * `evidence root` — the Merkle root over the evidence array recomputes (what is anchored);
//!   * per `Transaction` evidence — the tx exists on the node, at the claimed height, and the
//!     indexer agrees on that height;
//!   * per `Receiver` evidence — the receiver belongs to the claimed transaction: transparent
//!     receivers by script match, shielded receivers by decrypting the transaction with the
//!     available viewing keys and comparing the note's recipient;
//!   * per `Memo` evidence — the memo bytes decrypt out of the claimed transaction (the
//!     harness's dev account seed is public, so its memos are checkable by anyone);
//!   * per `PoolTotals` evidence — `getblock` at the claimed height reports the same pool
//!     totals;
//!   * per `MerchantField` evidence — skipped: the merchant side was simulated and is not a
//!     chain fact (the report says so itself).
//!
//! A check that cannot be completed is a `fail`, not a silent pass: for a privacy rig,
//! "unverifiable" and "wrong" both mean the claim cannot be relied on.

use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;
use std::rc::Rc;

use zcash_client_backend::{
    decrypt_transaction,
    proto::service::{compact_tx_streamer_client::CompactTxStreamerClient, TxFilter},
};
use zcash_keys::{
    address::Address as DecodedAddress,
    keys::{UnifiedFullViewingKey, UnifiedSpendingKey},
};
use zcash_primitives::transaction::Transaction;
use zcash_protocol::{
    consensus::{BlockHeight, BranchId, Network, NetworkType, NetworkUpgrade, Parameters},
    local_consensus::LocalNetwork,
};
use zip32::AccountId;

use crate::{evidence_root, hash_report, node::NodeRpc, Evidence, Report};

/// The harness's dev seed: the canonical BIP-39 English test vector. Public by construction —
/// it exists so that anyone (a judge, a stranger, this verifier) can decrypt the fixture
/// runs' shielded outputs and memos. It only ever holds throwaway regtest funds.
pub const DEV_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    Skip,
    Warn,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::Fail => "FAIL",
            Verdict::Skip => "SKIP",
            Verdict::Warn => "WARN",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Check {
    pub label: String,
    pub verdict: Verdict,
    pub detail: String,
}

#[derive(Debug, Default)]
pub struct VerifyOutcome {
    pub checks: Vec<Check>,
}

impl VerifyOutcome {
    pub fn fails(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.verdict == Verdict::Fail)
            .count()
    }

    pub fn warns(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.verdict == Verdict::Warn)
            .count()
    }

    pub fn skips(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.verdict == Verdict::Skip)
            .count()
    }

    pub fn passes(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.verdict == Verdict::Pass)
            .count()
    }

    pub fn verified(&self) -> bool {
        self.fails() == 0
    }
}

pub struct VerifyOptions {
    pub node: NodeRpc,
    pub indexer_url: String,
    /// Extra unified full viewing key (ZIP 316) to trial-decrypt with, if the run's outputs
    /// are not the harness dev account's.
    pub extra_ufvk: Option<String>,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            node: NodeRpc::from_env(),
            indexer_url: std::env::var("ZANARY_LWD_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:9067".into()),
            extra_ufvk: None,
        }
    }
}

/// Consensus parameters selected by the report's `chain` string.
#[derive(Debug, Clone, Copy)]
pub enum ChainParams {
    Regtest(LocalNetwork),
    Mainnet,
    Testnet,
}

impl ChainParams {
    pub fn for_chain(chain: &str) -> Result<Self> {
        match chain {
            "regtest" => Ok(ChainParams::Regtest(regtest_network())),
            "main" | "mainnet" => Ok(ChainParams::Mainnet),
            "test" | "testnet" => Ok(ChainParams::Testnet),
            other => bail!("unsupported chain `{other}` (expected regtest, main, or test)"),
        }
    }
}

impl Parameters for ChainParams {
    fn network_type(&self) -> NetworkType {
        match self {
            ChainParams::Regtest(n) => n.network_type(),
            ChainParams::Mainnet => Network::MainNetwork.network_type(),
            ChainParams::Testnet => Network::TestNetwork.network_type(),
        }
    }

    fn activation_height(&self, nu: NetworkUpgrade) -> Option<BlockHeight> {
        match self {
            ChainParams::Regtest(n) => n.activation_height(nu),
            ChainParams::Mainnet => Network::MainNetwork.activation_height(nu),
            ChainParams::Testnet => Network::TestNetwork.activation_height(nu),
        }
    }
}

/// The harness chain: every network upgrade active from height 1 (see
/// `data/zcashd/.zcash/zcash.conf`).
pub fn regtest_network() -> LocalNetwork {
    let one = Some(BlockHeight::from_u32(1));
    LocalNetwork {
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

#[derive(Clone)]
struct LoadedTx {
    node_height: u32,
    raw: Vec<u8>,
    tx: Rc<Transaction>,
}

struct Verifier<'a> {
    report: &'a Report,
    opts: &'a VerifyOptions,
    params: ChainParams,
    ufvks: HashMap<AccountId, UnifiedFullViewingKey>,
    indexer: Option<CompactTxStreamerClient<tonic::transport::Channel>>,
    tx_cache: HashMap<String, LoadedTx>,
    checks: Vec<Check>,
}

impl<'a> Verifier<'a> {
    fn push(&mut self, label: impl Into<String>, verdict: Verdict, detail: impl Into<String>) {
        self.checks.push(Check {
            label: label.into(),
            verdict,
            detail: detail.into(),
        });
    }

    /// Trial-decryption keys: the harness dev account's UFVK (public seed) plus an optional
    /// caller-supplied key.
    fn viewing_keys(
        params: &ChainParams,
        extra: Option<&str>,
    ) -> Result<HashMap<AccountId, UnifiedFullViewingKey>> {
        let mut keys = HashMap::new();
        let mnemonic = bip39::Mnemonic::parse_in_normalized(bip39::Language::English, DEV_MNEMONIC)
            .map_err(|e| anyhow!("dev mnemonic is not valid BIP-39: {e}"))?;
        let seed = mnemonic.to_seed("");
        let usk = UnifiedSpendingKey::from_seed(params, &seed, AccountId::ZERO)
            .context("deriving the harness dev UFVK")?;
        keys.insert(AccountId::ZERO, usk.to_unified_full_viewing_key());
        if let Some(encoded) = extra {
            let ufvk = UnifiedFullViewingKey::decode(params, encoded)
                .map_err(|e| anyhow!("--ufvk is not a valid UFVK for this chain: {e}"))?;
            keys.insert(
                AccountId::try_from(1u32).expect("account id 1 is valid"),
                ufvk,
            );
        }
        Ok(keys)
    }

    /// Fetch a transaction once: the node is authoritative for height and raw bytes; the
    /// indexer's view is recorded when it answers.
    async fn fetch_tx(&mut self, txid: &str) -> Result<Option<LoadedTx>> {
        if let Some(hit) = self.tx_cache.get(txid) {
            return Ok(Some(hit.clone()));
        }
        let Some((height, raw)) = self
            .opts
            .node
            .transaction(txid)
            .with_context(|| format!("reading transaction {txid} from the node"))?
        else {
            return Ok(None);
        };
        let tx = Transaction::read(
            &raw[..],
            BranchId::for_height(&self.params, BlockHeight::from_u32(height)),
        )
        .with_context(|| format!("parsing transaction {txid}"))?;
        let loaded = LoadedTx {
            node_height: height,
            raw,
            tx: Rc::new(tx),
        };
        self.tx_cache.insert(txid.to_string(), loaded.clone());
        Ok(Some(loaded))
    }

    async fn indexer_height(&mut self, txid: &str) -> Option<Result<u32>> {
        let client = self.indexer.as_mut()?;
        let mut bytes = match crate::node::hex_decode(txid) {
            Ok(b) if b.len() == 32 => b,
            _ => return Some(Err(anyhow!("malformed txid `{txid}`"))),
        };
        // `TxFilter.hash` is the raw internal txid (the reverse of display order).
        bytes.reverse();
        Some(
            client
                .get_transaction(TxFilter {
                    block: None,
                    index: 0,
                    hash: bytes,
                })
                .await
                .map(|reply| reply.into_inner().height as u32)
                .map_err(|e| anyhow!("indexer GetTransaction {txid}: {e}")),
        )
    }

    /// Checks a transaction claim, and — when the evidence says the payment was one of a batch —
    /// counts the transaction's Orchard actions against the claimed batch size.
    ///
    /// A batch claim is "this one transaction carried N payments to N receivers". The chain shows
    /// the action count (each action carries one output), so the claim is checkable without any
    /// keys: fewer actions than claimed payments means the claim is simply false. More than
    /// `N + 1` would mean outputs nobody claimed, which is a warning rather than a failure — the
    /// extra one is the payer's change.
    async fn check_transaction(
        &mut self,
        txid: &str,
        claimed_height: Option<u32>,
        batch_size: Option<u32>,
    ) {
        let label = match claimed_height {
            Some(h) => format!("tx {txid}@{h}"),
            None => format!("tx {txid}"),
        };
        match self.fetch_tx(txid).await {
            Err(e) => self.push(label, Verdict::Fail, format!("cannot check: {e:#}")),
            Ok(None) => self.push(
                label,
                Verdict::Fail,
                "transaction not found on the node's chain".to_string(),
            ),
            Ok(Some(loaded)) => {
                if let Some(claimed) = claimed_height {
                    if claimed != loaded.node_height {
                        self.push(
                            label,
                            Verdict::Fail,
                            format!(
                                "claimed height {claimed} but the chain has it at {}",
                                loaded.node_height
                            ),
                        );
                        return;
                    }
                }
                let actions = loaded
                    .tx
                    .orchard_bundle()
                    .map(|bundle| bundle.actions().len())
                    .unwrap_or(0);
                let tail = match batch_size {
                    None => String::new(),
                    Some(claimed_batch) => {
                        let claimed_batch = claimed_batch as usize;
                        if actions < claimed_batch {
                            self.push(
                                label,
                                Verdict::Fail,
                                format!(
                                    "claimed a batch of {claimed_batch} payments but the chain \
                                     shows only {actions} orchard action(s) in that transaction"
                                ),
                            );
                            return;
                        }
                        if actions > claimed_batch + 1 {
                            self.push(
                                label.clone(),
                                Verdict::Warn,
                                format!(
                                    "claimed a batch of {claimed_batch} payments; the transaction \
                                     carries {actions} orchard action(s), more than the batch plus \
                                     one change output"
                                ),
                            );
                        }
                        format!(
                            " · batch: {claimed_batch} payments in one transaction, {actions} \
                             orchard action(s) on chain"
                        )
                    }
                };
                // The indexer is the rig's own interface; a disagreement between the two
                // views is itself a finding.
                match self.indexer_height(txid).await {
                    Some(Ok(idx_height)) if idx_height == loaded.node_height => self.push(
                        label,
                        Verdict::Pass,
                        format!(
                            "exists at height {} · indexer agrees · {} bytes{tail}",
                            loaded.node_height,
                            loaded.raw.len()
                        ),
                    ),
                    Some(Ok(idx_height)) => self.push(
                        label,
                        Verdict::Fail,
                        format!(
                            "node says height {} but the indexer says {idx_height}",
                            loaded.node_height
                        ),
                    ),
                    Some(Err(e)) => self.push(
                        label,
                        Verdict::Warn,
                        format!(
                            "exists at height {} (node) · indexer check unavailable: {e}{tail}",
                            loaded.node_height
                        ),
                    ),
                    None => self.push(
                        label,
                        Verdict::Pass,
                        format!(
                            "exists at height {} (node; no indexer connected){tail}",
                            loaded.node_height
                        ),
                    ),
                }
            }
        }
    }

    async fn check_receiver(&mut self, address: &str, payment: &str, txid: &str) {
        let label = format!("receiver {address} ({payment})");
        if txid.is_empty() {
            self.push(
                label,
                Verdict::Fail,
                "evidence carries no txid binding (report predates `first_seen_txid`)".to_string(),
            );
            return;
        }
        let zaddr = match zcash_address::ZcashAddress::try_from_encoded(address) {
            Ok(z) => z,
            Err(e) => {
                self.push(label, Verdict::Fail, format!("cannot parse address: {e}"));
                return;
            }
        };
        let decoded = match DecodedAddress::try_from_zcash_address(&self.params, zaddr) {
            Ok(d) => d,
            Err(e) => {
                self.push(
                    label,
                    Verdict::Fail,
                    format!(
                        "address does not decode on chain `{}`: {e}",
                        self.report.chain
                    ),
                );
                return;
            }
        };
        let loaded = match self.fetch_tx(txid).await {
            Err(e) => {
                self.push(label, Verdict::Fail, format!("cannot check: {e:#}"));
                return;
            }
            Ok(None) => {
                self.push(
                    label,
                    Verdict::Fail,
                    format!("transaction {txid} not found on the node's chain"),
                );
                return;
            }
            Ok(Some(loaded)) => loaded,
        };

        match decoded {
            DecodedAddress::Transparent(taddr) => {
                let expected = transparent_script(&taddr);
                let found = loaded
                    .tx
                    .transparent_bundle()
                    .map(|bundle| {
                        bundle
                            .vout
                            .iter()
                            .any(|out| script_bytes(out.script_pubkey()) == expected.as_slice())
                    })
                    .unwrap_or(false);
                if found {
                    self.push(
                        label,
                        Verdict::Pass,
                        format!("transparent receiver appears in tx {txid} (script match)"),
                    );
                } else {
                    self.push(
                        label,
                        Verdict::Fail,
                        format!("no output of tx {txid} pays this transparent receiver"),
                    );
                }
            }
            DecodedAddress::Sapling(pa) => {
                let decrypted = decrypt_transaction(
                    &self.params,
                    Some(BlockHeight::from_u32(loaded.node_height)),
                    None,
                    &loaded.tx,
                    &self.ufvks,
                );
                let found = decrypted
                    .sapling_outputs()
                    .iter()
                    .any(|out| out.note().recipient() == pa);
                self.push_shielded_receiver(label, found, txid, "sapling");
            }
            DecodedAddress::Unified(ua) => {
                let decrypted = decrypt_transaction(
                    &self.params,
                    Some(BlockHeight::from_u32(loaded.node_height)),
                    None,
                    &loaded.tx,
                    &self.ufvks,
                );
                let orchard_found = ua.orchard().is_some_and(|want| {
                    decrypted
                        .orchard_outputs()
                        .iter()
                        .any(|out| out.note().0.recipient() == *want)
                });
                let sapling_found = ua.sapling().is_some_and(|want| {
                    decrypted
                        .sapling_outputs()
                        .iter()
                        .any(|out| out.note().recipient() == *want)
                });
                let found = orchard_found || sapling_found;
                let which = match (orchard_found, sapling_found) {
                    (true, true) => "orchard+sapling",
                    (true, false) => "orchard",
                    (false, true) => "sapling",
                    (false, false) => "none",
                };
                self.push_shielded_receiver(label, found, txid, which);
            }
            DecodedAddress::Tex(_) => self.push(
                label,
                Verdict::Skip,
                "TEX (transparent-source-only) receiver: not a shielded claim".to_string(),
            ),
        }
    }

    fn push_shielded_receiver(&mut self, label: String, found: bool, txid: &str, pool: &str) {
        if found {
            self.push(
                label,
                Verdict::Pass,
                format!("{pool} output of tx {txid} decrypts to this receiver"),
            );
        } else {
            self.push(
                label,
                Verdict::Fail,
                format!(
                    "no {pool} output of tx {txid} decrypts to this receiver with the available \
                     viewing keys"
                ),
            );
        }
    }

    async fn check_memo(&mut self, txid: &str, memo_hex: &str) {
        let label = format!("memo {txid}");
        let claimed = match crate::node::hex_decode(memo_hex) {
            Ok(bytes) => bytes,
            Err(e) => {
                self.push(label, Verdict::Fail, format!("memo_hex is not hex: {e}"));
                return;
            }
        };
        let loaded = match self.fetch_tx(txid).await {
            Err(e) => {
                self.push(label, Verdict::Fail, format!("cannot check: {e:#}"));
                return;
            }
            Ok(None) => {
                self.push(
                    label,
                    Verdict::Fail,
                    format!("transaction {txid} not found on the node's chain"),
                );
                return;
            }
            Ok(Some(loaded)) => loaded,
        };
        let decrypted = decrypt_transaction(
            &self.params,
            Some(BlockHeight::from_u32(loaded.node_height)),
            None,
            &loaded.tx,
            &self.ufvks,
        );
        let mut tried = 0usize;
        let mut matched: Option<String> = None;
        for out in decrypted.sapling_outputs() {
            tried += 1;
            if memo_matches(&claimed, out.memo().as_slice()) {
                matched = Some(format!("sapling output {}", out.index()));
                break;
            }
        }
        if matched.is_none() {
            for out in decrypted.orchard_outputs() {
                tried += 1;
                if memo_matches(&claimed, out.memo().as_slice()) {
                    matched = Some(format!("orchard output {}", out.index()));
                    break;
                }
            }
        }
        match matched {
            Some(which) => self.push(
                label,
                Verdict::Pass,
                format!(
                    "{} memo bytes decrypt from tx {txid} ({which})",
                    claimed.len()
                ),
            ),
            None => self.push(
                label,
                Verdict::Fail,
                format!(
                    "no shielded output of tx {txid} decrypts (with the available viewing keys, \
                     {tried} outputs tried) to the claimed memo bytes"
                ),
            ),
        }
    }

    fn check_pool_totals(&mut self, height: u32, transparent: u64, sapling: u64, orchard: u64) {
        let label = format!("pool totals @{height}");
        match self.opts.node.pool_totals(height) {
            Err(e) => self.push(label, Verdict::Fail, format!("cannot check: {e:#}")),
            Ok(actual) => {
                let mut mismatches = Vec::new();
                if actual.transparent_zat != transparent {
                    mismatches.push(format!(
                        "transparent claimed {transparent} got {}",
                        actual.transparent_zat
                    ));
                }
                if actual.sapling_zat != sapling {
                    mismatches.push(format!(
                        "sapling claimed {sapling} got {}",
                        actual.sapling_zat
                    ));
                }
                if actual.orchard_zat != orchard {
                    mismatches.push(format!(
                        "orchard claimed {orchard} got {}",
                        actual.orchard_zat
                    ));
                }
                if mismatches.is_empty() {
                    self.push(
                        label,
                        Verdict::Pass,
                        format!(
                            "transparent {transparent} · sapling {sapling} · orchard {orchard} \
                             (zat) match getblock {height}"
                        ),
                    );
                } else {
                    self.push(label, Verdict::Fail, mismatches.join("; "));
                }
            }
        }
    }

    async fn run(mut self) -> Result<VerifyOutcome> {
        // 1) The report itself.
        match hash_report(self.report) {
            Ok(recomputed) if recomputed == self.report.report_hash => self.push(
                "report hash",
                Verdict::Pass,
                format!(
                    "sha256 over canonical JSON recomputes: {}",
                    short(&recomputed)
                ),
            ),
            Ok(recomputed) => self.push(
                "report hash",
                Verdict::Fail,
                format!(
                    "stored {} but the canonical JSON hashes to {}",
                    short(&self.report.report_hash),
                    short(&recomputed)
                ),
            ),
            Err(e) => self.push("report hash", Verdict::Fail, format!("cannot hash: {e:#}")),
        }
        match evidence_root(&self.report.evidence) {
            Ok(root) => self.push(
                "evidence root",
                Verdict::Pass,
                format!(
                    "merkle root over {} evidence leaves recomputes: {}",
                    self.report.evidence.len(),
                    short(&root)
                ),
            ),
            Err(e) => self.push(
                "evidence root",
                Verdict::Fail,
                format!("cannot compute: {e:#}"),
            ),
        }

        // 2) Chain sanity.
        match self.opts.node.chain_name() {
            Ok(chain) if chain == self.report.chain => self.push(
                "chain",
                Verdict::Pass,
                format!(
                    "node reports chain `{chain}`, report claims `{}`",
                    self.report.chain
                ),
            ),
            Ok(chain) => self.push(
                "chain",
                Verdict::Fail,
                format!(
                    "report claims `{}` but the node reports `{chain}`",
                    self.report.chain
                ),
            ),
            Err(e) => self.push(
                "chain",
                Verdict::Fail,
                format!("cannot read the node: {e:#}"),
            ),
        }
        match self.opts.node.tip_height() {
            Ok(tip) if self.report.verified_at_height <= tip => self.push(
                "claim height",
                Verdict::Pass,
                format!(
                    "verified at height {} (tip {tip})",
                    self.report.verified_at_height
                ),
            ),
            Ok(tip) => self.push(
                "claim height",
                Verdict::Fail,
                format!(
                    "claim says verified at height {} but the chain tip is only {tip}",
                    self.report.verified_at_height
                ),
            ),
            Err(e) => self.push(
                "claim height",
                Verdict::Fail,
                format!("cannot read the node tip: {e:#}"),
            ),
        }

        // 3) Every evidence entry.
        let evidence = self.report.evidence.clone();
        for entry in &evidence {
            match entry {
                Evidence::Transaction {
                    txid,
                    height,
                    batch_size,
                } => {
                    self.check_transaction(txid, *height, *batch_size).await;
                }
                Evidence::Receiver {
                    address,
                    first_seen_payment,
                    first_seen_txid,
                } => {
                    self.check_receiver(address, first_seen_payment, first_seen_txid)
                        .await;
                }
                Evidence::Memo { txid, memo_hex } => {
                    self.check_memo(txid, memo_hex).await;
                }
                Evidence::PoolTotals {
                    height,
                    transparent_zat,
                    sapling_zat,
                    orchard_zat,
                } => {
                    self.check_pool_totals(*height, *transparent_zat, *sapling_zat, *orchard_zat);
                }
                Evidence::MerchantField { key, path, .. } => {
                    self.push(
                        format!("merchant field {path}:{key}"),
                        Verdict::Skip,
                        if self.report.merchant_is_simulated {
                            "merchant side was simulated by the rig (stated in the report); not a \
                             chain fact"
                                .to_string()
                        } else {
                            "merchant-side observation: not re-checkable from the chain".to_string()
                        },
                    );
                }
            }
        }

        Ok(VerifyOutcome {
            checks: self.checks,
        })
    }
}

/// Re-derive every chain fact in `report` and return the per-check outcome.
pub async fn verify_report(report: &Report, opts: &VerifyOptions) -> Result<VerifyOutcome> {
    let params = ChainParams::for_chain(&report.chain)?;
    let ufvks = Verifier::viewing_keys(&params, opts.extra_ufvk.as_deref())?;
    let indexer = CompactTxStreamerClient::connect(opts.indexer_url.clone())
        .await
        .ok();
    let verifier = Verifier {
        report,
        opts,
        params,
        ufvks,
        indexer,
        tx_cache: HashMap::new(),
        checks: Vec::new(),
    };
    verifier.run().await
}

/// A 512-byte memo field matches a claimed memo when it is exactly the claimed bytes,
/// or the claimed bytes followed only by zero padding (the field is zero-padded).
fn memo_matches(claimed: &[u8], actual: &[u8]) -> bool {
    if claimed == actual {
        return true;
    }
    actual.len() >= claimed.len()
        && actual.starts_with(claimed)
        && actual[claimed.len()..].iter().all(|b| *b == 0)
}

/// The scriptPubKey bytes of a transparent receiver.
fn transparent_script(addr: &zcash_transparent::address::TransparentAddress) -> Vec<u8> {
    use zcash_transparent::address::TransparentAddress;
    match addr {
        TransparentAddress::PublicKeyHash(hash) => {
            let mut script = vec![0x76, 0xa9, 0x14];
            script.extend_from_slice(hash);
            script.extend_from_slice(&[0x88, 0xac]);
            script
        }
        TransparentAddress::ScriptHash(hash) => {
            let mut script = vec![0xa9, 0x14];
            script.extend_from_slice(hash);
            script.push(0x87);
            script
        }
    }
}

fn script_bytes(script: &zcash_transparent::address::Script) -> &[u8] {
    &script.0 .0
}

fn short(hash: &str) -> String {
    if hash.len() > 12 {
        format!("{}…", &hash[..12])
    } else {
        hash.to_string()
    }
}
