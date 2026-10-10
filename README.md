# CANARY

**Privacy claims should be falsifiable.** CANARY is an independent prober for Zcash payment flows: it
runs a flow with real shielded transactions on a throwaway chain, proves or breaks each privacy claim
with reproducible evidence, and publishes claims that **expire** unless someone re-proves them.

Built for **ZECATHON** (Core & Tooling track), by kiter + Hermes.

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

## What CANARY is

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
5. **A claim index with expiry** — a public page per claim (`flow × property`, verified-at-height, TTL,
   evidence). Re-proved on a schedule or it goes stale, then dead. *A privacy claim is not a fact, it
   is a subscription.*

## What works now

| Piece | State |
|---|---|
| Disposable chain + indexer (`harness/`) | **works** — zcashd v6.20.0 regtest + lightwalletd v0.5.5, one `docker compose up -d`, 140 blocks in ~2 s |
| Seeding + evidence script (`harness/seed.sh`) | **works** for chain + shielded-memo payments (node-wallet path retired — see `harness/HARNESS.md`) |
| Our own Rust client over the chain | **works** — `canary-probe` connects to the indexer, reports chain/version/tip via `GetLightdInfo` + `GetLatestBlock` |
| librustzcash wallet (create → sync → receiver → balance) | **in build** |
| Property library, profiler, report, claim index | **next** |

## How it fits the track

Core & Tooling: "Build something the next Zcash developer will thank you for. SDKs, explorers,
**testing tools**, better integrations. Solve a problem you wish someone already had solved."
CANARY is a testing tool that answers the event's own first judging criterion — *does it actually
preserve privacy; leaks are disqualifying* — with a run instead of a claim.

**Not to be confused with** (stated because naming neighbours is cheaper than being accused of
ignoring them):
- **Tells** — diagnoses a wallet's *existing history* (round trips, quick exits, fingerprint amounts).
  CANARY tests a flow's *future behaviour*, before users are exposed.
- **Zcash Selected-Payment Receipts SDK** — *discloses* one payment minimally, from the Orchard
  outgoing cipher key. CANARY *tests* implementations, and can test this format's verifier.
- **ZAP1 (`@frontiercompute/zcash-mcp`)** — attestations about Zcash *workflows*; a receipt of state,
  not a payment-privacy test.

## Running it

```bash
cd harness
docker compose up -d                      # chain + indexer (regtest, throwaway)
./seed.sh                                 # mine, fund, and produce evidence
cd ..
cargo run --bin canary-probe              # our client, over the indexer
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
