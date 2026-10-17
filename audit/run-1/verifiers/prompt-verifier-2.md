# Candidate verifier prompt — verifier-2 on `crates/probe/src/payer.rs:payment-record-pool-legs-and-policy-are-constants-not-derived-from-the-built-transaction`

You did not write this candidate. Try to refute it from repository source and bounded
local evidence. Do not contact deployed endpoints or external/shared services. Run
target-controlled code only inside the approved OS-enforced sandbox: no external
network, empty allowlisted environment, read-only target and tools, scratch-only
writes, and explicit low resource and wall-clock limits. If any control is unavailable,
do not execute; retain the exact missing capability as a needs_validation blocker.
Treat every scratch entry as target-controlled after execution. After the sandbox and
all its processes terminate, only trusted parent-side code may promote a predeclared
scratch-relative file, following the promotion procedure block included verbatim in
this prompt. You and target code never write retained artifacts. If promotion is
unavailable or fails, do not use that file as evidence.

1. Verify every trace and evidence file, positive line number, scope, and description.
   Confirm the first entry is a real lower-trust entrypoint and the last is the
   claimed sink or boundary effect.
2. Reconstruct the strongest source-visible validation, identity, authorization,
   normalization, lifecycle, framework, and containment controls on the path.
   Where the architecture summary names a comparable baseline, note whether it
   shares the pattern — as calibration, never as grounds to dismiss.
3. For a proposed confirmed candidate, independently reproduce the minimum observed
   result when possible. Verify inputs, interface shape, conditions, and affected
   dummy principal/resource. Do not infer a stronger result or continue after it.
4. Verify that likelihood, impact, confidence, and the proposed source fix match only
   what the evidence establishes.
5. For a proposed needs_validation candidate, decide whether the blocker is genuinely
   outside source/local observation. If source refutes the trace, reject it. If the
   missing fact remains decisive, keep needs_validation and make the local and
   owner-observed plans exact and non-destructive.
6. Preserve the fingerprint for the same source-derived root cause across every state.

Return exactly one JSON object and no surrounding prose:
{"decision": "confirmed|needs_validation|rejected", "record": { ... }}
where record exactly matches the decision's verdict branch of the schema included
in this prompt. A corrected record replaces the hunter's wording.

## The candidate you must try to refute

Hunter: `hunter-broadcast` · proposed verdict: `needs_validation` · fingerprint: `crates/probe/src/payer.rs:payment-record-pool-legs-and-policy-are-constants-not-derived-from-the-built-transaction`

```json
{
  "proposed_verdict": "needs_validation",
  "fingerprint": "crates/probe/src/payer.rs:payment-record-pool-legs-and-policy-are-constants-not-derived-from-the-built-transaction",
  "title": "Bounded payer records every payment as Orchard -> Orchard FullPrivacy even when the transaction it broadcasts crosses a shielded pool boundary",
  "description": "payer::run pays each invoice it admitted through ops::propose_shielded_payment / ops::build_sign_broadcast and then records the payment with the constants pool_in: Orchard, pool_out: Orchard, kind: Shielded, policy: Some(\"FullPrivacy\") (crates/probe/src/payer.rs:347-352): no field is derived from the proposal or the transaction, and this path never applies ops::check_policy, which every other payment path in the workspace calls before it proposes (ops.rs:566, ops.rs:777, ops.rs:1020, flow.rs:634, flow.rs:827). The recipient comes from the merchant's invoice file, and decide only string-compares it against the allowlist (payer.rs:100) - with the documented empty allowlist (printed at payer.rs:222-224 as 'every receiver is allowed') an invoice naming a Sapling receiver is admitted. The observed local check shows that recipient passing the target's intake and reaching the wallet layer, while required_policy_for_recipient + check_policy refuse the same recipient under FullPrivacy with 'would reveal the amount (Orchard -> Sapling pool crossing)' (ops.rs:237-246). The wallet layer then delivers a Sapling-receiver payment through a Sapling output (zcash_client_backend 0.24.0 GreedyInputSelector::propose_transaction, input_selection.rs:966-968 and 999-1003, the selector and parameters ops.rs:929-936 constructs), so the transaction actually broadcast moves value across a shielded pool boundary and reveals the amount, while the run's record - and every verdict derived from it - asserts a same-pool, fully-shielded, FullPrivacy payment. policy-downgrade (Severity::Disqualifying, properties/src/properties/policy_downgrade.rs:76-113) reads exactly those two constant fields, so it reports PASS for such a run, and the report/index published from it carries that false clean verdict; nothing in the repo compares run.payments against the ledger or the transaction. Conditions: the invoice directory contains an invoice whose receiver is a Sapling address or a unified address with a Sapling receiver and no Orchard receiver (with a memo the payer can pass through), and the grant admits it (empty allowlist admits every receiver; an allowlist naming that receiver also admits it), on a chain where Sapling is active (core.rs:85). The reachable divergence is limited to shielded receivers: a bare transparent receiver is refused inside the target's own intake by zip321's memo rule (observed).",
  "claimed_root_cause": "Two coupled defects on the payer's pay path: (1) the payment record is a template - pool_in/pool_out/kind/policy are literals at crates/probe/src/payer.rs:347-352, never derived from the proposal's payment pools or from the built transaction, so the fields the leak properties read describe what the payer assumed rather than what it built; (2) the payer is the only payment path in the workspace that does not call ops::required_policy_for_recipient + ops::check_policy before proposing (0 occurrences in payer.rs against ops.rs:566/777/1020 and flow.rs:634/827), so a merchant-supplied recipient that makes the payment reveal more than FullPrivacy (a Sapling receiver, which the target's intake accepts and hands to the wallet layer) is paid instead of refused, and is then recorded as Orchard -> Orchard FullPrivacy.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/payer.rs",
      "line": 298,
      "scope": "payer::run, payment loop over the invoices decide allowed",
      "description": "The merchant-supplied receiver string (admitted by decide's allowlist string-compare at payer.rs:100) is decoded into the address the payment will be made to; there is no recipient-kind or privacy-policy gate anywhere on this path (payer.rs has 0 occurrences of check_policy / required_policy_for_recipient)."
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/ops.rs",
      "line": 900,
      "scope": "ops::propose_shielded_payments",
      "description": "The recipient becomes a ZIP-321 payment and is handed to propose_transfer with SpendPolicy::shielded_pools([Orchard]) (inputs only) and an Orchard change strategy (ops.rs:929-936), so nothing constrains the recipient's own pool; the observed check shows a Sapling receiver passing this intake and reaching the wallet layer."
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/ops.rs",
      "line": 360,
      "scope": "ops::build_sign_broadcast",
      "description": "Whatever the proposal built is proved, signed, read back from the wallet and sent to the indexer (ops.rs:376-415) without any comparison against the invoice the payer decided about."
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/payer.rs",
      "line": 347,
      "scope": "payer::run, the Payment record written for the payment just broadcast",
      "description": "pool_in, pool_out, kind and policy are written as constants (Orchard / Orchard / Shielded / FullPrivacy), so the run's evidence - and the report, index and matrix built from it - assert a same-pool fully-shielded payment regardless of the transaction's real shape."
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/payer.rs",
      "line": 347,
      "description": "the four fields the Disqualifying policy-downgrade property reads are literals; nothing in payer.rs reads the proposal back (no proposal.steps, no payment_pools, no transaction decryption)"
    },
    {
      "file": "crates/probe/src/ops.rs",
      "line": 243,
      "description": "required_policy_for_recipient maps a Sapling receiver (or a unified address with a Sapling receiver and no Orchard receiver) to AllowRevealedAmounts because the payment would reveal 'the amount (Orchard -> Sapling pool crossing)'"
    },
    {
      "file": "crates/probe/src/ops.rs",
      "line": 1020,
      "description": "the CLI pay path calls check_policy before proposing; the payer's loop has no equivalent (0 occurrences in payer.rs, counted from the compiled source in the local check)"
    },
    {
      "file": "crates/properties/src/properties/policy_downgrade.rs",
      "line": 92,
      "description": "the implicit-downgrade signal is computed from Payment.pool_in/pool_out alone (92-101) and the explicit one from Payment.policy (76-89), so the constant record decides the verdict"
    },
    {
      "file": "crates/observation/src/lib.rs",
      "line": 28,
      "description": "PaymentKind::Shielded is defined as 'shielded source and shielded destination, same pool' and pool_out is the payment's destination pool, so recording Orchard/Shielded for a Sapling-output payment is false by the model's own definitions"
    },
    {
      "file": "crates/properties/src/properties/fixtures.rs",
      "line": 240,
      "description": "the fixture that models a cross-pool payment records the real destination pool (pool_out Transparent, kind CrossPool, policy AllowRevealedAmounts) and drives the property's failing path"
    },
    {
      "file": "crates/probe/src/core.rs",
      "line": 85,
      "description": "the harness chain activates Sapling at height 1 while NU6.3/Ironwood stays unactivated, so a Sapling receiver is the shielded cross-pool delivery this build can produce"
    }
  ],
  "blockers": [
    "The decisive step - that the proposal/transaction built for a Sapling-receiver invoice carries a Sapling output - could not be executed in the approved sandbox: it needs a wallet holding a spendable Orchard note, and no offline route exists (zcash_client_backend's test-wallet framework needs the test-dependencies feature set -> proptest/ambassador/assert_matches, none of which are in the read-only registry cache, and --network=none permits no fetch).",
    "The end of the path - the broadcast and the resulting report - needs the regtest rig (its containers and host ports 18232/18233/9067/9068), which is out of bounds for hunters.",
    "Whether a Sapling-receiver invoice ever reaches a payer in practice is a deployment fact: the CI demo and every recorded run issue Orchard-only invoices via `zanary-probe invoice`, while the payer's contract is any invoice directory and the shipped grant's allowlist is empty."
  ],
  "validation_plan": {
    "local": "In the approved disposable copy (or a sandbox whose registry cache is populated), enable zcash_client_backend/test-dependencies and zcash_client_sqlite/test-dependencies, use their data_api::testing framework to fund one Orchard note in a test wallet, then call propose_transfer with the payer's exact arguments (params = core::regtest_network(), account, GreedyInputSelector, SingleOutputChangeStrategy::new(Zip317, None, ShieldedPool::Orchard, DustOutputPolicy::default()), ConfirmationsPolicy::default(), SpendPolicy::shielded_pools([Orchard]), request = one zip321 payment to a zregtestsapling1... receiver with the invoice memo) and print proposal.steps()[0].payment_pools() - expected {0: PoolType::SAPLING}, i.e. the payment leg is Sapling while payer.rs:350 would record Orchard.",
    "deployment": "Owner-observed on the regtest rig (throwaway funds, no external network): write one invoice JSON whose receiver is the dev account's own Sapling receiver with a self-consistent URI and a short memo, run `zanary-probe payer --grant <grant with an empty or matching allowlist> --invoices <dir>`, then read (a) run.json payments[0].pool_out/policy (expected the constants Orchard/FullPrivacy), (b) report.json policy-downgrade verdict (expected PASS), and (c) getrawtransaction/decoderawtransaction for that txid (expected a Sapling output / non-zero sapling valueBalance)."
  }
}
```

## Its linked coverage unit and the hunter's checks

- coverage_id: `crates%2Fprobe%2Fsrc%2Fops.rs%23propose_shielded_payment%20%2F%20build_sign_broadcast::the%20authority%20check%20%28decide%29%20-%3E%20the%20transaction%20actually%20built%20and%20broadcast::bounded-payer::ATTACK-CLASSES.md%23Business%20logic`
- surface: `crates/probe/src/ops.rs#propose_shielded_payment / build_sign_broadcast` (subsystem `bounded-payer`)
- boundary: `the authority check (decide) -> the transaction actually built and broadcast`
- attack class: `ATTACK-CLASSES.md#Business logic` · assignment owner: `hunter-broadcast`
- unit reviewed paths: ["crates/observation/src/lib.rs", "crates/probe/src/core.rs", "crates/probe/src/flow.rs", "crates/probe/src/ops.rs", "crates/probe/src/payer.rs", "crates/properties/src/properties/fixtures.rs", "crates/properties/src/properties/payer_policy.rs", "crates/properties/src/properties/policy_downgrade.rs"]

The unit's checks, including the local checks whose promoted artifacts you may read
(paths are relative to the run directory `/home/kiter/security-audit-skill/zanary/run-1`):

```json
[
  {
    "agent_id": "hunter-broadcast",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/probe/src/ops.rs",
      "crates/observation/src/lib.rs"
    ],
    "invariant": "the pool legs and privacy policy a payer payment records describe the transaction that was built, not a constant template",
    "method": "source",
    "result": "payer.rs:347-352 writes pool_in: Orchard, pool_out: Orchard, kind: Shielded, policy: Some(\"FullPrivacy\") as literals for every payment, and payer.rs never reads anything back from the proposal it built (no proposal.steps, no payment_pools, no transaction decryption anywhere in the file, unlike ops.rs:1076-1126 where `pay` decrypts the mined transaction). observation/src/lib.rs:28-36 and 56-60 define those fields as the payment's real pools and the 'privacy policy argument the flow passed to its wallet layer, verbatim' \u2014 the payer passes no such argument.",
    "artifact": null
  },
  {
    "agent_id": "hunter-broadcast",
    "reviewed_paths": [
      "crates/probe/src/ops.rs",
      "crates/probe/src/payer.rs"
    ],
    "invariant": "a merchant-supplied recipient whose payment would reveal more than FullPrivacy is refused before anything is proposed or built, as every other payment path in the workspace refuses it",
    "method": "local",
    "result": "Observed in the approved sandbox (check-intake.txt, 2 tests passed, exit 0): an invoice-style payment to a Sapling receiver (bare zregtestsapling1..., and a unified address with a Sapling receiver and no Orchard receiver) is ACCEPTED by the target's own intake in ops::propose_shielded_payments (it reaches the wallet layer: 'Must scan blocks first', with the invoice memo and with an empty memo), while ops::required_policy_for_recipient + ops::check_policy refuse the same recipient under FullPrivacy with 'would reveal the amount (Orchard -> Sapling pool crossing)'. payer.rs contains 0 occurrences of either function, against ops.rs:566 / ops.rs:777 / ops.rs:1020 / flow.rs:634 / flow.rs:827 on the other payment paths. A bare transparent receiver is refused inside the target's intake by zip321's memo rule, so the transparent variant is not reachable.",
    "artifact": "agents/hunter-broadcast/artifacts/check-intake.txt"
  },
  {
    "agent_id": "hunter-broadcast",
    "reviewed_paths": [
      "crates/probe/src/payer.rs"
    ],
    "invariant": "the run's per-invoice decisions and artifacts are committed even when a payment step fails after earlier payments were broadcast",
    "method": "source",
    "result": "payer.rs:246-270 decides every invoice first and only prints; the payment loop (291-363) propagates each step with `?` (decode 298-303, memo 304-305, propose 306-307, build_sign_broadcast 308, mine_and_resync 309, mined_height_of 310); ledger.jsonl and the run/report artifacts are written afterwards (408-411) and --expect-refused is evaluated later still (469-478). A mid-loop failure therefore returns Err with nothing recorded although earlier iterations already broadcast, and no code path in payer.rs can produce the `error:` refusal reason that properties/src/properties/payer_policy.rs:66-74 exists to flag.",
    "artifact": null
  },
  {
    "agent_id": "hunter-broadcast",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/probe/src/flow.rs"
    ],
    "invariant": "every invoice the decision layer admits can be executed by the payment loop (the decision covers the whole invoice, not only budget/cap/allowlist fields)",
    "method": "local",
    "result": "Observed in the approved sandbox (check-intake.txt): with the documented grant shape (empty allowlist, cap 10,000,000 zat, budget 50,000,000 zat) an invoice whose receiver is 'not-an-address' and whose amount is 5,000,000 is decided allowed=true ('paid: within the cap ... and the budget'), and Address::decode then returns NONE at payer.rs:298; an invoice whose memo is 600 bytes is likewise decided allowed=true and rejected by MemoBytes::from_bytes at payer.rs:304. The same run shows the payer's intake (serde, payer.rs:515-516) accepting an invoice whose URI pays the Orchard UA but whose receiver says Sapling, which flow.rs:1740-1768 (ops::pay --invoice) refuses as 'an invoice that disagrees with itself'.",
    "artifact": "agents/hunter-broadcast/artifacts/check-intake.txt"
  },
  {
    "agent_id": "hunter-broadcast",
    "reviewed_paths": [
      "crates/properties/src/properties/policy_downgrade.rs",
      "crates/properties/src/properties/fixtures.rs",
      "crates/properties/src/properties/payer_policy.rs"
    ],
    "invariant": "the leak verdict for a payment's pools is derived from the chain/transaction rather than from the writer's self-report, and the fixture set pins the encoding of a cross-pool payment",
    "method": "source",
    "result": "policy_downgrade.rs:76-101 computes both signals from Payment.policy and Payment.pool_in/pool_out alone (an explicit token match and pool_out in {Transparent, CrossPool}); nothing in the properties crate re-derives a pool leg from a transaction, and payer_policy.rs:63-103 likewise checks only the ledger's own amount/receiver/revoked fields against the grant. fixtures.rs:240-246 models a cross-pool payment with the real destination pool (pool_out Transparent, kind CrossPool, policy AllowRevealedAmounts), i.e. the record is the only evidence there is.",
    "artifact": null
  },
  {
    "agent_id": "hunter-broadcast",
    "reviewed_paths": [
      "crates/probe/src/core.rs"
    ],
    "invariant": "the chain the payment is built for activates the pools that decide what the transaction can reveal",
    "method": "source",
    "result": "core.rs:81-91 pins LocalNetwork with sapling: Some(1) (active from height 1) and nu6_1/nu6_2/nu6_3 unactivated, so an Orchard-receiver payment is an Orchard output and a Sapling receiver is the shielded cross-pool delivery this build can actually produce; the harness node (zcashd v6.20.0) matches.",
    "artifact": null
  }
]
```

Read the promoted files under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/` — those are the hunter's retained
local evidence. You may read a hunter's artifact, but every decisive claim must be re-established by you.

## Architecture facts (verbatim, written by the parent)

# Architecture — zanary (Phase 1 recon, run in the parent)

Target: `/home/kiter/hackathon/zanary` · repo `memeshee/zanary` · commit `7058406` · worktree clean at recon.
Profile `standard`, budget 14 agent invocations. Run dir outside the target: `~/security-audit-skill/zanary/run-1`.

## What the system is

A Rust workspace (38 `.rs` files, 14,465 lines, 100 tracked files) that runs a Zcash **regtest** payment
flow on a throwaway chain, observes what the chain/merchant/observer can see, scores it against 13 named
leak properties, hashes the evidence, and publishes claims that expire unless re-proved.

Crates: `probe` (CLI + rig driver + flow runner + bounded payer + CI gate), `properties` (the 13
properties + fixtures), `report` (report/anchor/verify/index/matrix + the renderer for the published page),
`observation` (the observation model, grants, ledger), plus fixtures/tests and `bin/zanary-seed`.

## Entry points and who can reach them

| entry point | reachable by | what it trusts |
|---|---|---|
| `zanary-probe {wallet,address,fund,shield,pay,anchor,flow,test,payer,invoice,verify-invoice}` | whoever runs the binary | CLI args, flow/grant/invoice JSON, env (`ZANARY_LWD_URL`, `ZANARY_UID/GID`, timeouts) |
| `zanary-report {verify,verify-receipt,build,index,matrix,verify-anchor}` | local user, CI | report/run/receipt JSON + a node/indexer URL |
| `harness/up.sh` + `harness/docker-compose.yml` | local user, CI, **any consumer of the composite action** | docker/podman availability, `ZANARY_UID/GID` |
| `action.yml` (composite GitHub Action, `uses: memeshee/zanary@main`) | **any third-party workflow** | action `inputs:` — interpolated into `run:` steps |
| `.github/workflows/privacy-gate.yml` (5 jobs: tests, private, control, payer, publish) | push/PR/dispatch/**weekly cron** | repo contents; publishes Pages + uploads artifacts |
| `scripts/{demo-run.sh,bounded-payer-demo.sh,ci-summary.py}` | local user, CI | env knobs `ZANARY_DEMO_RUNS`, `ZANARY_PAYER_DEMO_DIR`, `ZANARY_LOG` |
| published Pages site (`/`, `/video/`) + uploaded CI artifacts | **the public internet** (public repo, public Pages) | the evidence files it renders/copies |

## Trust boundaries

1. **Third-party CI ↔ this repo's automation.** `action.yml` is designed so a stranger's workflow runs it
   (`uses: memeshee/zanary@main` is the whole setup). Anything it interpolates from `inputs:` into a shell
   is a boundary between that consumer's workflow and their runner.
2. **Public repo/artifacts ↔ the rig's secrets.** Actions artifacts on a public repo are world-downloadable;
   the Pages deploy is world-readable. Wallet material, viewing keys, or RPC credentials reaching either is
   a disclosure boundary crossing.
3. **Untrusted data ↔ the probe CLI.** Flow definitions, invoices, grants and reports are JSON files that a
   user (or a judge, or CI) supplies; `--out`/`--definition`/`--grant` are caller-controlled paths.
4. **Host network ↔ the local rig.** `harness/docker-compose.yml` publishes zcashd JSON-RPC (18232/18233)
   and lightwalletd gRPC/HTTP (9067/9068) on the host.
5. **Authority model ↔ money movement.** The bounded payer decides whether a payment happens, from a grant
   (budget, per-payment cap, receiver allowlist, expiry, kill switch). Its ledger is the audit trail; the
   `payer-policy` property is the detector.
6. **Claim ↔ observer.** The published evidence (receivers, txids, memos, pool totals) is *intended* to be
   public; the tool's promise is that it never publishes more than the claim needs.

## Secrets and sensitive state

- `harness/zcash.conf` carries `rpcuser`/`rpcpassword` for the regtest node — **committed on purpose** and
  documented as throwaway, in a repo whose README tells people to run it locally. Not a secret in the usual
  sense; it *is* the credential that protects the JSON-RPC control plane (boundary 4).
- Wallet DBs, chain data, per-run evidence and keys live under `data/` and are gitignored (`/data`).
- No API keys, tokens, or `.env` in the tree (checked: no tracked file matches `*.env|secret|key|token|pem`).
- `video/zanary-demo.mp4` (3.6 MB) is a committed artifact, deliberately.

## What the tests need

`cargo test --workspace` is offline (fixtures + golden reports). The demo scripts, the flow runner and the
gate need **Docker plus the local regtest node/indexer over loopback** (`ZANARY_LWD_URL=http://127.0.0.1:9067`).
Anything requiring that rig is outside a `--network=none` sandbox and can only ever land as
`needs_validation` unless an offline path exists.

## Sandbox available

The methodology requires OS-enforced sandboxing for target-controlled execution: no external network,
read-only target, scratch-only writes, bounded resources.

Available: Docker with locally-present images (`rust:1-slim`, `node:22-slim`, `rust:1-slim` verified for
`cargo --version`), `--network=none`, a disposable `git archive HEAD` copy at
`/home/kiter/.hermes/cache/scratch/zanary-sandbox`, and the host cargo registry mounted read-only at
`/usr/local/cargo/registry` so `cargo test --offline` needs no network. Smoke-tested before dispatch —
see `hunter-protocol.md`.

**Sandbox proof (run before the hunters' results were collected).** From the disposable
`git archive HEAD` copy with `rust-toolchain.toml` removed (the pin names `channel = "stable"`, which rustup
can only resolve with network — the sandbox correctly refused: `dns error: Temporary failure in name
resolution`, no silent fetch) and the host registry mounted read-only:

```
docker run --rm --network=none -v $SB:/t -v ~/.cargo/registry:/usr/local/cargo/registry:ro -w /t \
  rust:1-slim cargo test --offline -p zanary-properties --lib
→ test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured
```

So: the unit suite runs offline inside the sandbox, target code executes with no network, and a local check
is available to hunters for this unit class.

Not available: any sandbox that can reach the regtest chain (the rig's containers are shared host state and
out of bounds for hunters), and any image that is not already local (no pulls, by policy).

## Parent-side scans performed (coverage, recorded here)

Run in the parent before wave 1, read-only, over the whole history (`git rev-list --all`) rather than the
working tree only:

| scan | result |
|---|---|
| secret-shaped patterns across every commit (`uview1`, `upriv1`, `secret-extended-key`, `ghp_`, `AKIA`, `AIza`, `BEGIN … PRIVATE KEY`, `lightwalletd.*password`) | **0 hits** |
| files ever added or deleted on secret-shaped paths | only source/docs names (`wallet.rs`, `WALLET-NOTES.md`, `harness/seed.sh`, the pre-rename `canary-seed.rs`) |
| was `data/` (wallet DB, chain data, per-run evidence) ever committed? | **never** — 0 paths in history |
| anything sensitive still tracked today (`wallet`, `*.key`, `seed`, `*.env`, `*.pem`) | only source files whose *names* match; no key material |
| viewing keys inside recorded evidence (`data/runs/demo/demo-private/{report,run}.json`) | 0 hits for `ufvk`/`uview`/`unified_full_viewing` |

One observation belongs to the published-evidence unit rather than here, and the parent is not the unit's
hunter: `docs/WALLET-NOTES.md` commits a dev **seed phrase** and dev **UFVK**. The phrase is the canonical
BIP-39 test vector (`abandon ×11 · about`) and the wallet is a throwaway regtest wallet, so nothing private
is disclosed — no key material of value exists to disclose. If a finding, it is a *hardening* note about the
habit (a doc-level pattern that would be a disclosure if repeated on a wallet with real history), not a
crossed boundary. Recorded here so the report does not have to rediscover it.

## Coverage note

No prior `run-*` directory exists for this target, so this is the first pass: one wave of 5 units, a
post-wave coverage critic and a distinct final-clean critic, and one fresh verifier per candidate. Absence
of findings on a unit means that unit's assignment was covered, never that the repo is clean.

## Companion validation blocks for this unit (verbatim)

### [companion validation block] PROTOCOLS-RPC-AND-MESSAGING.md#Validation rules — verbatim

## Validation rules (apply before reporting ANY finding here)

1. Name the realistic producer or peer, accepted message, authenticated channel identity, affected principal/resource, and unauthorized mutation or disclosure.
2. For disagreement claims, cite both parsers/consumers and the divergent decoded value. Safe rejection by either side prevents confirmation.
3. For replay/order claims, establish actual delivery guarantees and reproduce the invariant failure with a bounded local/in-memory transport.
4. For authorization and isolation, verify all interceptor, broker ACL, gateway, and consumer layers visible in source. External attachments make the candidate `needs_validation`.
5. Return `confirmed` only with the complete message lifecycle and observed meaningful result. Return `needs_validation` with the exact broker, service identity, route, or delivery fact required.

## Promotion procedure (verbatim; reference only — the parent performs it, you never do)

```text
Artifact promotion procedure (trusted parent-side code only):
Reference only for you: the parent performs these steps; you never perform them.

Before execution, the parent opens and retains trusted, non-inheritable directory
descriptors for the agent's scratch/ and artifacts/ roots, and records an allowlist
of expected scratch-relative artifact files plus explicit per-file and cumulative
byte limits. Never pass those descriptors to the agent or sandbox. After the sandbox
and all its processes terminate, trusted parent-side code promotes each allowlisted
file separately:

1. Validate the declared relative path: reject absolute, empty, `.`, `..`, or
   symlinked components.
2. Walk each parent component from the retained scratch-root descriptor with
   no-follow directory-relative operations; never reopen by path.
3. Open the leaf no-follow and nonblocking.
4. Verify with `fstat` that it is a regular file with link count exactly one and
   within the recorded per-file and cumulative byte limits.
5. Enforce those limits again while reading from that descriptor.
6. Copy exactly the verified size, repeat `fstat`, and reject a changed identity,
   type, link count, or size.
7. For the destination, walk every parent component from the retained
   artifacts-root descriptor with no-follow directory-relative operations; require
   each existing component to be a real directory, and create any missing directory
   exclusively before reopening and verifying it no-follow.
8. Create the leaf exclusively without following links, verify that the opened
   destination is a regular file with link count exactly one, and copy from the
   verified source descriptor without reopening either path.
9. Use equivalent race-safe APIs on non-POSIX systems.
10. Never recursively copy or glob scratch, extract an archive into artifacts, or
    open or promote a symlink, FIFO, socket, device, directory, hard-linked file,
    changing file, or file that exceeds its bound.
11. If any check is unavailable, cannot be enforced, or fails, discard the scratch
    entry; if it is decisive evidence, retain `needs_validation` with the exact
    promotion blocker.
```

## Source/local execution boundary and the approved sandbox

Target source is read-only: `/home/kiter/hackathon/zanary` (commit `7058406`, worktree clean — do not modify it).
Target-controlled code may run only inside the OS-enforced sandbox. The approved disposable copy of the
reviewed revision is `/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` (`git archive HEAD` copy with `rust-toolchain.toml` removed so the image's own
toolchain is used — do not restore the pin). Rebuild it yourself if needed:
`rm -rf /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && mkdir -p /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && cd /home/kiter/hackathon/zanary && git archive HEAD | tar -x -C /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && rm -f /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2/rust-toolchain.toml`

```bash
docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 \
  -v /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2:/t -v /home/kiter/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim \
  sh -c 'CARGO_BUILD_JOBS=6 cargo test --offline -p <crate> <filter> 2>&1 | tail -30'
```

No external network, no image pulls, no `sudo`, no host paths except `/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` and the read-only registry,
writes only inside `/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` and your own scratch. Out of bounds: the regtest rig's containers and host ports
(18232/18233/9067/9068), any network service, any public URL, and anything that mutates state outside your
scratch. If a decisive fact lives behind one of those, it stays a `needs_validation` blocker — name it exactly.

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-2/scratch` (scratch-relative files only, `check-*.txt` / `check-*.json` /
`evidence.txt`, 64 KiB per file, 256 KiB total); the parent promotes into `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-2/artifacts`. You never write
retained artifacts, another agent's directory, or any shared run file.

## Schema branches (verbatim from `report-schema.json`)

`confirmed`:

```json
{
  "type": "object",
  "description": "A source-grounded vulnerability that was independently demonstrated.",
  "properties": {
    "verdict": {
      "type": "string",
      "const": "confirmed"
    },
    "fingerprint": {
      "type": "string",
      "minLength": 1,
      "pattern": "^[A-Za-z0-9][A-Za-z0-9._:/@+-]*$",
      "description": "A stable source-derived identifier that does not change between validation states."
    },
    "title": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "description": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "root_cause": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "intended_behavior": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "trace": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "kind": {
            "type": "string",
            "enum": [
              "entrypoint",
              "propagation",
              "sink"
            ]
          },
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "scope": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "kind",
          "file",
          "line",
          "scope",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "evidence": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "file",
          "line",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "conditions": {
      "type": "array",
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "kind": {
            "type": "string",
            "enum": [
              "authentication_level",
              "authorization_role",
              "user_interaction",
              "system_configuration",
              "network_routing",
              "environmental_dependency",
              "data_state",
              "timing_dependency",
              "third_party_dependency"
            ]
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "kind",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "execution": {
      "type": "object",
      "description": "Target-neutral reproduction in the target's native interface.",
      "properties": {
        "attacker_perspective": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        },
        "payloads": {
          "type": "array",
          "minItems": 1,
          "uniqueItems": true,
          "items": {
            "type": "string"
          }
        },
        "instructions": {
          "type": "array",
          "minItems": 1,
          "items": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "observed_result": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        }
      },
      "required": [
        "attacker_perspective",
        "payloads",
        "instructions",
        "observed_result"
      ],
      "additionalProperties": false
    },
    "remediation": {
      "type": "object",
      "properties": {
        "strategy": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        },
        "code_changes": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "file_name": {
                "type": "string",
                "minLength": 1
              },
              "fixed_code": {
                "type": "string"
              }
            },
            "required": [
              "file_name",
              "fixed_code"
            ],
            "additionalProperties": false
          }
        }
      },
      "required": [
        "strategy"
      ],
      "additionalProperties": false
    },
    "severity": {
      "type": "object",
      "properties": {
        "likelihood": {
          "type": "object",
          "properties": {
            "score": {
              "type": "string",
              "enum": [
                "informational",
                "low",
                "medium",
                "high",
                "critical"
              ]
            },
            "reason": {
              "type": "string",
              "minLength": 1,
              "visibleContent": true
            }
          },
          "required": [
            "score",
            "reason"
          ],
          "additionalProperties": false
        },
        "impact": {
          "type": "object",
          "properties": {
            "score": {
              "type": "string",
              "enum": [
                "informational",
                "low",
                "medium",
                "high",
                "critical"
              ]
            },
            "reason": {
              "type": "string",
              "minLength": 1,
              "visibleContent": true
            }
          },
          "required": [
            "score",
            "reason"
          ],
          "additionalProperties": false
        },
        "overall_severity": {
          "type": "string",
          "enum": [
            "informational",
            "low",
            "medium",
            "high",
            "critical"
          ]
        }
      },
      "required": [
        "likelihood",
        "impact",
        "overall_severity"
      ],
      "additionalProperties": false
    },
    "confidence": {
      "type": "object",
      "properties": {
        "score": {
          "type": "string",
          "enum": [
            "low",
            "medium",
            "high"
          ]
        },
        "reason": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        }
      },
      "required": [
        "score",
        "reason"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "verdict",
    "fingerprint",
    "title",
    "description",
    "root_cause",
    "intended_behavior",
    "trace",
    "evidence",
    "conditions",
    "execution",
    "remediation",
    "severity",
    "confidence"
  ],
  "additionalProperties": false
}
```

`needs_validation`:

```json
{
  "type": "object",
  "description": "A source-grounded candidate whose decisive validation is blocked.",
  "properties": {
    "verdict": {
      "type": "string",
      "const": "needs_validation"
    },
    "fingerprint": {
      "type": "string",
      "minLength": 1,
      "pattern": "^[A-Za-z0-9][A-Za-z0-9._:/@+-]*$"
    },
    "title": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "description": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "claimed_root_cause": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "trace": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "kind": {
            "type": "string",
            "enum": [
              "entrypoint",
              "propagation",
              "sink"
            ]
          },
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "scope": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "kind",
          "file",
          "line",
          "scope",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "evidence": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "file",
          "line",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "blockers": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "string",
        "minLength": 1,
        "visibleContent": true
      }
    },
    "validation_plan": {
      "type": "object",
      "properties": {
        "local": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        },
        "deployment": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        }
      },
      "additionalProperties": false
    }
  },
  "required": [
    "verdict",
    "fingerprint",
    "title",
    "description",
    "claimed_root_cause",
    "trace",
    "evidence",
    "blockers",
    "validation_plan"
  ],
  "additionalProperties": false
}
```

`rejected`:

```json
{
  "type": "object",
  "description": "A source-grounded candidate refuted during validation.",
  "properties": {
    "verdict": {
      "type": "string",
      "const": "rejected"
    },
    "fingerprint": {
      "type": "string",
      "minLength": 1,
      "pattern": "^[A-Za-z0-9][A-Za-z0-9._:/@+-]*$"
    },
    "title": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "description": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "claimed_root_cause": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "trace": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "kind": {
            "type": "string",
            "enum": [
              "entrypoint",
              "propagation",
              "sink"
            ]
          },
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "scope": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "kind",
          "file",
          "line",
          "scope",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "evidence": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "file",
          "line",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "reason": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    }
  },
  "required": [
    "verdict",
    "fingerprint",
    "title",
    "description",
    "claimed_root_cause",
    "trace",
    "evidence",
    "reason"
  ],
  "additionalProperties": false
}
```

## Prior records with the same fingerprint

None. This target has no prior audit run, and no other current candidate carries this fingerprint.

## Your result

Return exactly one JSON object, no prose and no code fences:
`{"decision": "confirmed|needs_validation|rejected", "record": { ... }}` where `record` exactly matches
the decision's verdict branch above (`confirmed` records carry `root_cause`, `intended_behavior`,
`conditions`, `execution`, `remediation`, `severity`, `confidence`; `needs_validation` records carry
`claimed_root_cause`, `blockers`, a nonempty `validation_plan` with at least one applicable `local` or
`deployment` step and no severity or execution; `rejected` records carry `claimed_root_cause` and `reason`).
A corrected record replaces the hunter's wording. Keep the fingerprint unchanged unless you found a
genuinely different root cause — and then say so in the record.
