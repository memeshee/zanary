# ZANARY harness — a throwaway Zcash chain with an indexer

Everything here is a *test* chain. No real funds, no faucet, no sync, no network beyond this machine.
A judge can reproduce the whole environment from this directory.

## What runs

```
zcashd (regtest, v6.20.0)  ──RPC :18232──▶  lightwalletd (v0.5.5)  ──gRPC :9067──▶  zanary-probe (our Rust code)
   mines on demand, wallet                    CompactTxStreamer                   librustzcash wallet ops
   holds the shielded pool                    over block/tx RPC
```

## One command

```bash
harness/up.sh                 # config + `docker compose up -d` + wait until the rig is usable
```

`up.sh` exists because three steps that look like one are actually four, and skipping any of them
fails in a way that reads like a code bug:

1. **the node config.** `data/zcashd/.zcash/zcash.conf` is gitignored (it is live node state), so a
   fresh checkout has none: `up.sh` seeds it from the tracked `harness/zcash.conf` and never
   overwrites an existing one;
2. **`docker compose up -d`**;
3. **waiting for the node's RPC** (`getblockcount`);
4. **waiting for the indexer**, through the probe's own `GetLightdInfo` handshake when a binary
   exists — a listening port is not the same thing as a caught-up indexer.

Every one of those waits is *bounded*, and the probe on step 4 must answer **twice in a row with the
container still running** before the rig is declared usable. Both rules come from a CI runner rather
than from this machine: lightwalletd starts before zcashd's RPC answers, exits on its first failed
block fetch and is restarted by compose — and a gRPC call with no reply hangs instead of failing, so
an unbounded readiness probe can sit silently until a job's timeout. `ZANARY_PROBE_TIMEOUT_SECONDS`
(default 20) bounds each attempt, and each attempt prints the state it saw.

Block production is manual and instant: the rig decides when a block exists, so tests never race a
network. `generate N` mines N blocks in about two seconds, and a flow mines what it needs itself
(coinbase into the flow's own receiver — see *the node wallet is not part of the rig* below).

Without `up.sh` the raw commands are:

```bash
mkdir -p ../data/zcashd/.zcash && cp zcash.conf ../data/zcashd/.zcash/zcash.conf
docker compose up -d          # or: docker-compose up -d
until docker exec zanary-zcashd zcash-cli -regtest -rpcuser=zanary -rpcpassword=zanary getblockcount; do sleep 2; done
```

## Nodes

| Service | Image | Ports | Notes |
|---|---|---|---|
| `zcashd` | `electriccoinco/zcashd:latest` (v6.20.0) | 18232 RPC · 18233 P2P | Needs `data/zcashd/.zcash/zcash.conf` (mounted). Runs as uid 1000 so the bind mount is writable. |
| `lightwalletd` | `electriccoinco/lightwalletd:latest` (v0.5.5) | 9067 gRPC · 9068 HTTP | `--no-tls-very-insecure` is for this harness only; never expose it. |

## The config file that matters

`harness/zcash.conf` (tracked) is copied to `data/zcashd/.zcash/zcash.conf` (gitignored, live node
state) by `up.sh` — regtest, RPC open to the docker bridge, `txindex=1`, and the **network-upgrade
activations**, which zcashd does *not* apply on regtest by default:

```
nuparams=5ba81b19:1   # Overwinter
nuparams=76b809bb:1   # Sapling
nuparams=2bb40e60:1   # Blossom
nuparams=f5b9230b:1   # Heartwood
nuparams=e9ff75a6:1   # Canopy
nuparams=c2d6d0b4:1   # NU5 (Orchard)  ← without this, shielded sends fail with
nuparams=c8e71055:1   # NU6              "Cannot create shielded transactions before Sapling has activated"
```

Branch ids taken from zcashd v6.20.0 `src/consensus/upgrades.cpp`. Verify with
`getblockchaininfo | .upgrades` — every entry must read `"status": "active"`.

## Verified on 2026-10-11 (first working chain)

- 130 blocks mined in ~1.6 s; transparent pool 812.5 ZEC; wallet 100 ZEC spendable after maturity.
- `z_shieldcoinbase` moved 187.5 ZEC into the Orchard pool (txid `c022e98d8bc829b29e79f833641d70542457aa0f81578ff6f380dc9c578ddceb`).
- A **shielded Orchard payment carrying a 512-byte-class memo** landed and is readable at the recipient:
  - source/recipient: a *fresh diversified Orchard-only receiver* of the same account
    (`z_getaddressforaccount 0 '["orchard"]'`, diversifier index 6)
  - txid `58e828cfc9d0beacf825cad3a3193e64697bc668a40447b4d94bfbc075d5f7dd`
  - memo bytes `63616e6172792072656774657374…` = `zanary regtest memo: invoice 4471 / receipt seed 0xdeadbeef`
  - `z_listunspent` shows the 0.01 ZEC Orchard note with its memo intact — the primitive the rig's
    "memo-bound invoice" property sits on.

## Two things this harness taught us (both become rig properties)
1. **zcashd refuses to reveal amounts by default.** A shielded send that would have to spend from a
   different pool than the recipient fails with *"…would reveal transaction amounts. THIS MAY AFFECT
   YOUR PRIVACY. Resubmit with the privacyPolicy parameter set to AllowRevealedAmounts…"*.
   An application that sets `AllowRevealedAmounts` silently is downgrading privacy — that is a
   first-class leak class for the rig ("policy downgrade", testable, observable in review).
2. **API drift is real.** `z_getnewaddress` is disabled in v6.20.0 in favour of
   `z_getnewaccount` + `z_getaddressforaccount`; `z_shieldcoinbase` needs a real destination and
   mined blocks before shielded spends are possible. The rig must not assume any single node's
   RPC surface — which is exactly why the rig drives wallets through the **indexer gRPC**, not
   through node-specific RPCs.

## THE NODE WALLET IS NOT PART OF THE RIG (crash found 2026-10-11 19:06Z)

`seed.sh`'s node-wallet path crashed the node:

```
zcashd: wallet/orchard.cpp:22: OrchardWallet::GetSpendInfo(...)
  Assertion `walletAnchor.has_value() && walletAnchor.value() == anchor' failed.   → exit 139
```

It fired while the node's own Orchard wallet was listing/using notes after mining. Deprecated-code
fragility, and irrelevant to what the rig is for — but it *does* change the harness design:

- **zcashd is the chain and the miner. Nothing else.** No acceptance of, or dependence on, its wallet.
- **Every wallet in the rig is ours** — built on librustzcash (`zcash_client_backend` 0.24 +
  `zcash_client_sqlite` 0.22 + `zcash_keys` 0.16 / `zcash_address` 0.13), the same stack the test
  subjects use. This is strictly better for the rig: the property library must observe *application*
  behaviour, not a node's convenience methods.
- **Seeding moves to mining straight to our own address** (zcashd's `generateblock <address>` if
  available on v6.20.0, else a tiny getblocktemplate/submitblock miner). Coinbase pays our Rust
  wallet directly; maturity is 100 blocks (≈ a fifth of a second of regtest mining), and no
  node-wallet call is ever made.
- `restart: unless-stopped` is set on the node so a crash self-heals during long test runs.

**As built (2026-10-16):** `harness/zcash.conf` carries `mineraddress` = the dev account's
transparent receiver (derived from the public BIP-39 test-vector seed) with `minetolocalwallet=0`, so
coinbase pays the rig's own wallet while the node stays the chain and the miner. The flow runner
funds itself from that coinbase — mine to maturity (100 confirmations, 105 with margin), shield,
pay — and only falls back to a node-wallet transfer on a chain that mines to the node's wallet
(`top_up_payer` in `crates/probe/src/flow.rs`). That fallback is checked, never assumed: on this
conf the node wallet holds nothing to send. This is what makes a flow runnable on a chain at height
0 with no seeding step, locally and in CI (`docs/CI.md`).

## What is proven vs what is next

**Proven (evidence above):** a throwaway chain with all upgrades active; instant block production;
an Orchard payment carrying a structured memo, readable at the recipient; **and our own Rust binary
speaking gRPC to the indexer over that chain** — `GetLightdInfo` returning `chain regtest`,
`ECC LightWalletD v0.5.5`, `sapling at 1`, plus the tip hash from `GetLatestBlock`.

**Next (the D1 gate):** librustzcash wallet in `zanary-probe` — create account → sync → derive a
diversified Orchard receiver → print balance; then mine coinbase straight to that address and have
our own code shield/spend it with a memo. Nothing in that path may touch the node wallet.

## Known limits (stated, not hidden)

- This harness runs **zcashd**, whose shielded pool tops out at Orchard/NU6. Ironwood (NU6.3) is a
  Zebra-era pool; the rig therefore tests *property classes* (linkability, address reuse, memo
  binding, unshielding, policy downgrade) that are identical across pools, and keeps a Zebra-based
  path for the real-network run. Do not claim in the README that the harness chain is Ironwood.
- lightwalletd's regtest support is not part of its documented surface (its README covers Mainnet and
  Testnet). It runs here; if it ever fails to serve regtest blocks, the fallback is Zaino
  (`zingolabs/zaino`), which documents hermetic regtest chains, or Zebra regtest + Zaino.
- Host has ~3 GB RAM free: run one chain stack at a time.
