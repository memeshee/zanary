//! Shared plumbing for every `zanary-probe` subcommand.
//!
//! * regtest consensus parameters (matching the harness `nuparams`),
//! * the dev account's keys (BIP-39 test vector — TEST ONLY),
//! * wallet open/import (`zcash_client_sqlite`),
//! * the indexer sync loop — the documented `sync::run` workaround: the 0.24
//!   `sync::run` unconditionally fetches Ironwood subtree roots that the harness
//!   node (zcashd v6.20.0) cannot serve, so the probe inlines the same
//!   download -> `scan_cached_blocks` -> evict loop minus that fast-forward
//!   (a no-op on this chain: every pool reports zero complete subtrees).
//!
//! The zcashd node wallet is never touched: zcashd is only the chain and the miner.

use anyhow::{anyhow, Context, Result};
use std::{
    env,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};

use zcash_client_backend::{
    data_api::{
        chain::{error::Error as ChainError, scan_cached_blocks, BlockCache, BlockSource},
        scanning::ScanRange,
        wallet::decrypt_and_store_transaction,
        wallet::ConfirmationsPolicy,
        wallet::TargetHeight,
        Account as _, AccountBalance, AccountBirthday, AccountPurpose, WalletRead, WalletSummary,
        WalletWrite, Zip32Derivation,
    },
    proto::{
        compact_formats::CompactBlock,
        service::{
            compact_tx_streamer_client::CompactTxStreamerClient, BlockId, BlockRange, ChainSpec,
            GetAddressUtxosArg, TransparentAddressBlockFilter,
        },
    },
    wallet::WalletTransparentOutput,
};
use zcash_client_sqlite::{util::SystemClock, wallet::init::init_wallet_db, AccountUuid, WalletDb};
use zcash_keys::{
    encoding::AddressCodec as _,
    keys::{UnifiedFullViewingKey, UnifiedSpendingKey},
};
use zcash_protocol::{
    consensus::{BlockHeight, BranchId},
    local_consensus::LocalNetwork,
    value::Zatoshis,
    PoolType, TxId,
};
use zcash_primitives::transaction::Transaction;
use zcash_script::script;
use zcash_transparent::{
    address::Script,
    bundle::{OutPoint, TxOut},
};
use zip32::{fingerprint::SeedFingerprint, AccountId};

/// The dev seed phrase: the canonical BIP-39 English test vector. TEST ONLY — this
/// mnemonic is public, and the only chain it ever touches is the throwaway regtest
/// harness chain.
pub const DEV_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Name recorded for the dev account (ZIP-32 account 0) in the wallet database.
pub const DEV_ACCOUNT_NAME: &str = "zanary-dev";

/// The concrete wallet database type the probe uses everywhere.
pub type ProbeDb = WalletDb<rusqlite::Connection, LocalNetwork, SystemClock, rand::rngs::OsRng>;

/// The concrete indexer client type (tonic channel, no TLS — harness only).
pub type IndexerClient = CompactTxStreamerClient<tonic::transport::Channel>;

/// Consensus parameters for the harness chain: regtest with every network upgrade
/// zcashd v6.20.0 knows about activated at height 1 — exactly the `nuparams` in
/// `data/zcashd/.zcash/zcash.conf` (confirmed live via `getblockchaininfo.upgrades`).
///
/// `zcash_protocol` has no `Network::Regtest` variant; local chains are described by
/// [`LocalNetwork`], whose activation heights the caller must set to match their node.
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
        // zcashd v6.20.0 has no NU6.1/NU6.2/NU6.3, so they stay unactivated — same
        // as the node, which lists only the seven upgrades above.
        nu6_1: None,
        nu6_2: None,
        nu6_3: None,
    }
}

/// Indexer endpoint (`ZANARY_LWD_URL`, default `http://127.0.0.1:9067`).
pub fn endpoint() -> String {
    env::var("ZANARY_LWD_URL").unwrap_or_else(|_| "http://127.0.0.1:9067".to_string())
}

/// Wallet data directory (`ZANARY_DATA_DIR`, default `./data/wallet`).
pub fn data_dir() -> PathBuf {
    PathBuf::from(env::var("ZANARY_DATA_DIR").unwrap_or_else(|_| "./data/wallet".to_string()))
}

/// Path of the wallet database inside [`data_dir`].
pub fn db_path() -> PathBuf {
    data_dir().join("zanary-wallet.db")
}

/// Decodes the dev mnemonic into a 64-byte seed.
pub fn dev_seed() -> Result<[u8; 64]> {
    let mnemonic = bip39::Mnemonic::parse_in_normalized(bip39::Language::English, DEV_MNEMONIC)
        .map_err(|e| anyhow!("dev mnemonic is not valid BIP-39: {e}"))?;
    Ok(mnemonic.to_seed(""))
}

/// Name recorded for ZIP-32 account `account` in the wallet database.
pub fn account_name(account: u32) -> String {
    if account == 0 {
        DEV_ACCOUNT_NAME.to_string()
    } else {
        format!("{DEV_ACCOUNT_NAME}-{account}")
    }
}

/// Derives the unified spending key for ZIP-32 account `account` of the dev seed.
pub fn dev_usk(network: &LocalNetwork, account: u32) -> Result<UnifiedSpendingKey> {
    let seed = dev_seed()?;
    let account_id = AccountId::try_from(account)
        .map_err(|_| anyhow!("ZIP-32 account index {account} is out of range"))?;
    UnifiedSpendingKey::from_seed(network, &seed, account_id)
        .with_context(|| format!("deriving the unified spending key for account {account}"))
}

/// The BIP-39 test vector used for the *merchant* side of a flow.
///
/// A flow's merchant must be a genuinely independent party for its view of a payment
/// to be evidence: different seed, different wallet database, its own sync and its own
/// decryption. That is also what makes "the memo is readable at the recipient" a
/// third-party statement rather than a self-report from the payer's wallet.
pub const MERCHANT_MNEMONIC: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";

/// The anchor beacon's seed: the third canonical BIP-39 English test vector, so anchors come from a
/// wallet that is neither the payer's nor the merchant's. TEST ONLY — and the beacon's *viewing key*
/// is published on purpose: it can only ever decrypt a public log of hashes, which is what lets a
/// stranger check an anchor without trusting ZANARY.
pub const BEACON_MNEMONIC: &str =
    "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";

/// Which wallet a command operates on: its seed phrase, ZIP-32 account, the name it
/// is recorded under, and where its database lives.
#[derive(Clone, Debug)]
pub struct WalletProfile {
    /// BIP-39 seed phrase. Harness-only test vectors — never a real wallet.
    pub mnemonic: String,
    pub account: u32,
    /// Name the account is imported under.
    pub name: String,
    /// Free-text provenance recorded with the import.
    pub source: String,
    pub data_dir: PathBuf,
    /// Database file name inside [`WalletProfile::data_dir`].
    pub db_file: String,
}

impl WalletProfile {
    /// The probe's own wallet: the dev seed in `ZANARY_DATA_DIR` (default
    /// `./data/wallet`), account names exactly as before this type existed.
    pub fn dev(account: u32) -> Self {
        WalletProfile {
            mnemonic: DEV_MNEMONIC.to_string(),
            account,
            name: account_name(account),
            source: "zanary dev seed (BIP-39 test vector)".to_string(),
            data_dir: data_dir(),
            db_file: "zanary-wallet.db".to_string(),
        }
    }

    /// A merchant wallet: its own seed and its own database, so it is a third party.
    pub fn merchant(account: u32) -> Self {
        WalletProfile {
            mnemonic: MERCHANT_MNEMONIC.to_string(),
            account,
            name: if account == 0 {
                "zanary-merchant".to_string()
            } else {
                format!("zanary-merchant-{account}")
            },
            source: "zanary merchant seed (BIP-39 test vector)".to_string(),
            data_dir: PathBuf::from(
                env::var("ZANARY_MERCHANT_DATA_DIR")
                    .unwrap_or_else(|_| "./data/wallet-merchant".to_string()),
            ),
            db_file: "zanary-merchant.db".to_string(),
        }
    }

    /// The anchor beacon: its own seed and database, like the merchant, because an anchor must not
    /// be spendable by the wallet that ran the flow — and because its viewing key is published.
    pub fn beacon(account: u32) -> Self {
        WalletProfile {
            mnemonic: BEACON_MNEMONIC.to_string(),
            account,
            name: if account == 0 {
                "zanary-beacon".to_string()
            } else {
                format!("zanary-beacon-{account}")
            },
            source: "zanary anchor beacon seed (BIP-39 test vector)".to_string(),
            data_dir: PathBuf::from(
                env::var("ZANARY_BEACON_DATA_DIR")
                    .unwrap_or_else(|_| "./data/wallet-beacon".to_string()),
            ),
            db_file: "zanary-beacon.db".to_string(),
        }
    }

    /// Derives the seed from this profile's mnemonic.
    pub fn seed(&self) -> Result<[u8; 64]> {
        seed_from_mnemonic(&self.mnemonic)
    }

    /// Derives this profile's ZIP-32 account key.
    pub fn usk(&self, network: &LocalNetwork) -> Result<UnifiedSpendingKey> {
        let seed = self.seed()?;
        let account_id = AccountId::try_from(self.account)
            .map_err(|_| anyhow!("ZIP-32 account index {} is out of range", self.account))?;
        UnifiedSpendingKey::from_seed(network, &seed, account_id).with_context(|| {
            format!("deriving the unified spending key for account {}", self.account)
        })
    }

    /// Path of this profile's wallet database.
    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join(&self.db_file)
    }
}

/// Decodes a BIP-39 seed phrase into a 64-byte seed.
pub fn seed_from_mnemonic(mnemonic: &str) -> Result<[u8; 64]> {
    let mnemonic = bip39::Mnemonic::parse_in_normalized(bip39::Language::English, mnemonic)
        .map_err(|e| anyhow!("mnemonic is not valid BIP-39: {e}"))?;
    Ok(mnemonic.to_seed(""))
}

/// Creates (if needed) and opens the wallet database, running the schema migrations.
pub fn open_db(data_dir: &PathBuf) -> Result<(ProbeDb, PathBuf)> {
    let db_path = data_dir.join("zanary-wallet.db");
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("creating wallet data dir {}", data_dir.display()))?;
    let mut db = WalletDb::for_path(&db_path, regtest_network(), SystemClock, rand::rngs::OsRng)
        .context("opening the wallet database")?;
    // `for_path` only opens the SQLite connection; the schema migrations must be run
    // (idempotently) before the wallet APIs can be used.
    init_wallet_db(&mut db, None).context("initializing the wallet database schema")?;
    Ok((db, db_path))
}

/// [`open_db`], for any wallet profile (the merchant's database is its own file).
pub fn open_db_for(profile: &WalletProfile) -> Result<(ProbeDb, PathBuf)> {
    let db_path = profile.db_path();
    std::fs::create_dir_all(&profile.data_dir).with_context(|| {
        format!(
            "creating wallet data dir {}",
            profile.data_dir.display()
        )
    })?;
    let mut db = WalletDb::for_path(&db_path, regtest_network(), SystemClock, rand::rngs::OsRng)
        .with_context(|| format!("opening the wallet database {}", db_path.display()))?;
    init_wallet_db(&mut db, None).context("initializing the wallet database schema")?;
    Ok((db, db_path))
}

/// Connects to the harness indexer.
pub async fn connect(endpoint: &str) -> Result<IndexerClient> {
    CompactTxStreamerClient::connect(endpoint.to_string())
        .await
        .with_context(|| format!("connecting to the indexer at {endpoint}"))
}

/// Outcome of [`import_account_if_needed`].
pub struct ImportedAccount {
    pub id: AccountUuid,
    pub birthday_height: BlockHeight,
    pub newly_imported: bool,
}

/// Imports the UFVK for ZIP-32 account `account` of the dev seed into the wallet once;
/// reuses the stored account on later runs (idempotent).
pub async fn import_account_if_needed(
    db: &mut ProbeDb,
    client: &mut IndexerClient,
    ufvk: &UnifiedFullViewingKey,
    seed: &[u8; 64],
    account: u32,
) -> Result<ImportedAccount> {
    import_account_for(db, client, ufvk, seed, &WalletProfile::dev(account)).await
}

/// [`import_account_if_needed`], for any wallet profile: the account's recorded name,
/// provenance and birthday come from the profile, so a merchant wallet (different seed,
/// different database) imports its own account the same way the probe imports the dev one.
pub async fn import_account_for(
    db: &mut ProbeDb,
    client: &mut IndexerClient,
    ufvk: &UnifiedFullViewingKey,
    seed: &[u8; 64],
    profile: &WalletProfile,
) -> Result<ImportedAccount> {
    let account = profile.account;
    if let Some(account) = db
        .get_account_for_ufvk(ufvk)
        .context("checking whether the account is already imported")?
    {
        return Ok(ImportedAccount {
            id: account.id(),
            birthday_height: account.birthday_height(),
            newly_imported: false,
        });
    }

    // Birthday = the first block the wallet will scan; the chain state prior to it
    // comes from the indexer's tree state one block below.
    //
    // We take that tree state at height 1, not 0: lightwalletd treats a BlockId of
    // height 0 as "unset" (`must specify a block height or ID (hash)`) — and
    // `sync::run` itself asks for `scan_start - 1`, so the first block that can be
    // scanned through this indexer is 2. Nothing is missed on the harness chain:
    // block 1 is a coinbase-only transparent block.
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

    // Spending-purpose import: the probe holds the unified spending key derived from
    // the dev seed, so the wallet may track everything needed to spend from this
    // account. The derivation metadata ties the account to the dev seed's ZIP-32
    // fingerprint.
    let derivation = Zip32Derivation::new(
        SeedFingerprint::from_seed(seed)
            .ok_or_else(|| anyhow!("dev seed has an invalid length for ZIP-32"))?,
        AccountId::try_from(account)
            .map_err(|_| anyhow!("ZIP-32 account index {account} is out of range"))?,
    );
    let imported = db
        .import_account_ufvk(
            &profile.name,
            ufvk,
            &birthday,
            AccountPurpose::Spending {
                derivation: Some(derivation),
            },
            Some(profile.source.as_str()),
        )
        .with_context(|| {
            format!(
                "importing the UFVK for account {account} of wallet `{}` into the wallet",
                profile.name
            )
        })?;
    Ok(ImportedAccount {
        id: imported.id(),
        birthday_height: imported.birthday_height(),
        newly_imported: true,
    })
}

/// Syncs the wallet from the harness indexer: update the chain tip, refresh
/// transparent UTXOs, then download -> scan -> evict each suggested scan range.
///
/// This is the same sequence `zcash_client_backend::sync::run` (0.24) performs,
/// minus its unconditional Ironwood subtree-root fetch, which the harness node
/// cannot serve:
///   GetSubtreeRoots: z_getsubtreesbyindex, error: -8:
///   Requested pool must be "sapling" or "orchard"
/// Every pool on this chain reports zero complete subtrees, so there is nothing to
/// fast-forward and scanning the compact blocks maintains the note commitment trees
/// exactly.
pub async fn sync_wallet(
    db: &mut ProbeDb,
    client: &mut IndexerClient,
    network: &LocalNetwork,
    db_path: &Path,
) -> Result<()> {
    let timings = env::var("ZANARY_TIMINGS").is_ok();
    let t0 = std::time::Instant::now();
    let tip = client
        .get_latest_block(ChainSpec {})
        .await
        .context("GetLatestBlock")?
        .into_inner();
    let tip_height = BlockHeight::from_u32(tip.height as u32);
    db.update_chain_tip(tip_height)
        .context("updating the wallet chain tip")?;
    if timings {
        println!("  TIMING   tip: {:?}", t0.elapsed());
    }

    // Refresh transparent UTXOs for every account (mirrors `sync::run`'s
    // `refresh_utxos` step), so transparent funds received by the account's
    // receivers are detected. Note: this is the *only* transparent-output detection
    // path in this stack — an address must be registered in the wallet's `addresses`
    // table (see `ops::transparent_receiver`) for its UTXOs to be seen.
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
            .map(|address| address.encode(network))
            .collect();
        if addresses.is_empty() {
            continue;
        }
        let t_acct = std::time::Instant::now();
        // Unary `GetAddressUtxos` rather than the streaming variant: lightwalletd
        // v0.5.5's per-message stream flush is pathologically slow on this harness
        // (~300 ms per UTXO streamed, tens of seconds for a few dozen outputs), while
        // the unary reply returns the same list in one message. Same indexer call,
        // same data.
        let reply_list = client
            .get_address_utxos(GetAddressUtxosArg {
                addresses,
                start_height: u64::from(start_height),
                max_entries: 0,
            })
            .await
            .context("GetAddressUtxos")?
            .into_inner();
        let mut reply_count = 0usize;
        let mut stored_count = 0usize;
        for reply in reply_list.address_utxos {
            reply_count += 1;
            if env::var("ZANARY_TIMINGS").is_ok()
                && (reply_count == 1 || reply_count % 25 == 0)
            {
                println!(
                    "  TIMING     utxo reply #{reply_count}: {:?}",
                    t_acct.elapsed()
                );
            }

            let txid: [u8; 32] = reply.txid[..]
                .try_into()
                .map_err(|_| anyhow!("indexer returned a malformed txid"))?;
            let output_index: u32 = reply
                .index
                .try_into()
                .map_err(|_| anyhow!("indexer returned a malformed output index"))?;

            // Skip outputs the wallet already knows: the indexer re-reports every
            // UTXO in the gap window on every sync, and `put_received_transparent_utxo`
            // regenerates the account's gap window on each call — an address-derivation
            // loop that costs hundreds of milliseconds per output in a debug build.
            // Only genuinely new outputs are written.
            let known = db
                .get_received_outputs(
                    TxId::from_bytes(txid),
                    TargetHeight::from(tip_height + 1),
                    ConfirmationsPolicy::default(),
                )
                .context("checking whether a transparent output is already known")?
                .iter()
                .any(|output| {
                    output.pool_type() == PoolType::Transparent
                        && output.output_index() == output_index as usize
                });
            if known {
                continue;
            }

            let output = WalletTransparentOutput::from_parts(
                OutPoint::new(
                    txid,
                    output_index
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
            stored_count += 1;
        }
        if env::var("ZANARY_TIMINGS").is_ok() {
            println!(
                "  TIMING   utxo acct {account:?}: {reply_count} reply(s), {stored_count} stored, {:?}",
                t_acct.elapsed()
            );
        }
    }

    // Download -> scan -> evict, one suggested scan range at a time, until the
    // wallet has nothing left to scan.
    if timings {
        println!("  TIMING   utxo refresh: {:?}", t0.elapsed());
    }
    let cache = MemoryBlockCache::new();
    let mut iterations = 0u32;
    loop {
        iterations += 1;
        if iterations > 1_000 {
            anyhow::bail!("wallet sync did not converge after {iterations} scan ranges");
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

        // The chain state immediately below the range (block `start - 1`), so the
        // scan can maintain the note commitment trees from the right starting point.
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
        let summary = scan_cached_blocks(network, &cache, db, start, &from_state, limit)
            .map_err(|e| anyhow!("scanning blocks {start}..={end}: {e}"))?;
        cache.delete(range).await?;

        println!(
            "  scanned           : {} ..= {} ({} blocks, priority {priority:?})",
            summary.scanned_range().start,
            summary.scanned_range().end - 1,
            limit,
        );
    }

    if timings {
        println!("  TIMING   scan loop: {:?}", t0.elapsed());
    }

    // Record in-block transaction indices for the wallet's transparent outputs
    // (see `annotate_transparent_tx_indices`), so the wallet's coinbase-maturity
    // exclusion and coinbase filters work for outputs discovered via
    // `GetAddressUtxos` — the lightwalletd reply carries no `tx_index`.
    annotate_transparent_tx_indices(db_path).await?;
    if timings {
        println!("  TIMING   annotation: {:?}", t0.elapsed());
    }

    // Teach the wallet about transparent spends it did not initiate (see
    // `detect_transparent_spends`): without this, a UTXO spent by a transaction
    // built elsewhere stays "unspent" here, is selected as an input, and the node
    // rejects the result with `Missing inputs` (error_code -25).
    detect_transparent_spends(db, client, network).await?;
    if timings {
        println!("  TIMING   spend detection: {:?}", t0.elapsed());
    }
    Ok(())
}

/// Stores every transaction that involves one of the wallet's transparent receivers
/// and that the wallet has not seen yet, so that the wallet learns about transparent
/// spends it did not initiate.
///
/// Why this exists: a wallet only learns that one of its transparent outputs was
/// spent when it sees the spending transaction. `sync::run` hands that job to its
/// caller through `TransactionDataRequest::TransactionsInvolvingAddress` (one
/// request per stored UTXO, served by an address-scoped transaction lookup), and the
/// probe's sync loop does not run the wallet's request queue. The harness indexer
/// can answer the lookup directly, so the probe sweeps the account's receivers
/// instead of following the queue — a queue entry can also be dropped when a locally
/// built transaction that spends the UTXO is stored but never gets mined, which
/// would otherwise leave the spend undetectable forever.
///
/// Cost: one `GetTaddressTransactions` per receiver, plus a `get_transaction` check
/// per returned transaction. Only transactions the wallet has never seen are
/// decrypted and stored (the sweep is a no-op for a wallet that is already in sync).
async fn detect_transparent_spends(
    db: &mut ProbeDb,
    client: &mut IndexerClient,
    network: &LocalNetwork,
) -> Result<()> {
    let tip = db
        .chain_height()
        .context("reading the wallet chain height")?
        .context("the wallet chain height is unknown")?;

    for account_id in db.get_account_ids().context("listing wallet accounts")? {
        let birthday = db
            .get_account(account_id)
            .context("reading the wallet account")?
            .map(|account| account.birthday_height())
            .unwrap_or(BlockHeight::from_u32(1));
        let receivers = db
            .get_transparent_receivers(account_id, true, true)
            .context("listing the wallet's transparent receivers")?;

        for address in receivers.keys() {
            let mut stream = client
                .get_taddress_transactions(TransparentAddressBlockFilter {
                    address: address.encode(network),
                    range: Some(BlockRange {
                        start: Some(BlockId {
                            height: u64::from(birthday),
                            hash: vec![],
                        }),
                        end: Some(BlockId {
                            height: u64::from(tip),
                            hash: vec![],
                        }),
                        pool_types: vec![],
                    }),
                })
                .await
                .context("GetTaddressTransactions")?
                .into_inner();

            while let Some(raw) = stream
                .message()
                .await
                .context("reading the GetTaddressTransactions stream")?
            {
                if raw.data.is_empty() {
                    continue;
                }
                let mined_height = if raw.height == 0 {
                    None
                } else {
                    Some(BlockHeight::from_u32(raw.height as u32))
                };
                let tx = Transaction::read(
                    &raw.data[..],
                    BranchId::for_height(network, mined_height.unwrap_or(tip)),
                )
                .context("parsing a transaction returned by the indexer")?;
                if db
                    .get_transaction(tx.txid())
                    .context("looking up a transaction")?
                    .is_some()
                {
                    continue;
                }

                // Only a transaction that spends an output of a transaction the wallet
                // already knows can be spending the wallet's own funds: every transparent
                // output the wallet holds comes from a transaction it has stored (through
                // `GetAddressUtxos`, a send of its own, or an earlier sweep). Everything
                // else the indexer reports for these addresses — the coinbase
                // transactions that pay the wallet, transactions that only spend other
                // people's coins — is skipped without decrypting or storing it, which
                // keeps the sweep cheap on a wallet whose blocks arrive often.
                let spends_known_transaction = tx
                    .transparent_bundle()
                    .map(|bundle| {
                        bundle.vin.iter().any(|input| {
                            db.get_transaction(*input.prevout().txid())
                                .map(|known| known.is_some())
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false);
                if !spends_known_transaction {
                    continue;
                }

                decrypt_and_store_transaction(network, db, &tx, mined_height)
                    .context("storing a transaction that involves the wallet's transparent funds")?;
            }
        }
    }

    Ok(())
}

/// Records each mined transaction's index within its block (`transactions.tx_index`)
/// for transactions the wallet knows but has no index for.
///
/// Why this exists: transparent outputs are discovered through `GetAddressUtxos`,
/// whose reply has no `tx_index` field (a known upstream gap — the tracking ticket
/// is https://github.com/zcash/lightwallet-protocol/issues/17, and
/// `zcash_client_sqlite` carries a FIXME on `excluding_immature_coinbase_outputs`
/// for exactly this). The wallet therefore records such outputs with
/// `tx_index = NULL`, which its queries read as "not coinbase": the coinbase
/// maturity exclusion and the `CoinbaseFilter` become no-ops, and a shielding
/// transaction built from an immature coinbase UTXO is rejected by zcashd
/// (`bad-txns-premature-spend-of-coinbase`).
///
/// The probe re-derives the index from the node's block data — the node is the
/// chain, and its `getblock` RPC is the only place a block's transaction order is
/// visible (the indexer's compact blocks omit transparent-only transactions) — and
/// writes it into `transactions.tx_index`: the same column and value the wallet's
/// own scan path records via `put_tx_meta`. Rows that already have an index are
/// never touched. The write goes through a second connection to the same SQLite
/// file (the wallet API exposes no way to set the column), with a busy timeout so a
/// concurrent wallet transaction only delays it.
async fn annotate_transparent_tx_indices(db_path: &Path) -> Result<()> {
    let conn = rusqlite::Connection::open(db_path)
        .with_context(|| format!("opening {} for tx-index annotation", db_path.display()))?;
    conn.busy_timeout(Duration::from_secs(10))
        .context("setting the SQLite busy timeout")?;

    let missing: Vec<(Vec<u8>, u32)> = {
        let mut stmt = conn
            .prepare(
                "SELECT txid, mined_height FROM transactions
                 WHERE tx_index IS NULL AND mined_height IS NOT NULL",
            )
            .context("querying transactions without an in-block index")?;
        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, u32>(1)?)))
            .context("reading transactions without an in-block index")?
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("reading transactions without an in-block index")?;
        rows
    };
    let node = crate::node::NodeRpc::from_env()?;

    // The indexer's compact blocks cannot serve this: lightwalletd omits
    // transparent-only transactions from them (only txs with shielded components are
    // included), so the coinbase and plain transparent transactions are simply not
    // there. The node's chain RPC is the only place the block's transaction order is
    // visible, so fetch it per distinct height (2 calls per height, cached).
    let mut block_txids: std::collections::HashMap<u32, Vec<String>> =
        std::collections::HashMap::new();
    for height in missing.iter().map(|(_, h)| *h).collect::<std::collections::BTreeSet<_>>() {
        let hash = node
            .block_hash(height)
            .await
            .with_context(|| format!("getblockhash {height} for tx-index annotation"))?;
        let txids = node
            .block_txids(&hash)
            .await
            .with_context(|| format!("getblock {hash} for tx-index annotation"))?;
        block_txids.insert(height, txids);
    }

    // The wallet stores txids in internal byte order; the node displays them
    // reversed, so compare against the reversed hex of the stored bytes.
    let display_txid = |bytes: &[u8]| {
        bytes
            .iter()
            .rev()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };

    let mut updated = 0usize;
    {
        let transaction = conn
            .unchecked_transaction()
            .context("starting the tx-index annotation transaction")?;
        {
            let mut update = transaction
                .prepare("UPDATE transactions SET tx_index = ?1 WHERE txid = ?2 AND tx_index IS NULL")
                .context("preparing the tx-index update")?;
            for (txid, height) in &missing {
                if let Some(index) = block_txids
                    .get(height)
                    .and_then(|txids| txids.iter().position(|t| *t == display_txid(txid)))
                {
                    updated += update
                        .execute(rusqlite::params![index as i64, txid])
                        .context("writing a transaction's in-block index")?;
                }
            }
        }
        transaction
            .commit()
            .context("committing the tx-index annotation")?;
    }
    if updated > 0 && env::var("ZANARY_TIMINGS").is_ok() {
        println!("  tx indices        : recorded the in-block index for {updated} transaction(s)");
    }
    Ok(())
}

/// A fully bootstrapped probe: open wallet (imported + synced), connected indexer.
pub struct Probe {
    pub db: ProbeDb,
    pub client: IndexerClient,
    pub network: LocalNetwork,
    pub account: AccountUuid,
    pub usk: UnifiedSpendingKey,
    pub ufvk: UnifiedFullViewingKey,
    pub db_path: PathBuf,
}

/// Opens the wallet, imports ZIP-32 account `account` of the dev seed if needed,
/// syncs it from the indexer, and returns the ready-to-use [`Probe`].
pub async fn bootstrap(account: u32) -> Result<Probe> {
    bootstrap_with(WalletProfile::dev(account)).await
}

/// [`bootstrap`], for any wallet profile: opens *that* wallet's database, imports the
/// profile's account if needed, syncs it, and returns a ready-to-use [`Probe`].
///
/// A merchant wallet goes through exactly this path with its own seed and database,
/// which is what makes its observations third-party evidence rather than a report from
/// the payer's own wallet.
pub async fn bootstrap_with(profile: WalletProfile) -> Result<Probe> {
    let t0 = std::time::Instant::now();
    let timings = env::var("ZANARY_TIMINGS").is_ok();
    let endpoint = endpoint();
    let network = regtest_network();
    let seed = profile.seed()?;
    let usk = profile.usk(&network)?;
    let ufvk = usk.to_unified_full_viewing_key();
    if timings {
        println!("  TIMING keys: {:?}", t0.elapsed());
    }

    let (mut db, db_path) = open_db_for(&profile)?;
    if timings {
        println!("  TIMING open_db: {:?}", t0.elapsed());
    }
    let mut client = connect(&endpoint).await?;
    if timings {
        println!("  TIMING connect: {:?}", t0.elapsed());
    }
    let imported = import_account_for(&mut db, &mut client, &ufvk, &seed, &profile).await?;
    if timings {
        println!("  TIMING import: {:?}", t0.elapsed());
    }
    sync_wallet(&mut db, &mut client, &network, &db_path).await?;
    if timings {
        println!("  TIMING sync: {:?}", t0.elapsed());
    }

    Ok(Probe {
        db,
        client,
        network,
        account: imported.id,
        usk,
        ufvk,
        db_path,
    })
}

/// Reads the wallet summary at the default confirmation policy.
pub fn wallet_summary(db: &ProbeDb) -> Result<Option<WalletSummary<AccountUuid>>> {
    db.get_wallet_summary(ConfirmationsPolicy::default())
        .context("reading the wallet summary")
}

/// A copy of one account's balances, in zatoshis (signed so deltas can go negative).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BalanceSnapshot {
    pub total: i128,
    pub orchard: i128,
    pub sapling: i128,
    pub transparent: i128,
}

impl BalanceSnapshot {
    /// Captures the account's current balances (all-zero if the wallet has no
    /// summary yet).
    pub fn capture(db: &ProbeDb, account: AccountUuid) -> Result<Self> {
        let summary = wallet_summary(db)?;
        let balance = summary
            .as_ref()
            .and_then(|summary| summary.account_balances().get(&account));
        Ok(match balance {
            Some(balance) => BalanceSnapshot {
                total: i128::from(balance.total().into_u64()),
                orchard: i128::from(balance.orchard_balance().total().into_u64()),
                sapling: i128::from(balance.sapling_balance().total().into_u64()),
                transparent: i128::from(balance.unshielded_balance().total().into_u64()),
            },
            None => BalanceSnapshot::default(),
        })
    }

    /// Per-pool `after - before` deltas.
    pub fn delta(&self, after: &BalanceSnapshot) -> BalanceSnapshot {
        BalanceSnapshot {
            total: after.total - self.total,
            orchard: after.orchard - self.orchard,
            sapling: after.sapling - self.sapling,
            transparent: after.transparent - self.transparent,
        }
    }
}

/// Prints the per-pool balances of one account (same format as `wallet`).
pub fn print_balances(balance: &AccountBalance) {
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

/// Prints a signed per-pool balance delta (e.g. after a send).
pub fn print_delta(delta: &BalanceSnapshot) {
    println!("  balance delta    : {}", format_zats_signed(delta.total));
    println!("    orchard        : {}", format_zats_signed(delta.orchard));
    println!("    sapling        : {}", format_zats_signed(delta.sapling));
    println!("    transparent    : {}", format_zats_signed(delta.transparent));
}

/// Formats zatoshis as a ZEC amount with 8 decimal places, e.g. `0.00000000 ZEC`.
pub fn format_zec(value: Zatoshis) -> String {
    format_zats_signed(i128::from(value.into_u64()))
}

/// Formats a signed zatoshis amount, e.g. `-0.00010000 ZEC`.
pub fn format_zats_signed(value: i128) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let abs = value.unsigned_abs();
    format!("{sign}{}.{:08} ZEC", abs / 100_000_000, abs % 100_000_000)
}

/// A minimal in-memory compact-block cache for the wallet sync loop.
///
/// `zcash_client_sqlite` 0.22 exposes no public `BlockCache` implementation
/// (`FsBlockDb` sits behind its `unstable` feature, and the other cache is
/// test-only), so the probe carries its own, modelled on the trait's documented
/// example. The harness chain is tiny and fully local, so cached compact blocks
/// live in RAM and die with the process.
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
