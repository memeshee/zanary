# Candidate verifier prompt — verifier-16 on `zanary/published-summary-carries-invocation-paths`

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

Hunter: `hunter-evidence` · proposed verdict: `needs_validation` · fingerprint: `zanary/published-summary-carries-invocation-paths`

```json
{
  "proposed_verdict": "needs_validation",
  "fingerprint": "zanary/published-summary-carries-invocation-paths",
  "title": "Published claim files carry the CI invocation paths verbatim (absolute runner paths in the documented usage)",
  "description": "The publish job republishes each run's test.json into public/claims/<run>/, and that summary stores the raw path arguments the gate was called with: flow_definition, run_dir and report_file. In the documented CI usage the action passes ${{ github.workspace }}-prefixed paths, so the claim a stranger downloads would embed the runner's absolute workspace paths, including the job's working directory - information no published claim or verification step consumes. The gate log and the uploaded artifact carry the same raw paths.",
  "claimed_root_cause": "the summary writer records the CLI path arguments (definition.display(), out_dir.display(), report_path.display()) without normalising them to workspace-relative paths, and the publish step copies test.json into the published claims directory",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "action.yml",
      "line": 103,
      "scope": "composite action",
      "description": "OUT is built as \"${{ github.workspace }}/${{ inputs.out }}\" and the flow candidate is resolved against ${{ github.workspace }} (line 96)"
    },
    {
      "kind": "propagation",
      "file": "action.yml",
      "line": 113,
      "scope": "composite action",
      "description": "zanary-probe test is invoked with --definition \"$FLOW\" --out \"$OUT\" and --summary \"$OUT/test.json\", so the absolute paths are the argument values"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/gate.rs",
      "line": 336,
      "scope": "summary writer",
      "description": "the machine-readable summary stores flow_definition = definition.display(), run_dir = outcome.out_dir.display(), report_file = outcome.report_path.display() - the raw arguments"
    },
    {
      "kind": "sink",
      "file": ".github/workflows/privacy-gate.yml",
      "line": 161,
      "scope": "publish job",
      "description": "cp \"$dir\"report.json \"$dir\"anchor.json \"$dir\"test.json \"public/claims/$name/\" republishes the summary, and upload-pages-artifact/deploy-pages (lines 209-215) serve it publicly"
    }
  ],
  "evidence": [
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 154,
      "description": "the index is rendered from claim-input and the per-run files are copied into public/claims/ - test.json included"
    },
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 97,
      "description": "the gate job uploads runs/${{ matrix.id }}/ as a 30-day world-downloadable artifact, which contains test.json and gate.log"
    },
    {
      "file": "data/runs/ci-checkout/test.json",
      "line": 17,
      "description": "recorded instance of the summary: flow_definition/run_dir/report_file are the raw invocation strings (relative here because the local recording invoked the probe with relative paths); the CI invocation passes the same fields as ${{ github.workspace }}-prefixed absolute arguments"
    },
    {
      "file": "crates/probe/src/gate.rs",
      "line": 365,
      "description": "the summary is written to the path given by --summary with no sanitisation of the three path fields"
    }
  ],
  "blockers": [
    "the published bytes live in the GitHub Pages claim files and in the run's uploaded artifact, which this read-only review may not fetch (no live URL or artifact access)",
    "whether github.workspace is an absolute host path in the runner is GitHub provider behaviour, not repository source"
  ],
  "validation_plan": {
    "local": "Run the documented gate locally the way the workflow does, with absolute paths (harness/up.sh, then ./target/release/zanary-probe test --definition \"$PWD/flows/ci-checkout.json\" --out \"$PWD/runs/private\" --summary \"$PWD/runs/private/test.json\") and read runs/private/test.json: the recorded copies in data/ used relative arguments, which is why the local fixture alone cannot settle it. A unit test on the summary writer asserting workspace-relative fields closes it permanently.",
    "deployment": "Owner opens the published claim file for one run (for example https://memeshee.github.io/zanary/claims/zanary-private-evidence/test.json, and one uploaded run artifact) and reads flow_definition, run_dir and report_file: if they are absolute runner paths (/home/runner/work/...), the disclosure is confirmed. Fix: write the three fields relative to the workspace (or drop test.json from the public/claims/ copy list)."
  }
}
```

## Its linked coverage unit and the hunter's checks

- coverage_id: `crates%2Freport%2Fsrc%2Flib.rs%23index-renderer::.github%2Fworkflows%2Fprivacy-gate.yml%23publish::published-evidence::ATTACK-CLASSES.md%23Cryptography%20and%20secrets`
- surface: `crates/report/src/lib.rs#index-renderer` (subsystem `published-evidence`)
- boundary: `.github/workflows/privacy-gate.yml#publish`
- attack class: `ATTACK-CLASSES.md#Cryptography and secrets` · assignment owner: `hunter-evidence`
- unit reviewed paths: [".github/workflows/privacy-gate.yml", ".gitignore", "action.yml", "crates/observation/src/lib.rs", "crates/probe/src/core.rs", "crates/probe/src/gate.rs", "crates/probe/src/ops.rs", "crates/properties/src/properties/report_leak.rs", "crates/report/fixtures/index.golden.html", "crates/report/fixtures/reports/report-2026-10-10.json", "crates/report/src/anchor.rs", "crates/report/src/index.rs", "crates/report/src/lib.rs", "crates/report/src/main.rs", "crates/report/src/verify.rs", "crates/report/tests/golden.rs", "data/claims/2026-10-11-checkout-private.json", "data/claims/index.html", "data/demo-payer/invoices/8301.json", "data/runs/ci-checkout/report.json", "data/runs/ci-checkout/test.json", "data/runs/demo/demo-control/report.json", "data/runs/demo/demo-private/anchor.json", "data/runs/demo/demo-private/report.json", "data/runs/demo/demo-private/run.json", "data/runs/demo/demo-private/test.json", "docs/ANCHOR-NOTES.md", "docs/CI.md", "docs/CLAIMS.md", "docs/LEAK-MATRIX.md", "docs/SUBMISSION.md", "docs/THREAT-MODEL.md", "flows/ci-checkout.json", "scratch/ci-private.log", "scripts/bounded-payer-demo.sh"]

The unit's checks, including the local checks whose promoted artifacts you may read
(paths are relative to the run directory `/home/kiter/security-audit-skill/zanary/run-1`):

```json
[
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/report/src/index.rs",
      "crates/report/src/main.rs",
      "crates/report/src/lib.rs",
      "crates/report/fixtures/index.golden.html"
    ],
    "invariant": "the published claim page shows the claim (flow id, clock, hashes, anchor txid, verdicts) and must never render the evidence payload or the anchor record's key material",
    "method": "source",
    "result": "render_index_with_anchors formats only flow_id/flow_commit(short)/chain/verified_at/height/ttl/countdown/evidence-root+report-hash (full only in title attributes)/anchor txid+height/evidence count/merchant_is_simulated/finding property-severity-verdict-title-note/regression links; no Evidence value (txid, receiver, memo, pool totals, merchant field) is ever formatted; every interpolated value passes esc(); the golden fixture and the four recorded renderings (crates/report/fixtures/index.golden.html, data/claims/index.html, data/runs/index.html, data/runs/demo/index.html) contain zero occurrences of any receiver address or memo hex",
    "artifact": null
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      ".github/workflows/privacy-gate.yml",
      "action.yml",
      ".gitignore"
    ],
    "invariant": "the publish job must publish a fixed allowlist of claim files and never widen to the rest of the run directory",
    "method": "source",
    "result": "publish (privacy-gate.yml:110-215) copies only index.html plus per-run report.json/anchor.json/test.json (line 161) and the tracked fixture reports (lines 143-148, 163); run.json, findings.json, gate.log and claim-input/ are not republished; the job is main-only and reads artifacts of its own workflow run; .gitignore excludes /data, claim-input/, public/, *.env, *.key",
    "artifact": null
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/report/src/anchor.rs",
      "crates/probe/src/ops.rs",
      "crates/probe/src/core.rs",
      "docs/THREAT-MODEL.md",
      "docs/ANCHOR-NOTES.md",
      "docs/CI.md"
    ],
    "invariant": "the only key material the pipeline publishes is the beacon viewing key its documented verifier needs; no seed, spending key or wallet credential reaches published output",
    "method": "source",
    "result": "anchor.json/test.json publish the beacon account-0 UFVK; the beacon wallet (WalletProfile::beacon) is instantiated only in the anchor path (ops.rs:540) and is paid only the 1000-zat anchor notes, so that key decrypts the anchor log and nothing else; no mnemonic, seed, USK, transparent key or node credential is written into report.json/anchor.json/test.json or the page (the docs state the beacon key must be public: THREAT-MODEL.md:48-56, ANCHOR-NOTES.md:32-36, CI.md:99)",
    "artifact": null
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/observation/src/lib.rs",
      "crates/report/src/verify.rs",
      "crates/properties/src/properties/report_leak.rs",
      "docs/CLAIMS.md",
      "docs/LEAK-MATRIX.md"
    ],
    "invariant": "published report evidence is a whitelist that excludes payer-side and receipt material, and every published evidence entry is re-checkable",
    "method": "source",
    "result": "collect_evidence (lib.rs:209-254) whitelists transaction/batch_size, recipient receivers, memo bytes, pool totals and merchant fields; Payment.sender_receivers, funding receivers, receipts (with signer ak and wallet attestation), merchant headers, policy strings and free-form observed values are never copied into a report; verify re-checks tx/receiver/memo/pool entries against the chain and deliberately SKIPs merchant fields as not-chain facts (verify.rs:774-781)",
    "artifact": null
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "docs/SUBMISSION.md",
      "data/claims/index.html",
      "data/claims/2026-10-11-checkout-private.json",
      "crates/report/fixtures/reports/report-2026-10-10.json",
      "flows/ci-checkout.json",
      "scripts/bounded-payer-demo.sh"
    ],
    "invariant": "what the documented publish shape actually contains for a recorded claim",
    "method": "source",
    "result": "the recorded claim file and page carry the full evidence array (txids, regtest receivers, 20-byte invoice memos, merchant webhook fields, pool totals) next to the page - intended and requirable by 'zanary-report verify'; the flat fixture report published as history is already tracked in the repo, and the bounded-payer demo dir that the publish job also downloads has no top-level report.json/anchor.json/test.json, so no payer data reaches public/",
    "artifact": null
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/probe/src/gate.rs",
      "data/runs/demo/demo-private/report.json",
      "data/runs/demo/demo-private/run.json",
      "data/runs/demo/demo-private/anchor.json",
      "data/runs/demo/demo-private/test.json",
      "data/runs/demo/demo-control/report.json",
      "data/runs/ci-checkout/test.json",
      "data/runs/ci-checkout/report.json",
      "data/demo-payer/invoices/8301.json",
      "scratch/ci-private.log"
    ],
    "invariant": "recorded run-dir contents vs what the publish job copies (does any un-copied file carry material that would be excess if it were published)",
    "method": "source",
    "result": "run.json carries receipts with signer+attestation (merchant wallet ak), merchant-side headers, invoice ids and observed values, and gate.log carries the payer/merchant AccountUuid lines (scratch/ci-private.log:21-22) - the publish job copies none of them, but the gate/payer jobs upload those files as world-downloadable artifacts; test.json carries the raw flow_definition/run_dir/report_file argument strings plus the anchor's beacon_ufvk",
    "artifact": null
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/report/src/index.rs",
      "data/runs/demo/demo-private/report.json",
      "data/runs/demo/demo-private/anchor.json"
    ],
    "invariant": "what the page actually renders when the claim input carries full evidence and an anchor record",
    "method": "local",
    "result": "rendering the built tool at commit 7058406 over a claim-input copy of the recorded private-flow report (32 evidence entries) plus its anchor.json produced a page that contains the flow id, short commit, report hash, evidence root, anchor txid+height, evidence count and finding text, and none of the 32 evidence entries' values, no anchor-record field except the txid, and no unescaped markup from a hostile flow_id/note/title (LEAKMARKER values absent)",
    "artifact": "agents/hunter-evidence/artifacts/check-r1-render.json"
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/report/src/index.rs"
    ],
    "invariant": "the index must not abort on a report whose hash-like field is not pure ASCII at byte 12",
    "method": "local",
    "result": "a report whose flow_commit is 11 ASCII chars followed by U+00E9 panics at crates/report/src/index.rs:138 ('end byte index 12 is not a char boundary'), exit 101, no page written - byte-index truncation in short()",
    "artifact": "agents/hunter-evidence/artifacts/check-r2-slice.txt"
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/report/tests/golden.rs",
      "crates/report/src/index.rs"
    ],
    "invariant": "the published page's bytes are pinned by tests, so an accidental widening of what the page shows fails CI",
    "method": "local",
    "result": "cargo test --offline -p zanary-report in the offline sandbox at commit 7058406: exit 0, 13 unit tests and 4 golden tests pass, including index_matches_golden_html which compares the renderer's output byte-for-byte against crates/report/fixtures/index.golden.html",
    "artifact": "agents/hunter-evidence/artifacts/check-r3-tests.txt"
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-16/scratch` (scratch-relative files only, `check-*.txt` / `check-*.json` /
`evidence.txt`, 64 KiB per file, 256 KiB total); the parent promotes into `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-16/artifacts`. You never write
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
