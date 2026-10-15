# ZANARY v1 — flow runner plan (my lane: the rig)

Status: **in build**. This file is the durable plan + progress log for the piece that turns
the rig from "a wallet that can pay" into "a flow that produces verifiable evidence".

## What exists already (siblings, do not duplicate)

| crate | what it gives us |
|---|---|
| `observation` | the shared `FlowRun` data model (Payments, Funding, MerchantEvent, Receipt, PoolTotals) — data only, additive changes |
| `properties` | 12 named leak properties + `evaluate(&FlowRun) -> Vec<Finding>` |
| `report` | `build(&run, findings) -> Report`, canonical JSON + hash + evidence root, `index_claims` (expiry/stale/dead/regression), standalone `verify` |
| `zanary-seed` | seeds the chain to our wallet and makes a shielded payment with a memo (separate wallet DB) |
| `zanary-probe` | wallet, address, fund, shield, pay — the executor (Phase 0 gate, closed) |

## The gap this closes

Nothing yet *executes a flow*: there is no merchant side, no per-invoice fresh receiver, no
`run.json` produced by a run, and no glue from `run.json` → findings → report. The
`crates/report/fixtures/` runs were hand-assembled from `zanary-seed` output.

## v1 scope (`zanary-flow`, new bin in `crates/probe`)

```
zanary-flow run --definition flows/checkout-private.json [--out data/runs/<id>]
  → data/runs/<id>/run.json      (observation::FlowRun)
  → data/runs/<id>/findings.json (properties::evaluate)
  → data/runs/<id>/report.json   (report::build)
  → stdout: per-invoice table + verdict summary
```

Definition (JSON, hashed into `flow_commit`):

```json
{
  "flow_id": "zanary-checkout",
  "invoices": [{"invoice_id": "4471", "amount_zec": 0.25, "memo_template": "zanary: invoice {invoice_id}"}],
  "receiver_strategy": "fresh",        // "fresh" (private) | "reused" (control)
  "memo": true,                        // false = control
  "policy": "AllowRevealedAmounts",    // verbatim, recorded per payment
  "payer": {"profile": "dev", "account": 0},
  "merchant": {"profile": "merchant", "account": 0, "seed": "<BIP-39 test vector 2>"},
  "funding": {"target_zec": 5.0}
}
```

Behaviour per invoice:

1. **Merchant issues the invoice** — a *fresh diversified Orchard receiver* from the
   merchant wallet (`get_next_available_address`), memo rendered from the template, a
   ZIP-321 URI recorded on the invoice.
2. **Payer pays** — the Phase-0 executor path (propose → prove → sign → broadcast → mine),
   Orchard → Orchard, memo = ZIP-302 bytes, `--policy` recorded verbatim.
3. **Merchant verifies independently** — the merchant wallet (different seed, different DB)
   syncs and decrypts the mined transaction with *its own* UFVK, matches amount + memo +
   invoice id, and emits a `MerchantEvent` (the webhook the rig itself received) plus a
   `Receipt`.
4. **Rig records** the payment, the funding, and pool totals before/after.

The **control run** is the same definition with `receiver_strategy: "reused"` / `memo: false`
(and optionally a weaker policy): same runner, same evidence shape, so
`profiler vs control` is one command apart.

## Increments (each one verified on the harness before the next)

- [ ] **A · wallets are profiles** — generalize the wallet layer: seed phrase + account +
      data dir per wallet, so the merchant is a genuinely independent party (its own seed,
      its own DB). `zanary-probe`'s existing paths keep the dev profile unchanged.
- [ ] **B · merchant + fresh receivers** — merchant wallet bootstrap, per-invoice fresh
      diversified receiver, ZIP-321 invoice rendering.
- [ ] **C · the runner** — definition parsing + `flow_commit`, N-invoice execution through
      the existing pay path, `run.json` emission; merchant-side decryption per payment.
- [ ] **D · evidence glue** — `findings.json` + `report.json` in the same command; verify a
      passing private run and a control run that FAILs `receiver-reuse` / `memo-absent`.
- [ ] **E · third-party memo reader closed** — the merchant's read is the Phase-1 gate's
      "memo readable in a third-party wallet view" (its own seed + DB, not the payer's
      wallet).
- [ ] **F · docs + README row** — `docs/FLOW-RUNNER-NOTES.md` with commands, raw output,
      txids, traps; README "What works now" updated.

## Progress log

- (start) plan written; recon done: `observation`/`properties`/`report` interfaces read;
  `crates/report/fixtures/run.json` is the target shape for the private run, and the
  fixtures README confirms those runs were assembled by hand — which is what the flow
  runner replaces.
- **A · wallets are profiles — done.** `core::WalletProfile` + `bootstrap_with` +
  `open_db_for` + `import_account_for`; `bootstrap(account)` is now a thin wrapper over the
  dev profile, so `zanary-probe wallet/address/fund/shield/pay` are unchanged. The merchant
  is the BIP-39 test vector 2 seed in `data/wallet-merchant/zanary-merchant.db`
  (`ZANARY_MERCHANT_DATA_DIR` overrides the directory).
- **B · merchant + fresh receivers — done.** The merchant wallet bootstraps through the same
  path, syncs, and issues an Orchard-only diversified receiver per invoice via
  `get_next_available_address(account, UnifiedAddressRequest::ORCHARD)`.
- **C · the runner — done (first real run in flight).** `zanary-probe flow --definition
  <path> [--out <dir>]`: parses the definition, hashes it into `flow_commit`, records
  funding + pool totals, pays each invoice through `ops::propose_shielded_payment` +
  `ops::build_sign_broadcast` (the same code path `zanary-probe pay` uses — extracted, not
  duplicated), mines 3 blocks per payment, then has the merchant read each payment with its
  own wallet (`GetTransaction` → parse → `decrypt_transaction` with the merchant's UFVK).
- **D · evidence glue — done in the same command.** `run.json` (observation::FlowRun),
  `findings.json` (`zanary_properties::evaluate`), `report.json`
  (`zanary_report::build_with_claim` with an RFC3339 claim clock). Exit code is non-zero when
  any property FAILs, so a control run is a failing run by design.
- **E · third-party memo reader — closed by B**: the merchant's read is its own seed, its own
  database, its own trial decryption, and it is recorded on each payment as
  `observed.merchant_verified`.
- Definitions: `flows/checkout-private.json` (5 invoices, fresh receiver per invoice, memo
  per payment, `FullPrivacy`, non-round distinct amounts) and `flows/checkout-control.json`
  (5 invoices, ONE reused receiver, no memo, `AllowRevealedAmounts`, identical round
  amounts) — the same runner, one definition apart.
- **G · signed receipts + offline verification — done.** `observation::Receipt` gained optional
  `signer`/`signature` (additive, skipped when absent); the runner signs each receipt with the
  merchant's Orchard spend-authorizing key (RedPallas, `randomize` at the identity scalar);
  `crates/report/src/receipt.rs` holds the signing payload and the checks; `zanary-report
  verify-receipt <receipt.json> [--network …] [--fvk …] [--signer …]` verifies offline. Proven on the
  `9774534a…` run (12 pass · 0 fail): good receipt → VALID/exit 0, tampered body → Invalid
  signature/exit 2, wrong wallet's `--fvk` → FAIL/exit 1, wrong `--signer` → exit 1, and with the node
  and indexer pointed at dead ports the check still passes (the Phase-2 "engine stopped" gate).
  Traps (FVK-encoded `ak`, orchard's `[u8; 64]` signature conversion, `LocalNetwork` for regtest
  keys) are in `docs/FLOW-RUNNER-NOTES.md`.

## Next increments (in order)

1. **Machine-readable claims + a third-party receipt format — done.** `schemas/zanary-receipt-v1.json`
   (JSON Schema 2020-12, shipped for third-party tools) and `receipt::validate_claim`, which
   `verify-receipt` runs on every receipt: closed body shape, `txid`/`memo_sha256` as 64-hex,
   integer `amount_zat`, claims that state the body's figures, and an empty `discloses`. The verifier
   now reports *all* failures instead of stopping at the first (the tampered receipt names both the
   broken signature and the mismatched claim). Proven on five receipts: good → exit 0; tampered,
   missing `txid`, extra body field, claims-vs-body disagreement → exit 1 with a precise message each.
2. **Timing jitter — done.** `FlowDefinition` gained `jitter_blocks`/`jitter_seed` (both skipped when
   absent, so existing `flow_commit`s are unchanged — verified against the recorded value); the
   runner draws the block count per payment and reports it in the header and the summary.
   `flows/checkout-private-jitter.json` (`[1, 8]`, seed `20261011`) ran `12 pass · 0 fail`, report
   `49d61e58…`, with height deltas `+7, +6, +4, +2` matching the draws `[7, 6, 4, 2, 3]` — evidence
   that settlement is no longer a fixed distance, and that the node mines a just-broadcast payment in
   the first block it generates. Trap recorded: a piped build in a chained command hides compile
   failures and runs a stale binary.
3. **A dedicated receipt key** so signing does not publish the merchant's wallet identity.
4. **Batched settlement with randomised output order** (one transaction, N invoices) and the property
   that names what batching gives up (the batch links the invoices to each other by construction).
5. **The leak matrix — done.** `zanary-report matrix --runs data/runs --out docs/LEAK-MATRIX.md`
   renders one property × flow table over every run directory (run.json + findings.json), with each
   column pinned to its `flow_commit`, the severity of every property, the per-run verdict row, and
   a re-check command per report. Current artifact: `zanary-checkout` (12/0/0), `zanary-checkout-control`
   (7/4/1 — the four designed failures), `zanary-checkout-jittered` (12/0/0). It regenerates as new
   variants land (batching is the next one).
6. **The agent run — done.** `scripts/agent-pay-loop.sh` runs three invoice → pay → verify cycles
   unassisted: `zanary-probe invoice` (fresh diversified receiver + ZIP-321 URI on disk), `pay
   --invoice` (the URI decides recipient/amount/memo, cross-checked against the file), `verify-invoice`
   (the merchant's own wallet reads the memo and confirms the invoice id). All three settled —
   invoices 8101/8102/8103, txids `e73f4926…`, `4ae3f0be…`, `cf0789ee…` — with the merchant's
   read-back recorded in `docs/FLOW-RUNNER-NOTES.md`. Traps found by running it: displayed txids are
   the reverse of the transaction's internal byte order, and a piped command hides a real exit status.
