//! Minimal read-only JSON-RPC client for the harness node (zcashd).
//!
//! The verifier reads the node for facts the indexer cannot give it — per-height pool totals
//! (`getblock`), a transaction's mined height (`getrawtransaction` + `getblockheader`) — and
//! only those. No wallet RPCs, ever: the node is the chain, not a party to the claim.

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct NodeRpc {
    hostport: String,
    user: String,
    pass: String,
}

impl NodeRpc {
    /// Defaults match the harness (`CANARY_NODE_URL` / `CANARY_NODE_USER` / `CANARY_NODE_PASS`).
    pub fn from_env() -> Self {
        Self {
            hostport: env_hostport(),
            user: std::env::var("CANARY_NODE_USER").unwrap_or_else(|_| "canary".into()),
            pass: std::env::var("CANARY_NODE_PASS").unwrap_or_else(|_| "canary".into()),
        }
    }

    pub fn new(hostport: impl Into<String>, user: impl Into<String>, pass: impl Into<String>) -> Self {
        Self {
            hostport: hostport.into(),
            user: user.into(),
            pass: pass.into(),
        }
    }

    pub fn call(&self, method: &str, params: Value) -> Result<Value> {
        let body = serde_json::json!({
            "jsonrpc": "1.0",
            "id": "canary-report",
            "method": method,
            "params": params,
        })
        .to_string();
        let auth = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", self.user, self.pass),
        );
        let request = format!(
            "POST / HTTP/1.1\r\nHost: {}\r\nAuthorization: Basic {auth}\r\n\
             Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            self.hostport,
            body.len()
        );
        let mut stream = std::net::TcpStream::connect(&self.hostport)
            .with_context(|| format!("connecting to the node at {}", self.hostport))?;
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
        let json: Value = serde_json::from_str(response_body).context("parsing the node response")?;
        if let Some(error) = json.get("error") {
            if !error.is_null() {
                bail!("node RPC `{method}` failed: {error}");
            }
        }
        Ok(json.get("result").cloned().unwrap_or(Value::Null))
    }

    /// The node's view of a transaction: `(height, raw bytes)` or `None` when unknown.
    pub fn transaction(&self, txid: &str) -> Result<Option<(u32, Vec<u8>)>> {
        let raw = match self.call("getrawtransaction", serde_json::json!([txid])) {
            Ok(Value::String(hex)) => hex,
            Ok(_) => bail!("getrawtransaction {txid} returned no data"),
            // -5: no such mempool or blockchain transaction.
            Err(e) if e.to_string().contains("-5") => return Ok(None),
            Err(e) => return Err(e),
        };
        let decoded = self.call("getrawtransaction", serde_json::json!([txid, 1]))?;
        let blockhash = decoded
            .get("blockhash")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("transaction {txid} is not mined (no blockhash)"))?;
        let header = self.call("getblockheader", serde_json::json!([blockhash]))?;
        let height = header
            .get("height")
            .and_then(Value::as_u64)
            .ok_or_else(|| anyhow!("getblockheader {blockhash} returned no height"))?;
        Ok(Some((height as u32, hex_decode(&raw)?)))
    }

    /// Value-pool totals at a height, from that height's block.
    pub fn pool_totals(&self, height: u32) -> Result<PoolTotals> {
        let hash = self
            .call("getblockhash", serde_json::json!([height]))?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| anyhow!("getblockhash {height} returned no hash"))?;
        let block = self.call("getblock", serde_json::json!([hash, 1]))?;
        let mut totals = PoolTotals::default();
        let pools = block
            .get("valuePools")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("block {height} has no valuePools"))?;
        for pool in pools {
            let id = pool.get("id").and_then(Value::as_str).unwrap_or_default();
            let value = pool.get("chainValueZat").and_then(Value::as_u64);
            match (id, value) {
                ("transparent", Some(v)) => totals.transparent_zat = v,
                ("sapling", Some(v)) => totals.sapling_zat = v,
                ("orchard", Some(v)) => totals.orchard_zat = v,
                _ => {}
            }
        }
        Ok(totals)
    }

    /// Every transaction in a block as `(txid, raw bytes)`.
    ///
    /// Verbosity 2 returns each transaction with its raw hex, so a full-chain scan costs two RPC
    /// calls per block instead of three per transaction — the difference between seconds and
    /// minutes when the question is "is this evidence root anchored anywhere?".
    pub fn block_transactions(&self, height: u32) -> Result<Vec<(String, Vec<u8>)>> {
        let hash = self
            .call("getblockhash", serde_json::json!([height]))?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| anyhow!("getblockhash {height} returned no hash"))?;
        let block = self.call("getblock", serde_json::json!([hash, 2]))?;
        let transactions = block
            .get("tx")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("block {height} has no transactions"))?;
        let mut out = Vec::with_capacity(transactions.len());
        for tx in transactions {
            // A coinbase with no shielded data still parses; one whose hex is missing is a node
            // that cannot be asked the question, not an answer.
            let txid = tx
                .get("txid")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("a transaction in block {height} has no txid"))?;
            let hex = tx
                .get("hex")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("transaction {txid} in block {height} has no raw hex"))?;
            out.push((txid.to_string(), hex_decode(hex)?));
        }
        Ok(out)
    }

    pub fn chain_name(&self) -> Result<String> {
        let info = self.call("getblockchaininfo", serde_json::json!([]))?;
        info.get("chain")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| anyhow!("getblockchaininfo returned no chain name"))
    }

    pub fn tip_height(&self) -> Result<u32> {
        let info = self.call("getblockchaininfo", serde_json::json!([]))?;
        info.get("blocks")
            .and_then(Value::as_u64)
            .map(|h| h as u32)
            .ok_or_else(|| anyhow!("getblockchaininfo returned no height"))
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PoolTotals {
    pub transparent_zat: u64,
    pub sapling_zat: u64,
    pub orchard_zat: u64,
}

fn env_hostport() -> String {
    let url = std::env::var("CANARY_NODE_URL").unwrap_or_else(|_| "http://127.0.0.1:18232".into());
    url.strip_prefix("http://")
        .unwrap_or(&url)
        .trim_end_matches('/')
        .to_string()
}

pub fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 {
        bail!("odd-length hex string");
    }
    let mut out = vec![0u8; hex.len() / 2];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let s = std::str::from_utf8(chunk)?;
        out[i] = u8::from_str_radix(s, 16).with_context(|| format!("bad hex `{s}`"))?;
    }
    Ok(out)
}
