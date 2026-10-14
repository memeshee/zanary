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

use anyhow::{anyhow, Context, Result};

use zcash_client_backend::data_api::{Account as _, WalletRead};
use zcash_keys::keys::UnifiedAddressRequest;
use zip32::DiversifierIndex;

use crate::core;

/// Runs the `wallet` subcommand.
pub async fn run() -> Result<()> {
    let endpoint = core::endpoint();
    let data_dir = core::data_dir();
    let db_path = core::db_path();
    let network = core::regtest_network();

    println!(
        "canary-probe {} wallet — CANARY (ZECATHON, Core & Tooling)",
        env!("CARGO_PKG_VERSION")
    );
    println!("  network           : regtest (throwaway harness chain)");
    println!("  indexer endpoint  : {endpoint}");
    println!("  wallet db         : {}", db_path.display());
    println!("  dev seed phrase   : {}", core::DEV_MNEMONIC);
    println!("                      ^ BIP-39 English test vector — TEST ONLY, holds no real funds");

    // --- 1) Derive the dev account's keys from the seed phrase (zcash_keys). ---
    let seed = core::dev_seed()?;
    let usk = core::dev_usk(&network, 0)?;
    let ufvk = usk.to_unified_full_viewing_key();
    println!("  dev UFVK          : {}", ufvk.encode(&network));

    // --- 2) Create or open the wallet database (zcash_client_sqlite). ---
    let (mut db, _) = core::open_db(&data_dir)?;

    let mut client = core::connect(&endpoint).await?;

    // --- 3) Import the dev account once; reuse it on later runs (idempotent). ---
    let imported =
        core::import_account_if_needed(&mut db, &mut client, &ufvk, &seed, 0).await?;
    let account_id = imported.id;
    println!(
        "  account           : {} ({:?}, birthday height {})",
        if imported.newly_imported {
            "imported"
        } else {
            "already imported"
        },
        imported.id,
        imported.birthday_height
    );

    // --- 4) Sync the wallet from the harness indexer. ---
    //
    // `zcash_client_backend::sync::run` (0.24) cannot talk to this harness: it always
    // fetches Ironwood subtree roots, and the harness node (zcashd v6.20.0) has no
    // Ironwood pool — its `z_getsubtreesbyindex` accepts only "sapling"/"orchard" — so
    // `sync::run` dies with:
    //   GetSubtreeRoots: z_getsubtreesbyindex, error: -8:
    //   Requested pool must be "sapling" or "orchard"
    //
    // This performs the same steps `sync::run` does — update the chain tip, refresh
    // transparent UTXOs, then download -> scan -> evict each suggested scan range —
    // minus that fast-forward, which is a no-op on this chain: every pool reports zero
    // complete subtrees, so there is nothing to fast-forward and scanning the compact
    // blocks maintains the wallet's note commitment trees exactly.
    core::sync_wallet(&mut db, &mut client, &network, &db_path).await?;

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

    let summary = core::wallet_summary(&db)?;

    match &summary {
        Some(summary) => {
            println!("  chain tip height  : {}", summary.chain_tip_height());
            println!("  fully scanned     : {}", summary.fully_scanned_height());
            match summary.account_balances().get(&account_id) {
                Some(balance) => core::print_balances(balance),
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


