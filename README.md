# ZANARY

**Privacy claims should be falsifiable.** ZANARY is an independent prober for Zcash payment flows: it
runs a flow with real shielded transactions on a throwaway chain, proves or breaks each privacy claim
with reproducible evidence, and publishes claims that **expire** unless someone re-proves them.

Built for **ZECATHON** (Core & Tooling track), by kiter + Hermes.

[![privacy gate](https://github.com/memeshee/zanary/actions/workflows/privacy-gate.yml/badge.svg)](https://github.com/memeshee/zanary/actions/workflows/privacy-gate.yml)
— two flows per push: one that must pass, one that must be **caught**. Both jobs green means the rig
worked, not that nothing was found; open the control job's summary for the leaks it caught.

> **On the name:** this project ran as *CANARY* until Oct 15, 2026 and is **ZANARY** since. Evidence
> recorded before the rename — `crates/report/fixtures/`, the claim rows in `docs/LEAK-MATRIX.md` —
> is kept byte-identical, because editing a recorded run invalidates the hashes that make it
> checkable. New runs carry ZANARY's flow ids (`zanary-checkout*`).

> Status: **in build** (Oct 11 → Oct 28, 2026). What runs today is listed under *What works now* —
> nothing below claims more than the code does. The README is updated as phases land; the
> "real vs mocked" section is the contract, not decoration.

---

## The problem

Every Zcash integration is asked the same question by its users: *"does this actually preserve
privacy, or does it only look like it does?"* Today the answer is a paragraph in a README, a screenshot,
or an audit that costs more than the app. Meanwhile the failure modes are structural and boring —
a transparent fallback nobody noticed, one receiver reused across invoices, a wallet quietly told to
`AllowRevealedAmounts`, amounts that echo a funding deposit so the round trip is linkable by amount and
timing alone.

The Zcash ecosystem funds audits, privacy research and formal verification, but a payment flow still
has **no executable privacy test**: nothing a developer can run in CI that fails the build when the
flow starts leaking.

## What ZANARY is

1. **A rig** — one command brings up a disposable Zcash chain (instant blocks, no sync, no faucet) and
   an indexer; flows execute against it with real shielded transactions.
2. **A property library** — each leak class is a named test with a severity, a detection method and
   evidence a stranger can check: ([`docs/PROPERTIES.md`](docs/PROPERTIES.md)).
3. **An adversarial profiler** — the rig acts as the attacker: it tries to cluster the flow's payments
   into a payer graph and reports the confidence it achieved, next to a control flow that must *not*
   cluster.
4. **A report you can verify without trusting the rig** — `report.json` with txids, receivers, memos
   and heights; hashed, with the root anchored in a shielded memo so the evidence is publicly
   timestamped without revealing who ran it.
5. **A gate you can put in CI** — one command (`zanary-probe test <flow>`), one exit code: `0` the run
   matched its expectation and the chain agrees, `1` a flow leaked or a control flow the library was
   supposed to bite was not caught, `2` the rig itself could not check. The workflow in this repo runs
   a private flow and a deliberately leaky control side by side, and uploads the evidence
   ([`docs/CI.md`](docs/CI.md)).
6. **A claim index with expiry** — a public page per claim (`flow × property`, verified-at-height, TTL,
   evidence). Re-proved on a schedule or it goes stale, then dead. *A privacy claim is not a fact, it
   is a subscription.*

## What works now

| Piece | State |
|---|---|
| Disposable chain + indexer (`harness/`) | **works** — zcashd v6.20.0 regtest + lightwalletd v0.5.5, one `docker compose up -d`, 140 blocks in ~2 s |
| Seeding + evidence script (`harness/seed.sh`) | **works** for chain + shielded-memo payments (node-wallet path retired — see `harness/HARNESS.md`) |
| Our own Rust client over the chain | **works** — `zanary-probe` connects to the indexer, reports chain/version/tip via `GetLightdInfo` + `GetLatestBlock` |
| librustzcash wallet (create → sync → receiver → balance) | **works** — `zanary-probe wallet`, `address --transparent/--orchard` (see `docs/WALLET-NOTES.md`) |
| Shielded send path — fund → shield → pay with a ZIP-302 memo | **works** — `zanary-probe fund/shield/pay`, all built/proved/signed in our code, txids + before/after balances + recipient-side memo read in `docs/WALLET-SEND-NOTES.md` |
| Flow runner — one command, real evidence | **works** — `zanary-probe flow --definition flows/checkout-private.json` executes N invoices (fresh diversified receiver + ZIP-302 memo each, verified by a separate merchant wallet), then writes `run.json` + `findings.json` + `report.json`; the private flow scores 12 pass / 0 fail, the control flow fails by design (`docs/FLOW-RUNNER-NOTES.md`) |
| Batched settlement — N invoices, one transaction | **works** — `settlement: "batch"` puts every invoice in one transaction (one txid, one fee, shuffled output order, a distinct amount per invoice enforced before anything is built) and reads each payment back at the merchant by amount + memo; `flows/checkout-private-batch.json` scores 11 pass / 1 fail — the failure is `payer-linkability`, because a batch links its own invoices by construction (`docs/THREAT-MODEL.md`) |
| Signed receipts, verified offline | **works** — each receipt is signed by a *dedicated receipt key* (ZIP-32 account 1 of the merchant seed), never the wallet's own spend authority, and the wallet **attests** that key with its own signature; `zanary-report verify-receipt` checks the receipt with no node, no indexer and no rig (proven with both pointed at dead ports), and `--fvk` binds receipt key → wallet through the attestation — a receipt published without it is bound to no wallet, which is the point |
| Leak matrix | **works** — `zanary-report matrix --runs data/runs --out docs/LEAK-MATRIX.md`: 12 properties × 4 flows, each column pinned to its `flow_commit`; the control shows its four designed failures, and the batched column shows the one the batch discloses, next to two clean private runs |
| Agent payment loop | **works** — `scripts/agent-pay-loop.sh`: three invoice → pay → verify cycles unassisted, with the merchant's own wallet reading each memo back (invoices 8101–8103, txids in `docs/FLOW-RUNNER-NOTES.md`) |
| Anchored reports — a claim with a chain timestamp | **works** — `zanary-probe anchor --report <report.json>` puts the report's `evidence_root` + `report_hash` in a shielded memo to a public beacon, and `zanary-report verify-anchor` finds the earliest anchor for that root with a node and the beacon's *published* viewing key, nothing from ZANARY (640 blocks / 693 txs in ~3 s); unanchored reports, flipped roots, disagreeing `--root`s and post-hoc edits all exit 1 (`docs/ANCHOR-NOTES.md`) |
| CI gate — one command, an exit code a build can read | **works** — `zanary-probe test --definition <flow.json> --expect pass\|fail [--anchor] [--summary <path>]` runs the flow, re-derives every evidence line from the chain, anchors the evidence root in a shielded memo, prints the claim (height, report hash, expiry) and exits `0`/`1`/`2`; the two CI flows score `12 pass · 0 fail` (private, `4d8279bb`) and `4 fail` (control — the planted leaks, caught) and the GitHub Action runs both on every push ([`docs/CI.md`](docs/CI.md), [`.github/workflows/privacy-gate.yml`](.github/workflows/privacy-gate.yml)) |
| Claim index with expiry (renderer) | **works** — `zanary-report index --reports <dir> --out <index.html>` renders one page per claim with `live`/`stale`/`dead`/`regression` against a `--now` clock and the anchor's height ([`crates/report/fixtures/index.golden.html`](crates/report/fixtures/index.golden.html)); publishing it on a schedule is next |
| The reference implementation that passes | **in build** — the bounded-payer engine (grants: budget, per-payment cap, allowlist, expiry, kill) as the flagship *subject*; today `scripts/agent-pay-loop.sh` is the closest thing running unassisted |

## How it fits the track

Core & Tooling: "Build something the next Zcash developer will thank you for. SDKs, explorers,
**testing tools**, better integrations. Solve a problem you wish someone already had solved."
ZANARY is a testing tool that answers the event's own first judging criterion — *does it actually
preserve privacy; leaks are disqualifying* — with a run instead of a claim.

**Not to be confused with** (stated because naming neighbours is cheaper than being accused of
ignoring them):
- **Tells** — diagnoses a wallet's *existing history* (round trips, quick exits, fingerprint amounts).
  ZANARY tests a flow's *future behaviour*, before users are exposed.
- **Zcash Selected-Payment Receipts SDK** — *discloses* one payment minimally, from the Orchard
  outgoing cipher key. ZANARY *tests* implementations, and can test this format's verifier.
- **ZAP1 (`@frontiercompute/zcash-mcp`)** — attestations about Zcash *workflows*; a receipt of state,
  not a payment-privacy test.

## Running it

```bash
harness/up.sh                             # throwaway chain + indexer, and wait until they are usable
cargo build --release --bin zanary-probe --bin zanary-report

# the whole point, in one command: run a flow, verify it against the chain, anchor it, judge it
./target/release/zanary-probe test --definition flows/ci-checkout.json --expect pass \
    --anchor --ttl-days 30 --summary data/runs/ci/test.json

# and the other half of the pair: this one must be caught
./target/release/zanary-probe test --definition flows/ci-checkout-control.json --expect fail
```

Requirements: Docker, Rust (stable, ≥ 1.88 — the Zcash stack refuses older toolchains), ~3 GB RAM free.
Full harness notes, traps, and evidence: [`harness/HARNESS.md`](harness/HARNESS.md).

## Real vs mocked

- The chain is **real** (a real Zcash node in regtest mode, real shielded transactions, real memos) and
  **local** — no mainnet funds are used or needed.
- Wallets in the rig are **ours**, built on librustzcash; the node is only the chain and the miner.
- Anything simulated (for example a merchant endpoint the rig runs itself) is labelled as such in the
  report, and the property it exercises is stated in plain words.

## Limits, stated up front

- The local chain's shielded pool tops out at Orchard/NU6 (Ironwood is a Zebra-era pool); the rig tests
  **property classes** that are pool-independent and keeps a Zebra path for real-network runs.
- Network-layer deanonymisation and protocol soundness are out of scope (`docs/PROPERTIES.md`).
- Fees on real networks (~5000 zat/action) make sub-dollar payments uneconomic — a rig finding that
  affects what a flow *should* do, not what it can do.

## Licence

MIT (see `LICENSE` when it lands with the first public push).
