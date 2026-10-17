# Candidate verifier prompt — verifier-9 on `video/zanary-demo.mp4:publish-video-unverifiable-media-content`

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

Hunter: `hunter-video` · proposed verdict: `needs_validation` · fingerprint: `video/zanary-demo.mp4:publish-video-unverifiable-media-content`

```json
{
  "proposed_verdict": "needs_validation",
  "fingerprint": "video/zanary-demo.mp4:publish-video-unverifiable-media-content",
  "title": "Publish-video step serves a media asset whose disclosed content no repository artifact constrains or verifies",
  "description": "The publish job copies video/*.mp4 verbatim into the repo's world-readable Pages site (privacy-gate.yml:172, 209-215) and the player page asserts \"What is on screen comes from a real run\" (:198). At the reviewed commit one file matches: video/zanary-demo.mp4 (sha256 38a6b73fb25bab4e3888988639658e603336a8784e5503af7783179cb159ec75, 3717435 B) - a 112 s rendered cut of a local rehearsal take whose source log and whole recording/rendering pipeline sit outside the repository (docs/VIDEO-SCRIPT.md:8-9; privacy-gate.yml:169-170). Sandbox container inspection found no stowaway content (no text track, no trailing bytes, no metadata beyond encoder tags - check-video-container.txt), so the remaining disclosure surface is exactly the decoded pixels and narration audio. Whether those 112 s carry only the intended-public run data enumerated by the cut sheet (commands, txids/heights, receivers, property table, ledger lines) or also captured non-claimed values (e.g. the dev seed phrase the CLI prints by design, crates/probe/src/wallet.rs:38; host paths; environment strings) is decisive for the world-readable-Pages vs rig-data disclosure boundary and is not establishable from source.",
  "claimed_root_cause": "The publisher ships a committed binary media asset to a world-readable site on the strength of a manual, out-of-tree review: no in-tree control (hash pin on the published bytes, content scan, per-take review, cut-sheet diff) constrains what the frames and audio of video/*.mp4 show, and the producer pipeline and its source take are deliberately outside the repository - so the artifact's actual disclosure content is unverified by construction, not by policy.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "video/zanary-demo.mp4",
      "line": 1,
      "scope": "the committed media asset (ISO-BMFF, avc1 + mp4a, 112 s, sha256 38a6b73f...)",
      "description": "The disclosure surface: a rendered cut whose on-screen content is claimed in docs/VIDEO-SCRIPT.md but not derivable or checkable from any repository artifact."
    },
    {
      "kind": "propagation",
      "file": "docs/VIDEO-SCRIPT.md",
      "line": 9,
      "scope": "cut sheet - origin of every on-screen block",
      "description": "\"every line below is verbatim from ~/hackathon/zanary-rehearsal.log, a real take that exited 0\" - the content's source is a log outside the reviewed revision; the TTS/render/ffmpeg pipeline is out of tree."
    },
    {
      "kind": "propagation",
      "file": ".github/workflows/privacy-gate.yml",
      "line": 170,
      "scope": "publish job, comment above the demo step",
      "description": "The workflow itself records that the recording pipeline stays out of the repository, so no CI step can re-derive or check the media content."
    },
    {
      "kind": "sink",
      "file": ".github/workflows/privacy-gate.yml",
      "line": 172,
      "scope": "publish job, step 'publish the two-minute demo next to the claims'",
      "description": "cp video/*.mp4 public/video/ - unconditional verbatim copy with no check; runs on main pushes and the weekly re-probe."
    },
    {
      "kind": "sink",
      "file": ".github/workflows/privacy-gate.yml",
      "line": 215,
      "scope": "uses: actions/upload-pages-artifact (:209) + actions/deploy-pages (:215)",
      "description": "public/ is deployed to memeshee.github.io/zanary/video/ - world-readable and submission-critical; no content review stands between the committed bytes and the world."
    }
  ],
  "evidence": [
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 172,
      "description": "Verbatim copy of every video/*.mp4 into the published site: no hash pin, no scan, no transform; the \"|| echo no video in the tree - skipping\" fallback also lets a missing or renamed file deploy a page whose player 404s without failing the job."
    },
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 170,
      "description": "\"The recording pipeline itself stays out of the repository\" - the producer of the published bytes is not reviewable in-tree."
    },
    {
      "file": "docs/VIDEO-SCRIPT.md",
      "line": 8,
      "description": "\"Nothing is staged: every line below is verbatim from ~/hackathon/zanary-rehearsal.log, a real take that exited 0\" - the asset's central content claim rests on an out-of-tree log."
    },
    {
      "file": "docs/VIDEO-SCRIPT.md",
      "line": 34,
      "description": "The enumerated intended on-screen blocks (typed command; run output; payment lines with txids/heights; property table; claim block) plus the payer ledger and kill-switch block (:55-68), the control 'caught' block (:76-82) and the index/Actions shots (:90) - the intended-public content list a frame/audio review would diff against."
    },
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 198,
      "description": "The published page asserts \"What is on screen comes from a real run ...\" - a content claim the repository cannot verify."
    },
    {
      "file": "crates/properties/src/properties/report_leak.rs",
      "line": 7,
      "description": "The project's excess-disclosure rule for exported artifacts (key-material tokens, unknown addresses, emails) evaluates run.receipts only; no counterpart binds the media asset."
    },
    {
      "file": "video/zanary-demo.mp4",
      "line": 1,
      "description": "Sandbox container inspection (check-video-container.txt): avc1 + mp4a tracks only; no text/subtitle/chapter track; 0 trailing bytes after the last box; creation times 0; metadata only \u00a9too=Lavf63.1.101; raw-byte token scan 0 hits - the undecoded pixels/audio remain the only unexamined region."
    },
    {
      "file": "crates/probe/src/wallet.rs",
      "line": 38,
      "description": "The CLI prints the dev seed phrase by design; the demo scripts never invoke `wallet`, so source gives no reason to expect it on screen - but only the pixels can settle what a take captured."
    }
  ],
  "blockers": [
    "Decoding is impossible under the sandbox policy: the approved image (rust:1-slim) has no ffmpeg/ffprobe/python3, --network=none forbids fetching one, image pulls are out of policy, and the read-only offline registry contains no media crates; running the host's ffmpeg on the file would process a target fixture outside the OS-enforced sandbox.",
    "The producer is out of tree: the source take (~/hackathon/zanary-rehearsal.log) and the TTS/frame-render/ffmpeg pipeline are outside the reviewed revision by design (docs/VIDEO-SCRIPT.md:8-9; privacy-gate.yml:169-170), so the frames cannot be re-derived from the repository or diffed against a committed take.",
    "The take cannot be re-recorded to compare: the regtest rig's containers and host ports are out of bounds for this audit, and the recording procedure depends on that rig."
  ],
  "validation_plan": {
    "local": "Owner-side, offline, one bounded pass over the committed regular file (sha256 38a6b73f..., already verified in-sandbox): extract frames (ffmpeg -vf fps=2 PNGs + OCR) and audio (-f wav + transcription) and diff every extracted token against the on-screen block list in docs/VIDEO-SCRIPT.md:34-99; fail on any report-leak token class (ivk/ovk/ufvk/fvk/viewing key/spending key/seed phrase/mnemonic), any absolute host path or username, any address/hex string absent from the run's own records, or any audio beyond the narration script.",
    "deployment": "Owner-observed on the published site (not a probe of a live target): confirm the bytes served at memeshee.github.io/zanary/video/zanary-demo.mp4 equal the reviewed revision's asset (same sha256) and that the player page loads exactly that one file; if a frame/audio value is found that the claim does not cover, replace the asset and re-deploy - the published copy (Pages, git history, crawler mirrors) cannot be recalled, only rewritten."
  }
}
```

## Its linked coverage unit and the hunter's checks

- coverage_id: `video%2Fzanary-demo.mp4%23published-asset::.github%2Fworkflows%2Fprivacy-gate.yml%23publish-video::published-evidence::ATTACK-CLASSES.md%23Feature%20abuse%20and%20data%20leakage`
- surface: `video/zanary-demo.mp4#published-asset` (subsystem `published-evidence`)
- boundary: `.github/workflows/privacy-gate.yml#publish-video`
- attack class: `ATTACK-CLASSES.md#Feature abuse and data leakage` · assignment owner: `hunter-video`
- unit reviewed paths: [".github/workflows/privacy-gate.yml", "README.md", "crates/properties/src/properties/report_leak.rs", "docs/CLAIMS.md", "docs/PROPERTY-IMPL-NOTES.md", "docs/SUBMISSION.md", "docs/THREAT-MODEL.md", "docs/VIDEO-SCRIPT.md", "video/zanary-demo.mp4"]

The unit's checks, including the local checks whose promoted artifacts you may read
(paths are relative to the run directory `/home/kiter/security-audit-skill/zanary/run-1`):

```json
[
  {
    "agent_id": "hunter-video",
    "reviewed_paths": [
      ".github/workflows/privacy-gate.yml"
    ],
    "invariant": "The publish-video step reproduces exactly the reviewed revision's committed media bytes into the Pages output path: a verbatim local copy, gated to main after the gate jobs, with no network fetch, transform, or content-dependent branch a contributor could steer.",
    "method": "source",
    "result": "Lines 166-172 copy video/*.mp4 to public/video/ verbatim (no hash pin, no scan, no transform); the publish job runs only when github.ref == 'refs/heads/main' (line 113) and after needs: gate; lines 209-215 upload public/ and deploy it; the player page hardcodes the served file name zanary-demo.mp4 (line 196). Nothing else decides what is served, and at the reviewed commit the glob matches exactly one file (video/zanary-demo.mp4, 3717435 B).",
    "artifact": null
  },
  {
    "agent_id": "hunter-video",
    "reviewed_paths": [
      "docs/VIDEO-SCRIPT.md",
      "README.md",
      "docs/SUBMISSION.md"
    ],
    "invariant": "What the published asset is claimed to show, and where its on-screen text comes from, are stated in-tree well enough to name the intended-public content the pixels would have to match.",
    "method": "source",
    "result": "The cut sheet claims a 112 s cut of one real exit-0 take, every on-screen block verbatim from an out-of-tree rehearsal log (lines 8-9), and enumerates the blocks (typed command, payment lines with txids/heights, property table, claim block, payer ledger + kill-switch block, control 'caught' block, index and Actions shots). README lines 15-18 and SUBMISSION.md lines 99/111 surface the same claims to judges. Source states the claim but cannot exhibit the pixels; the take log and the render pipeline (TTS beats, frame renderer, ffmpeg) are out of tree by stated design (privacy-gate.yml:169-170).",
    "artifact": null
  },
  {
    "agent_id": "hunter-video",
    "reviewed_paths": [
      "video/zanary-demo.mp4"
    ],
    "invariant": "The published container carries no stowaway payload: no text/subtitle/chapter track, no bytes after the last top-level box, no author/path/creation metadata, and no plaintext key-material or address-shaped strings in any uncompressed region.",
    "method": "local",
    "result": "Sandboxed parse (rust:1-slim, --network=none, disposable copy) of the committed bytes (sha256 38a6b73fb25bab4e3888988639658e603336a8784e5503af7783179cb159ec75; 3717435 B): one avc1 video track (112.13 s, Lavc63.1.101 libx264) plus one mp4a audio track (112.15 s); no text/subtitle/chapter track; the file ends exactly at the end of the last top-level box (0 trailing bytes, no appended polyglot payload); creation/modification times 0; udta/meta carries only \u00a9too=Lavf63.1.101. Raw-byte token scan: 0 hits for abandon / seed phrase / mnemonic / ufvk / uview / rpcpassword / zanary-rehearsal / kiter / uregtest / allowlist. Decoder probe: no ffmpeg/ffprobe/python3 in the sandbox and no media crates in the read-only offline registry, so decoded frames/audio - the only remaining content surface - are not inspectable.",
    "artifact": "agents/hunter-video/artifacts/check-video-container.txt"
  },
  {
    "agent_id": "hunter-video",
    "reviewed_paths": [
      "crates/properties/src/properties/report_leak.rs",
      "docs/PROPERTY-IMPL-NOTES.md",
      "docs/THREAT-MODEL.md",
      "docs/CLAIMS.md"
    ],
    "invariant": "One of the project's own disclosure rules for published artifacts reaches the media asset (no key-material tokens, no disclosure beyond a claim), or their absence is explicit and acknowledged.",
    "method": "source",
    "result": "report-leak - the excess-disclosure rule (discloses list, key-material tokens, addresses not in the run's records, emails) - inspects flow-exported receipts only and skips runs without receipts (report_leak.rs:81-89); the threat model's disclosure inventory (THREAT-MODEL.md:3, 43-46) and CLAIMS.md's publishing rules (36-57) cover claim artifacts, not the video or the Pages video publish. No in-tree rule or CI check reviews the media content.",
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

### [companion validation block] DATA-ISOLATION-AND-LIFECYCLE.md#Validation rules — verbatim

## Validation rules (apply before reporting ANY finding here)

1. Name attacker or lower-trust principal, protected data/state, affected owner/tenant, alternate copy or operation, and unauthorized disclosure or mutation.
2. Cite both intended source-of-truth policy and the path that omits or disagrees with it. Confirm another layer does not enforce the same tenant/lifecycle condition.
3. Use local dummy tenants and non-sensitive fixtures to prove cross-scope access or stale lifecycle behavior. Stop at the minimum observable record or operation.
4. If external cache, object storage, replicas, analytics, backup, or retention policy is required, classify `needs_validation` and state the owner-observed check.
5. Return `confirmed` only with complete lineage and concrete boundary impact. Return `needs_validation` with the exact unresolved storage, ACL, invalidation, retention, or restore fact.

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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-9/scratch` (scratch-relative files only, `check-*.txt` / `check-*.json` /
`evidence.txt`, 64 KiB per file, 256 KiB total); the parent promotes into `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-9/artifacts`. You never write
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
