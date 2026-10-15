# WALLET-NOTES — `zanary-probe wallet` (the D1 gate)

**Date:** 2026-10-11 · **Status:** working, verified against the live harness (140-block regtest chain)
**Scope:** create/open a `zcash_client_sqlite` wallet, import the dev account's UFVK, sync from
`lightwalletd`, print a diversified Orchard-only receiver + balances + synced height. No sending yet.

## The command (and what it really printed)

```bash
cd /home/kiter/hackathon/zanary
ZANARY_LWD_URL=http://127.0.0.1:9067 cargo run -q --bin zanary-probe -- wallet
```

Real output (fresh wallet DB; the 73 `INFO schemerz: Applying migration …` lines are elided):

```text
zanary-probe 0.0.1 wallet — ZANARY (ZECATHON, Core & Tooling)
  network           : regtest (throwaway harness chain)
  indexer endpoint  : http://127.0.0.1:9067
  wallet db         : ./data/wallet/zanary-wallet.db
  dev seed phrase   : abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
                      ^ BIP-39 English test vector — TEST ONLY, holds no real funds
  dev UFVK          : uviewregtest1td4fxalun47yu3erk09atltkwh84wcxzz067cyual0wrs4vskmwym6fshe0v0l2juv4wuc9uxv4kjyhrshfszzexstaxt58965k8zrru6738vds2skke86wfx5m28sz79cxuxlm3tmn7l35t9d2ckr2wxh6z842d09d72cqqc4hkjvth7tx9njslgsjdn9uxrcdkc3wlkp83ljfn264q9la72tzf3rmqma0n7pqy754g4ank5f6dhx89nhhmz5z30emt246l5kuagm5fxsq6mzu8v4uey2wf85n0lxjd4qk8angn3e47dekxpz0fs3495uwmj6mqc28c2ftcyatr3flqay5m0pdrlk3n35jzsa27fn8g2ujuwxgwlzjmzu5fx4gpthhjeweclygmvuvd4rpx0ezzfp2upuu054w26pyyjra9ruql00cmgsndlmzxpm0fdypszupc32qgemav29wax0esy5thphwr5svt4m8szkemqu4cs2wn
  account           : imported (AccountUuid(8e3641a6-65ef-4bcd-95e3-cc0795747ac8), birthday height 2)
  scanned           : 2 ..= 140 (139 blocks, priority Historic)

  --- wallet state ---
  orchard receiver  : uregtest106rqhh28qq6fhdtay885kgugvq9f7sncurjhnkq5z75gu0km9jvgedvvfj093gwkx4egted9q8j82ezwhjjcqv94l2ky8694kyck5c80
  diversifier index : DiversifierIndex([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
  chain tip height  : 140
  fully scanned     : 140
  balance (total)   : 0.00000000 ZEC
    shielded orchard: 0.00000000 ZEC
    shielded sapling: 0.00000000 ZEC
    transparent     : 0.00000000 ZEC
    spendable now   : 0.00000000 ZEC

  next: mine coinbase to the receiver above, then shield and spend it with a memo
```

Checks that matter:

- `uregtest1…` receiver, Orchard-only by construction (`UnifiedAddressRequest::ORCHARD`).
- synced height **140 = `getblockcount` 140** (both `chain tip height` and `fully scanned`).
- all balances zero, exit code 0.
- re-running is idempotent: `account: already imported`, no re-scan, same receiver.
- dead indexer → exit 1 with `transport error → tcp connect error → Connection refused`.
- no subcommand → unchanged Phase-0 behaviour (GetLightdInfo + latest tip), exit 0.

## Stack actually resolved (Cargo.lock)

`zcash_client_sqlite 0.22.0` · `zcash_keys 0.16.1` · `zcash_protocol 0.10.6` (+`local-consensus`) ·
`zip32 0.2.1` · `zcash_transparent 0.10.0` · `zcash_script 0.4.5` · `rand 0.8.8` · `bip39 2.2.2` ·
`async-trait 0.1.92`. `zcash_client_backend 0.24.0` gained the `transparent-inputs` feature.

## Exact APIs used

| Step | API |
|---|---|
| Open DB | `zcash_client_sqlite::WalletDb::for_path(path, params, SystemClock, rand::rngs::OsRng)` |
| Init schema | `zcash_client_sqlite::wallet::init::init_wallet_db(&mut db, None)` — **required** |
| Seed → USK | `zcash_keys::keys::UnifiedSpendingKey::from_seed(&params, &seed, zip32::AccountId::ZERO)` then `.to_unified_full_viewing_key()` |
| Seed fingerprint | `zip32::fingerprint::SeedFingerprint::from_seed(&seed)` + `data_api::Zip32Derivation::new(fp, AccountId::ZERO)` (2 args; the 3rd only exists with `zcashd-compat`) |
| Import account | `WalletWrite::import_account_ufvk(name, &ufvk, &birthday, AccountPurpose::Spending { derivation: Some(..) }, key_source)` (returns `Account`; idempotence via `WalletRead::get_account_for_ufvk` first) |
| Birthday | `AccountBirthday::from_treestate(client.get_tree_state(BlockId{height:1,..}), None)` → scans from height 2 |
| Receiver | `UnifiedFullViewingKey::find_address(DiversifierIndex::from(0u32), UnifiedAddressRequest::ORCHARD)` → `UnifiedAddress::encode(&params)` |
| Sync | **not** `sync::run` (see trap 4) — loop of `client.get_block_range` → `MemoryBlockCache` → `data_api::chain::scan_cached_blocks(&params, &cache, &mut db, from_height, &from_state, limit)` per `WalletRead::suggest_scan_ranges`, plus `WalletWrite::update_chain_tip` and a `GetAddressUtxos` refresh mirroring sync's `refresh_utxos` (`WalletTransparentOutput::from_parts`, `put_received_transparent_utxo`) |
| Block cache | own `MemoryBlockCache` implementing `data_api::chain::{BlockSource, BlockCache}` (`#[async_trait]`), modelled on the trait's documented example |
| Balances | `WalletRead::get_wallet_summary(ConfirmationsPolicy::default())` → `WalletSummary::{chain_tip_height, fully_scanned_height, account_balances}` → `AccountBalance::{total, orchard_balance, sapling_balance, unshielded_balance, spendable_value}` |

## Traps hit (each cost a run)

1. **`Network::Regtest` does not exist** in `zcash_protocol 0.10.6` (`Network` has only
   MainNetwork/TestNetwork). Local chains use `zcash_protocol::local_consensus::LocalNetwork`
   (feature `local-consensus`), whose activation heights must be set to match the node.
   Verified live via `getblockchaininfo.upgrades`: zcashd v6.20.0 knows exactly 7 upgrades
   (Overwinter…NU6), all at height 1 — so `nu6_1/nu6_2/nu6_3 = None`. Setting NU6.1+ to `Some(1)`
   would make the wallet expect branch ids the node cannot produce.
2. **`WalletDb::for_path` only opens the SQLite connection — it does not create the schema.**
   Without `init_wallet_db(&mut db, None)` every wallet call dies with `no such table: accounts`.
3. **lightwalletd rejects `GetTreeState` at height 0**: `must specify a block height or ID (hash)`
   (Go zero-value check — height 0 is indistinguishable from "unset"). `sync::run` asks for
   `scan_start - 1`, so the first scannable height through this indexer is **2**. The account
   birthday is therefore built from the tree state at height 1 (block 1 is coinbase-only on the
   harness chain, so nothing is missed). `zcashd z_gettreestate 0` itself works — it is only the
   indexer API that cannot express it.
4. **`sync::run` (zcash_client_backend 0.24) cannot sync against this harness.** It unconditionally
   fetches Ironwood subtree roots; lightwalletd v0.5.5 forwards `z_getsubtreesbyindex "ironwood"`
   to zcashd v6.20.0, which has no Ironwood pool (Zebra-era), and fails with:
   `GetSubtreeRoots: z_getsubtreesbyindex, error: -8: Requested pool must be "sapling" or "orchard"`.
   Workaround (in `wallet.rs`): the same download → `scan_cached_blocks` → evict loop that
   `sync::run` runs internally, minus the subtree-root fast-forward — a **no-op on this chain**:
   every pool reports 0 complete subtrees (sapling and orchard calls succeed and return empty),
   so scanning the compact blocks maintains the note commitment trees exactly. If the harness ever
   moves to Zebra+Zaino, `sync::run` can come back.
5. **No public `BlockCache` ships for downstream use** in zcash_client_sqlite 0.22 (`FsBlockDb` is
   behind `unstable`; the other caches are test-only), so the probe carries a small in-memory cache
   (`BlockSource::with_blocks` must honour the "requested start height present" rule and `read`
   the contiguous-prefix rule).
6. **rand 0.8, not 0.9:** the sqlite bounds are `R: rand::RngCore` (rand 0.8 → rand_core 0.6);
   `OsRng` from `rand::rngs`, clock from `zcash_client_sqlite::util::SystemClock`.
7. **`zcash_keys` default features are only `["std"]`** — `orchard`/`sapling`/`transparent-inputs`
   must be enabled explicitly, and the whole stack's `transparent-inputs` features must be turned
   on **together** (backend + sqlite + keys), or sync's UTXO step hits the trait's `unimplemented!()`
   default. Enabling them now also gives the account a full UFVK (transparent+sapling+orchard),
   which the funding milestone needs for coinbase-to-t-addr; importing a narrower UFVK first would
   collide on later import ("FVK collision even if UFVKs do not match exactly").
8. **Spending-purpose import:** `AccountPurpose::Spending { derivation: Some(dev-seed fingerprint) }`
   so the wallet tracks spend data — the USK is derived from the dev seed in code, and the next
   milestone spends from this account. `AccountUuid` is random per import; the receiver is
   deterministic (same UFVK → same address).

## Reproduce

```bash
cd /home/kiter/hackathon/zanary/harness && docker compose up -d
docker exec zanary-zcashd zcash-cli -regtest -rpcuser=zanary -rpcpassword=zanary generate 140
cd .. && ZANARY_LWD_URL=http://127.0.0.1:9067 cargo run -q --bin zanary-probe -- wallet
```

Env knobs: `ZANARY_LWD_URL` (default `http://127.0.0.1:9067`), `ZANARY_DATA_DIR`
(default `./data/wallet`; the DB is `zanary-wallet.db`, git-ignored under `/data`).
