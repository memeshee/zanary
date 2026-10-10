//! CANARY probe — Phase 0 walking skeleton.
//!
//! No subcommand (unchanged since step 2): talk to the harness indexer over its
//! CompactTxStreamer gRPC interface and report what chain we are actually attached to.
//!
//! `canary-probe wallet` (step 3, the D1 gate): create/open the probe's own
//! librustzcash wallet, import the dev account, sync it from the indexer, and print
//! an Orchard-only receiver plus balances. See `src/wallet.rs`.
//!
//! Next: build a shielded self-payment carrying a memo, prove + sign it, broadcast it,
//! and print the txid — all from our own wallet, never the node wallet.

use anyhow::{bail, Context, Result};
use std::env;
use zcash_client_backend::proto::service::{
    compact_tx_streamer_client::CompactTxStreamerClient, ChainSpec, Empty,
};

mod wallet;

#[tokio::main]
async fn main() -> Result<()> {
    let filter = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt().with_env_filter(filter).init();

    match env::args().nth(1).as_deref() {
        None => probe_indexer().await,
        Some("wallet") => wallet::run().await,
        Some(other) => bail!(
            "unknown subcommand `{other}` — usage: `canary-probe` (indexer probe) \
             or `canary-probe wallet` (wallet bootstrap + sync)"
        ),
    }
}

/// The original no-subcommand behaviour: report the indexer we are attached to.
async fn probe_indexer() -> Result<()> {
    let endpoint =
        env::var("CANARY_LWD_URL").unwrap_or_else(|_| "http://127.0.0.1:9067".to_string());
    let network = env::var("CANARY_NETWORK").unwrap_or_else(|_| "regtest".to_string());

    println!(
        "canary-probe {} — CANARY (ZECATHON, Core & Tooling)",
        env!("CARGO_PKG_VERSION")
    );
    println!("  configured network : {network}");
    println!("  indexer endpoint   : {endpoint}");

    let mut client = CompactTxStreamerClient::connect(endpoint.clone())
        .await
        .with_context(|| format!("connecting to indexer at {endpoint}"))?;

    let info = client
        .get_lightd_info(Empty {})
        .await
        .context("GetLightdInfo")?
        .into_inner();

    println!("  --- GetLightdInfo ---");
    println!("  chain       : {}", info.chain_name);
    println!("  version     : {}", info.version);
    println!("  vendor      : {}", info.vendor);
    println!("  sapling at  : {}", info.sapling_activation_height);
    println!("  branch id   : {}", info.consensus_branch_id);

    let tip = client
        .get_latest_block(ChainSpec {})
        .await
        .context("GetLatestBlock")?
        .into_inner();

    println!(
        "  latest tip  : height {} hash {}",
        tip.height,
        hex32(&tip.hash)
    );

    println!("\n  next: lightd info -> sync -> shielded self-payment with memo -> txid");
    Ok(())
}

/// The indexer returns the block hash little-endian; display it reversed, like explorers do.
fn hex32(bytes: &[u8]) -> String {
    bytes.iter().rev().map(|b| format!("{b:02x}")).collect()
}
