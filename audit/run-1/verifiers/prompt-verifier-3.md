# Candidate verifier prompt — verifier-3 on `crates/report/src/main.rs:short-helper-byte-slice-on-untrusted-json-fields`

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

Hunter: `hunter-intake` · proposed verdict: `confirmed` · fingerprint: `crates/report/src/main.rs:short-helper-byte-slice-on-untrusted-json-fields`

```json
{
  "proposed_verdict": "confirmed",
  "fingerprint": "crates/report/src/main.rs:short-helper-byte-slice-on-untrusted-json-fields",
  "title": "Report CLI truncates untrusted report/run JSON fields by byte index and panics (exit 101) instead of returning a verdict",
  "description": "The zanary-report CLI deserializes a report/run file supplied by the claim's publisher and passes its flow_commit and report_hash into the display helpers short()/short_tail(), which truncate with byte indices. Any value whose truncation offset falls inside a multi-byte UTF-8 character aborts the process with a char-boundary panic, before or while the checks are printed. The verifier is documented as the stranger's check on a claim it does not trust ('nothing here may depend on the reporter's honesty'; the fixtures README invites editing a report and re-running), and the tool's contract is exit 0 on pass, 1 on any mismatch, 2 on error; the trigger reaches the panic with no node, no indexer and no keys.",
  "root_cause": "short() (crates/report/src/main.rs:760-765) formats `&hash[..12]` and short_tail() (main.rs:283-288) formats `&value[..8]`/`&value[value.len() - 8..]`; the matrix renderer (matrix.rs:109 and :148, and the CLI summary at main.rs:226) slices `&column.commit[..8.min(len)]`. The inputs are Report.flow_commit / Report.report_hash / FlowRun.flow_commit deserialized from caller-supplied JSON (crates/report/src/lib.rs:33-55, crates/observation/src/lib.rs:154-157) and never validated as ASCII or hex, so the slices are not on guaranteed char boundaries. The identical helper shape is duplicated in crates/report/src/verify.rs:849-855 (used at verify.rs:673, the report-hash-mismatch branch).",
  "intended_behavior": "Fields that stand in for hashes should be validated at intake (or the truncation made character-safe), and every command should end in its documented verdict: verify exits 0 when every check passes, 1 on any mismatch, 2 on error; matrix either renders or fails with a diagnostic. A report the verifier does not trust must be handled as data, never as something that can terminate the process.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/report/src/main.rs",
      "line": 517,
      "scope": "fn verify(args: &[String])",
      "description": "`zanary-report verify <report.json>` reads and deserializes the caller-supplied report (main.rs:521-523) with no validation of flow_commit or report_hash."
    },
    {
      "kind": "propagation",
      "file": "crates/report/src/main.rs",
      "line": 542,
      "scope": "fn verify \u2014 banner println",
      "description": "The untrusted report.flow_commit (and, in build, the untrusted run.flow_commit) is passed to the display helper short() while formatting the first output line."
    },
    {
      "kind": "propagation",
      "file": "crates/report/src/verify.rs",
      "line": 673,
      "scope": "fn Verifier::run \u2014 report-hash-mismatch branch",
      "description": "The same untrusted report.report_hash reaches verify.rs's own copy of short() (verify.rs:851) exactly when the report has been altered \u2014 the branch that exists to print the mismatch."
    },
    {
      "kind": "propagation",
      "file": "crates/report/src/matrix.rs",
      "line": 109,
      "scope": "fn render_markdown",
      "description": "matrix --runs <dir> copies run.flow_commit into RunColumn.commit (matrix.rs:69) and byte-slices it at [..8] (also matrix.rs:148, main.rs:226), aborting before the LEAK-MATRIX artifact is written."
    },
    {
      "kind": "sink",
      "file": "crates/report/src/main.rs",
      "line": 762,
      "scope": "fn short",
      "description": "`&hash[..12]` panics with 'end byte index 12 is not a char boundary' when the field's byte 12 falls inside a multi-byte UTF-8 character: stdout stays empty, no check line and no verdict is produced, exit code 101."
    }
  ],
  "evidence": [
    {
      "file": "crates/report/src/main.rs",
      "line": 762,
      "description": "The failing expression `&hash[..12]` in short(); used by verify (main.rs:545), build (main.rs:629) and verify-anchor (main.rs:392)."
    },
    {
      "file": "crates/report/src/main.rs",
      "line": 287,
      "description": "short_tail()'s `&value[..8]` and `&value[value.len() - 8..]`, reached from verify-anchor with an operator --root that disagrees with the report."
    },
    {
      "file": "crates/report/src/verify.rs",
      "line": 851,
      "description": "Second copy of the same helper, used at verify.rs:673 on report.report_hash in the hash-mismatch branch."
    },
    {
      "file": "crates/report/src/matrix.rs",
      "line": 109,
      "description": "`&column.commit[..8.min(column.commit.len())]`; commit comes straight from the parsed run.json at matrix.rs:69."
    },
    {
      "file": "crates/report/src/lib.rs",
      "line": 33,
      "description": "Report is a plain derive with flow_commit/report_hash as unvalidated Strings \u2014 the untrusted source of both fields."
    },
    {
      "file": "crates/report/src/main.rs",
      "line": 49,
      "description": "main() documents the exit contract: Ok(code) -> exit(code), Err -> exit(2); a panic bypasses it with exit 101."
    }
  ],
  "conditions": [
    {
      "kind": "data_state",
      "description": "a report/run JSON field (flow_commit or report_hash) contains a multi-byte UTF-8 character straddling byte 12 for short(), byte 8 for the matrix/8-byte slices, or bytes 8 / len-8 for short_tail(); the rest of the file only has to parse"
    },
    {
      "kind": "user_interaction",
      "description": "the operator runs zanary-report verify/build/matrix/verify-anchor over a report or run directory written by someone else \u2014 the documented stranger-checks-a-claim usage (fixtures README: 'try editing one hex digit of a txid in a copy and re-running')"
    }
  ],
  "execution": {
    "attacker_perspective": "Whoever publishes the claim sets flow_commit (or report_hash) to a string with a non-ASCII character at the truncation offset. Nothing else in the report needs to be valid: the verifier, the builder and the matrix renderer all abort while formatting their first output. The verifier needs no node, no indexer and no keys for the crash, so the trigger works against an offline check.",
    "payloads": [
      "Report JSON: {\"version\":1,\"flow_id\":\"regtest-checkout\",\"flow_commit\":\"aaaaaaaaaaa\u00e9\",\"chain\":\"regtest\",\"verified_at_height\":100,\"findings\":[],\"evidence\":[],\"merchant_is_simulated\":true,\"report_hash\":\"<64 zeros>\"} -> zanary-report verify v1_hostile_commit_report.json",
      "Report JSON, same shape, with \"report_hash\":\"bbbbbbbbbbb\u00e9\" and an ASCII flow_commit -> zanary-report verify v2_hostile_hash_report.json",
      "FlowRun JSON in a run directory (with findings.json []): \"flow_commit\":\"abcdefg\u00e9\" -> zanary-report matrix --runs runs2-hostile --out out2/m1.md",
      "FlowRun JSON: \"flow_commit\":\"aaaaaaaaaaa\u00e9\" -> zanary-report build --run hostile_run_build.json --out out/b1_report.json",
      "CLI arguments only: zanary-report verify-anchor --root \"ccccccccccc\u00e9\" --beacon-ufvk uview1invalid ; and with a parseable report beside it: zanary-report verify-anchor --report benign_report_for_anchor.json --root \"aaaaaaa\u00e9zzzzzzzzzzzzzzzz\" --beacon-ufvk uview1invalid"
    ],
    "instructions": [
      "Use the approved disposable copy /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 (git archive HEAD with rust-toolchain.toml removed; crates/report/src and crates/properties/src diff-verified identical to the reviewed revision before running).",
      "Write the minimal fixtures above into /t/hunter-intake-checks/ inside that copy (a valid report/run with only the named field hostile, plus benign controls).",
      "Run inside the sandbox only: docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 -v /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2:/t -v /home/kiter/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim sh -c 'sh /t/hunter-intake-checks/run_checks.sh' (and run_checks2.sh for matrix); /t/target/debug/zanary-report is built offline from the reviewed revision.",
      "Capture stdout, stderr and the exit code per command; no node, indexer, network or wallet is involved (the panics happen before any connect)."
    ],
    "observed_result": "verify with the hostile flow_commit: exit 101, stdout EMPTY, stderr `thread 'main' (16) panicked at crates/report/src/main.rs:762:29: end byte index 12 is not a char boundary; it is inside '\u00e9' (bytes 11..13 of string)`. verify with the hostile report_hash: exit 101, stdout banner + chain line only, stderr `panicked at crates/report/src/verify.rs:851:29` (same boundary message). build: stdout `built out/b1_report.json`, out/b1_report.json exists (4122 bytes), exit 101, stderr `panicked at crates/report/src/main.rs:762:29`. verify-anchor --root: exit 101, `panicked at crates/report/src/main.rs:762:29`; with --report and mismatching --root: exit 101, `panicked at crates/report/src/main.rs:287:28: end byte index 8 is not a char boundary`. matrix: exit 101, `panicked at crates/report/src/matrix.rs:109:27`, out2/m1.md not written. Benign controls: verify exit 1 with the full check list and `NOT VERIFIED`; matrix exit 0 with the rendered table; no memory-safety, confidentiality or privilege effect observed in any run."
  },
  "remediation": {
    "strategy": "Make the display truncation character-safe and stop accepting non-hash text in hash fields. Either reject at intake (validate that flow_commit/report_hash are ASCII hex before anything else runs, exiting 2 with the existing error style) or truncate on character boundaries; add one regression case per command that feeds a multi-byte field and asserts the documented exit code. The same two-line helper shape is duplicated in the index and receipt modules; the fix should replace every copy so a later caller cannot reintroduce the byte slice.",
    "code_changes": [
      {
        "file_name": "crates/report/src/main.rs",
        "fixed_code": "fn short(hash: &str) -> String {\n    // Char-safe: never split a UTF-8 character. Validate: reject non-ASCII in hash fields.\n    let head: String = hash.chars().take(12).collect();\n    if hash.chars().count() > 12 {\n        format!(\"{head}\u2026\")\n    } else {\n        hash.to_string()\n    }\n}\n\nfn short_tail(value: &str) -> String {\n    if value.chars().count() <= 20 {\n        return value.to_string();\n    }\n    let tail: Vec<char> = value.chars().collect();\n    let head: String = tail[..8].iter().collect();\n    let end: String = tail[tail.len() - 8..].iter().collect();\n    format!(\"{head}\u2026{end}\")\n}\n"
      },
      {
        "file_name": "crates/report/src/matrix.rs",
        "fixed_code": "let short_commit: String = column.commit.chars().take(8).collect();\nout.push_str(&format!(\" {} `{}` |\", column.flow_id, short_commit));\n// same for matrix.rs:148 (and the CLI summary at main.rs:226)"
      },
      {
        "file_name": "crates/report/src/verify.rs",
        "fixed_code": "fn short(hash: &str) -> String {\n    let head: String = hash.chars().take(12).collect();\n    if hash.chars().count() > 12 { format!(\"{head}\u2026\") } else { hash.to_string() }\n}\n\n// plus a regression test:\n//   let report = serde_json::from_str(r#\"{\"version\":1,\"flow_id\":\"x\",\"flow_commit\":\"aaaaaaaaaaa\u00e9\",\n//     \"chain\":\"regtest\",\"verified_at_height\":1,\"findings\":[],\"evidence\":[],\n//     \"merchant_is_simulated\":true,\"report_hash\":\"0\".repeat(64)}\"#).unwrap();\n//   assert!(hash_report(&report).is_ok()); // verify_report must return a verdict, not abort"
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "low",
      "reason": "The trigger is 5 bytes in a JSON field the attacker fully controls and no layer validates it, but this rig's own pipeline always writes ASCII hex hashes, so it takes an adversarial or hand-authored claimant rather than ordinary operation."
    },
    "impact": {
      "score": "low",
      "reason": "Observable effect is safe process termination: the checking process aborts (exit 101) and produces no verdict, replacing the documented 0/1/2 contract; build additionally exits 101 after writing its artifact, so automation sees a failure with output on disk. No memory-safety, confidentiality or privilege effect was observed, and the failure is fail-closed for a verifier."
    },
    "overall_severity": "low"
  },
  "confidence": {
    "score": "high",
    "reason": "Complete source trace from the parsed JSON field to the exact panicking expression, reproduced by running the binary built from the reviewed revision inside the approved sandbox, with a benign control for every command showing the documented behaviour."
  }
}
```

## Its linked coverage unit and the hunter's checks

- coverage_id: `crates%2Freport%2Fsrc%2Fmain.rs%23run-and-report-intake::crates%2Freport%2Fsrc%2Flib.rs%23renderers-and-property-evaluation::published-evidence::ATTACK-CLASSES.md%23Resource%20and%20file%20handling`
- surface: `crates/report/src/main.rs#run-and-report-intake` (subsystem `published-evidence`)
- boundary: `crates/report/src/lib.rs#renderers-and-property-evaluation`
- attack class: `ATTACK-CLASSES.md#Resource and file handling` · assignment owner: `hunter-intake`
- unit reviewed paths: ["crates/observation/src/lib.rs", "crates/properties/src/lib.rs", "crates/properties/src/properties/amount_fingerprint.rs", "crates/properties/src/properties/common.rs", "crates/properties/src/properties/memo_absent.rs", "crates/properties/src/properties/memo_content.rs", "crates/properties/src/properties/payer_linkability.rs", "crates/properties/src/properties/policy_downgrade.rs", "crates/properties/src/properties/receiver_reuse.rs", "crates/properties/src/properties/report_leak.rs", "crates/properties/src/properties/same_block_roundtrip.rs", "crates/properties/src/properties/server_metadata.rs", "crates/properties/src/properties/timing_correlation.rs", "crates/properties/src/properties/transparent_fallback.rs", "crates/properties/src/properties/unshielding.rs", "crates/report/src/anchor.rs", "crates/report/src/lib.rs", "crates/report/src/main.rs", "crates/report/src/matrix.rs", "crates/report/src/verify.rs"]

The unit's checks, including the local checks whose promoted artifacts you may read
(paths are relative to the run directory `/home/kiter/security-audit-skill/zanary/run-1`):

```json
[
  {
    "agent_id": "hunter-intake",
    "reviewed_paths": [
      "crates/report/src/main.rs",
      "crates/report/src/verify.rs",
      "crates/report/src/lib.rs",
      "crates/observation/src/lib.rs"
    ],
    "invariant": "The display helpers short()/short_tail() truncate report/run JSON fields by byte index; a field carrying a multi-byte UTF-8 character at the truncation offset must not abort the process (documented exit contract: 0 pass / 1 mismatch / 2 error).",
    "method": "local",
    "result": "Ran the binary built from the reviewed revision in the sandbox. verify on a report whose flow_commit is 'aaaaaaaaaaa\u00e9' (byte 12 inside the character): exit 101, stdout empty, panic at crates/report/src/main.rs:762:29 'end byte index 12 is not a char boundary'. Same shape in report_hash: panic at crates/report/src/verify.rs:851:29, i.e. inside the report-hash-mismatch branch that exists to report an altered claim. build --run <hostile run.json>: wrote its --out report.json, then panicked at main.rs:762:29, exit 101. verify-anchor --root <13-byte multibyte>: panicked offline at main.rs:762:29; with --report and a mismatching 25-byte --root: panic at main.rs:287:28 (short_tail). Benign controls: verify exited 1 with a full check list, verify-anchor/parse paths clean.",
    "artifact": "agents/hunter-intake/artifacts/check-intake-panics.txt"
  },
  {
    "agent_id": "hunter-intake",
    "reviewed_paths": [
      "crates/report/src/matrix.rs",
      "crates/report/src/main.rs",
      "crates/observation/src/lib.rs"
    ],
    "invariant": "matrix --runs <dir> must render every run directory it accepts or fail with a diagnostic; it must not abort while truncating a run.json field.",
    "method": "local",
    "result": "run directory whose run.json has flow_commit 'abcdefg\u00e9' (byte 8 inside the character): exit 101, panic at crates/report/src/matrix.rs:109:27, out file not written; benign run directory exits 0 and renders (out2/m0.md). Same expression also at matrix.rs:148 and main.rs:226.",
    "artifact": "agents/hunter-intake/artifacts/check-matrix-panic.txt"
  },
  {
    "agent_id": "hunter-intake",
    "reviewed_paths": [
      "crates/report/src/main.rs"
    ],
    "invariant": "Every filesystem path the report CLI writes or reads comes from the operator's own arguments or env; no path component is derived from parsed JSON (no traversal, no zip slip); the index walk stays one level under --reports.",
    "method": "source",
    "result": "Confirmed: verify/build/matrix/index/verify-anchor/verify-receipt open only --run/--findings/--report/--reports/--root/--out paths plus the anchor sidecar built from the directory entry (main.rs:711-714); no crate in the intake handles archives; the index walk follows symlinks only inside the directory the operator named (same principal).",
    "artifact": null
  },
  {
    "agent_id": "hunter-intake",
    "reviewed_paths": [
      "crates/report/src/main.rs",
      "crates/report/src/verify.rs",
      "crates/report/src/lib.rs"
    ],
    "invariant": "Caller-supplied numbers in the intake are parsed with checks and are never used as lengths, indices or offsets.",
    "method": "source",
    "result": "Confirmed: --ttl-days uses checked_mul(86_400) with an explicit overflow error (main.rs:609-617); --verified-at is RFC3339-validated in build_with_claim (lib.rs:130-134); --since/--now parse with context; batch_size as usize (verify.rs:334) is a widening cast compared against an Orchard action count; indexer height as u32 (verify.rs:282) only feeds the node-vs-indexer agreement line (the node stays authoritative). The only byte-index arithmetic over data is the display truncation, carried as the candidate.",
    "artifact": null
  },
  {
    "agent_id": "hunter-intake",
    "reviewed_paths": [
      "crates/report/src/main.rs",
      "crates/report/src/lib.rs",
      "crates/report/src/anchor.rs"
    ],
    "invariant": "Untrusted JSON deserialization is derive-only (no untagged/flatten/custom Deserialize, no data-driven recursion) and malformed input produces a clean error or an explicit skip, never partial state.",
    "method": "source",
    "result": "Confirmed: Report/Evidence/Finding/FlowRun/Receipt/AnchorRecord are plain serde derives; index skips unparseable files with a stderr warning (main.rs:698-701); verify/build/verify-receipt exit 2 with a context message; Anchor::from_memo_text refuses unknown fields and unknown payload versions (anchor.rs:82-109). No unsafe-deserialization surface reachable from this intake.",
    "artifact": null
  },
  {
    "agent_id": "hunter-intake",
    "reviewed_paths": [
      "crates/properties/src/lib.rs",
      "crates/properties/src/properties/amount_fingerprint.rs",
      "crates/properties/src/properties/common.rs",
      "crates/properties/src/properties/memo_absent.rs",
      "crates/properties/src/properties/memo_content.rs",
      "crates/properties/src/properties/payer_linkability.rs",
      "crates/properties/src/properties/policy_downgrade.rs",
      "crates/properties/src/properties/receiver_reuse.rs",
      "crates/properties/src/properties/report_leak.rs",
      "crates/properties/src/properties/same_block_roundtrip.rs",
      "crates/properties/src/properties/server_metadata.rs",
      "crates/properties/src/properties/timing_correlation.rs",
      "crates/properties/src/properties/transparent_fallback.rs",
      "crates/properties/src/properties/unshielding.rs",
      "crates/observation/src/lib.rs"
    ],
    "invariant": "Property evaluation reached from `build --run <untrusted run.json>` (zanary_properties::evaluate) must not panic or mis-bound on hostile heights, amounts or strings.",
    "method": "source",
    "result": "No defect found. timing-correlation's u32 subtraction is preceded by its `f.height <= h` filter and expect(\"delay came from this set\") filters on the same expression that produced the minimum (reachable-safe); same-block-roundtrip compares Options before unwrap; unshielding widens to u128 and uses saturating_sub; payer-linkability uses fixed weights, abs_diff and Dsu indices bounded by n; amount-fingerprint uses abs_diff and a len==1 guard before candidates[0]; common.rs slices only after a char-boundary find('@'). payer_policy.rs excluded (peer-owned bounded-payer unit).",
    "artifact": null
  },
  {
    "agent_id": "hunter-intake",
    "reviewed_paths": [
      "crates/report/src/main.rs",
      "crates/report/src/matrix.rs"
    ],
    "invariant": "Resource bounds on intake: file size, array lengths and output escaping for data the CLI ingests.",
    "method": "source",
    "result": "No bound exists: fs::read_to_string with no size cap, evidence/findings/merchant_events accepted unbounded, hash_report clones the whole Report, pairwise property loops are O(n^2), and matrix markdown interpolates run-supplied strings unescaped. All same-principal input (the operator names the file), so recorded as hardening rather than a finding.",
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

### [companion validation block] MEMORY-SAFETY-AND-BINARY.md#Validation rules — verbatim

## Validation rules (apply before reporting ANY finding here)

1. Establish a realistic untrusted entry and exact operation that violates a bounds, type, lifetime, ABI, concurrency, loader, or authority invariant.
2. Classify the observable effect: invalid read, invalid write, stale alias, wrong object, uninitialized output, unauthorized image load, deadlock, or safe process termination. Do not claim a stronger effect than observed.
3. Run the narrowest local harness, existing test, sanitizer, or fuzzer needed to reproduce the effect. Verify sanitizer coverage of the faulting operation and record architecture/build conditions.
4. For concurrency, use a deterministic schedule or sanitizer trace. For binary loading, prove the checked identity differs from the mapped identity and name the lower-trust writer.
5. Return `confirmed` findings only with exact input, source trace, and observed result. Return `needs_validation` for a specific unresolved reachability, ABI, build, deployment, or runtime fact and state the bounded check needed.

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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-3/scratch` (scratch-relative files only, `check-*.txt` / `check-*.json` /
`evidence.txt`, 64 KiB per file, 256 KiB total); the parent promotes into `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-3/artifacts`. You never write
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
