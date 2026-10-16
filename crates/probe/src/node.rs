//! Minimal zcashd JSON-RPC client for the harness node.
//!
//! The node is only ever used as *the chain and the miner*: this client speaks
//! transparent-only calls (`getblockcount`, `getwalletinfo`, `sendtoaddress`) plus
//! `generate`. It must never call the node's shielded wallet RPCs (`z_*`) — the
//! node's Orchard wallet is deprecated and crashed zcashd with an assertion when a
//! previous agent used it.
//!
//! HTTP is hand-rolled over a tokio TCP socket: the harness endpoint is plain
//! `http://127.0.0.1:18232` with HTTP basic auth, so no TLS and no HTTP stack are
//! needed. The request is HTTP/1.0 with `Connection: close`, so the response body
//! ends at EOF (chunked transfer encoding cannot occur on HTTP/1.0).

use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::{json, Value};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};

/// Default harness node RPC endpoint (the docker-compose zcashd).
const DEFAULT_RPC_URL: &str = "http://127.0.0.1:18232";

/// A connected-on-demand zcashd JSON-RPC client.
pub struct NodeRpc {
    host: String,
    port: u16,
    user: String,
    password: String,
}

/// Value-pool totals at a height, as the node reports them.
#[derive(Debug, Default, Clone, Copy)]
pub struct PoolTotals {
    pub height: u32,
    pub transparent_zat: u64,
    pub sapling_zat: u64,
    pub orchard_zat: u64,
}

impl NodeRpc {
    /// Builds the client from `ZANARY_NODE_RPC_URL` (default
    /// `http://127.0.0.1:18232`), `ZANARY_NODE_RPC_USER` and
    /// `ZANARY_NODE_RPC_PASSWORD` (both default `zanary`).
    pub fn from_env() -> Result<Self> {
        let url =
            std::env::var("ZANARY_NODE_RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_string());
        let rest = url.strip_prefix("http://").ok_or_else(|| {
            anyhow!("ZANARY_NODE_RPC_URL must be a plain http:// URL, got `{url}`")
        })?;
        let (host, port) = rest
            .split_once(':')
            .ok_or_else(|| anyhow!("ZANARY_NODE_RPC_URL must include a port, got `{url}`"))?;
        let port: u16 = port
            .parse()
            .with_context(|| format!("invalid port in ZANARY_NODE_RPC_URL `{url}`"))?;
        Ok(Self {
            host: host.to_string(),
            port,
            user: std::env::var("ZANARY_NODE_RPC_USER").unwrap_or_else(|_| "zanary".to_string()),
            password: std::env::var("ZANARY_NODE_RPC_PASSWORD")
                .unwrap_or_else(|_| "zanary".to_string()),
        })
    }

    /// Performs one JSON-RPC call and returns its `result` (errors become `Err`).
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let body = json!({
            "jsonrpc": "1.0",
            "id": "zanary-probe",
            "method": method,
            "params": params,
        })
        .to_string();
        let auth = STANDARD.encode(format!("{}:{}", self.user, self.password));
        let request = format!(
            "POST / HTTP/1.0\r\n\
             Host: {host}:{port}\r\n\
             Authorization: Basic {auth}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {len}\r\n\
             Connection: close\r\n\
             \r\n\
             {body}",
            host = self.host,
            port = self.port,
            len = body.len(),
        );

        let mut stream = timeout(
            Duration::from_secs(30),
            TcpStream::connect((self.host.as_str(), self.port)),
        )
        .await
        .with_context(|| {
            format!(
                "connecting to the zcashd RPC at {}:{} (is the harness up? \
                 `cd harness && docker compose up -d`)",
                self.host, self.port
            )
        })?
        .with_context(|| {
            format!(
                "connecting to the zcashd RPC at {}:{}",
                self.host, self.port
            )
        })?;
        stream
            .write_all(request.as_bytes())
            .await
            .context("sending the zcashd JSON-RPC request")?;
        stream
            .flush()
            .await
            .context("flushing the zcashd JSON-RPC request")?;

        let mut raw = Vec::new();
        timeout(Duration::from_secs(300), stream.read_to_end(&mut raw))
            .await
            .with_context(|| format!("zcashd RPC `{method}` timed out"))?
            .context("reading the zcashd JSON-RPC response")?;

        let text = String::from_utf8_lossy(&raw);
        let (headers, body) = text.split_once("\r\n\r\n").ok_or_else(|| {
            anyhow!("malformed HTTP response from the zcashd RPC (no header/body separator)")
        })?;
        let status_line = headers.lines().next().unwrap_or("");
        if !status_line.contains(" 200 ") {
            bail!(
                "zcashd RPC `{method}` returned an HTTP error: {status_line}\n  \
                 (check ZANARY_NODE_RPC_USER / ZANARY_NODE_RPC_PASSWORD)"
            );
        }

        let response: Value = serde_json::from_str(body.trim())
            .with_context(|| format!("parsing the zcashd JSON-RPC response for `{method}`"))?;
        if !response["error"].is_null() {
            bail!("zcashd RPC `{method}` failed: {}", response["error"]);
        }
        Ok(response["result"].clone())
    }

    /// Current chain height (`getblockcount`).
    pub async fn block_count(&self) -> Result<u64> {
        self.call("getblockcount", json!([]))
            .await?
            .as_u64()
            .ok_or_else(|| anyhow!("getblockcount returned a non-integer result"))
    }

    /// Value-pool totals from `getblockchaininfo` — the node's own view of how much value
    /// sits in each pool.
    ///
    /// Pool crossings are only checkable after the fact with these numbers: an unshielding
    /// shows up as orchard value falling while transparent value rises, a shielding as the
    /// reverse.
    pub async fn pool_totals(&self, height: u32) -> Result<PoolTotals> {
        let info = self.call("getblockchaininfo", json!([])).await?;
        let pools = info
            .get("valuePools")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("getblockchaininfo has no valuePools array"))?;
        let mut totals = PoolTotals {
            height,
            ..Default::default()
        };
        for pool in pools {
            let id = pool.get("id").and_then(Value::as_str).unwrap_or_default();
            let value = pool
                .get("chainValueZat")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            match id {
                "transparent" => totals.transparent_zat = value,
                "sapling" => totals.sapling_zat = value,
                "orchard" => totals.orchard_zat = value,
                _ => {}
            }
        }
        Ok(totals)
    }

    /// Block hash at `height` (`getblockhash`), in display order.
    pub async fn block_hash(&self, height: u32) -> Result<String> {
        self.call("getblockhash", json!([height]))
            .await?
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| anyhow!("getblockhash {height} returned a non-string result"))
    }

    /// The transaction ids of the block with the given hash, in block order
    /// (`getblock <hash> 1`), in display order.
    pub async fn block_txids(&self, hash: &str) -> Result<Vec<String>> {
        let block = self.call("getblock", json!([hash, 1])).await?;
        block["tx"]
            .as_array()
            .map(|txids| {
                txids
                    .iter()
                    .filter_map(|txid| txid.as_str().map(str::to_string))
                    .collect()
            })
            .ok_or_else(|| anyhow!("getblock {hash} returned no `tx` array"))
    }

    /// The node wallet's transparent balances in zatoshis: `(spendable, immature)`.
    ///
    /// Reads `getwalletinfo` only — a transparent-view RPC. (The node's shielded
    /// wallet is deprecated and must not be touched.)
    pub async fn transparent_balances(&self) -> Result<(u64, u64)> {
        let info = self.call("getwalletinfo", json!([])).await?;
        let spendable = zec_to_zats(
            info["balance"]
                .as_f64()
                .ok_or_else(|| anyhow!("getwalletinfo returned no `balance`"))?,
        );
        let immature = zec_to_zats(
            info["immature_balance"]
                .as_f64()
                .ok_or_else(|| anyhow!("getwalletinfo returned no `immature_balance`"))?,
        );
        Ok((spendable, immature))
    }

    /// Mines `blocks` regtest blocks (`generate`) and returns the new block hashes.
    pub async fn generate(&self, blocks: u64) -> Result<Vec<String>> {
        let result = self.call("generate", json!([blocks])).await?;
        Ok(result
            .as_array()
            .map(|hashes| {
                hashes
                    .iter()
                    .filter_map(|hash| hash.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Sends transparent funds to `address` via the node's own transparent wallet
    /// (`sendtoaddress`) and returns the transaction id.
    ///
    /// The amount is passed as a decimal string, never a float, so no precision is
    /// lost on the way to the node.
    pub async fn send_to_address(&self, address: &str, zats: u64) -> Result<String> {
        let amount = format!("{}.{:08}", zats / 100_000_000, zats % 100_000_000);
        self.call("sendtoaddress", json!([address, amount]))
            .await?
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| anyhow!("sendtoaddress returned a non-string txid"))
    }
}

/// Converts a ZEC amount from `getwalletinfo` (JSON float) to zatoshis.
fn zec_to_zats(zec: f64) -> u64 {
    (zec * 100_000_000.0).round() as u64
}
