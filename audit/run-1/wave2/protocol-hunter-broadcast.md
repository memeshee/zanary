# Wave-2 hunter prompt — hunter-broadcast (payer-authority-to-broadcast)

You are hunter `hunter-broadcast` in a security-audit run whose run directory is `/home/kiter/security-audit-skill/zanary/run-1` (outside the target).
Find source-grounded security invariant failures in your assigned coverage unit and return exactly one JSON object
matching the "Structured hunter result" contract at the end of this prompt — no prose, no code fences.

Target: `/home/kiter/hackathon/zanary` — repo `memeshee/zanary`, reviewed commit `7058406`, worktree clean; no real funds exist
anywhere in this system (Zcash **regtest** only). Zero confirmed findings on a unit is a legitimate result;
inventing low-severity findings is a failure.

## 1. Architecture (verbatim, written by the parent)

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

## 2. Your assignment

- canonical agent id: `hunter-broadcast`
- label: `payer-authority-to-broadcast`
- coverage_id: `crates%2Fprobe%2Fsrc%2Fops.rs%23propose_shielded_payment%20%2F%20build_sign_broadcast::the%20authority%20check%20%28decide%29%20-%3E%20the%20transaction%20actually%20built%20and%20broadcast::bounded-payer::ATTACK-CLASSES.md%23Business%20logic`
- subsystem: `bounded-payer`
- boundary: `the authority check (decide) -> the transaction actually built and broadcast`
- surface: `crates/probe/src/ops.rs#propose_shielded_payment / build_sign_broadcast`
- attack class: `ATTACK-CLASSES.md#Business logic`
- wave: 2 · profile: standard · no prior runs for this target

Repository-relative starting paths:
- `crates/probe/src/ops.rs`
- `crates/probe/src/payer.rs`

Assignment block map (this unit's ledger record, authoritative):

```json
{
  "coverage_id": "crates%2Fprobe%2Fsrc%2Fops.rs%23propose_shielded_payment%20%2F%20build_sign_broadcast::the%20authority%20check%20%28decide%29%20-%3E%20the%20transaction%20actually%20built%20and%20broadcast::bounded-payer::ATTACK-CLASSES.md%23Business%20logic",
  "canonical_refs": {
    "surface": "crates/probe/src/ops.rs#propose_shielded_payment / build_sign_broadcast",
    "boundary": "the authority check (decide) -> the transaction actually built and broadcast",
    "subsystem": "bounded-payer",
    "attack_class": "ATTACK-CLASSES.md#Business logic"
  },
  "surface": "crates/probe/src/ops.rs#propose_shielded_payment / build_sign_broadcast",
  "boundary": "the authority check (decide) -> the transaction actually built and broadcast",
  "subsystem": "bounded-payer",
  "attack_class": "ATTACK-CLASSES.md#Business logic",
  "starting_paths": [
    "crates/probe/src/ops.rs",
    "crates/probe/src/payer.rs"
  ],
  "ordinary_attack_class_block": "ATTACK-CLASSES.md#Business logic",
  "selected_companion_blocks": [
    "PROTOCOLS-RPC-AND-MESSAGING.md#Core discipline",
    "PROTOCOLS-RPC-AND-MESSAGING.md#Replay, ordering, and transaction attack classes",
    "PROTOCOLS-RPC-AND-MESSAGING.md#Universal moves",
    "PROTOCOLS-RPC-AND-MESSAGING.md#Validation rules"
  ],
  "excluded_blocks": [],
  "prior_status": "none",
  "attempts": [],
  "wave": 2,
  "status": "in_progress"
}
```

Why this unit exists: decide() authorises a receiver string and an amount; the shielded transaction — fees, change, pool selection, actual addressees — is built by the shared wallet layer the payer calls (payer.rs:306-308), and neither payer-policy nor any other check compares run.payments against the ledger. A divergence between what the grant authorised and what the transaction does would be invisible. That is a different surface from this unit (decide/payer_policy) and from the cli-inputs unit (flow parsing + record writing), so it needs its own coverage unit.

## 3. The exact selected blocks (copied verbatim — this is your assignment's class text)

### [ordinary attack-class block] ATTACK-CLASSES.md#Business logic — verbatim from `/home/kiter/.hermes/skills/security/security-audit/ATTACK-CLASSES.md`

**Business logic** (subagent_type: `general`)
Hunt logic errors by hand: standard scanners cannot find them, and they yield high-impact findings. For each major workflow:
- **State machine violations**: Can you skip steps? Go backwards? Reach an invalid state? What happens if you replay a completed flow? What about partial failure — if step 2 of 3 fails, is step 1 rolled back?
- **Race conditions with business impact**: Concurrent operations that produce invalid states (double-spend, double-approve, lost updates). Focus on operations that check-then-act non-atomically.
- **Numeric/quantity manipulation**: Negative values, zero values, overflow, precision loss, type coercion between string and number.
- **Access boundary violations**: Not "does the permission check exist" but "is it the right check for the business rule?" Can input to one operation bypass a restriction enforced on a different operation for the same effect?
- **Implicit trust assumptions**: Data from storage, config, other components, or plugins assumed safe because "we validated it on the way in." What if a different code path wrote it?
- **Time-based logic**: Expiry checks, scheduling, rate windows, clock skew. What happens at exact boundary moments? What about timezone differences between components?
- **Default and fallback behavior**: What is the security posture when config is missing? When a feature flag is off? When a dependency is unavailable? When the system is mid-migration?

### [companion block] PROTOCOLS-RPC-AND-MESSAGING.md#Core discipline — verbatim from `/home/kiter/.hermes/skills/security/security-audit/PROTOCOLS-RPC-AND-MESSAGING.md`

## Core discipline (include in every agent prompt for this domain)

```
- "Internal" is not authentication. Name the peer identity at every hop and show how it becomes the application principal used for authorization.
- Schema validation proves message shape, not provenance, resource authority, ordering, or safe values. Follow decoded fields to policy and side effects.
- Broker guarantees and application guarantees differ. Write down retry, ordering, acknowledgement, deduplication, and transaction behavior before evaluating state changes.
- Parser disagreement requires two concrete consumers, schema versions, or wire representations and one security-relevant divergent value.
- Use `confirmed` for source-complete paths plus bounded local producer/consumer tests. Use `needs_validation` for broker ACL, service-mesh identity, topic attachment, or compatibility behavior outside the repository.
```

### [companion block] PROTOCOLS-RPC-AND-MESSAGING.md#Replay, ordering, and transaction attack classes — verbatim from `/home/kiter/.hermes/skills/security/security-audit/PROTOCOLS-RPC-AND-MESSAGING.md`

## Replay, ordering, and transaction attack classes (subagent_type: `general`)

**Duplicate delivery and idempotency gaps**
Retries or redelivery repeat a side effect because deduplication is absent, occurs after mutation, or uses a key that collides across tenants or operations. Confirm the broker's delivery model and the side effect that is not naturally idempotent.

**Out-of-order and stale message acceptance**
Older state, revoked membership, canceled work, or pre-step-up authorization arrives after newer state and overwrites it. Review sequence/version checks, tombstones, partition changes, and restore/replay workflows.

**Acknowledgment/commit ordering defects**
Acknowledgment occurs before durable commit and loses security-relevant work, or commit happens before an unreliable acknowledgment and duplicates a mutation. Evaluate transactional outbox/inbox behavior and failure recovery.

**Partial multi-consumer transitions**
Several consumers jointly implement one authorization or business transition, but retries and partial failure leave only a subset committed. Identify invariants that must become durable atomically or compensate with current authorization.

### [companion block] PROTOCOLS-RPC-AND-MESSAGING.md#Universal moves — verbatim from `/home/kiter/.hermes/skills/security/security-audit/PROTOCOLS-RPC-AND-MESSAGING.md`

## Universal moves (apply across the above)

- Draw producer → broker/transport → gateway → consumer → storage for each message family. At each hop record authenticated peer, authoritative tenant/resource fields, validation, and side effect.
- Feed the same small fixture to every in-repo schema version or language binding. Test duplicate, missing, unknown, boundary, replayed, and reordered messages without producing load.
- Compare normal, retry, dead-letter, replay, migration, reflection, stream, and gateway-transcoded routes. Security policy must survive transport changes.

### [companion block] PROTOCOLS-RPC-AND-MESSAGING.md#Validation rules — verbatim from `/home/kiter/.hermes/skills/security/security-audit/PROTOCOLS-RPC-AND-MESSAGING.md`

## Validation rules (apply before reporting ANY finding here)

1. Name the realistic producer or peer, accepted message, authenticated channel identity, affected principal/resource, and unauthorized mutation or disclosure.
2. For disagreement claims, cite both parsers/consumers and the divergent decoded value. Safe rejection by either side prevents confirmation.
3. For replay/order claims, establish actual delivery guarantees and reproduce the invariant failure with a bounded local/in-memory transport.
4. For authorization and isolation, verify all interceptor, broker ACL, gateway, and consumer layers visible in source. External attachments make the candidate `needs_validation`.
5. Return `confirmed` only with the complete message lifecycle and observed meaningful result. Return `needs_validation` with the exact broker, service identity, route, or delivery fact required.

## 4. Explicit exclusions

- Ordinary attack-class blocks other than `ATTACK-CLASSES.md#Business logic`: owned by their own ledger units, listed under "peer-owned coverage ids" below. Do not hunt them.
- `CLIENT-SIDE.md`, `DESKTOP-MOBILE-AND-LOCAL-IPC.md`: no such surface exists in this target (no client application, no IPC/IPC-transport layer); they are excluded for every unit of this run.
- Companion files and sections not listed above: their domains are covered by peer units (e.g. published-evidence disclosure by the `crates/report/src/lib.rs#index-renderer` unit, ci-and-release injection by the `action.yml#inputs-and-steps` unit). Excluded as duplicate coverage, not as unimportant.
- Nothing in the target outside this run's scope: the run is unscoped, so only the boundaries above are excluded.

## 5. Hunting method (verbatim from the methodology)

```text
## Defensive vulnerability-finding method

Your goal is to find source-grounded security invariant failures and the smallest fix,
not to expand harm beyond the boundary result. Stay within source review and bounded local execution.
Do not contact deployed endpoints, provider APIs, registries, identity systems,
message brokers, shared services, or other users. Use local dummy data only.

READ THE CODE AT DEPTH. Follow each assigned input through parsing, identity,
authorization, normalization, state, derived copies, and the final sink. Read sibling,
legacy, batch, retry, cancellation, migration, and error paths that produce the same
effect. Compare sibling controls for equivalence, not only presence, and compare what
one component guarantees with what the next component assumes.

WORK FROM A CONCRETE INVARIANT:
1. Name the lower-trust principal and starting capability.
2. Name the accepted value, action, state transition, or resource selector.
3. Locate the control that should reject, bind, isolate, limit, or revoke it.
4. Trace the exact source path after that decision.
5. Stop at the smallest affected dummy record, wrong return value, process-integrity
   effect, or locally observable shared-resource effect.
6. State a source-level change and regression case that enforce the invariant.

DEPTH BOUND: trace only paths that can reach your assigned boundary or whose
guarantees that boundary relies on. Stop a line of investigation as soon as the
invariant is settled either way, and record the result in your structured output —
a covered, candidate, or blocked disposition, or an `uncovered` entry — instead of
continuing to search.

TEST SAD PATHS AND DISAGREEMENTS. Check absent, empty, zero, negative, maximum,
over-limit, duplicate, mixed encoding, stale, revoked, reordered, concurrent,
partially migrated, failed dependency, and rollback state only where the interface
accepts them. Compare canonicalization and units at every parser or policy handoff.
For multi-step issues, treat each output as a prerequisite and do not assume a later
boundary. If any prerequisite is not established, record a blocker.

When a proposed high or critical candidate reveals a reusable root cause, search paths
owned by the assigned coverage IDs for lexical, structural, and logical variants.
Consolidate the same root cause, but establish each variant's conditions and impact
independently. Do not investigate peer-owned units. Return a variant with no current
coverage unit as `uncovered`.

USE THE NARROWEST LOCAL CHECK THAT SETTLES THE CLAIM. Target-controlled builds,
tests, processes, browsers, emulators, fuzzers, and fixture processing may run only
inside the parent-approved OS-enforced sandbox. It must disable external networking,
start from an empty allowlisted environment, expose target and tools read-only, permit
writes only to your scratch directory, and apply low CPU, memory, process, file-size,
disk, and wall-clock limits. Isolated loopback is allowed only for a local fixture.
If any control is unavailable, do not execute: return needs_validation with that exact
blocker. Prefer an existing unit test, minimal function harness, dummy-tenant service
call, small malformed fixture, deterministic race schedule, or locally rendered policy.
Do not install or fetch tools.

Record the exact input, command, limits, and minimum result. For the environment,
record only allowlisted variable names and safe non-secret values needed to reproduce
the check. Never capture the ambient environment, inherited variables, credentials,
authentication state, or unrelated host paths. The target-controlled process writes
only in scratch. After the sandbox and all its processes terminate, only trusted
parent-side code may promote predeclared scratch-relative files, following the
promotion procedure block included verbatim in this prompt. You and target code never
write retained artifacts. If promotion is unavailable or fails for decisive evidence,
return needs_validation with the exact promotion blocker.
Never stress availability, invoke a live target, use a real credential, publish an
artifact, or continue past the minimum observed effect.

A deployment, browser, provider, broker, OS, proxy, package, secret, or identity fact
outside source is not proof either way. If one such fact is decisive, return a
needs_validation record with the exact missing observation and safe owner-observed check.
```

## 6. Promotion procedure (verbatim; reference only — you never perform it)

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

## 7. Validation rules (verbatim)

```text
## Candidate gate

1. A candidate needs a complete repository-relative source trace and evidence for the
   claimed root cause, including the strongest source-visible control.
2. A proposed confirmed record needs a bounded local observed result, meaningful impact
   across a stated boundary, complete conditions, and no visible preventing layer.
3. Do not strengthen a crash into code execution, ordinary work into shared availability,
   or a same-principal action into privilege gain.
4. If a required fact is not source-visible or locally observable, use
   needs_validation. Name exact blockers; do not give it severity or speculative completion.
5. A missing best practice with no affected principal/resource is excluded or hardening,
   not a finding. A candidate disproved by source is not needs_validation.
6. Use the same source-derived fingerprint for the same root cause in every state.
   It must match `^[A-Za-z0-9][A-Za-z0-9._:/@+-]*$` and must not include a line,
   wave, agent, severity, or verdict.
7. Return an empty candidate array when nothing survives these gates.
```

## 8. Prior evidence and peer units

- Carried same-source prior confirmed exclusions: none. This target has no prior audit runs (`prior_status: none` for every unit), so nothing is excluded from hunting on those grounds.
- Peer-owned current coverage ids you must not duplicate:
- `action.yml%23inputs-and-steps::.github%2Fworkflows%2Fprivacy-gate.yml%23jobs::ci-and-release::ATTACK-CLASSES.md%23Injection`
- `crates%2Fprobe%2Fsrc%2Fbin%2Fzanary-seed.rs%23cli-and-wallet-writes::crates%2Fprobe%2Fsrc%2Fbin%2Fzanary-seed.rs%23seed-db-and-chain-intake::cli-inputs::ATTACK-CLASSES.md%23Resource%20and%20file%20handling`
- `crates%2Fprobe%2Fsrc%2Fflow.rs%23parse-args-and-flow-load::crates%2Fprobe%2Fsrc%2Frecord.rs%23write-artifacts::cli-inputs::ATTACK-CLASSES.md%23Resource%20and%20file%20handling`
- `crates%2Fprobe%2Fsrc%2Fgate.rs%23test-summary-and-run-artifacts::.github%2Fworkflows%2Fprivacy-gate.yml%23evidence-artifact-upload::ci-and-release::ATTACK-CLASSES.md%23Feature%20abuse%20and%20data%20leakage`
- `crates%2Fprobe%2Fsrc%2Fpayer.rs%23decide::crates%2Fproperties%2Fsrc%2Fproperties%2Fpayer_policy.rs%23evaluate::bounded-payer::ATTACK-CLASSES.md%23Business%20logic`
- `crates%2Fprobe%2Fsrc%2Fpayer.rs%23load-invoices::merchant-supplied%20invoice%20directory%20-%3E%20payer%20memory%20and%20payment%20work::bounded-payer::RESOURCE-EXHAUSTION-AND-AVAILABILITY.md%23Resource%20accumulation%20attack%20classes`
- `crates%2Freport%2Fsrc%2Flib.rs%23index-renderer::.github%2Fworkflows%2Fprivacy-gate.yml%23publish::published-evidence::ATTACK-CLASSES.md%23Cryptography%20and%20secrets`
- `crates%2Freport%2Fsrc%2Fmain.rs%23run-and-report-intake::crates%2Freport%2Fsrc%2Flib.rs%23renderers-and-property-evaluation::published-evidence::ATTACK-CLASSES.md%23Resource%20and%20file%20handling`
- `crates%2Freport%2Fsrc%2Freceipt.rs%23verify_signature%20%2F%20verify_attestation%20%2F%20validate_claim::a%20flow-signed%20receipt%20%28signer%20ak%20%2B%20optional%20wallet%20attestation%29%20-%3E%20the%20offline%20%60verify-receipt%60%20verdict%3A%20signature%20validity%2C%20attestation-to-wallet%20binding%2C%20claim%2Fbody%20agreement%2C%20exit%20code::published-evidence::ATTACK-CLASSES.md%23Cryptography%20and%20secrets`
- `harness%2Fdocker-compose.yml%23services::electriccoinco%2Fzcashd%3Alatest%20and%20electriccoinco%2Flightwalletd%3Alatest%20image%20references::local-rig::SUPPLY-CHAIN-AND-RELEASE.md%23Dependency%20and%20build-input%20attack%20classes`
- `harness%2Fdocker-compose.yml%23services::harness%2Fzcash.conf%23rpc-credentials::local-rig::ATTACK-CLASSES.md%23Resource%20and%20file%20handling`
- `video%2Fzanary-demo.mp4%23published-asset::.github%2Fworkflows%2Fprivacy-gate.yml%23publish-video::published-evidence::ATTACK-CLASSES.md%23Feature%20abuse%20and%20data%20leakage`

## 9. Sandbox, scratch, promotion allowlist, and result contract

Target code may run only in the OS-enforced sandbox. The approved disposable copy of the reviewed
revision is `/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` (a `git archive HEAD` copy with `rust-toolchain.toml` removed so the image's own
toolchain is used — do not restore it). Rebuild it yourself if it does not exist:
`rm -rf /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && mkdir -p /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && cd /home/kiter/hackathon/zanary && git archive HEAD | tar -x -C /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && rm -f /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2/rust-toolchain.toml`

```bash
docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 \
  -v /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2:/t -v /home/kiter/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim \
  sh -c 'CARGO_BUILD_JOBS=6 cargo test --offline -p <crate> <filter> 2>&1 | tail -30'
```

Rules: no external network, no image pulls, no `sudo`, no host paths besides `/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` and the read-only
registry, no writes outside `/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` and your own scratch dir, and no change to the target repository
itself (read-only review; this audit describes fixes and never applies them). Bounded checks only —
one test, one fixture, one small invocation — and stop at the first observed boundary result.
Out of bounds: the regtest rig's containers and host ports (18232/18233/9067/9068), any network
service, any public URL (GitHub, Pages, Actions logs and artifacts are data you may read only if the
parent supplies it), fuzzing for its own sake, and anything that mutates state outside your scratch.
If a decisive fact lives behind one of these, the candidate is `needs_validation` and you name the
blocker exactly.

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/hunter-broadcast/scratch`; the parent promotes into `/home/kiter/security-audit-skill/zanary/run-1/agents/hunter-broadcast/artifacts`.
Allowed scratch names: `evidence.txt`, `check-*.txt`, `check-*.json`. Limits: 64 KiB per file,
256 KiB total for your scratch tree. You never write retained artifacts, another agent's directory,
or any shared run file (`coverage-ledger.json`, `findings.json`, `architecture.md`, `REPORT.md`, …).

### Structured hunter result (verbatim from the methodology)

```json
{
  "units": [
    {
      "coverage_id": "one assigned ID",
      "disposition": "covered|candidate|blocked",
      "reviewed_paths": ["repo/relative/path"],
      "checks": [
        {
          "agent_id": "canonical owner of this check",
          "reviewed_paths": ["repo/relative/path owned by this check"],
          "invariant": "specific control checked for this unit",
          "method": "source|local",
          "result": "what source or the bounded check established",
          "artifact": "agents/<agent-id>/artifacts/file for local, null for source"
        }
      ],
      "candidate_fingerprints": [],
      "unresolved": []
    }
  ],
  "candidates": [],
  "hardening": ["concrete non-finding note"],
  "uncovered": [
    {
      "surface": "...",
      "boundary": "...",
      "subsystem": "...",
      "attack_class": "...",
      "starting_paths": ["repo/relative/path"],
      "reason": "why this needs its own deterministic coverage unit"
    }
  ]
}
```

Each `candidates` entry is schema-shaped except that it uses `proposed_verdict` in place of `verdict`:

- `proposed_verdict: "confirmed"`: include every field required by the `confirmed` branch of `report-schema.json` other than `verdict`: `fingerprint`, title, description, `root_cause`, `intended_behavior`, ordered `trace`, `evidence`, `conditions`, target-neutral `execution`, `remediation`, `severity`, and `confidence`. The execution instructions describe only the bounded local check already performed. `payloads` holds the minimum test input, fixture, or native invocation. `observed_result` records actual local output. Overall severity must not exceed observed impact.
- `proposed_verdict: "needs_validation"`: include every field required by that schema branch other than `verdict`: `fingerprint`, title, description, `claimed_root_cause`, ordered `trace`, `evidence`, nonempty `blockers`, and `validation_plan` with at least one applicable `local` or `deployment` step. Do not invent an inapplicable context. Do not include severity, execution, remediation, reason, or a confirmed `root_cause`. `deployment` is an owner-observed check, not a request to probe a live target.

Every assigned coverage ID appears exactly once in `units`. A `covered` unit needs an owner, nonempty `reviewed_paths` and `checks`, no unresolved fact, and no candidate. A `candidate` unit has the same owned evidence and is the only state that carries linked fingerprints. A `blocked` unit is an owned partial review with nonempty paths, checks, and unresolved facts but no fingerprint. All source paths are repository-relative, never absolute or traversal paths. A trace with several entries begins at `entrypoint`, ends at `sink`, and labels intermediate steps `propagation`. Every check has its own canonical lowercase `agent_id` and nonempty `reviewed_paths`; the unit-level list is exactly the union of those owned paths. A `source` check uses `artifact: null`. A `local` check uses one successfully parent-promoted regular file beneath `agents/<check.agent_id>/artifacts/`; this permits a verifier to add independently owned evidence without taking ownership from the hunter. Never link scratch, an output-root file, or another check owner's artifact.

The `confirmed` branch of `report-schema.json` (verbatim; your `confirmed` candidates must satisfy it
field-for-field, with `proposed_verdict: "confirmed"` in place of `verdict`):

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

The `needs_validation` branch (verbatim; use `proposed_verdict: "needs_validation"` instead of `verdict`,
and include no severity, execution, remediation or confirmed `root_cause`):

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

Return exactly one JSON object with keys `units`, `candidates`, `hardening`, `uncovered` and nothing else.
