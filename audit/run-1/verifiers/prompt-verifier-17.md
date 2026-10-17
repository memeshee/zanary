# Candidate verifier prompt — verifier-17 on `zanary/rig-image-tags-unpinned`

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

Hunter: `hunter-images` · proposed verdict: `needs_validation` · fingerprint: `zanary/rig-image-tags-unpinned`

```json
{
  "proposed_verdict": "needs_validation",
  "fingerprint": "zanary/rig-image-tags-unpinned",
  "title": "The rig that decides every published privacy claim runs from two mutable :latest image tags",
  "description": "Both rig containers are selected by floating tags (harness/docker-compose.yml:9 electriccoinco/zcashd:latest, :34 electriccoinco/lightwalletd:latest). Nothing in the reviewed revision pins a digest, sets pull_policy, or checks a repo digest, so the content that runs is chosen by whoever can re-point those Docker Hub tags, not by this repository. That content is not marginal: zcashd is the chain and the miner, and lightwalletd is the CompactTxStreamer the wallets sync against, so the pair produces every block, transaction and balance the flow runner observes. The same pair is then re-read by the gate's chain verification (crates/probe/src/gate.rs:165) and by the anchor, which is a memo on that chain, so the 'independent' re-derivation and the timestamp share the unbound producer. The resolution point is reached by (a) any third-party workflow that uses the composite action (action.yml:85 runs harness/up.sh on the caller's runner), (b) this repo's own gate and payer jobs (.github/workflows/privacy-gate.yml:87 via uses: ./ and :239 directly), and (c) any judge or user following the documented one command (README.md:111/114, docs/SUBMISSION.md:36). A substituted image therefore both executes on those machines and can fabricate the data a 'privacy claim' is made of, while the recorded claim carries no image, digest or container identity at all (crates/report/src/lib.rs:33-54 binds flow_id/flow_commit/chain/height/report_hash only) - so a claim cannot be tied to the container content that produced it, and a drift is invisible in the evidence, the artifacts and the published page. The docs assert fixed identities that no mechanism enforces (harness/HARNESS.md:54-55 and README.md:75 name v6.20.0 / v0.5.5 next to the mutable tags). Whether the tags have in fact been re-pointed, and who can re-point them, is a registry fact this run cannot observe - hence needs_validation rather than confirmed.",
  "claimed_root_cause": "harness/docker-compose.yml selects both rig services by mutable tag and the repository never binds the running container to a reviewed artifact: no image digest, no pull_policy, no repo-digest comparison anywhere in the tree (grep @sha256: / pull_policy / repoDigests over the tracked tree: 0 files), and harness/up.sh:116 inspects the indexer container only for .State.Running. The one control that could bind identity - the resolver call at harness/up.sh:60 - is a bare `docker compose up -d`, and the claim schema has no field to record what it resolved, so the gate verdict and the published evidence are produced by an unbound third-party component.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "harness/docker-compose.yml",
      "line": 9,
      "scope": "services.zcashd.image (and services.lightwalletd.image at :34)",
      "description": "The chain/miner container and the indexer container are selected by floating tags (electriccoinco/zcashd:latest, electriccoinco/lightwalletd:latest); the file pins no digest, platform or pull policy, so the satisfying artifact is decided outside this repository."
    },
    {
      "kind": "propagation",
      "file": "harness/up.sh",
      "line": 60,
      "scope": "the documented one-command bring-up",
      "description": "`docker compose up -d` resolves those tags on whichever machine runs the harness - necessarily from the registry on a fresh CI runner - and the script records nothing about what it resolved; its only identity-adjacent call (:116) reads .State.Running."
    },
    {
      "kind": "propagation",
      "file": "action.yml",
      "line": 85,
      "scope": "composite action step 'start the throwaway chain'",
      "description": "A third-party workflow that consumes `uses: memeshee/zanary@main` executes this step, so the current content of both tags is pulled and run inside the caller's runner/job, which is the execution boundary the tag controller reaches."
    },
    {
      "kind": "propagation",
      "file": ".github/workflows/privacy-gate.yml",
      "line": 87,
      "scope": "jobs.gate (uses: ./ at :87, job payer at :239)",
      "description": "This repository's own gate and payer jobs run the same harness/up.sh, and the publish job (:110-163) renders the resulting runs to Pages, so the artifact published as the project's public privacy evidence is decided downstream of the unpinned containers."
    },
    {
      "kind": "propagation",
      "file": "crates/report/src/lib.rs",
      "line": 33,
      "scope": "Report / claim schema",
      "description": "What a claim binds: version, flow_id, flow_commit, chain ('regtest' as a self-asserted string), heights, findings, evidence, report_hash - no image, digest, container or environment identity, so the container content behind a claim is unrecorded and uncheckable after the fact."
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/gate.rs",
      "line": 165,
      "scope": "gate verdict (chain_verified) and its published summary",
      "description": "The pass/fail verdict is re-derived from the same unbound node+indexer pair (and the anchor it may publish is a memo on the chain those containers produce), so the value that reaches the build result, the uploaded artifacts and the public claim index is produced by a component whose identity this repository never fixed or recorded."
    }
  ],
  "evidence": [
    {
      "file": "harness/docker-compose.yml",
      "line": 9,
      "description": "image: electriccoinco/zcashd:latest - floating tag, no @sha256 digest, no pull_policy"
    },
    {
      "file": "harness/docker-compose.yml",
      "line": 34,
      "description": "image: electriccoinco/lightwalletd:latest - same for the indexer; the second service mounts nothing and adds no identity requirement"
    },
    {
      "file": "harness/up.sh",
      "line": 60,
      "description": "the only resolution point: bare `docker compose up -d` (the compose header documents the same command for the user)"
    },
    {
      "file": "harness/up.sh",
      "line": 116,
      "description": "the rig's only docker inspect call reads .State.Running - liveness, not image identity"
    },
    {
      "file": "action.yml",
      "line": 85,
      "description": "run: harness/up.sh - the composite action third parties are told to use brings up the unpinned rig in their runner"
    },
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 239,
      "description": "run: harness/up.sh in the payer job; the gate job reaches the same step through uses: ./ at :87"
    },
    {
      "file": "harness/HARNESS.md",
      "line": 54,
      "description": "documents `electriccoinco/zcashd:latest` (v6.20.0) - a fixed version asserted next to a mutable tag, with no mechanism enforcing either the version or the identity"
    },
    {
      "file": "README.md",
      "line": 75,
      "description": "'zcashd v6.20.0 regtest + lightwalletd v0.5.5, one `docker compose up -d`' - the reproducibility the README sells is not bound to the pulled artifact"
    },
    {
      "file": "docs/SUBMISSION.md",
      "line": 36,
      "description": "'One command: harness/up.sh && scripts/demo-run.sh' - the judge-facing instruction resolves the unpinned tags on the judge's machine"
    },
    {
      "file": "docs/CI.md",
      "line": 113,
      "description": "'the run has a timestamp nobody had to be trusted for' - on this rig the timestamp (an anchor memo) and the run share the unpinned containers as their producer"
    },
    {
      "file": "crates/report/src/lib.rs",
      "line": 33,
      "description": "Report struct: the published claim carries no container/image/digest identity, so drift between what produced it and what the docs name is undetectable from the evidence"
    },
    {
      "file": "scripts/phase0-warmup.sh",
      "line": 29,
      "description": "even the repo's own warm-up pulls the same floating tag (`electriccoinco/zcashd:latest`) with a bare docker pull, recording no digest"
    }
  ],
  "blockers": [
    "Registry fact, not observable from source and out of bounds for this run (no registry access, no pull or inspect of the rig's images): which digest docker.io/electriccoinco/zcashd:latest and docker.io/electriccoinco/lightwalletd:latest resolve to, and whether either tag has been re-pointed since the reviewed revision (the first commit, e79f4e6, that introduced :latest on 2026-10-11).",
    "Hosted-control fact: who holds write access to the two Docker Hub repositories and whether tag immutability/protection is enabled there - this decides whether a lower-trust writer of the tag exists at all, which is the precondition of the handoff.",
    "Consumer fact: whether any run (this repo's weekly schedule, a push, or a third-party workflow using the action) executed between the reviewed commit and now against container content that differs from the reviewed-era image; decidable only from the registry plus Actions logs and artifact digests that are outside this unit's reach.",
    "No bounded local check can settle it: the decisive experiment needs a substituted node/indexer pair (a hostile lightwalletd answering CompactTxStreamer plus a node answering JSON-RPC) reachable by the probe, i.e. the rig's own images and host ports - both explicitly out of bounds - and the approved sandbox (--network=none, no image pulls) cannot build or run them."
  ],
  "validation_plan": {
    "local": "On a machine allowed to reach the registry: (1) record `docker manifest inspect electriccoinco/zcashd:latest | jq -r .config.digest` (and the same for lightwalletd:latest) at two separated times; (2) in a disposable checkout of the reviewed commit run `harness/up.sh` and immediately capture `docker inspect --format '{{index .RepoDigests 0}}' zanary-zcashd` and `... zanary-lwd`. A digest that changes while harness/docker-compose.yml is unchanged, or a digest that does not match the image the recorded/golden evidence was produced with, is the missing observation; a check needing no registry at all is `docker compose -f harness/docker-compose.yml config --images`, which shows the resolved references carry no digest.",
    "deployment": "Owner-observed, no probing: on the CI runner used by the privacy-gate jobs (and for one third-party caller of the action), print `docker inspect --format '{{.Config.Image}} {{index .RepoDigests 0}}' zanary-zcashd zanary-lwd` in the rig-diagnostics step for two runs a week apart; if the digests differ on an unchanged compose file, the tag drifted under a claim that is still published, and pinning both services to the observed digests plus comparing that digest in harness/up.sh before the flow runs is the closure."
  }
}
```

## Its linked coverage unit and the hunter's checks

- coverage_id: `harness%2Fdocker-compose.yml%23services::electriccoinco%2Fzcashd%3Alatest%20and%20electriccoinco%2Flightwalletd%3Alatest%20image%20references::local-rig::SUPPLY-CHAIN-AND-RELEASE.md%23Dependency%20and%20build-input%20attack%20classes`
- surface: `harness/docker-compose.yml#services` (subsystem `local-rig`)
- boundary: `electriccoinco/zcashd:latest and electriccoinco/lightwalletd:latest image references`
- attack class: `SUPPLY-CHAIN-AND-RELEASE.md#Dependency and build-input attack classes` · assignment owner: `hunter-images`
- unit reviewed paths: [".github/workflows/privacy-gate.yml", "README.md", "action.yml", "crates/probe/src/core.rs", "crates/probe/src/gate.rs", "crates/probe/src/main.rs", "crates/report/src/lib.rs", "docs/CI.md", "docs/SUBMISSION.md", "harness/HARNESS.md", "harness/docker-compose.yml", "harness/up.sh", "rust-toolchain.toml", "scripts/phase0-warmup.sh"]

The unit's checks, including the local checks whose promoted artifacts you may read
(paths are relative to the run directory `/home/kiter/security-audit-skill/zanary/run-1`):

```json
[
  {
    "agent_id": "hunter-images",
    "reviewed_paths": [
      "harness/docker-compose.yml",
      "harness/up.sh",
      "harness/HARNESS.md",
      "action.yml",
      ".github/workflows/privacy-gate.yml",
      "README.md",
      "docs/CI.md",
      "scripts/phase0-warmup.sh"
    ],
    "invariant": "both rig images are selected by an identity bound to the reviewed revision, and every consumer of the rig (the composite action, this repo's gate/payer jobs, the documented one-command setup) resolves exactly that artifact",
    "method": "source",
    "result": "refuted: docker-compose.yml:9 and :34 use floating :latest tags with no digest, no pull_policy and no platform pin; the tracked tree contains zero occurrences of @sha256: / pull_policy / repoDigests; resolution happens at the bare `docker compose up -d` (up.sh:60) and the rig's only inspect (up.sh:116) reads .State.Running; the callers are action.yml:85 (third-party consumers), .github/workflows/privacy-gate.yml:87/:239 (this repo's jobs); docs assert v6.20.0 / v0.5.5 (HARNESS.md:54-55, README.md:75) with nothing enforcing them.",
    "artifact": null
  },
  {
    "agent_id": "hunter-images",
    "reviewed_paths": [
      "harness/docker-compose.yml",
      "harness/up.sh",
      "action.yml",
      ".github/workflows/privacy-gate.yml",
      "crates/report/src/lib.rs",
      "harness/HARNESS.md"
    ],
    "invariant": "the repo-side facts above hold under mechanical re-derivation on a disposable copy, with no registry contact, and the recorded claim format really carries no environment identity",
    "method": "local",
    "result": "re-derived inside the approved sandbox (docker run --network=none, rust:1-slim, script on stdin, no target code executed, no image pulled or inspected): two image references and nothing else; 0 files matching @sha256:/pull_policy/repoDigests; callers action.yml:85 + privacy-gate.yml:239 (gate job via uses: ./ at :87); tracked claim fixtures mention no image/version identity (files=0) and the Report fields (crates/report/src/lib.rs:33-54) bind flow/chain/height/hash only; cited-file sha256 values match the target checkout (harness/docker-compose.yml 7d6e6470..., harness/up.sh 0958b05d...). Full transcript in the artifact.",
    "artifact": "agents/hunter-images/artifacts/check-images-refs.txt"
  },
  {
    "agent_id": "hunter-images",
    "reviewed_paths": [
      "crates/probe/src/main.rs",
      "crates/probe/src/core.rs",
      "crates/probe/src/gate.rs"
    ],
    "invariant": "the target code that consumes the rig can distinguish a substituted node/indexer from the reviewed one (any version, digest or chain-identity check on the client side, including at the gate's re-derivation)",
    "method": "source",
    "result": "no such control exists: crates/probe/src/main.rs:97-107 prints the indexer's self-reported chain_name/version/vendor and compares them to nothing; core.rs connects to the endpoint and waits for a tip height only (wait_for_indexer_height, core.rs:128-160); the flow runner's 'chain: regtest' is a self-asserted string in the artifact (flow.rs:396, payer.rs:394); the gate's chain_verified (gate.rs:165, summary fields :342-348) re-reads evidence from that same unbound pair. A substituted pair is therefore indistinguishable to the client, which is why the impact rides entirely on image identity - this unit's root cause.",
    "artifact": null
  },
  {
    "agent_id": "hunter-images",
    "reviewed_paths": [
      "rust-toolchain.toml",
      "docs/SUBMISSION.md"
    ],
    "invariant": "the entry points a judge or third-party consumer follows are bound to reviewed artifacts - the images the one command resolves, and the toolchain the build step resolves",
    "method": "source",
    "result": "refuted on both: docs/SUBMISSION.md:36 (and README.md:111) tells a judge to run `harness/up.sh && scripts/demo-run.sh`, which resolves the unpinned images with no digest check, and rust-toolchain.toml:4 sets channel = \"stable\" (its own comment :1-2 presents it as the pin against a pre-1.88 rustc) which action.yml:59 and privacy-gate.yml:50/:130/:231 install with a bare `rustup toolchain install` - a second unbound build input, recorded in this result as an uncovered variant because no current unit owns toolchain identity.",
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

### [companion validation block] SUPPLY-CHAIN-AND-RELEASE.md#Validation rules — verbatim

## Validation rules (apply before reporting ANY finding here)

1. Name the lower-trust actor, controllable source/cache/artifact/metadata, consuming trusted job or updater, and resulting unauthorized publication, code inclusion, secret disclosure, or privileged execution.
2. Prove artifact identity across the broken handoff. A different mutable name or unbound digest must reach a real consumer.
3. Verify built-in package-manager, repository-host, registry, and signing defaults for the pinned version. Unknown hosted controls require `needs_validation`.
4. Keep local validation bounded: use a harmless fixture repository, dummy credential marker, local registry/config, and non-production artifact namespace. Do not publish or alter a real release.
5. Return `confirmed` only with a complete source-visible handoff and meaningful result. Return `needs_validation` with the precise branch, runner, registry, signing, or deployment fact an owner must observe.

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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-17/scratch` (scratch-relative files only, `check-*.txt` / `check-*.json` /
`evidence.txt`, 64 KiB per file, 256 KiB total); the parent promotes into `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-17/artifacts`. You never write
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
