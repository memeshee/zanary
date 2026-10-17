# Candidate verifier prompt — verifier-14 on `zanary/payer-expiry-text-compare`

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

Hunter: `hunter-payer` · proposed verdict: `confirmed` · fingerprint: `zanary/payer-expiry-text-compare`

```json
{
  "proposed_verdict": "confirmed",
  "fingerprint": "zanary/payer-expiry-text-compare",
  "title": "Bounded payer enforces grant expiry as a text comparison, so an already-passed expiry written with a UTC offset (or malformed) still pays",
  "description": "decide() refuses an invoice only when the ASCII comparison `now >= grant.expires_at` holds (crates/probe/src/payer.rs:91). Both sides are unvalidated strings: `now` comes from the workspace clock (fractional seconds, RFC3339 Z) and a grant file's `expires_at` is used verbatim (payer.rs:173-184 parses only the *default* path). Equal instants written differently therefore get opposite verdicts, and any string that sorts after the clock never expires. The lower-trust principal is whoever supplies the grant JSON to `zanary-probe payer --grant` (architecture.md boundary 3: caller-supplied JSON and paths into the probe CLI); a tool that stamps the expiry in local time, or a malformed value, is enough. The control that should have rejected it is the grant's own expiry rule ('the instant after which the authority is void', docs/BOUNDED-PAYER.md). The failure is invisible after the fact: the ledger records no decision time and payer-policy has no expiry rule.",
  "root_cause": "Text comparison of two RFC3339 strings instead of an instant comparison, plus no validation or canonicalisation of `expires_at` on the grant-file path. `if now >= grant.expires_at.as_str()` (payer.rs:91) is only correct when both strings are the same encoding \u2014 canonical UTC 'Z', whole seconds, fixed width. The clock producer and the grant author are different components with different conventions (record.rs:132-137 emits fractional seconds; `data/grants/*.json` are whole-second Z; RFC3339 permits offsets).",
  "intended_behavior": "An invoice shown at or after the grant's expiry instant must be refused ('now >= expires_at? -> refuse', docs/BOUNDED-PAYER.md:42-47), and that refusal must hold for any valid RFC3339 representation of that instant.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/payer.rs",
      "line": 149,
      "scope": "zanary-probe payer --grant",
      "description": "the grant path is taken from argv and the file is read; `expires_at` enters as an operator/tool-supplied string (parsed into Grant at 167)"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 173,
      "scope": "Grant -> PayerGrant",
      "description": "a file-supplied expires_at is used verbatim; only the absent (None) case parses and re-formats a timestamp (178-182)"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 250,
      "scope": "decision loop",
      "description": "decide(&grant, spent, invoice, &now) is called once per invoice with the startup clock string"
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/payer.rs",
      "line": 91,
      "scope": "decide() expiry rule",
      "description": "lexicographic comparison: a past instant written with a positive offset sorts after the clock and is treated as unexpired; the invoice is then pushed to to_pay (258-259) and broadcast (306-308)"
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/payer.rs",
      "line": 91,
      "description": "`if now >= grant.expires_at.as_str()` \u2014 the expiry rule is a text comparison"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 173,
      "description": "a grant file's expires_at is taken verbatim; the RFC3339 parser (178) runs only for the default"
    },
    {
      "file": "crates/probe/src/record.rs",
      "line": 132,
      "description": "utc_now_rfc3339 formats fractional seconds; data/runs/payer-pass/report.json shows verified_at \"2026-10-16T10:57:34.300715717Z\" while data/grants/*.json are whole-second Z"
    },
    {
      "file": "crates/observation/src/lib.rs",
      "line": 194,
      "description": "PayerGrant.expires_at is a String \u2014 the observation model never derives an instant"
    },
    {
      "file": "crates/properties/src/properties/payer_policy.rs",
      "line": 105,
      "description": "payer-policy echoes grant.expires_at in its summary but never compares it to anything"
    }
  ],
  "conditions": [
    {
      "kind": "data_state",
      "description": "the grant file's expires_at is a valid RFC3339 instant already in the past but written with a positive UTC offset (a local-time writer), or is any string that sorts after the clock text ('never')"
    },
    {
      "kind": "timing_dependency",
      "description": "at the exact expiry second, the payer's own clock text carries fractional seconds while a whole-second grant no longer compares as expired"
    },
    {
      "kind": "system_configuration",
      "description": "the payer is invoked with that grant file; nothing (schema, flag, validation) rejects a non-canonical or malformed expires_at"
    }
  ],
  "execution": {
    "attacker_perspective": "whoever supplies the grant JSON to `zanary-probe payer --grant` (the CLI's documented untrusted-input surface); no privileged access needed \u2014 a grant written from local time is enough",
    "payloads": [
      "grant.json: {\"budget_zat\":1000,\"per_payment_cap_zat\":400,\"allowlist\":[\"uregtest1merchant\"],\"expires_at\":\"2026-11-02T13:00:00+14:00\"}",
      "grant.json: \"expires_at\": \"never\" or \"2030-99-99T00:00:00Z\"",
      "--now 2026-11-02T00:00:00Z with the two encodings of 2026-11-01T23:00:00Z (2026-11-01T23:00:00Z vs 2026-11-02T13:00:00+14:00)"
    ],
    "instructions": [
      "git archive HEAD the repo into a disposable /t sandbox copy (the reviewed repository is not modified)",
      "append the bounded unit-test module (check-payer-decide-src.txt) to /t/crates/probe/src/payer.rs",
      "docker run --rm --network=none --cpus=4 --memory=2800m --memory-swap=4g --pids-limit=1024 -v <sandbox>:/t -v <cargo registry>:ro -w /t -e CARGO_BUILD_JOBS=4 rust:1-slim sh -c 'cargo test --offline -p zanary-probe --bin zanary-probe -- hunter_payer --nocapture --test-threads=1'"
    ],
    "observed_result": "7 passed; 0 failed. H1a: expires_at 2026-11-01T23:00:00Z (canonical) -> allowed=false \"refused: the grant expired at 2026-11-01T23:00:00Z (this invoice was shown at 2026-11-02T00:00:00Z)\"; the SAME instant as 2026-11-02T13:00:00+14:00 -> allowed=true \"paid: within the cap (400 zat) and the budget (10 of 1000 zat spent after this)\". H1b: now 2026-11-01T00:00:00.300715717Z vs expiry 2026-11-01T00:00:00Z -> allowed=true. H1c: \"never\", \"2030-99-99T00:00:00Z\", \"9999-01-01T00:00:00Z\" -> allowed=true."
  },
  "remediation": {
    "strategy": "compare instants: parse both `now` and the grant's expires_at with the RFC3339 parser the crate already depends on (time::OffsetDateTime::parse) and refuse the run when expires_at is not a valid timestamp; additionally record `decided_at` on every decision so the property can re-derive the expiry rule after the fact.",
    "code_changes": [
      {
        "file_name": "crates/probe/src/payer.rs",
        "fixed_code": "let now_dt = time::OffsetDateTime::parse(now, &time::format_description::well_known::Rfc3339)\n    .map_err(|e| anyhow!(\"the clock is not RFC3339: {e}\"))?;\nlet expires_dt = time::OffsetDateTime::parse(\n    &grant.expires_at,\n    &time::format_description::well_known::Rfc3339,\n)\n    .map_err(|e| anyhow!(\"the grant's expires_at {} is not RFC3339: {e}\", grant.expires_at))?;\nif now_dt >= expires_dt {\n    return Decision {\n        allowed: false,\n        reason: format!(\n            \"refused: the grant expired at {} (this invoice was shown at {now})\",\n            grant.expires_at\n        ),\n    };\n}"
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "medium",
      "reason": "needs an expires_at that is not canonical UTC-Z text: a local-time writer, a tool that emits an offset or fractional seconds, or a typo. The repo's own grants and fixtures are canonical Z, so the shipped demo data does not trigger it."
    },
    "impact": {
      "score": "medium",
      "reason": "a payment made after the authority was void; the ledger cannot show it (no decision timestamp, payer_policy has no expiry rule) so the run still reports PASS \u2014 bounded by the offset window, and the chain here is regtest-only."
    },
    "overall_severity": "medium"
  },
  "confidence": {
    "score": "high",
    "reason": "the real decide() was exercised in the sandbox: equal instants, two encodings, opposite verdicts; the trace is four lines and the fix is local (one comparison and one validation)."
  }
}
```

## Its linked coverage unit and the hunter's checks

- coverage_id: `crates%2Fprobe%2Fsrc%2Fpayer.rs%23decide::crates%2Fproperties%2Fsrc%2Fproperties%2Fpayer_policy.rs%23evaluate::bounded-payer::ATTACK-CLASSES.md%23Business%20logic`
- surface: `crates/probe/src/payer.rs#decide` (subsystem `bounded-payer`)
- boundary: `crates/properties/src/properties/payer_policy.rs#evaluate`
- attack class: `ATTACK-CLASSES.md#Business logic` · assignment owner: `hunter-payer`
- unit reviewed paths: ["crates/observation/src/lib.rs", "crates/probe/src/flow.rs", "crates/probe/src/ops.rs", "crates/probe/src/payer.rs", "crates/probe/src/record.rs", "crates/properties/src/properties/payer_policy.rs"]

The unit's checks, including the local checks whose promoted artifacts you may read
(paths are relative to the run directory `/home/kiter/security-audit-skill/zanary/run-1`):

```json
[
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/observation/src/lib.rs"
    ],
    "invariant": "decide() is the single decision point for an invoice and applies the five grant rules in the documented order (revoked -> expired -> allowlist -> cap -> budget) over the grant actually recorded for the run",
    "method": "source",
    "result": "verified for the reference payer: decide is called once per invoice before any payment (payer.rs:246-270), the grant recorded in the run is the same object decide used (payer.rs:193-199, 388), and every refusal/allowance carries the first rule that fired. The rule bodies themselves are where the authority model leaks: the expiry rule is a text comparison (payer.rs:91), the allowlist rule never validates the receiver (100), the budget rule counts invoice principal only (119-128).",
    "artifact": null
  },
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/properties/src/properties/payer_policy.rs",
      "crates/observation/src/lib.rs",
      "crates/probe/src/record.rs"
    ],
    "invariant": "the ledger a payer records carries everything the payer-policy boundary needs to re-derive every grant rule, and the boundary re-derives the same verdict as the payer's own decide()",
    "method": "source",
    "result": "not re-derivable: PayerDecision has no decision time (observation:200-214), so the expiry rule cannot be checked at all (payer_policy only echoes expires_at into its summary, 105-116); the property never cross-checks the ledger against run.payments; and its internal-error rule keys on the literal reason prefix \"error:\" (68) which the reference payer never writes (it aborts, payer.rs:298-309).",
    "artifact": null
  },
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/probe/src/flow.rs"
    ],
    "invariant": "the payer's own gate holds: an expired grant cannot pay (as an instant), a repeated invoice_id is decided once, the budget bounds what the run spends, a misspelled grant key cannot widen authority, and an invoice the payment loop cannot pay is not decided payable",
    "method": "local",
    "result": "all five fail against the reviewed decide() (7/7 tests passed, i.e. the failures are the observed behaviour): H1a same expiry instant two encodings -> refused (canonical Z) vs ALLOWED (offset +14:00); H1b fractional clock against a whole-second expiry -> ALLOWED; H1c \"never\"/\"2030-99-99T00:00:00Z\" -> ALLOWED; H2 invoice dup-8301 decided ALLOW twice (spent 300 then 600 of 1000); H3 1-zat invoices -> 10,000,000 zat of fees for a 1,000-zat budget; H4 typo'd allow_list/expiry -> allowlist [] (any receiver), expires_at None; H8 allowlisted non-address receiver and 600-byte memo -> ALLOWED",
    "artifact": "agents/hunter-payer/artifacts/check-payer-decide.txt"
  },
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/properties/src/properties/payer_policy.rs",
      "crates/observation/src/lib.rs",
      "crates/probe/src/flow.rs"
    ],
    "invariant": "the payer-policy boundary re-derives the grant verdict from the run record for invoice identity, for whether a paid row is backed by a payment, and for the grant's expiry",
    "method": "local",
    "result": "passes in all three blind spots (3/3 tests passed): H5 two paid rows sharing one invoice_id (the second with no txid) -> PASS; H6 a paid row with zero run.payments entries -> PASS; H7 a ledger whose recorded grant expired in 2020 -> PASS, with the expiry echoed in the summary text only",
    "artifact": "agents/hunter-payer/artifacts/check-payer-policy.txt"
  },
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/probe/src/ops.rs"
    ],
    "invariant": "what the payer spends and does per allowed invoice is what the grant's budget accounts for (the cost of paying at the shared wallet layer is visible to the authority model)",
    "method": "source",
    "result": "the budget comparison never sees the payment layer's cost: every allowed invoice is one shielded transaction plus mine_and_resync(11 blocks) (payer.rs:291-310; ops.rs:59) and the payer's own funding model budgets 10,000 zat of ZIP-317 fee per payment (payer.rs:530), none of which the budget rule (119-128) or payer_policy (77-90) counts \u2014 the fee the payer knows it will pay is invisible to the authority it enforces.",
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

### [companion validation block] RESOURCE-EXHAUSTION-AND-AVAILABILITY.md#Validation rules — verbatim

## Validation rules (apply before reporting ANY finding here)

1. Name untrusted input, requester work, service amplification or retained resource, shared blast radius, and recovery. Missing limits without concrete shared impact are hardening.
2. Confirm no source-visible upstream, parser, queue, tenant, or framework bound prevents the path. Unknown deployed controls require `needs_validation`.
3. For superlinear behavior, establish the accepted complexity and bounded local growth. For leaks, show repeatable retention after cleanup should occur. For fatal paths, identify process/supervisor isolation.
4. Prioritize by low requester work, unauthenticated reachability, cross-tenant scope, persistence, and poor recovery; do not validate with availability impact.
5. Return `confirmed` only with safe local proof and meaningful shared effect. Return `needs_validation` with the exact upstream limit, topology, quota, or recovery observation an owner must check.

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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-14/scratch` (scratch-relative files only, `check-*.txt` / `check-*.json` /
`evidence.txt`, 64 KiB per file, 256 KiB total); the parent promotes into `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-14/artifacts`. You never write
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
