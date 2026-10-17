# Candidate verifier prompt — verifier-10 on `zanary/composite-action-input-shell-expansion`

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

Hunter: `hunter-ci` · proposed verdict: `confirmed` · fingerprint: `zanary/composite-action-input-shell-expansion`

```json
{
  "proposed_verdict": "confirmed",
  "fingerprint": "zanary/composite-action-input-shell-expansion",
  "title": "Composite action expands caller-supplied with: inputs into shell script text",
  "description": "action.yml is published as a composite action that strangers run (uses: memeshee/zanary@main). Its five inputs (flow, expect, anchor, ttl-days, out) are free-form strings, and the action interpolates them directly into the script text of its run: steps as ${{ inputs.* }} expressions (action.yml:95, 100, 103, 106, 116, 117, 140, 177-178) instead of receiving them through env: or argv. GitHub substitutes those expressions into the script before bash parses it, so a value containing shell syntax (a quote plus ;, $( ), backticks, #) is executed by the calling job shell. The value also leaves the gate step as a step output -- the raw, newline-unescaped out= write into GITHUB_OUTPUT (action.yml:107) -- and re-enters script text at action.yml:136 and :153 (${{ steps.gate.outputs.out }}) and at :175 (${{ steps.gate.outputs.exit_code }}, the output that decides the build verdict). The documented consumer setup passes a literal path (docs/CI.md:49-55) and this repository's own gate does the same (privacy-gate.yml:87-93), so nothing fires in this repository; the defect is in the action as a library: it hands every consumer a code-execution sink for any input value the consumer templates.",
  "root_cause": "The action treats caller-supplied input values as script fragments rather than data: ${{ inputs.* }} values are placed in run: script text at eight sites, values derived from them are written into GITHUB_OUTPUT without key/newline escaping (action.yml:107) and re-interpolated into later steps (action.yml:136, 153, 175). No value is passed through env:/argv -- the only form that keeps it data -- and nothing validates or rejects shell syntax, so double quotes are the sole barrier and $( ) / backticks defeat them.",
  "intended_behavior": "with: inputs are data, not code: each step should take them through env: (or argv) so a value cannot change which commands run; out should be written to GITHUB_OUTPUT with the multiline-safe key<<DELIM form; and steps.gate.outputs.*, which are derived from the inputs, must be passed the same way instead of being re-inserted into script text.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "action.yml",
      "line": 23,
      "scope": "composite action inputs",
      "description": "The action accepts five caller-supplied strings (flow, expect, anchor, ttl-days, out; lines 23-46) with no format validation; a third-party workflow supplies them in its own repository via with:."
    },
    {
      "kind": "propagation",
      "file": "action.yml",
      "line": 95,
      "scope": "step run the gate - run: script text",
      "description": "The runner substitutes ${{ inputs.flow }} into the script text (a double-quoted assignment) before bash parses it; the identical pattern sits at lines 100, 103, 106, 116, 117 and in the job summary / verdict steps (140, 177-178)."
    },
    {
      "kind": "propagation",
      "file": "action.yml",
      "line": 107,
      "scope": "step run the gate - step output write",
      "description": "The out value is appended raw to GITHUB_OUTPUT (no key/newline escaping), so it also leaves the step as a step output and re-enters later scripts at 136, 153 and 175."
    },
    {
      "kind": "sink",
      "file": "action.yml",
      "line": 103,
      "scope": "step run the gate - command line",
      "description": "OUT is assigned from ${{ github.workspace }} and ${{ inputs.out }}, so the expansion lands in command context: a value with $( ), backticks, a closing quote plus ;, or # is executed by the caller shell (observed locally on this line and on line 95 with marker payloads)."
    }
  ],
  "evidence": [
    {
      "file": "action.yml",
      "line": 95,
      "description": "FLOW is assigned from ${{ inputs.flow }} inside the run the gate step - flow is expanded into script text."
    },
    {
      "file": "action.yml",
      "line": 100,
      "description": "second expansion of flow in the same step (the no-flow-definition error path) - the payload ran here too (observed twice on stdout)."
    },
    {
      "file": "action.yml",
      "line": 103,
      "description": "OUT is assigned from ${{ github.workspace }} and ${{ inputs.out }} - out expanded into a command line (the local check run B)."
    },
    {
      "file": "action.yml",
      "line": 106,
      "description": "the anchor input is expanded into script text in the [ anchor = true ] test."
    },
    {
      "file": "action.yml",
      "line": 116,
      "description": "expect is expanded inside the probe command line (--expect argument)."
    },
    {
      "file": "action.yml",
      "line": 117,
      "description": "ttl-days is expanded the same way (--ttl-days argument)."
    },
    {
      "file": "action.yml",
      "line": 107,
      "description": "the echo of out=$OUT into $GITHUB_OUTPUT - raw write, no newline escaping; the value leaves the step as a step output."
    },
    {
      "file": "action.yml",
      "line": 136,
      "description": "OUT is assigned from ${{ steps.gate.outputs.out }} in the always-run job summary step - the stored value re-enters script text (same at 153 in rig diagnostics)."
    },
    {
      "file": "action.yml",
      "line": 175,
      "description": "code is assigned from ${{ steps.gate.outputs.exit_code }} in the verdict step - the output that decides the build result is also script text."
    },
    {
      "file": "action.yml",
      "line": 140,
      "description": "the title argument of the job summary step expands inputs.flow and inputs.expect; that step runs with if: always()."
    },
    {
      "file": "action.yml",
      "line": 177,
      "description": "inputs.expect is expanded inside the verdict step case branches (also line 178)."
    },
    {
      "file": "docs/CI.md",
      "line": 51,
      "description": "the documented consumer setup passes literal values (flow: flows/my-checkout.json), which is why no in-repo caller triggers the injection."
    }
  ],
  "conditions": [
    {
      "kind": "third_party_dependency",
      "description": "A calling workflow passes a non-constant value (reusable-workflow input, matrix value, or event-derived string) to one of flow, expect, anchor, ttl-days, out. The documented setup (docs/CI.md:49-55) and this repository's own gate (privacy-gate.yml:87-93) pass literals, which do not carry shell syntax."
    }
  ],
  "execution": {
    "attacker_perspective": "Whoever controls a value that a consumer workflow passes into the action inputs -- a contributor-influenced branch/PR-derived string, a reusable-workflow parameter, or the consumer own templated value -- gets the calling job shell: the payload runs in the action steps with that job environment (GITHUB_* variables, the step outputs, any job/env secrets the step can see). A consumer that passes a literal path cannot be attacked.",
    "payloads": [
      "x\"; printf 'ZANARY-INPUT-INJECTION: flow value executed as shell code\\n'; printf 'injected-by-flow\\n' > /t/ci-check/marker-flow.txt; #",
      "z\"; printf 'ZANARY-INPUT-INJECTION: out value executed as shell code\\n'; printf 'injected-by-out\\n' > /t/ci-check/marker-out.txt; #"
    ],
    "instructions": [
      "Read action.yml from the target (read-only) and take the verbatim run-block of the run the gate step (lines 92-129), asserting the exact content of every interpolation site (95, 96, 100, 103, 106, 107, 116, 117) and of the later sites (136, 140, 153, 175, 177).",
      "Render the block the way the Actions runner does -- literal replacement of the ${{ inputs.* }} expressions plus ${{ github.workspace }} and ${{ github.action_path }} -- with a payload in one input and benign literals elsewhere (run A: flow; run B: out).",
      "Run each rendered script under bash inside docker run --rm --network=none --user 1000:1000 -v <disposable copy>:/t -w /t/ci-check/action rust:1-slim, with GITHUB_OUTPUT / GITHUB_WORKSPACE / GITHUB_ACTION_PATH set and a placeholder ./target/release/zanary-probe (the real rig is out of bounds), clearing previous markers first.",
      "Observe the marker files, stdout and the step GITHUB_OUTPUT file; stop at the first observed boundary result (no network, no rig, no exploration)."
    ],
    "observed_result": "Both rendered runs executed the injected command: marker-flow.txt and marker-out.txt were created inside the sandbox copy and the ZANARY-INPUT-INJECTION lines appeared on stdout. Run A (payload in flow) went through the step normal error path and exited 2 with the marker still written; run B (payload in out) exited 0 and the step wrote its normal outputs to GITHUB_OUTPUT: out=/t/ci-check/ws/z and exit_code=0. Nothing outside the disposable copy was written; the check used --network=none, a placeholder probe and no rig."
  },
  "remediation": {
    "strategy": "Keep inputs as data on both sides of every step: declare them under env: and reference the environment variables inside the scripts; validate the shapes the action actually needs (flow/out caller-workspace-relative paths; expect in {pass, fail}; anchor in {true, false}; ttl-days a positive integer) and fail early with a clear error; write out to GITHUB_OUTPUT with the multiline-safe key<<DELIM form; and in the job summary, rig diagnostics and verdict steps pass steps.gate.outputs.out / steps.gate.outputs.exit_code through env: instead of re-interpolating them.",
    "code_changes": [
      {
        "file_name": "action.yml",
        "fixed_code": "    - name: run the gate\n      id: gate\n      shell: bash\n      working-directory: ${{ github.action_path }}\n      env:                                    # every input arrives as data\n        INPUT_FLOW: ${{ inputs.flow }}\n        INPUT_OUT: ${{ inputs.out }}\n        INPUT_EXPECT: ${{ inputs.expect }}\n        INPUT_ANCHOR: ${{ inputs.anchor }}\n        INPUT_TTL_DAYS: ${{ inputs.ttl-days }}\n      run: |\n        case \"$INPUT_EXPECT\" in pass|fail) ;; *) echo \"::error::expect must be pass or fail\"; exit 2 ;; esac\n        anchor_flag=\"\"; [ \"$INPUT_ANCHOR\" = \"true\" ] && anchor_flag=\"--anchor\"\n        OUT=\"$GITHUB_WORKSPACE/$INPUT_OUT\"; mkdir -p \"$OUT\"\n        { echo \"out<<ZANARY_OUT\"; printf '%s\\n' \"$OUT\"; echo \"ZANARY_OUT\"; } >> \"$GITHUB_OUTPUT\"\n        timeout \"${ZANARY_GATE_TIMEOUT_SECONDS:-2700}\" ./target/release/zanary-probe test \\\n          --definition \"$INPUT_FLOW\" --out \"$OUT\" --expect \"$INPUT_EXPECT\" \\\n          --ttl-days \"$INPUT_TTL_DAYS\" $anchor_flag --summary \"$OUT/test.json\" 2>&1 | tee \"$OUT/gate.log\"\n        code=${PIPESTATUS[0]}; [ \"$code\" = \"124\" ] && code=2\n        echo \"exit_code=$code\" >> \"$GITHUB_OUTPUT\"\n        exit 0\n    # later steps (job summary, rig diagnostics, verdict) take GATE_OUT / GATE_EXIT through env:,\n    # never through ${{ }} inside the script."
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "low",
      "reason": "No in-repo caller passes a non-constant value into the inputs (privacy-gate.yml uses literals; the documented consumer setup does too), so exploitation requires a consumer that templates one of the five inputs -- a caller-side pattern the defect makes dangerous but cannot guarantee."
    },
    "impact": {
      "score": "high",
      "reason": "When it fires, arbitrary commands run in the consumer CI job with that job environment: the action own steps, its step outputs (including the exit_code the verdict step trusts) and any job-level env or token the step can see."
    },
    "overall_severity": "medium"
  },
  "confidence": {
    "score": "high",
    "reason": "Every interpolation site and the output round-trip are in source (line-verified this run), execution was observed in the offline sandbox on the rendered script, and the substitution semantics is the documented Actions behavior the same file already depends on elsewhere (${{ github.action_path }} in working-directory: and in the script own path search)."
  }
}
```

## Its linked coverage unit and the hunter's checks

- coverage_id: `action.yml%23inputs-and-steps::.github%2Fworkflows%2Fprivacy-gate.yml%23jobs::ci-and-release::ATTACK-CLASSES.md%23Injection`
- surface: `action.yml#inputs-and-steps` (subsystem `ci-and-release`)
- boundary: `.github/workflows/privacy-gate.yml#jobs`
- attack class: `ATTACK-CLASSES.md#Injection` · assignment owner: `hunter-ci`
- unit reviewed paths: [".github/workflows/privacy-gate.yml", "action.yml", "harness/up.sh", "scripts/bounded-payer-demo.sh", "scripts/ci-summary.py", "scripts/demo-run.sh"]

The unit's checks, including the local checks whose promoted artifacts you may read
(paths are relative to the run directory `/home/kiter/security-audit-skill/zanary/run-1`):

```json
[
  {
    "agent_id": "hunter-ci",
    "reviewed_paths": [
      "action.yml"
    ],
    "invariant": "composite-action inputs must stay data: no value from `${{ inputs.* }}` may be placed in the script text of a run: step (pass through env:/argv instead)",
    "method": "source",
    "result": "violated at action.yml:95, 100, 103, 106, 116, 117 (step run the gate), :140 (step job summary) and :177-178 (step verdict); the same class re-enters through `${{ steps.gate.outputs.out }}` at :136 and :153 and `${{ steps.gate.outputs.exit_code }}` at :175 because :107 stores the input-derived value raw in `$GITHUB_OUTPUT`",
    "artifact": null
  },
  {
    "agent_id": "hunter-ci",
    "reviewed_paths": [
      "action.yml"
    ],
    "invariant": "an input value containing shell syntax must not execute in the action steps",
    "method": "local",
    "result": "violated: the run the gate run-block (lines 92-129) was rendered with literal expression substitution (as the runner does) and executed under bash in `docker run --rm --network=none`; payloads in inputs.flow and inputs.out both executed (marker files created), and run B still finished with the step normal outputs `out=...` and `exit_code=0`",
    "artifact": "agents/hunter-ci/artifacts/check-injection-rce.txt"
  },
  {
    "agent_id": "hunter-ci",
    "reviewed_paths": [
      ".github/workflows/privacy-gate.yml"
    ],
    "invariant": "no event- or contributor-controlled value may reach shell script text or command arguments in the workflow jobs",
    "method": "source",
    "result": "covered, no sink: all run: scripts are built from static matrix values and constants; github.ref appears only in concurrency.group and in an if: predicate, never in script text; artifact names and run directories derive from static matrix ids; the sed-derived rust-toolchain channel is passed quoted to rustup",
    "artifact": null
  },
  {
    "agent_id": "hunter-ci",
    "reviewed_paths": [
      "scripts/demo-run.sh",
      "scripts/bounded-payer-demo.sh",
      "harness/up.sh",
      "scripts/ci-summary.py"
    ],
    "invariant": "caller-tunable env values and file contents are consumed as data (quoted expansions, no eval, no re-entry into shell text)",
    "method": "source",
    "result": "covered, no sink: `set -euo pipefail` in the shell scripts, every ZANARY_* caller value used inside quotes and confined to paths/timeouts; quoted python heredocs; ci-summary.py prints gate JSON fields as argv/Markdown into the caller own job summary only",
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-10/scratch` (scratch-relative files only, `check-*.txt` / `check-*.json` /
`evidence.txt`, 64 KiB per file, 256 KiB total); the parent promotes into `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-10/artifacts`. You never write
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
