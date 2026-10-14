//! `canary-seed` — seed the throwaway chain **to our own wallet** and make a real
//! shielded payment carrying a memo, without ever touching the node wallet.
//!
//! The harness node (zcashd) is the chain and the miner, nothing else: `seed.sh`'s
//! node-wallet path crashed the node (see `harness/HARNESS.md`), and the rig's rule is
//! that every wallet in it is ours. So this tool:
//!
//!   1. `addr`   — open (or create) the seed wallet, import the public dev account, and
//!                 print the account's default transparent receiver (the address the node
//!                 will mine to via `-mineraddress`) and a fresh Orchard-only receiver.
//!   2. `sync`   — sync the wallet from the harness indexer (lightwalletd gRPC), scanning
//!                 compact blocks and refreshing transparent UTXOs.
//!   3. `shield` — take the wallet's *coinbase* transparent outputs and shield them into
//!                 an Orchard output **carrying a memo**, using
//!                 `zcash_client_backend::data_api::wallet::propose_shielding_coinbase`
//!                 (which exists precisely for this: coinbase-only inputs, no change),
//!                 then broadcast the signed transaction through the indexer.
//!   4. `tx`     — print what the indexer knows about a txid (height + raw bytes).
//!
//! The wallet database is separate from `canary-probe wallet`'s
//! (`$CANARY_SEED_DB`, default `./data/wallet-seed/canary-seed.db`) so this tool never
//! rewrites state another process depends on. It derives the same dev account from the
//! same public BIP-39 test-vector mnemonic.
//!
//! The dev mnemonic is public and this chain is disposable: nothing here is a secret.

use anyhow::{anyhow, bail, Context, Result};
use std::{convert::Infallible, env, path::PathBuf, sync::Mutex};

use sapling_crypto::{
    bundle::GrothProofBytes,
    circuit,
    keys::{Diversifier, EphemeralSecretKey, ProofGenerationKey},
    note::Rseed,
    prover::{OutputProver, SpendProver},
    value::{NoteValue, ValueCommitTrapdoor},
    MerklePath, PaymentAddress,
};
use zcash_client_backend::{
    data_api::{
        chain::{error::Error as ChainError, scan_cached_blocks, BlockCache, BlockSource},
        scanning::ScanRange,
        wallet::{
            create_proposed_transactions, decrypt_and_store_transaction,
            input_selection::GreedyInputSelector, propose_shielding_coinbase, ConfirmationsPolicy,
            SpendingKeys,
        },
        Account as _, AccountBalance, AccountBirthday, AccountPurpose, WalletRead, WalletWrite,
        Zip32Derivation,
    },
    fees::StandardFeeRule,
    proto::{
        compact_formats::CompactBlock,
        service::{
            compact_tx_streamer_client::CompactTxStreamerClient, BlockId, BlockRange, ChainSpec,
            GetAddressUtxosArg, RawTransaction, TxFilter,
        },
    },
    wallet::{OvkPolicy, WalletTransparentOutput},
};
use zcash_client_sqlite::{util::SystemClock, wallet::init::init_wallet_db, WalletDb};
use zcash_keys::{
    encoding::AddressCodec as _,
    keys::{UnifiedAddressRequest, UnifiedFullViewingKey, UnifiedSpendingKey},
};
use zcash_protocol::{
    consensus::BlockHeight, local_consensus::LocalNetwork, memo::Memo, memo::MemoBytes,
    value::Zatoshis,
};
use zcash_script::script;
use zcash_transparent::{
    address::Script,
    bundle::{OutPoint, TxOut},
    keys::IncomingViewingKey as _,
};
use zip32::{fingerprint::SeedFingerprint, AccountId, DiversifierIndex};

/// The dev seed phrase: the canonical BIP-39 English test vector. TEST ONLY — public,
/// and the only chain it ever touches is the throwaway regtest harness chain.
const DEV_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const DEV_ACCOUNT_NAME: &str = "canary-dev";

type SeedDb = WalletDb<rusqlite::Connection, LocalNetwork, SystemClock, rand::rngs::OsRng>;

/// Consensus parameters matching `data/zcashd/.zcash/zcash.conf` (all upgrades at 1).
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
        nu6_1: None,
        nu6_2: None,
        nu6_3: None,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    match env::args().nth(1).as_deref() {
        Some("addr") => addr().await,
        Some("sync") => sync().await,
        Some("learn-coinbase") => learn_coinbase().await,
        Some("shield") => shield().await,
        Some("tx") => tx_info().await,
        Some(other) => bail!(
            "unknown subcommand `{other}` — usage: canary-seed addr | sync | learn-coinbase \
             --from <h> --to <h> | shield --memo <text> [--to <addr>] [--limit <n>] | \
             tx --txid <txid>"
        ),
        None => bail!("usage: canary-seed addr | sync | learn-coinbase | shield | tx"),
    }
}

fn arg_value(flag: &str) -> Option<String> {
    let args: Vec<String> = env::args().collect();
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn indexer_endpoint() -> String {
    env::var("CANARY_LWD_URL").unwrap_or_else(|_| "http://127.0.0.1:9067".to_string())
}

fn seed_db_path() -> PathBuf {
    PathBuf::from(
        env::var("CANARY_SEED_DB").unwrap_or_else(|_| "./data/wallet-seed/canary-seed.db".into()),
    )
}

async fn connect_client() -> Result<CompactTxStreamerClient<tonic::transport::Channel>> {
    let endpoint = indexer_endpoint();
    CompactTxStreamerClient::connect(endpoint.clone())
        .await
        .with_context(|| format!("connecting to indexer at {endpoint}"))
}

/// Derives the dev unified spending key from the public test mnemonic.
fn dev_usk(network: &LocalNetwork) -> Result<UnifiedSpendingKey> {
    let mnemonic = bip39::Mnemonic::parse_in_normalized(bip39::Language::English, DEV_MNEMONIC)
        .map_err(|e| anyhow!("dev mnemonic is not valid BIP-39: {e}"))?;
    let seed = mnemonic.to_seed("");
    UnifiedSpendingKey::from_seed(network, &seed, AccountId::ZERO)
        .context("deriving the dev unified spending key")
}

/// Opens (or creates) the seed wallet database and ensures the dev account is imported.
async fn open_wallet(network: &LocalNetwork) -> Result<(SeedDb, UnifiedSpendingKey)> {
    let db_path = seed_db_path();
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating seed wallet dir {}", parent.display()))?;
    }
    let usk = dev_usk(network)?;
    let ufvk = usk.to_unified_full_viewing_key();

    let mut db = WalletDb::for_path(&db_path, *network, SystemClock, rand::rngs::OsRng)
        .context("opening the seed wallet database")?;
    init_wallet_db(&mut db, None).context("initializing the wallet database schema")?;

    if db
        .get_account_for_ufvk(&ufvk)
        .context("checking dev account")?
        .is_none()
    {
        // The seed wallet scans from block 1; the birthday tree state comes from the
        // indexer (BlockId height 0 is treated as unset by lightwalletd, so use 1 and
        // let the sync loop start at 2 — block 1 is a coinbase-only block).
        let mut client = connect_client().await?;
        let tree_state = client
            .get_tree_state(BlockId {
                height: 1,
                hash: vec![],
            })
            .await
            .context("GetTreeState at height 1")?
            .into_inner();
        let birthday = AccountBirthday::from_treestate(tree_state, None)
            .map_err(|e| anyhow!("building account birthday: {e}"))?;
        let mnemonic = bip39::Mnemonic::parse_in_normalized(bip39::Language::English, DEV_MNEMONIC)
            .expect("dev mnemonic is valid");
        let seed = mnemonic.to_seed("");
        let derivation = Zip32Derivation::new(
            SeedFingerprint::from_seed(&seed)
                .ok_or_else(|| anyhow!("dev seed has an invalid length for ZIP-32"))?,
            AccountId::ZERO,
        );
        db.import_account_ufvk(
            DEV_ACCOUNT_NAME,
            &ufvk,
            &birthday,
            AccountPurpose::Spending {
                derivation: Some(derivation),
            },
            Some("canary dev seed (BIP-39 test vector)"),
        )
        .context("importing the dev UFVK")?;
    }
    Ok((db, usk))
}

/// The dev account's default external transparent receiver (what `-mineraddress` points at).
fn transparent_receiver(ufvk: &UnifiedFullViewingKey, network: &LocalNetwork) -> Result<String> {
    let account_pubkey = ufvk
        .transparent()
        .ok_or_else(|| anyhow!("dev account has no transparent component"))?;
    let external_ivk = account_pubkey
        .derive_external_ivk()
        .map_err(|e| anyhow!("deriving the external transparent IVK: {e}"))?;
    Ok(external_ivk.default_address().0.encode(network))
}

/// `addr` — print the transparent receiver to mine to and an Orchard-only receiver.
async fn addr() -> Result<()> {
    let network = regtest_network();
    let (_db, usk) = open_wallet(&network).await?;
    let ufvk = usk.to_unified_full_viewing_key();
    let t_addr = transparent_receiver(&ufvk, &network)?;
    let (orchard_receiver, diversifier) = ufvk
        .find_address(DiversifierIndex::from(0u32), UnifiedAddressRequest::ORCHARD)
        .map_err(|e| anyhow!("deriving orchard-only receiver: {e}"))?;
    println!(
        "{}",
        serde_json::json!({
            "transparent_receiver": t_addr,
            "orchard_receiver": orchard_receiver.encode(&network),
            "diversifier_index": format!("{diversifier:?}"),
        })
    );
    Ok(())
}

/// One full wallet sync pass: chain tip, transparent UTXOs, then compact-block scan loop.
async fn sync_wallet(
    db: &mut SeedDb,
    client: &mut CompactTxStreamerClient<tonic::transport::Channel>,
    network: &LocalNetwork,
) -> Result<u64> {
    let tip = client
        .get_latest_block(ChainSpec {})
        .await
        .context("GetLatestBlock")?
        .into_inner();
    let tip_height = u64::from(tip.height);
    db.update_chain_tip(BlockHeight::from_u32(tip.height as u32))
        .context("updating wallet chain tip")?;

    // Refresh transparent UTXOs for every account (mirrors sync::run).
    for account in db.get_account_ids().context("listing accounts")? {
        let start_height = db
            .utxo_query_height(account)
            .context("reading utxo query height")?;
        let addresses: Vec<String> = db
            .get_transparent_receivers(account, true, true)
            .context("listing transparent receivers")?
            .keys()
            .map(|address| address.encode(network))
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
        while let Some(reply) = stream.message().await.context("reading UTXO stream")? {
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

    // Download -> scan -> evict, one suggested scan range at a time.
    let cache = MemoryBlockCache::new();
    let mut iterations = 0u32;
    loop {
        iterations += 1;
        if iterations > 1_000 {
            bail!("wallet sync did not converge after {iterations} ranges");
        }
        let ranges = db.suggest_scan_ranges().context("suggesting scan ranges")?;
        let Some(range) = ranges.into_iter().find(|range| !range.is_empty()) else {
            break;
        };
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
        while let Some(block) = stream.message().await.context("reading block stream")? {
            blocks.push(block);
        }
        cache.insert(blocks).await?;

        let tree_state = client
            .get_tree_state(BlockId {
                height: u64::from(start - 1),
                hash: vec![],
            })
            .await
            .with_context(|| format!("GetTreeState at {}", start - 1))?
            .into_inner();
        let from_state = tree_state
            .to_chain_state()
            .context("decoding tree state below scan range")?;
        let limit = usize::try_from(u32::from(end) - u32::from(start) + 1)
            .expect("scan range length fits in usize");
        scan_cached_blocks(network, &cache, db, start, &from_state, limit)
            .map_err(|e| anyhow!("scanning blocks {start}..={end}: {e}"))?;
        cache.delete(range).await?;
    }
    Ok(tip_height)
}

/// Minimal read-only JSON-RPC client for the harness node (getblockhash/getblock/
/// getrawtransaction only — the node is the chain and the miner, never a wallet here).
fn node_rpc(method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
    use std::io::{Read, Write};
    let url = env::var("CANARY_NODE_URL").unwrap_or_else(|_| "http://127.0.0.1:18232".into());
    let user = env::var("CANARY_NODE_USER").unwrap_or_else(|_| "canary".into());
    let pass = env::var("CANARY_NODE_PASS").unwrap_or_else(|_| "canary".into());
    let hostport = url.strip_prefix("http://").unwrap_or(&url).trim_end_matches('/');
    let body = serde_json::json!({
        "jsonrpc": "1.0",
        "id": "canary-seed",
        "method": method,
        "params": params,
    })
    .to_string();
    let auth = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        format!("{user}:{pass}"),
    );
    let request = format!(
        "POST / HTTP/1.1\r\nHost: {hostport}\r\nAuthorization: Basic {auth}\r\n\
         Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let mut stream = std::net::TcpStream::connect(hostport)
        .with_context(|| format!("connecting to the node at {hostport}"))?;
    stream
        .write_all(request.as_bytes())
        .context("writing the node request")?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .context("reading the node response")?;
    let (_, response_body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| anyhow!("malformed HTTP response from the node"))?;
    let json: serde_json::Value =
        serde_json::from_str(response_body).context("parsing the node response")?;
    if let Some(error) = json.get("error") {
        if !error.is_null() {
            bail!("node RPC `{method}` failed: {error}");
        }
    }
    Ok(json
        .get("result")
        .cloned()
        .unwrap_or(serde_json::Value::Null))
}

/// `learn-coinbase` — fetch each height's coinbase transaction and store it in the wallet, so
/// the wallet learns `tx_index = 0` (coinbase) for the outputs it received through the
/// transparent-UTXO stream. Without this, the coinbase-only shielding selector conservatively
/// treats unknown outputs as non-coinbase and refuses to select them (see the FIXME in
/// `zcash_client_sqlite`'s `excluding_immature_coinbase_outputs`).
///
/// Coinbase txids come from the node (`getblock`, read-only); the raw transaction bytes come
/// from the indexer (`GetTransaction`) with the node as fallback.
async fn learn_coinbase() -> Result<()> {
    let from: u32 = arg_value("--from")
        .ok_or_else(|| anyhow!("learn-coinbase requires --from <height>"))?
        .parse()
        .context("parsing --from")?;
    let to: u32 = arg_value("--to")
        .ok_or_else(|| anyhow!("learn-coinbase requires --to <height>"))?
        .parse()
        .context("parsing --to")?;
    if to < from {
        bail!("--to {to} is below --from {from}");
    }
    let network = regtest_network();
    let (mut db, _usk) = open_wallet(&network).await?;
    let mut client = connect_client().await?;

    let tip = client
        .get_latest_block(ChainSpec {})
        .await
        .context("GetLatestBlock")?
        .into_inner();
    db.update_chain_tip(BlockHeight::from_u32(tip.height as u32))
        .context("updating wallet chain tip")?;

    let mut stored = 0usize;
    let mut skipped = 0usize;
    for height in from..=to {
        let block_hash = node_rpc("getblockhash", serde_json::json!([height]))?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| anyhow!("getblockhash {height} returned no hash"))?;
        let block = node_rpc("getblock", serde_json::json!([block_hash, 1]))?;
        let Some(txid_hex) = block
            .get("tx")
            .and_then(|txs| txs.get(0))
            .and_then(|txid| txid.as_str())
        else {
            skipped += 1;
            continue;
        };
        let mut txid = [0u8; 32];
        // The node returns txids in display (big-endian) order; the indexer wants the
        // raw internal order, which is the reverse.
        let mut bytes = vec![0u8; 32];
        hex_decode_into(txid_hex, &mut bytes)?;
        bytes.reverse();
        txid.copy_from_slice(&bytes);

        let reply = client
            .get_transaction(TxFilter {
                block: None,
                index: 0,
                hash: txid.to_vec(),
            })
            .await
            .with_context(|| format!("GetTransaction for the coinbase at height {height}"))?
            .into_inner();
        let raw = if reply.data.is_empty() {
            // Fall back to the node for the raw bytes (read-only).
            let hex = node_rpc("getrawtransaction", serde_json::json!([txid_hex]))
                .with_context(|| format!("getrawtransaction {txid_hex}"))?
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| anyhow!("getrawtransaction returned no data"))?;
            hex_decode(&hex)?
        } else {
            reply.data
        };

        let tx = zcash_primitives::transaction::Transaction::read(
            &raw[..],
            zcash_protocol::consensus::BranchId::for_height(
                &network,
                BlockHeight::from_u32(height),
            ),
        )
        .with_context(|| format!("parsing the coinbase at height {height}"))?;
        decrypt_and_store_transaction(&network, &mut db, &tx, Some(BlockHeight::from_u32(height)))
            .with_context(|| format!("storing the coinbase at height {height}"))?;
        stored += 1;
    }

    println!(
        "{}",
        serde_json::json!({
            "from": from,
            "to": to,
            "stored": stored,
            "skipped": skipped,
        })
    );
    Ok(())
}

fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 {
        bail!("odd-length hex string");
    }
    let mut out = vec![0u8; hex.len() / 2];
    hex_decode_into(hex, &mut out)?;
    Ok(out)
}

/// `sync` — run one sync and print the wallet status.
async fn sync() -> Result<()> {
    let network = regtest_network();
    let (mut db, usk) = open_wallet(&network).await?;
    let ufvk = usk.to_unified_full_viewing_key();
    let account_id = db
        .get_account_for_ufvk(&ufvk)
        .context("reading dev account")?
        .ok_or_else(|| anyhow!("dev account missing from seed wallet"))?
        .id();
    let mut client = connect_client().await?;
    let tip = sync_wallet(&mut db, &mut client, &network).await?;
    let summary = db.get_wallet_summary(ConfirmationsPolicy::default())?;
    let mut out = serde_json::json!({
        "chain_tip_height": tip,
        "account": format!("{account_id:?}"),
    });
    if let Some(summary) = &summary {
        out["fully_scanned_height"] = serde_json::json!(u32::from(summary.fully_scanned_height()));
        if let Some(bal) = summary.account_balances().get(&account_id) {
            out["balance"] = balance_json(bal);
        }
    }
    println!("{out}");
    Ok(())
}

fn balance_json(balance: &AccountBalance) -> serde_json::Value {
    serde_json::json!({
        "total_zat": balance.total().into_u64(),
        "orchard_zat": balance.orchard_balance().total().into_u64(),
        "sapling_zat": balance.sapling_balance().total().into_u64(),
        "transparent_zat": balance.unshielded_balance().total().into_u64(),
        "spendable_zat": balance.spendable_value().into_u64(),
    })
}

/// `shield` — shield coinbase outputs into an Orchard payment carrying a memo, and broadcast.
async fn shield() -> Result<()> {
    let memo_text = arg_value("--memo").ok_or_else(|| anyhow!("shield requires --memo <text>"))?;
    let limit: Option<usize> = arg_value("--limit")
        .map(|v| v.parse::<usize>())
        .transpose()
        .context("parsing --limit")?;
    let network = regtest_network();
    let (mut db, usk) = open_wallet(&network).await?;
    let ufvk = usk.to_unified_full_viewing_key();

    // Destination: an Orchard-only receiver of the dev account (or an explicit --to).
    let to_address = match arg_value("--to") {
        Some(addr) => {
            zcash_address::ZcashAddress::try_from_encoded(&addr).context("parsing --to address")?
        }
        None => {
            let (receiver, _) = ufvk
                .find_address(DiversifierIndex::from(0u32), UnifiedAddressRequest::ORCHARD)
                .map_err(|e| anyhow!("deriving orchard-only receiver: {e}"))?;
            let encoded = receiver.encode(&network);
            zcash_address::ZcashAddress::try_from_encoded(&encoded)
                .context("parsing derived orchard receiver")?
        }
    };
    let to_str = to_address.to_string();

    let mut client = connect_client().await?;

    // 1) Sync so UTXOs and commitment trees are current.
    let tip = sync_wallet(&mut db, &mut client, &network).await?;

    // 2) Which transparent receivers may fund the shielding.
    let from_addrs = vec![
        ufvk.transparent()
            .ok_or_else(|| anyhow!("dev account has no transparent component"))?
            .derive_external_ivk()
            .map_err(|e| anyhow!("deriving the external transparent IVK: {e}"))?
            .default_address()
            .0,
    ];

    // 3) Proposal over coinbase-only inputs, no change, memo attached.
    let selector = GreedyInputSelector::new();
    let fee_rule = StandardFeeRule::Zip317;
    let proposal = propose_shielding_coinbase::<_, _, _, _, Infallible>(
        &mut db,
        &network,
        &selector,
        &fee_rule,
        Zatoshis::ZERO,
        &from_addrs,
        to_address,
        Some(MemoBytes::from(
            memo_text
                .parse::<Memo>()
                .map_err(|e| anyhow!("memo is not valid: {e}"))?,
        )),
        limit,
        None,
    )
    .map_err(|e| anyhow!("propose_shielding_coinbase: {e}"))?;

    // 4) Build + sign. The transaction is Orchard-only by construction (coinbase
    //    transparent inputs -> Orchard output, no change), so the Sapling provers below
    //    are never invoked; they exist only to satisfy the API and fail loudly if that
    //    assumption is ever violated.
    let txids = create_proposed_transactions::<_, _, Infallible, _, Infallible, _>(
        &mut db,
        &network,
        &NoSaplingProver,
        &NoSaplingProver,
        &SpendingKeys::from_unified_spending_key(usk.clone()),
        OvkPolicy::Sender,
        &proposal,
        None,
    )
    .map_err(|e| anyhow!("create_proposed_transactions: {e}"))?;

    let txid = txids.first();
    let tx = db
        .get_transaction(*txid)
        .context("reading the built transaction from the wallet")?
        .ok_or_else(|| anyhow!("built transaction {txid} missing from wallet db"))?;
    let mut bytes = Vec::new();
    tx.write(&mut bytes).context("serializing transaction")?;

    // 5) Broadcast through the indexer (lightwalletd -> zcashd sendrawtransaction).
    let response = client
        .send_transaction(RawTransaction {
            data: bytes.clone(),
            height: 0,
        })
        .await
        .context("SendTransaction")?
        .into_inner();
    if response.error_code != 0 {
        bail!(
            "indexer rejected the transaction: code {} {}",
            response.error_code,
            response.error_message
        );
    }

    println!(
        "{}",
        serde_json::json!({
            "txid": txid.to_string(),
            "to": to_str,
            "memo_text": memo_text,
            "memo_hex": hex_encode(memo_text.as_bytes()),
            "built_against_height": tip,
            "tx_bytes": bytes.len(),
        })
    );
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// `tx` — ask the indexer what it knows about a txid (display-order hex, as explorers show it).
async fn tx_info() -> Result<()> {
    let txid_hex = arg_value("--txid").ok_or_else(|| anyhow!("tx requires --txid <txid>"))?;
    let mut client = connect_client().await?;
    let mut bytes = vec![0u8; 32];
    hex_decode_into(&txid_hex, &mut bytes)?;
    // The indexer's `TxFilter.hash` is the raw internal txid, the reverse of display order.
    bytes.reverse();
    let reply = client
        .get_transaction(TxFilter {
            block: None,
            index: 0,
            hash: bytes,
        })
        .await
        .context("GetTransaction")?
        .into_inner();
    println!(
        "{}",
        serde_json::json!({
            "txid": txid_hex,
            "height": reply.height,
            "bytes": reply.data.len(),
        })
    );
    Ok(())
}

fn hex_decode_into(hex: &str, out: &mut [u8]) -> Result<()> {
    if hex.len() != out.len() * 2 {
        bail!("expected {} hex chars, got {}", out.len() * 2, hex.len());
    }
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let s = std::str::from_utf8(chunk)?;
        out[i] = u8::from_str_radix(s, 16).with_context(|| format!("bad hex `{s}`"))?;
    }
    Ok(())
}

/// A Sapling prover that refuses to run: this seeding flow is Orchard-only by
/// construction, so `prepare_circuit`/`create_proof` being reached means the
/// assumption broke and the transaction shape is not what the tool promised.
struct NoSaplingProver;

impl SpendProver for NoSaplingProver {
    type Proof = GrothProofBytes;

    fn prepare_circuit(
        _proof_generation_key: ProofGenerationKey,
        _diversifier: Diversifier,
        _rseed: Rseed,
        _value: NoteValue,
        _alpha: jubjub::Fr,
        _rcv: ValueCommitTrapdoor,
        _anchor: bls12_381::Scalar,
        _merkle_path: MerklePath,
    ) -> Option<circuit::Spend> {
        panic!("canary-seed: a Sapling spend appeared in an Orchard-only shielding flow")
    }

    fn create_proof<R: rand_core::RngCore>(
        &self,
        _circuit: circuit::Spend,
        _rng: &mut R,
    ) -> Self::Proof {
        panic!("canary-seed: a Sapling spend proof was requested in an Orchard-only flow")
    }

    fn encode_proof(proof: Self::Proof) -> GrothProofBytes {
        proof
    }
}

impl OutputProver for NoSaplingProver {
    type Proof = GrothProofBytes;

    fn prepare_circuit(
        _esk: &EphemeralSecretKey,
        _payment_address: PaymentAddress,
        _rcm: jubjub::Fr,
        _value: NoteValue,
        _rcv: ValueCommitTrapdoor,
    ) -> circuit::Output {
        panic!("canary-seed: a Sapling output appeared in an Orchard-only shielding flow")
    }

    fn create_proof<R: rand_core::RngCore>(
        &self,
        _circuit: circuit::Output,
        _rng: &mut R,
    ) -> Self::Proof {
        panic!("canary-seed: a Sapling output proof was requested in an Orchard-only flow")
    }

    fn encode_proof(proof: Self::Proof) -> GrothProofBytes {
        proof
    }
}

/// Minimal in-memory compact-block cache (same shape as `wallet.rs`'s).
struct MemoryBlockCache {
    blocks: Mutex<Vec<CompactBlock>>,
}

impl MemoryBlockCache {
    fn new() -> Self {
        Self {
            blocks: Mutex::new(Vec::new()),
        }
    }

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
        let start = range.block_range().start;
        let mut expected = u32::from(start);
        let mut out = Vec::new();
        for block in self.blocks_in(range) {
            match (block.height as u32).cmp(&expected) {
                std::cmp::Ordering::Equal => {
                    out.push(block);
                    expected += 1;
                }
                std::cmp::Ordering::Greater => break,
                std::cmp::Ordering::Less => {}
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
