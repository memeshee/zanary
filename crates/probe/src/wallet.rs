//! `canary-probe wallet` — bootstrap the CANARY probe's own librustzcash wallet.
//!
//! * creates or opens a `zcash_client_sqlite` wallet database under `$CANARY_DATA_DIR`
//!   (default `./data/wallet`),
//! * derives the unified spending key for ZIP-32 account 0 from the fixed dev seed
//!   phrase — the BIP-39 English test vector, obviously not a real wallet seed — and
//!   imports its unified full viewing key into the wallet for the regtest network
//!   (`LocalNetwork`, matching the harness `nuparams`),
//! * syncs the wallet from the harness indexer at `$CANARY_LWD_URL`
//!   (default `http://127.0.0.1:9067`),
//! * prints a diversified Orchard-only receiver for account 0, the wallet balances
//!   (shielded + transparent), and the synced height.
//!
//! The zcashd node wallet is never touched: zcashd is only the chain and the miner.

use anyhow::{anyhow, bail, Context, Result};
use std::{env, path::PathBuf, sync::Mutex};

use zcash_client_backend::{
    data_api::{
        chain::{error::Error as ChainError, scan_cached_blocks, BlockCache, BlockSource},
        scanning::ScanRange,
        wallet::ConfirmationsPolicy,
        Account as _, AccountBalance, AccountBirthday, AccountPurpose, WalletRead, WalletWrite,
        Zip32Derivation,
    },
    proto::{
        compact_formats::CompactBlock,
        service::{
            compact_tx_streamer_client::CompactTxStreamerClient, BlockId, BlockRange, ChainSpec,
            GetAddressUtxosArg,
        },
    },
    wallet::WalletTransparentOutput,
};
use zcash_client_sqlite::{util::SystemClock, wallet::init::init_wallet_db, WalletDb};
use zcash_keys::{
    encoding::AddressCodec as _,
    keys::{UnifiedAddressRequest, UnifiedSpendingKey},
};
use zcash_protocol::{consensus::BlockHeight, local_consensus::LocalNetwork, value::Zatoshis};
use zcash_script::script;
use zcash_transparent::{
    address::Script,
    bundle::{OutPoint, TxOut},
};
use zip32::{fingerprint::SeedFingerprint, AccountId, DiversifierIndex};

/// The dev seed phrase: the canonical BIP-39 English test vector. TEST ONLY — this
/// mnemonic is public, and the only chain it ever touches is the throwaway regtest
/// harness chain.
const DEV_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Name recorded for the dev account in the wallet database.
const DEV_ACCOUNT_NAME: &str = "canary-dev";

/// Consensus parameters for the harness chain: regtest with every network upgrade
/// zcashd v6.20.0 knows about activated at height 1 — exactly the `nuparams` in
/// `data/zcashd/.zcash/zcash.conf` (confirmed live via `getblockchaininfo.upgrades`).
///
/// `zcash_protocol` has no `Network::Regtest` variant; local chains are described by
/// [`LocalNetwork`], whose activation heights the caller must set to match their node.
fn regtest_network() -> LocalNetwork {
    let one = Some(BlockHeight::from_u32(1));
    LocalNetwork {
        overwinter: one,
        sapling: one,
        blossom: one,
        heartwood: one,
        canopy: one,
        nu5: one,
        nu6: one,
        // zcashd v6.20.0 has no NU6.1/NU6.2/NU6.3, so they stay unactivated — same
        // as the node, which lists only the seven upgrades above.
        nu6_1: None,
        nu6_2: None,
        nu6_3: None,
    }
}

/// Runs the `wallet` subcommand.
pub async fn run() -> Result<()> {
    let endpoint =
        env::var("CANARY_LWD_URL").unwrap_or_else(|_| "http://127.0.0.1:9067".to_string());
    let data_dir =
        PathBuf::from(env::var("CANARY_DATA_DIR").unwrap_or_else(|_| "./data/wallet".to_string()));
    let db_path = data_dir.join("canary-wallet.db");
    let network = regtest_network();

    println!(
        "canary-probe {} wallet — CANARY (ZECATHON, Core & Tooling)",
        env!("CARGO_PKG_VERSION")
    );
    println!("  network           : regtest (throwaway harness chain)");
    println!("  indexer endpoint  : {endpoint}");
    println!("  wallet db         : {}", db_path.display());
    println!("  dev seed phrase   : {DEV_MNEMONIC}");
    println!("                      ^ BIP-39 English test vector — TEST ONLY, holds no real funds");

    // --- 1) Derive the dev account's keys from the seed phrase (zcash_keys). ---
    let mnemonic = bip39::Mnemonic::parse_in_normalized(bip39::Language::English, DEV_MNEMONIC)
        .map_err(|e| anyhow!("dev mnemonic is not valid BIP-39: {e}"))?;
    let seed = mnemonic.to_seed("");
    let usk = UnifiedSpendingKey::from_seed(&network, &seed, AccountId::ZERO)
        .context("deriving unified spending key from the dev seed")?;
    let ufvk = usk.to_unified_full_viewing_key();
    println!("  dev UFVK          : {}", ufvk.encode(&network));

    // --- 2) Create or open the wallet database (zcash_client_sqlite). ---
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("creating wallet data dir {}", data_dir.display()))?;
    let mut db = WalletDb::for_path(&db_path, network, SystemClock, rand::rngs::OsRng)
        .context("opening the wallet database")?;
    // `for_path` only opens the SQLite connection; the schema migrations must be run
    // (idempotently) before the wallet APIs can be used.
    init_wallet_db(&mut db, None).context("initializing the wallet database schema")?;

    let mut client = CompactTxStreamerClient::connect(endpoint.clone())
        .await
        .with_context(|| format!("connecting to the indexer at {endpoint}"))?;

    // --- 3) Import the dev account once; reuse it on later runs (idempotent). ---
    let account_id = match db
        .get_account_for_ufvk(&ufvk)
        .context("checking whether the dev account is already imported")?
    {
        Some(account) => {
            println!(
                "  account           : already imported ({:?}, birthday height {})",
                account.id(),
                account.birthday_height()
            );
            account.id()
        }
        None => {
            // Birthday = the first block the wallet will scan; the chain state prior to
            // it comes from the indexer's tree state one block below.
            //
            // We take that tree state at height 1, not 0: lightwalletd treats a BlockId
            // of height 0 as "unset" (`must specify a block height or ID (hash)`) — and
            // `sync::run` itself asks for `scan_start - 1`, so the first block that can
            // be scanned through this indexer is 2. Nothing is missed on the harness
            // chain: block 1 is a coinbase-only transparent block.
            let tree_state = client
                .get_tree_state(BlockId {
                    height: 1,
                    hash: vec![],
                })
                .await
                .context("GetTreeState at height 1")?
                .into_inner();
            let birthday = AccountBirthday::from_treestate(tree_state, None)
                .map_err(|e| anyhow!("building the account birthday from the tree state: {e}"))?;

            // Spending-purpose import: the probe holds the unified spending key derived
            // from the dev seed, so the wallet may track everything needed to spend from
            // this account (the next milestone shields and spends). The derivation
            // metadata ties the account to the dev seed's ZIP-32 fingerprint.
            let derivation = Zip32Derivation::new(
                SeedFingerprint::from_seed(&seed)
                    .ok_or_else(|| anyhow!("dev seed has an invalid length for ZIP-32"))?,
                AccountId::ZERO,
            );
            let account = db
                .import_account_ufvk(
                    DEV_ACCOUNT_NAME,
                    &ufvk,
                    &birthday,
                    AccountPurpose::Spending {
                        derivation: Some(derivation),
                    },
                    Some("canary dev seed (BIP-39 test vector)"),
                )
                .context("importing the dev UFVK into the wallet")?;
            println!(
                "  account           : imported ({:?}, birthday height {})",
                account.id(),
                account.birthday_height()
            );
            account.id()
        }
    };

    // --- 4) Sync the wallet from the harness indexer. ---
    //
    // `zcash_client_backend::sync::run` (0.24) cannot talk to this harness: it always
    // fetches Ironwood subtree roots, and the harness node (zcashd v6.20.0) has no
    // Ironwood pool — its `z_getsubtreesbyindex` accepts only "sapling"/"orchard" — so
    // `sync::run` dies with:
    //   GetSubtreeRoots: z_getsubtreesbyindex, error: -8:
    //   Requested pool must be "sapling" or "orchard"
    //
    // This loop performs the same steps `sync::run` does — update the chain tip, refresh
    // transparent UTXOs, then download -> scan -> evict each suggested scan range —
    // minus that fast-forward, which is a no-op on this chain: every pool reports zero
    // complete subtrees, so there is nothing to fast-forward and scanning the compact
    // blocks maintains the wallet's note commitment trees exactly.
    let tip = client
        .get_latest_block(ChainSpec {})
        .await
        .context("GetLatestBlock")?
        .into_inner();
    let tip_height = BlockHeight::from_u32(tip.height as u32);
    db.update_chain_tip(tip_height)
        .context("updating the wallet chain tip")?;

    // Refresh transparent UTXOs for every account (mirrors `sync::run`'s `refresh_utxos`
    // step), so transparent funds received by the account's receivers are detected.
    for account in db
        .get_account_ids()
        .context("listing the wallet's accounts")?
    {
        let start_height = db
            .utxo_query_height(account)
            .context("reading the wallet's UTXO query height")?;
        let addresses: Vec<String> = db
            .get_transparent_receivers(account, true, true)
            .context("listing the account's transparent receivers")?
            .keys()
            .map(|address| address.encode(&network))
            .collect();
        if addresses.is_empty() {
            continue;
        }
        let mut stream = client
            .get_address_utxos_stream(GetAddressUtxosArg {
                addresses,
                start_height: u64::from(start_height),
                max_entries: 0,
            })
            .await
            .context("GetAddressUtxos")?
            .into_inner();
        while let Some(reply) = stream
            .message()
            .await
            .context("reading the GetAddressUtxos stream")?
        {
            let output = WalletTransparentOutput::from_parts(
                OutPoint::new(
                    reply.txid[..]
                        .try_into()
                        .map_err(|_| anyhow!("indexer returned a malformed txid"))?,
                    reply
                        .index
                        .try_into()
                        .map_err(|_| anyhow!("indexer returned a malformed output index"))?,
                ),
                TxOut::new(
                    Zatoshis::from_nonnegative_i64(reply.value_zat)
                        .map_err(|_| anyhow!("indexer returned an invalid output value"))?,
                    Script(script::Code(reply.script)),
                ),
                Some(
                    BlockHeight::try_from(reply.height)
                        .map_err(|_| anyhow!("indexer returned an invalid block height"))?,
                ),
                Some(account),
                None,
                None,
            )
            .ok_or_else(|| anyhow!("indexer returned a UTXO with an unrecognized script"))?;
            db.put_received_transparent_utxo(&output)
                .context("storing a transparent UTXO")?;
        }
    }

    // Download -> scan -> evict, one suggested scan range at a time, until the wallet
    // has nothing left to scan.
    let cache = MemoryBlockCache::new();
    let mut iterations = 0u32;
    loop {
        iterations += 1;
        if iterations > 1_000 {
            bail!("wallet sync did not converge after {iterations} scan ranges");
        }
        let ranges = db.suggest_scan_ranges().context("suggesting scan ranges")?;
        let Some(range) = ranges.into_iter().find(|range| !range.is_empty()) else {
            break;
        };

        let priority = range.priority();
        let start = range.block_range().start;
        let end = range.block_range().end - 1;

        let mut stream = client
            .get_block_range(BlockRange {
                start: Some(BlockId {
                    height: u64::from(start),
                    hash: vec![],
                }),
                end: Some(BlockId {
                    height: u64::from(end),
                    hash: vec![],
                }),
                pool_types: vec![],
            })
            .await
            .with_context(|| format!("GetBlockRange {start}..={end}"))?
            .into_inner();
        let mut blocks = Vec::new();
        while let Some(block) = stream
            .message()
            .await
            .with_context(|| format!("reading the GetBlockRange stream for {start}..={end}"))?
        {
            blocks.push(block);
        }
        cache.insert(blocks).await?;

        // The chain state immediately below the range (block `start - 1`), so the scan
        // can maintain the note commitment trees from the right starting point.
        let tree_state = client
            .get_tree_state(BlockId {
                height: u64::from(start - 1),
                hash: vec![],
            })
            .await
            .with_context(|| format!("GetTreeState at height {}", start - 1))?
            .into_inner();
        let from_state = tree_state
            .to_chain_state()
            .context("decoding the tree state below the scan range")?;

        let limit = usize::try_from(u32::from(end) - u32::from(start) + 1)
            .expect("scan range length fits in usize");
        let summary = scan_cached_blocks(&network, &cache, &mut db, start, &from_state, limit)
            .map_err(|e| anyhow!("scanning blocks {start}..={end}: {e}"))?;
        cache.delete(range).await?;

        println!(
            "  scanned           : {} ..= {} ({} blocks, priority {priority:?})",
            summary.scanned_range().start,
            summary.scanned_range().end - 1,
            limit,
        );
    }

    // --- 5) Report: Orchard-only receiver, balances, synced height. ---
    let stored_ufvk = db
        .get_account(account_id)
        .context("reading the dev account back from the wallet")?
        .and_then(|account| account.ufvk().cloned())
        .context("the dev account has no UFVK stored in the wallet")?;

    // `UnifiedAddressRequest::ORCHARD` requires an Orchard receiver and no others, so
    // the encoded address below is a `uregtest1…` Orchard-only receiver.
    let (orchard_receiver, diversifier_index) = stored_ufvk
        .find_address(DiversifierIndex::from(0u32), UnifiedAddressRequest::ORCHARD)
        .map_err(|e| anyhow!("deriving an Orchard-only receiver for account 0: {e}"))?;

    println!();
    println!("  --- wallet state ---");
    println!(
        "  orchard receiver  : {}",
        orchard_receiver.encode(&network)
    );
    println!("  diversifier index : {diversifier_index:?}");

    let summary = db
        .get_wallet_summary(ConfirmationsPolicy::default())
        .context("reading the wallet summary")?;

    match &summary {
        Some(summary) => {
            println!("  chain tip height  : {}", summary.chain_tip_height());
            println!("  fully scanned     : {}", summary.fully_scanned_height());
            match summary.account_balances().get(&account_id) {
                Some(balance) => print_balances(balance),
                None => println!("  balance           : no entry for the dev account"),
            }
        }
        None => {
            let tip = db
                .chain_height()
                .context("reading the wallet chain height")?;
            println!("  chain tip height  : {tip:?}");
            println!("  balance           : no wallet summary available yet");
        }
    }

    println!();
    println!("  next: mine coinbase to the receiver above, then shield and spend it with a memo");
    Ok(())
}

/// Prints the per-pool balances of one account.
fn print_balances(balance: &AccountBalance) {
    println!("  balance (total)   : {}", format_zec(balance.total()));
    println!(
        "    shielded orchard: {}",
        format_zec(balance.orchard_balance().total())
    );
    println!(
        "    shielded sapling: {}",
        format_zec(balance.sapling_balance().total())
    );
    println!(
        "    transparent     : {}",
        format_zec(balance.unshielded_balance().total())
    );
    println!(
        "    spendable now   : {}",
        format_zec(balance.spendable_value())
    );
}

/// Formats zatoshis as a ZEC amount with 8 decimal places, e.g. `0.00000000 ZEC`.
fn format_zec(value: Zatoshis) -> String {
    let zats = value.into_u64();
    format!("{}.{:08} ZEC", zats / 100_000_000, zats % 100_000_000)
}

/// A minimal in-memory compact-block cache for the wallet sync loop.
///
/// `zcash_client_sqlite` 0.22 exposes no public `BlockCache` implementation (`FsBlockDb`
/// sits behind its `unstable` feature, and the other cache is test-only), so the probe
/// carries its own, modelled on the trait's documented example. The harness chain is tiny
/// and fully local, so cached compact blocks live in RAM and die with the process.
struct MemoryBlockCache {
    blocks: Mutex<Vec<CompactBlock>>,
}

impl MemoryBlockCache {
    fn new() -> Self {
        Self {
            blocks: Mutex::new(Vec::new()),
        }
    }

    /// Cached blocks whose height falls in `range`, ascending by height.
    fn blocks_in(&self, range: &ScanRange) -> Vec<CompactBlock> {
        let mut blocks: Vec<CompactBlock> = self
            .blocks
            .lock()
            .expect("block cache mutex poisoned")
            .iter()
            .filter(|block| {
                range
                    .block_range()
                    .contains(&BlockHeight::from_u32(block.height as u32))
            })
            .cloned()
            .collect();
        blocks.sort_by_key(|block| block.height);
        blocks
    }
}

/// Error type for [`MemoryBlockCache`]; the block traits need `std::error::Error + Send + Sync`.
#[derive(Debug)]
struct CacheError(String);

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "block cache: {}", self.0)
    }
}

impl std::error::Error for CacheError {}

impl BlockSource for MemoryBlockCache {
    type Error = CacheError;

    fn with_blocks<F, WalletErrT>(
        &self,
        from_height: Option<BlockHeight>,
        limit: Option<usize>,
        mut with_block: F,
    ) -> Result<(), ChainError<WalletErrT, Self::Error>>
    where
        F: FnMut(CompactBlock) -> Result<(), ChainError<WalletErrT, Self::Error>>,
    {
        let mut blocks: Vec<CompactBlock> = self
            .blocks
            .lock()
            .expect("block cache mutex poisoned")
            .iter()
            .filter(|block| {
                from_height.is_none_or(|from| BlockHeight::from_u32(block.height as u32) >= from)
            })
            .cloned()
            .collect();
        blocks.sort_by_key(|block| block.height);

        // Like `FsBlockDb`: a requested start height must actually be present.
        if let (Some(from), Some(first)) = (from_height, blocks.first()) {
            if BlockHeight::from_u32(first.height as u32) != from {
                return Err(ChainError::BlockSource(CacheError(format!(
                    "requested height {from} is not present in the block cache"
                ))));
            }
        }

        for block in blocks.into_iter().take(limit.unwrap_or(usize::MAX)) {
            with_block(block)?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl BlockCache for MemoryBlockCache {
    fn get_tip_height(
        &self,
        range: Option<&ScanRange>,
    ) -> Result<Option<BlockHeight>, Self::Error> {
        let blocks = self.blocks.lock().expect("block cache mutex poisoned");
        Ok(blocks
            .iter()
            .filter(|block| {
                range.is_none_or(|range| {
                    range
                        .block_range()
                        .contains(&BlockHeight::from_u32(block.height as u32))
                })
            })
            .map(|block| block.height)
            .max()
            .map(|height| BlockHeight::from_u32(height as u32)))
    }

    async fn read(&self, range: &ScanRange) -> Result<Vec<CompactBlock>, Self::Error> {
        // Short reads are allowed, but what is returned must be contiguous and start at
        // `range.start` (see the `BlockCache` trait docs).
        let start = range.block_range().start;
        let mut expected = u32::from(start);
        let mut out = Vec::new();
        for block in self.blocks_in(range) {
            match (block.height as u32).cmp(&expected) {
                std::cmp::Ordering::Equal => {
                    out.push(block);
                    expected += 1;
                }
                std::cmp::Ordering::Greater => break, // gap: stop at the contiguous prefix
                std::cmp::Ordering::Less => {}        // below the range start; ignore
            }
        }
        if out.is_empty() && !range.block_range().is_empty() {
            return Err(CacheError(format!(
                "no cached block at requested start height {start}"
            )));
        }
        Ok(out)
    }

    async fn insert(&self, compact_blocks: Vec<CompactBlock>) -> Result<(), Self::Error> {
        self.blocks
            .lock()
            .expect("block cache mutex poisoned")
            .extend(compact_blocks);
        Ok(())
    }

    async fn delete(&self, range: ScanRange) -> Result<(), Self::Error> {
        self.blocks
            .lock()
            .expect("block cache mutex poisoned")
            .retain(|block| {
                !range
                    .block_range()
                    .contains(&BlockHeight::from_u32(block.height as u32))
            });
        Ok(())
    }
}
