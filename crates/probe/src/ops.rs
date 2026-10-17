//! `zanary-probe address | fund | shield | pay` — move real value on the harness
//! chain with the probe's own librustzcash wallet.
//!
//! * `address` — derive the wallet's transparent / Orchard receivers from the
//!   imported dev UFVK.
//! * `fund` — move transparent coinbase from the node wallet (mined if needed) to
//!   the probe wallet's transparent receiver, using the node's transparent-only
//!   `sendtoaddress` RPC.
//! * `shield` — spend the wallet's transparent UTXOs into its own Orchard pool:
//!   propose (`propose_shielding`) -> build/prove/sign
//!   (`create_proposed_transactions`, our own librustzcash code) -> broadcast via
//!   the indexer (`SendTransaction`).
//! * `pay` — send a shielded Orchard payment with a ZIP-302 memo, with an optional
//!   explicit zcashd-style privacy policy that this code enforces itself.
//!
//! The zcashd node wallet is only ever used through `sendtoaddress` (transparent);
//! no `z_*` RPC is called. All shielded operations are the probe's own code.

use anyhow::{anyhow, bail, Context, Result};
use std::{collections::HashMap, convert::Infallible, path::PathBuf};

use zcash_client_backend::{
    data_api::{
        wallet::{
            create_proposed_transactions,
            input_selection::{GreedyInputSelector, SpendPolicy},
            propose_shielding, propose_transfer, ConfirmationsPolicy, SpendingKeys,
        },
        Account as _, CoinbaseFilter, InputSource, WalletRead, WalletWrite,
    },
    decrypt_transaction,
    fees::{standard::SingleOutputChangeStrategy, DustOutputPolicy, StandardFeeRule},
    proposal::Proposal,
    proto::service::{RawTransaction, TxFilter},
    wallet::OvkPolicy,
};
use zcash_client_sqlite::AccountUuid;
use zcash_keys::{
    address::Address,
    encoding::AddressCodec as _,
    keys::{ReceiverRequirement, UnifiedAddressRequest},
};
use zcash_primitives::transaction::fees::FeeRule;
use zcash_proofs::prover::LocalTxProver;
use zcash_protocol::{
    local_consensus::LocalNetwork, memo::MemoBytes, value::Zatoshis, ShieldedPool, TxId,
};
use zcash_transparent::address::TransparentAddress;
use zip32::DiversifierIndex;

use crate::{
    core::{self, BalanceSnapshot, Probe, ProbeDb, WalletProfile},
    node::NodeRpc,
};

/// Blocks mined to confirm a broadcast transaction (ZIP 315 default policy wants
/// 10 confirmations for outputs received from others; one extra block keeps the
/// just-mined note firmly inside that window).
pub(crate) const CONFIRM_BLOCKS: u64 = 11;

// ---------------------------------------------------------------------------
// argument parsing
// ---------------------------------------------------------------------------

/// A tiny `--flag value` parser (no external argument crate): every argument is a
/// `--name`; `--name value` binds a value, a bare `--name` is a boolean `true`.
pub(crate) struct Args {
    values: HashMap<String, String>,
    usage: &'static str,
}

impl Args {
    pub(crate) fn parse(args: &[String], usage: &'static str) -> Result<Self> {
        let mut values = HashMap::new();
        let mut iter = args.iter().peekable();
        while let Some(arg) = iter.next() {
            if !arg.starts_with("--") {
                bail!("unexpected argument `{arg}`\nusage: {usage}");
            }
            let value = match iter.peek() {
                Some(next) if !next.starts_with("--") => iter.next().expect("peeked").clone(),
                _ => "true".to_string(),
            };
            if values.insert(arg.clone(), value).is_some() {
                bail!("duplicate argument `{arg}`\nusage: {usage}");
            }
        }
        Ok(Self { values, usage })
    }

    pub(crate) fn value(&mut self, name: &str) -> Option<String> {
        self.values.remove(name)
    }

    pub(crate) fn flag(&mut self, name: &str) -> bool {
        self.values.remove(name).is_some_and(|v| v == "true")
    }

    pub(crate) fn require(&mut self, name: &str) -> Result<String> {
        self.value(name)
            .ok_or_else(|| anyhow!("missing required argument `{name}`\nusage: {}", self.usage))
    }

    pub(crate) fn optional_u32(&mut self, name: &str) -> Result<Option<u32>> {
        self.value(name)
            .map(|v| {
                v.parse::<u32>()
                    .with_context(|| format!("`{name} {v}` is not a valid number"))
            })
            .transpose()
    }

    pub(crate) fn finish(self) -> Result<()> {
        if !self.values.is_empty() {
            let mut unknown: Vec<String> = self.values.into_keys().collect();
            unknown.sort();
            bail!(
                "unknown argument(s) {}\nusage: {}",
                unknown.join(", "),
                self.usage
            );
        }
        Ok(())
    }
}

/// Parses a decimal ZEC amount into zatoshis without touching floats.
pub(crate) fn parse_zec(s: &str) -> Result<u64> {
    let (whole_s, frac_s) = match s.split_once('.') {
        Some((whole, frac)) => (whole, frac),
        None => (s, ""),
    };
    if whole_s.is_empty() && frac_s.is_empty() {
        bail!("`{s}` is not a ZEC amount (use a decimal like `1` or `0.01`)");
    }
    if !whole_s.chars().all(|c| c.is_ascii_digit()) || !frac_s.chars().all(|c| c.is_ascii_digit()) {
        bail!("`{s}` is not a valid ZEC amount (use a decimal like `1` or `0.01`)");
    }
    if frac_s.len() > 8 {
        bail!("`{s}` has more than 8 decimal places (1 zatoshi = 0.00000001 ZEC)");
    }
    let whole: u64 = if whole_s.is_empty() {
        0
    } else {
        whole_s.parse().map_err(|_| anyhow!("`{s}` is too large"))?
    };
    let frac: u64 = if frac_s.is_empty() {
        0
    } else {
        format!("{frac_s:0<8}")
            .parse()
            .map_err(|_| anyhow!("`{s}` is not a valid ZEC amount"))?
    };
    let zats = whole
        .checked_mul(100_000_000)
        .and_then(|v| v.checked_add(frac))
        .ok_or_else(|| anyhow!("`{s}` is too large"))?;
    if zats == 0 {
        bail!("amount must be greater than zero");
    }
    Ok(zats)
}

// ---------------------------------------------------------------------------
// privacy policy (zcashd `privacyPolicy` semantics, enforced by our own code)
// ---------------------------------------------------------------------------

/// The zcashd `privacyPolicy` ladder, least to most permissive. Each value permits
/// its own class of revelation and every class below it (zcashd's cumulative
/// ordering), so `FullPrivacy` allows only fully-shielded same-pool payments while
/// `NoPrivacy` allows everything.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PrivacyPolicy {
    FullPrivacy,
    AllowRevealedAmounts,
    AllowRevealedRecipients,
    AllowRevealedSenders,
    AllowFullyTransparent,
    NoPrivacy,
}

impl PrivacyPolicy {
    pub(crate) fn parse(s: &str) -> Result<Self> {
        Ok(
            match s.to_ascii_lowercase().replace(['_', '-'], "").as_str() {
                "fullprivacy" => Self::FullPrivacy,
                "allowrevealedamounts" => Self::AllowRevealedAmounts,
                "allowrevealedrecipients" => Self::AllowRevealedRecipients,
                "allowrevealedsenders" => Self::AllowRevealedSenders,
                "allowfullytransparent" => Self::AllowFullyTransparent,
                "noprivacy" => Self::NoPrivacy,
                _ => bail!(
                    "unknown privacy policy `{s}` — expected one of FullPrivacy, \
                 AllowRevealedAmounts, AllowRevealedRecipients, AllowRevealedSenders, \
                 AllowFullyTransparent, NoPrivacy"
                ),
            },
        )
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::FullPrivacy => "FullPrivacy",
            Self::AllowRevealedAmounts => "AllowRevealedAmounts",
            Self::AllowRevealedRecipients => "AllowRevealedRecipients",
            Self::AllowRevealedSenders => "AllowRevealedSenders",
            Self::AllowFullyTransparent => "AllowFullyTransparent",
            Self::NoPrivacy => "NoPrivacy",
        }
    }
}

/// Refuses the operation when the requested policy does not permit what the
/// transaction would reveal. Nothing is proposed, built or broadcast when this
/// fails.
pub(crate) fn check_policy(
    requested: PrivacyPolicy,
    required: PrivacyPolicy,
    revealed: &str,
    operation: &str,
) -> Result<()> {
    if requested < required {
        bail!(
            "privacy policy {} is too strict for this {operation}: it would reveal {revealed}.\n  \
             Re-run with `--policy {}` (or a more permissive policy). Nothing was broadcast.",
            requested.name(),
            required.name()
        );
    }
    Ok(())
}

/// What a payment to `recipient` would reveal, given that the probe spends only
/// Orchard notes and sends Orchard change. Returns the least permissive policy
/// that permits the payment, plus a description of the revelation (empty when
/// nothing is revealed).
pub(crate) fn required_policy_for_recipient(recipient: &Address) -> (PrivacyPolicy, &'static str) {
    match recipient {
        Address::Transparent(_) | Address::Tex(_) => (
            PrivacyPolicy::AllowRevealedRecipients,
            "the transparent recipient and the payment amount",
        ),
        Address::Sapling(_) => (
            PrivacyPolicy::AllowRevealedAmounts,
            "the amount (Orchard -> Sapling pool crossing)",
        ),
        Address::Unified(ua) => {
            if ua.has_orchard() {
                (PrivacyPolicy::FullPrivacy, "")
            } else if ua.has_sapling() {
                (
                    PrivacyPolicy::AllowRevealedAmounts,
                    "the amount (Orchard -> Sapling pool crossing)",
                )
            } else {
                (
                    PrivacyPolicy::AllowRevealedRecipients,
                    "the transparent recipient and the payment amount",
                )
            }
        }
    }
}

/// The pool the payment's value lands in, from the address that receives it.
pub(crate) fn recipient_pool(recipient: &Address) -> zanary_observation::Pool {
    // One function decides this for both the record and the spend policy above: a unified address
    // that carries an Orchard receiver is paid *into Orchard* even when it also carries Sapling or
    // transparent receivers, because the wallet picks the Orchard receiver — which is what
    // `required_policy_for_recipient` already assumes. Keeping them in one place means the recorded
    // pool legs and the policy can never disagree.
    match recipient {
        Address::Transparent(_) | Address::Tex(_) => zanary_observation::Pool::Transparent,
        Address::Sapling(_) => zanary_observation::Pool::Sapling,
        Address::Unified(ua) => {
            if ua.has_orchard() {
                zanary_observation::Pool::Orchard
            } else if ua.has_sapling() {
                zanary_observation::Pool::Sapling
            } else {
                zanary_observation::Pool::Transparent
            }
        }
    }
}

/// Human-readable description of the recipient's receiver set.
pub(crate) fn describe_address(recipient: &Address) -> String {
    match recipient {
        Address::Transparent(_) => "transparent".to_string(),
        Address::Tex(_) => "transparent source-only (TEX)".to_string(),
        Address::Sapling(_) => "sapling".to_string(),
        Address::Unified(ua) => {
            let mut pools = Vec::new();
            if ua.has_orchard() {
                pools.push("orchard");
            }
            if ua.has_sapling() {
                pools.push("sapling");
            }
            if ua.has_transparent() {
                pools.push("transparent");
            }
            format!("unified ({} receivers)", pools.join(" + "))
        }
    }
}

// ---------------------------------------------------------------------------
// shared helpers
// ---------------------------------------------------------------------------

fn print_header(command: &str) {
    println!(
        "zanary-probe {} {command} — ZANARY (ZECATHON, Core & Tooling)",
        env!("CARGO_PKG_VERSION")
    );
    println!("  network           : regtest (throwaway harness chain)");
    println!("  indexer endpoint  : {}", core::endpoint());
    println!("  wallet db         : {}", core::db_path().display());
}

/// Returns the wallet's default external transparent receiver (or the one at
/// child index `index`), registering the BIP-44 gap window first if the account
/// has no transparent receivers yet.
///
/// This registration matters: transparent-output detection in this stack happens
/// only through the `GetAddressUtxos` refresh for *registered* receivers, so an
/// address that is not in the wallet's `addresses` table would never see its funds.
fn transparent_receiver(
    db: &mut ProbeDb,
    account: AccountUuid,
    index: Option<u32>,
) -> Result<TransparentAddress> {
    let mut receivers = db
        .get_transparent_receivers(account, false, false)
        .context("listing the account's transparent receivers")?;
    if receivers.is_empty() {
        // Pre-generate the external gap window and reserve the first unexposed
        // receiver: a unified-address request that *requires* a p2pkh receiver
        // derives and caches the account's default transparent address.
        let request = UnifiedAddressRequest::custom(
            ReceiverRequirement::Allow,
            ReceiverRequirement::Allow,
            ReceiverRequirement::Require,
        )
        .map_err(|e| anyhow!("building the transparent address request: {e}"))?;
        db.get_next_available_address(account, request)
            .context("generating the account's default transparent receiver")?
            .ok_or_else(|| anyhow!("the wallet could not generate a transparent receiver"))?;
        receivers = db
            .get_transparent_receivers(account, false, false)
            .context("listing the account's transparent receivers")?;
    }

    let mut known: Vec<(u32, TransparentAddress)> = receivers
        .iter()
        .filter_map(|(address, meta)| meta.address_index().map(|i| (i.index(), *address)))
        .collect();
    known.sort_by_key(|(index, _)| *index);
    if known.is_empty() {
        bail!("the wallet has no derived transparent receivers for this account");
    }
    match index {
        None => Ok(known[0].1),
        Some(wanted) => known
            .iter()
            .find(|(index, _)| *index == wanted)
            .map(|(_, address)| *address)
            .ok_or_else(|| {
                anyhow!(
                    "no transparent receiver at child index {wanted} is registered \
                     (known indices: {:?})",
                    known.iter().map(|(index, _)| *index).collect::<Vec<_>>()
                )
            }),
    }
}

/// Builds, proves and signs the proposal with our own librustzcash code, then
/// broadcasts the resulting transaction through the harness indexer.
pub(crate) async fn build_sign_broadcast<FeeRuleT, NoteRefT>(
    probe: &mut Probe,
    proposal: &Proposal<FeeRuleT, NoteRefT>,
    ovk: OvkPolicy,
) -> Result<TxId>
where
    FeeRuleT: FeeRule,
    <FeeRuleT as FeeRule>::Error: std::fmt::Display,
    NoteRefT: std::fmt::Display,
{
    // Our own code end to end: the wallet builder selects the inputs, constructs
    // the Orchard bundle (its proving key is built internally by
    // `zcash_primitives`), proves and signs; the Sapling prover is required by the
    // API's type even though these transactions carry no Sapling components.
    let keys = SpendingKeys::from_unified_spending_key(probe.usk.clone());
    let prover = LocalTxProver::bundled();
    let txids = create_proposed_transactions::<
        ProbeDb,
        LocalNetwork,
        Infallible,
        FeeRuleT,
        Infallible,
        NoteRefT,
    >(
        &mut probe.db,
        &probe.network,
        &prover,
        &prover,
        &keys,
        ovk,
        proposal,
        None,
    )
    .map_err(|e| anyhow!("building, proving and signing the transaction: {e}"))?;
    let txid = *txids.first();

    let tx = probe
        .db
        .get_transaction(txid)
        .context("reading the built transaction back from the wallet")?
        .ok_or_else(|| {
            anyhow!("transaction {txid} was built but is not present in the wallet database")
        })?;
    let mut bytes = Vec::new();
    tx.write(&mut bytes)
        .context("serializing the transaction")?;

    let response = probe
        .client
        .send_transaction(RawTransaction {
            data: bytes.clone(),
            height: 0,
        })
        .await
        .context("SendTransaction to the indexer")?
        .into_inner();
    if response.error_code != 0 {
        bail!(
            "the indexer rejected the transaction: {} (error_code {})\n  \
             nothing was mined; the wallet has recorded the transaction as sent",
            response.error_message,
            response.error_code
        );
    }
    println!(
        "  broadcast         : accepted by the indexer ({} bytes)",
        bytes.len()
    );
    Ok(txid)
}

/// Mines `blocks` blocks with the node and re-syncs the wallet so it sees the
/// mined transaction.
pub(crate) async fn mine_and_resync(probe: &mut Probe, blocks: u64) -> Result<()> {
    let node = NodeRpc::from_env()?;
    let before = node.block_count().await?;
    node.generate(blocks).await?;
    let after = node.block_count().await?;
    println!(
        "  mined             : {} block(s) (chain height {} -> {})",
        after - before,
        before,
        after
    );
    core::sync_wallet(
        &mut probe.db,
        &mut probe.client,
        &probe.network,
        &probe.db_path,
    )
    .await?;
    Ok(())
}

/// Renders decrypted memo bytes: trailing NUL padding removed, lossy UTF-8.
pub(crate) fn memo_text(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .rposition(|b| *b != 0)
        .map(|i| i + 1)
        .unwrap_or(0);
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

// ---------------------------------------------------------------------------
// anchor
// ---------------------------------------------------------------------------

/// `zanary-probe anchor --report <report.json> [--out <anchor.json>] [--amount-zat N] [--account N]`
///
/// Turns a report into a claim with a public timestamp: the report's `evidence_root` and
/// `report_hash` go into a **shielded memo** paid to the anchor beacon's next fresh receiver, and
/// the chain then says for everyone that this evidence existed by that height.
///
/// The beacon is a third wallet (its own seed, its own database), its viewing key is published, and
/// the anchor carries no identity beyond the hashes — the design and its disclosures are in
/// `zanary_report::anchor` and `docs/CLAIMS.md`.
///
/// Nothing here trusts the report file: the hash and the root are recomputed from the report's own
/// fields, and the run is refused if the file does not hash to itself. Anchoring a report that has
/// been edited by hand would publish a claim about a run nobody made.
pub async fn anchor(raw: &[String]) -> Result<()> {
    const USAGE: &str = "zanary-probe anchor --report <report.json> [--out <anchor.json>] \
                         [--amount-zat N] [--account N]";
    let mut args = Args::parse(raw, USAGE)?;
    let report_path = args.require("--report")?;
    let out = args.value("--out");
    let amount_zat = args
        .optional_u32("--amount-zat")?
        .map(u64::from)
        .unwrap_or(zanary_report::anchor::ANCHOR_AMOUNT_ZAT);
    let account_no = args.optional_u32("--account")?.unwrap_or(0);
    args.finish()?;

    let report: zanary_report::Report = serde_json::from_str(
        &std::fs::read_to_string(&report_path)
            .with_context(|| format!("reading the report {report_path}"))?,
    )
    .with_context(|| format!("parsing the report {report_path}"))?;

    // Recompute, never trust: the file's `report_hash` must be the hash of its own canonical JSON,
    // and the root must be the Merkle root of its own evidence.
    let report_hash = zanary_report::hash_report(&report)?;
    if report.report_hash != report_hash {
        bail!(
            "the report's stored hash is {} but its canonical JSON hashes to {report_hash} — \
             anchoring it would publish a claim no run supports",
            report.report_hash
        );
    }
    let evidence_root = zanary_report::evidence_root(&report.evidence)?;

    let anchor = zanary_report::anchor::Anchor {
        v: zanary_report::anchor::ANCHOR_VERSION,
        chain: report.chain.clone(),
        flow_commit: report.flow_commit.clone(),
        report_hash: report_hash.clone(),
        evidence_root: evidence_root.clone(),
        verified_at_height: report.verified_at_height,
    };
    let memo_text = anchor.memo_text()?;
    let memo = MemoBytes::from_bytes(memo_text.as_bytes())
        .map_err(|e| anyhow!("the anchor memo was rejected: {e}"))?;

    print_header("anchor");
    println!("  report            : {report_path}");
    println!(
        "  flow              : {} @ {}",
        report.flow_id,
        &report.flow_commit[..8.min(report.flow_commit.len())]
    );
    println!("  report hash       : {}", report.report_hash);
    println!("  evidence root     : {evidence_root}");
    println!("  verified height   : {}", report.verified_at_height);
    println!(
        "  anchor amount     : {} (the note the memo rides on, not a payment)",
        core::format_zats_signed(amount_zat as i128)
    );

    let mut payer = core::bootstrap(account_no).await?;
    let mut beacon = core::bootstrap_with(WalletProfile::beacon(0)).await?;
    // A fresh diversified receiver per anchor: anchors are countable by index (CAₙ) without being
    // linkable by destination address.
    let (beacon_address, beacon_index) = beacon
        .db
        .get_next_available_address(beacon.account, UnifiedAddressRequest::ORCHARD)
        .context("deriving the beacon's next Orchard receiver")?
        .ok_or_else(|| anyhow!("the beacon wallet could not derive another Orchard receiver"))?;
    let beacon_receiver = beacon_address.encode(&beacon.network);
    println!("  beacon            : {beacon_receiver}");
    println!(
        "  beacon tag        : {beacon_index:?} (this anchor's tag; the wallet draws diversifier \
         indices, so the tag is a label, not a sequence number)"
    );
    println!(
        "  beacon UFVK       : {}",
        beacon.ufvk.encode(&beacon.network)
    );
    println!("  anchor memo       : {memo_text:?}");

    let recipient = Address::decode(&payer.network, &beacon_receiver).ok_or_else(|| {
        anyhow!("the beacon issued an address this chain cannot decode: {beacon_receiver}")
    })?;
    let (required, revealed) = required_policy_for_recipient(&recipient);
    // Anchoring is a fully-shielded, same-pool payment, so the strictest policy is the only one
    // that could be satisfied: a revealed amount here would publish nothing but would look wrong.
    check_policy(PrivacyPolicy::FullPrivacy, required, revealed, "anchor")?;
    let proposal = propose_shielded_payment(&mut payer, &recipient, amount_zat, &memo)?;
    let txid = build_sign_broadcast(&mut payer, &proposal, OvkPolicy::Sender).await?;
    println!("  anchor txid       : {txid}");
    mine_and_resync(&mut payer, CONFIRM_BLOCKS).await?;
    let height = payer
        .client
        .get_transaction(TxFilter {
            block: None,
            index: 0,
            hash: txid.as_ref().to_vec(),
        })
        .await
        .context("GetTransaction from the indexer")?
        .into_inner()
        .height;
    if height == 0 {
        bail!("the anchor transaction {txid} is not mined yet, so it has no height to record");
    }
    let height = height as u32;

    let record = zanary_report::anchor::AnchorRecord {
        anchor,
        memo: memo_text.clone(),
        txid: txid.to_string(),
        height,
        beacon_receiver: beacon_receiver.clone(),
        beacon_receiver_index: format!("{beacon_index:?}"),
        beacon_ufvk: beacon.ufvk.encode(&beacon.network),
        chain: report.chain.clone(),
        payer_label: "zanary-probe dev wallet (regtest harness)".to_string(),
    };
    let out_path = match &out {
        Some(path) => PathBuf::from(path),
        None => anchor_path_for(&report_path),
    };
    record.write(&out_path)?;

    println!("  anchored at       : height {height}");
    println!("  anchor record     : {}", out_path.display());
    println!();
    println!("  A stranger can now check this with no trust in this rig:");
    println!(
        "    zanary-report verify-anchor --report {report_path} \\\n     \
         --beacon-ufvk {} --node http://127.0.0.1:18232",
        record.beacon_ufvk
    );
    Ok(())
}

/// Where an anchor record goes when `--out` is not given: beside the report it anchors, so a run
/// directory carries `run.json`, `findings.json`, `report.json` and — once anchored — `anchor.json`.
fn anchor_path_for(report_path: &str) -> PathBuf {
    match report_path.strip_suffix("/report.json") {
        Some(dir) => PathBuf::from(format!("{dir}/anchor.json")),
        None => PathBuf::from(format!("{report_path}.anchor.json")),
    }
}

// ---------------------------------------------------------------------------
// address
// ---------------------------------------------------------------------------

pub async fn address(raw: &[String]) -> Result<()> {
    const USAGE: &str =
        "zanary-probe address [--transparent | --orchard] [--index N] [--account N]";
    let mut args = Args::parse(raw, USAGE)?;
    let want_transparent = args.flag("--transparent");
    let want_orchard = args.flag("--orchard");
    let index = args.optional_u32("--index")?;
    let account_no = args.optional_u32("--account")?.unwrap_or(0);
    args.finish()?;
    if want_transparent && want_orchard {
        bail!("choose one of --transparent or --orchard\nusage: {USAGE}");
    }

    let mut probe = core::bootstrap(account_no).await?;
    print_header("address");
    println!(
        "  account           : {} ({:?})",
        core::account_name(account_no),
        probe.account
    );
    println!("  --- receivers ---");

    if !want_orchard {
        let taddr = transparent_receiver(&mut probe.db, probe.account, index)?;
        println!("  transparent       : {}", taddr.encode(&probe.network));
    }
    if !want_transparent {
        let index = index.unwrap_or(0);
        let (ua, diversifier) = probe
            .ufvk
            .find_address(
                DiversifierIndex::from(index),
                UnifiedAddressRequest::ORCHARD,
            )
            .map_err(|e| anyhow!("deriving the Orchard-only receiver at index {index}: {e}"))?;
        println!("  orchard           : {}", ua.encode(&probe.network));
        println!("  diversifier index : {diversifier:?}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// fund
// ---------------------------------------------------------------------------

pub async fn fund(raw: &[String]) -> Result<()> {
    const USAGE: &str = "zanary-probe fund --zec <amount>";
    let mut args = Args::parse(raw, USAGE)?;
    let amount = args.require("--zec")?;
    args.finish()?;
    let zats = parse_zec(&amount)?;

    let mut probe = core::bootstrap(0).await?;
    print_header("fund");
    println!(
        "  account           : {} ({:?})",
        core::DEV_ACCOUNT_NAME,
        probe.account
    );

    let recipient = transparent_receiver(&mut probe.db, probe.account, None)?;
    let recipient = recipient.encode(&probe.network);
    println!("  recipient (t)     : {recipient}");
    println!(
        "  amount            : {}",
        core::format_zats_signed(zats as i128)
    );

    let node = NodeRpc::from_env()?;
    let mut height = node.block_count().await?;
    let (mut spendable, immature) = node.transparent_balances().await?;
    println!(
        "  node wallet       : {} spendable · {} immature · chain height {height}",
        core::format_zats_signed(spendable as i128),
        core::format_zats_signed(immature as i128)
    );

    // The node funds itself from mined coinbase: if it has no mature coinbase to
    // cover the transfer, mine enough blocks to mature some (100-block maturity).
    const NODE_FEE_MARGIN: u64 = 100_000; // 0.001 ZEC headroom for the node's send fee
    if spendable < zats.saturating_add(NODE_FEE_MARGIN) {
        println!(
            "  node wallet       : no mature coinbase covers the transfer — mining 110 blocks \
             to mature coinbase..."
        );
        node.generate(110).await?;
        height = node.block_count().await?;
        (spendable, _) = node.transparent_balances().await?;
        println!(
            "  node wallet       : after mining: {} spendable · chain height {height}",
            core::format_zats_signed(spendable as i128)
        );
    }
    if spendable < zats.saturating_add(NODE_FEE_MARGIN) {
        bail!(
            "the node wallet still has only {} spendable after mining — cannot fund {} \
             (the harness node funds itself from mined coinbase; mine more blocks or ask \
             for a smaller amount)",
            core::format_zats_signed(spendable as i128),
            core::format_zats_signed(zats as i128)
        );
    }

    let txid = node.send_to_address(&recipient, zats).await?;
    println!("  sendtoaddress     : {txid}");

    let hashes = node.generate(CONFIRM_BLOCKS).await?;
    height = node.block_count().await?;
    println!(
        "  mined             : {} block(s) to confirm (chain height {height})",
        hashes.len()
    );

    core::sync_wallet(
        &mut probe.db,
        &mut probe.client,
        &probe.network,
        &probe.db_path,
    )
    .await?;
    let snapshot = BalanceSnapshot::capture(&probe.db, probe.account)?;
    println!(
        "  wallet view       : transparent {} · orchard {} · total {}",
        core::format_zats_signed(snapshot.transparent),
        core::format_zats_signed(snapshot.orchard),
        core::format_zats_signed(snapshot.total)
    );
    println!();
    println!("  next: zanary-probe shield");
    Ok(())
}

// ---------------------------------------------------------------------------
// shield
// ---------------------------------------------------------------------------

pub async fn shield(raw: &[String]) -> Result<()> {
    const USAGE: &str = "zanary-probe shield [--account N] [--policy <privacyPolicy>]";
    let mut args = Args::parse(raw, USAGE)?;
    let account_no = args.optional_u32("--account")?.unwrap_or(0);
    let policy = match args.value("--policy") {
        Some(policy) => PrivacyPolicy::parse(&policy)?,
        // Shielding's own minimum: the transparent senders and amounts are public
        // by construction, so the default is the least permissive policy that
        // permits the operation.
        None => PrivacyPolicy::AllowRevealedSenders,
    };
    args.finish()?;
    check_policy(
        policy,
        PrivacyPolicy::AllowRevealedSenders,
        "the transparent senders and their amounts",
        "shielding transaction",
    )?;

    let mut probe = core::bootstrap(account_no).await?;
    print_header("shield");
    println!(
        "  account           : {} ({:?})",
        core::account_name(account_no),
        probe.account
    );
    println!(
        "  policy            : {} (shielding reveals the transparent senders and amounts)",
        policy.name()
    );

    let before = BalanceSnapshot::capture(&probe.db, probe.account)?;
    println!(
        "  before            : transparent {} · orchard {} · total {}",
        core::format_zats_signed(before.transparent),
        core::format_zats_signed(before.orchard),
        core::format_zats_signed(before.total)
    );
    if before.transparent == 0 {
        bail!(
            "no transparent funds in account `{}` — fund the wallet first:\n  \
             zanary-probe fund --zec <amount>   (then the funds need 10 confirmations)",
            core::account_name(account_no)
        );
    }

    let from_addrs: Vec<TransparentAddress> = probe
        .db
        .get_transparent_receivers(probe.account, true, true)
        .context("listing the account's transparent receivers")?
        .into_keys()
        .collect();
    if from_addrs.is_empty() {
        bail!("no transparent receivers are registered for this account");
    }

    let selector = GreedyInputSelector::<ProbeDb>::new();
    let change_strategy = SingleOutputChangeStrategy::<ProbeDb>::new(
        StandardFeeRule::Zip317,
        None,
        ShieldedPool::Orchard,
        DustOutputPolicy::default(),
    );
    let proposal = propose_shielding::<
        ProbeDb,
        LocalNetwork,
        GreedyInputSelector<ProbeDb>,
        SingleOutputChangeStrategy<ProbeDb>,
        Infallible,
    >(
        &mut probe.db,
        &probe.network,
        &selector,
        &change_strategy,
        Zatoshis::ZERO,
        &from_addrs,
        probe.account,
        ConfirmationsPolicy::default(),
        CoinbaseFilter::AllTransparentOutputs,
        None,
    )
    .map_err(|e| anyhow!("could not propose a shielding transaction: {e}"))?;

    let step = proposal.steps().first();
    println!(
        "  proposal          : {} step(s) · fee {} · shielding {}",
        proposal.steps().len(),
        core::format_zec(step.balance().fee_required()),
        core::format_zec(step.balance().total())
    );

    let txid = build_sign_broadcast(&mut probe, &proposal, OvkPolicy::Sender).await?;
    println!("  txid              : {txid}");

    mine_and_resync(&mut probe, CONFIRM_BLOCKS).await?;

    let after = BalanceSnapshot::capture(&probe.db, probe.account)?;
    println!(
        "  after             : transparent {} · orchard {} · total {}",
        core::format_zats_signed(after.transparent),
        core::format_zats_signed(after.orchard),
        core::format_zats_signed(after.total)
    );
    core::print_delta(&before.delta(&after));
    println!();
    println!("  next: zanary-probe pay --to <address> --zec <amount> --memo <text>");
    Ok(())
}

// ---------------------------------------------------------------------------
// pay
// ---------------------------------------------------------------------------

/// Proposes an Orchard-to-shielded payment carrying a ZIP-302 memo, under the probe's
/// own spend policy: inputs come only from the account's Orchard pool (never transparent
/// or Sapling), ZIP-317 fees, Orchard change.
///
/// Shared by `zanary-probe pay` and the flow runner, so a flow pays through exactly the
/// path the CLI documents — one implementation, one policy.
pub(crate) fn propose_shielded_payment(
    probe: &mut Probe,
    recipient: &Address,
    zats: u64,
    memo: &MemoBytes,
) -> Result<Proposal<StandardFeeRule, <ProbeDb as InputSource>::NoteRef>> {
    propose_shielded_payments(probe, &[(recipient.clone(), zats, memo.clone())])
}

/// [`propose_shielded_payment`], for a request carrying several payments at once.
///
/// This is the batch settlement path: one ZIP-321 request with N payments becomes *one*
/// proposal, so the wallet selects inputs once, builds one Orchard bundle with N outputs
/// plus change, pays one ZIP-317 fee for the bundle, and broadcasts one transaction. The
/// order of `payments` is the order they are added to the request, which is the caller's
/// business (the flow runner shuffles it and records what it drew).
pub(crate) fn propose_shielded_payments(
    probe: &mut Probe,
    payments: &[(Address, u64, MemoBytes)],
) -> Result<Proposal<StandardFeeRule, <ProbeDb as InputSource>::NoteRef>> {
    if payments.is_empty() {
        bail!("a transaction request needs at least one payment");
    }
    let mut built: Vec<zip321::Payment> = Vec::with_capacity(payments.len());
    for (recipient, zats, memo) in payments {
        built.push(
            zip321::Payment::new(
                recipient.to_zcash_address(&probe.network),
                Some(Zatoshis::from_u64(*zats).map_err(|e| anyhow!("invalid amount: {e}"))?),
                Some(memo.clone()),
                None,
                None,
                vec![],
            )
            .map_err(|e| {
                anyhow!(
                    "building the ZIP-321 payment: {e}\n  \
                     (a memo requires a shielded recipient)"
                )
            })?,
        );
    }
    let request = zip321::TransactionRequest::new(built)
        .map_err(|e| anyhow!("building the ZIP-321 transaction request: {e}"))?;

    let spend_policy = SpendPolicy::shielded_pools([ShieldedPool::Orchard]);
    let selector = GreedyInputSelector::<ProbeDb>::new();
    let change_strategy = SingleOutputChangeStrategy::<ProbeDb>::new(
        StandardFeeRule::Zip317,
        None,
        ShieldedPool::Orchard,
        DustOutputPolicy::default(),
    );
    propose_transfer::<
        ProbeDb,
        LocalNetwork,
        GreedyInputSelector<ProbeDb>,
        SingleOutputChangeStrategy<ProbeDb>,
        Infallible,
    >(
        &mut probe.db,
        &probe.network,
        probe.account,
        &selector,
        &change_strategy,
        request,
        ConfirmationsPolicy::default(),
        &spend_policy,
        None,
        None,
    )
    .map_err(|e| {
        anyhow!(
            "could not propose the payment: {e}\n  \
             (InsufficientFunds? shield first with `zanary-probe shield`; freshly shielded \
             funds need 10 confirmations before they are spendable)"
        )
    })
}

pub async fn pay(raw: &[String]) -> Result<()> {
    const USAGE: &str = "zanary-probe pay --to <address> --zec <amount> --memo <text> \
                         [--policy <privacyPolicy>] [--account N]\n  \
                         zanary-probe pay --invoice <file.json> [--policy <privacyPolicy>] \
                         [--account N]";
    let mut args = Args::parse(raw, USAGE)?;
    // `--invoice <file>`: pay a merchant's invoice. The ZIP-321 URI inside it decides the
    // recipient, the amount and the memo — parsed and cross-checked by `Invoice::from_path` — so
    // this is the standard's intake path, not a second implementation of it.
    let invoice = match args.value("--invoice") {
        Some(path) => Some(crate::flow::Invoice::from_path(&path)?),
        None => None,
    };
    let to = match &invoice {
        Some(invoice) => invoice.receiver.clone(),
        None => args.require("--to")?,
    };
    let amount = match &invoice {
        Some(invoice) => core::format_zec(
            Zatoshis::from_u64(invoice.amount_zat).map_err(|e| anyhow!("invalid amount: {e}"))?,
        ),
        None => args.require("--zec")?,
    };
    let memo_input = match &invoice {
        Some(invoice) => invoice.memo.clone(),
        None => args.require("--memo")?,
    };
    let policy = match args.value("--policy") {
        Some(policy) => PrivacyPolicy::parse(&policy)?,
        // Default: only fully-shielded, same-pool payments pass. Loosening this is
        // an explicit choice, never a silent downgrade.
        None => PrivacyPolicy::FullPrivacy,
    };
    let account_no = args.optional_u32("--account")?.unwrap_or(0);
    args.finish()?;
    let zats = match &invoice {
        Some(invoice) => invoice.amount_zat,
        None => parse_zec(&amount)?,
    };

    let mut probe = core::bootstrap(account_no).await?;
    print_header("pay");
    println!(
        "  from account      : {} ({:?})",
        core::account_name(account_no),
        probe.account
    );

    let recipient = Address::decode(&probe.network, &to).ok_or_else(|| {
        anyhow!(
            "`{to}` is not a valid address for the regtest harness chain\n  \
             (harness addresses: transparent `tm…`/`t2…`, sapling `zregtestsapling1…`, \
             unified `uregtest1…`)"
        )
    })?;
    let (required, revealed) = required_policy_for_recipient(&recipient);
    check_policy(policy, required, revealed, "payment")?;

    let memo = MemoBytes::from_bytes(memo_input.as_bytes())
        .map_err(|e| anyhow!("memo rejected: {e}\n  (ZIP-302 memos are at most 512 bytes)"))?;
    // The ZIP-321 request, spend policy and change strategy live in
    // `propose_shielded_payment` — shared with the flow runner, so both pay through the
    // same code path and the same policy.
    println!("  to                : {to}");
    println!("  recipient         : {}", describe_address(&recipient));
    println!(
        "  amount            : {}",
        core::format_zats_signed(zats as i128)
    );
    println!(
        "  memo              : {:?} ({} bytes, ZIP-302)",
        memo_input,
        memo_input.len()
    );
    println!("  policy            : {}", policy.name());

    let before = BalanceSnapshot::capture(&probe.db, probe.account)?;
    println!(
        "  before            : transparent {} · orchard {} · total {}",
        core::format_zats_signed(before.transparent),
        core::format_zats_signed(before.orchard),
        core::format_zats_signed(before.total)
    );

    // Spend only from the account's Orchard pool (never transparent or Sapling
    // inputs): a shielded payment from Orchard, as the probe's own design demands.
    let proposal = propose_shielded_payment(&mut probe, &recipient, zats, &memo)?;

    let step = proposal.steps().first();
    println!(
        "  proposal          : {} step(s) · fee {}",
        proposal.steps().len(),
        core::format_zec(step.balance().fee_required())
    );

    let txid = build_sign_broadcast(&mut probe, &proposal, OvkPolicy::Sender).await?;
    println!("  txid              : {txid}");

    mine_and_resync(&mut probe, CONFIRM_BLOCKS).await?;

    let after = BalanceSnapshot::capture(&probe.db, probe.account)?;
    println!(
        "  after             : transparent {} · orchard {} · total {}",
        core::format_zats_signed(after.transparent),
        core::format_zats_signed(after.orchard),
        core::format_zats_signed(after.total)
    );
    core::print_delta(&before.delta(&after));

    // --- recipient-side check: decrypt the mined transaction with our own code. ---
    println!();
    println!("  --- recipient-side check (our code decrypts the mined transaction) ---");
    let tx = probe
        .db
        .get_transaction(txid)
        .context("reading the sent transaction back from the wallet")?
        .ok_or_else(|| anyhow!("transaction {txid} is not present in the wallet database"))?;
    let mut ufvks: HashMap<AccountUuid, zcash_keys::keys::UnifiedFullViewingKey> = HashMap::new();
    for account in probe
        .db
        .get_account_ids()
        .context("listing the wallet's accounts")?
    {
        if let Some(account) = probe
            .db
            .get_account(account)
            .context("reading an account")?
        {
            if let Some(ufvk) = account.ufvk() {
                ufvks.insert(account.id(), ufvk.clone());
            }
        }
    }
    let tip = probe
        .db
        .chain_height()
        .context("reading the wallet chain height")?
        .ok_or_else(|| anyhow!("wallet chain height is unknown"))?;
    let decrypted = decrypt_transaction(&probe.network, None, Some(tip), &tx, &ufvks);
    if decrypted.orchard_outputs().is_empty() {
        println!(
            "  (no Orchard output decryptable with this wallet's account UFVKs — \
             the recipient is external to the wallet)"
        );
    }
    for output in decrypted.orchard_outputs() {
        let text = memo_text(output.memo().as_slice());
        let note_value = output.note().0.value().inner();
        println!(
            "  orchard output #{} → account {:?} · {:?} · {} · memo: {}",
            output.index(),
            output.account(),
            output.transfer_type(),
            core::format_zec(
                Zatoshis::from_u64(note_value).expect("consensus-validated note value")
            ),
            if text.is_empty() {
                "(none)".to_string()
            } else {
                format!("{text:?}")
            }
        );
    }
    Ok(())
}
