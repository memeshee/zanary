//! ZANARY probe — the rig's own librustzcash wallet on the harness chain.
//!
//! No subcommand (unchanged since step 2): talk to the harness indexer over its
//! CompactTxStreamer gRPC interface and report what chain we are actually attached to.
//!
//! `zanary-probe wallet` (the D1 gate): create/open the probe's own librustzcash
//! wallet, import the dev account, sync it from the indexer, and print an
//! Orchard-only receiver plus balances. See `src/wallet.rs`.
//!
//! `zanary-probe address | fund | shield | pay`: move real value on the harness
//! chain — derive receivers, fund from the node's transparent wallet, shield into
//! the Orchard pool, and send shielded payments with ZIP-302 memos. All shielded
//! operations are the probe's own code (propose -> build/prove/sign -> broadcast
//! via the indexer); see `src/ops.rs`, `src/node.rs` and `src/core.rs`.

use anyhow::{bail, Context, Result};
use std::env;
use zcash_client_backend::proto::service::{
    compact_tx_streamer_client::CompactTxStreamerClient, ChainSpec, Empty,
};

mod core;
mod flow;
mod gate;
mod node;
mod ops;
mod wallet;

#[tokio::main]
async fn main() -> Result<()> {
    let filter = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let argv: Vec<String> = env::args().collect();
    let rest: &[String] = argv.get(2..).unwrap_or(&[]);
    match argv.get(1).map(String::as_str) {
        None => probe_indexer().await,
        Some("wallet") => wallet::run().await,
        Some("address") => ops::address(rest).await,
        Some("fund") => ops::fund(rest).await,
        Some("shield") => ops::shield(rest).await,
        Some("pay") => ops::pay(rest).await,
        Some("anchor") => ops::anchor(rest).await,
        Some("flow") => flow::run(rest).await,
        // The gate exits with its own code (0 = as expected, 1 = not what was expected,
        // 2 = the rig could not check), so it does not go through the `Result` -> exit 1 path.
        Some("test") => gate::test(rest).await.map(|code| std::process::exit(code)),
        Some("invoice") => flow::issue_invoice(rest).await,
        Some("verify-invoice") => flow::verify_invoice(rest).await,
        Some(other) => bail!(
            "unknown subcommand `{other}`\nusage:\n  \
             zanary-probe                        indexer probe (GetLightdInfo + latest tip)\n  \
             zanary-probe wallet                 create/import the dev wallet, sync, print receivers + balances\n  \
             zanary-probe address [--transparent|--orchard] [--index N] [--account N]\n  \
             zanary-probe fund --zec <amount>    fund the wallet's transparent receiver from the node wallet\n  \
             zanary-probe shield [--policy <p>]  shield transparent funds into the wallet's own Orchard pool\n  \
             zanary-probe pay --to <address> --zec <amount> --memo <text> [--policy <p>] [--account N]\n  \
             zanary-probe anchor --report <report.json> [--out <anchor.json>] [--amount-zat N]\n  \
             zanary-probe flow --definition <path> [--out <dir>]   run a payment flow, record run.json + findings + report\n  \
             zanary-probe test --definition <path> [--out <dir>]   the same run as a build gate:\n  \
             \x20   [--expect pass|fail] [--anchor] [--ttl-days <n>] [--summary <path>]\n  \
             \x20   [--node <url>] [--indexer <url>]\n  \
             \x20   exit 0 = the run matched its expectation and the chain agrees,\n  \
             \x20   exit 1 = it did not (a leak on a flow that must pass, or a control flow that\n  \
             \x20   the property library failed to bite), exit 2 = the rig could not check"
        ),
    }
}

/// The original no-subcommand behaviour: report the indexer we are attached to.
async fn probe_indexer() -> Result<()> {
    let endpoint =
        env::var("ZANARY_LWD_URL").unwrap_or_else(|_| "http://127.0.0.1:9067".to_string());
    let network = env::var("ZANARY_NETWORK").unwrap_or_else(|_| "regtest".to_string());

    println!(
        "zanary-probe {} — ZANARY (ZECATHON, Core & Tooling)",
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
