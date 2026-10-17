# Phase-5 record verifier — verifier-22 on `crates/report/src/main.rs:short-helper-byte-slice-on-untrusted-json-fields`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`confirmed`)

```json
{
  "verdict": "confirmed",
  "fingerprint": "crates/report/src/main.rs:short-helper-byte-slice-on-untrusted-json-fields",
  "title": "Report CLI truncates untrusted report/run JSON fields by byte index and panics (exit 101) instead of returning a verdict",
  "description": "The zanary-report CLI deserializes a report/run file supplied by the claim's publisher and passes its flow_commit and report_hash into the display helpers short()/short_tail(), which truncate with byte indices. Any value whose truncation offset falls inside a multi-byte UTF-8 character aborts the process with a char-boundary panic, before or while the checks are printed. The verifier is documented as the stranger's check on a claim it does not trust ('Nothing here may depend on the reporter's honesty', crates/report/src/lib.rs:10; the fixtures README invites editing a report and re-running), and the tool's contract is exit 0 on pass, 1 on any mismatch, 2 on error; the trigger reaches the panic with no node, no indexer and no keys.",
  "root_cause": "short() (crates/report/src/main.rs:760-765) formats `&hash[..12]` (line 762) and short_tail() (main.rs:283-288) formats `&value[..8]`/`&value[value.len() - 8..]` (line 287); the matrix renderer (matrix.rs:109 and :148, and the CLI summary at main.rs:226) slices `&column.commit[..8.min(len)]`. The inputs are Report.flow_commit / Report.report_hash / FlowRun.flow_commit deserialized from caller-supplied JSON (crates/report/src/lib.rs:33-55, crates/observation/src/lib.rs:154-157) and never validated as ASCII or hex, so the slices are not on guaranteed char boundaries. The identical helper shape is duplicated in crates/report/src/verify.rs:849-855 (used at verify.rs:673, the report-hash-mismatch branch, which a hostile report_hash always takes because the recomputed value is a 64-char hex digest), and also at crates/report/src/index.rs:136-138 (applied to report.flow_commit/report_hash at index.rs:298/314/326) and crates/report/src/receipt.rs:188-190 (applied to attestation key strings at receipt.rs:145-153).",
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
      "line": 545,
      "scope": "fn verify \u2014 banner println (statement at 542-546)",
      "description": "The untrusted report.flow_commit (and, in build, the untrusted run.flow_commit) is passed to the display helper short() at the call on line 545 while the first output line is formatted."
    },
    {
      "kind": "propagation",
      "file": "crates/report/src/verify.rs",
      "line": 673,
      "scope": "fn Verifier::run \u2014 report-hash-mismatch branch",
      "description": "The same untrusted report.report_hash reaches verify.rs's own copy of short() (verify.rs:851) exactly in the branch that exists to print the mismatch \u2014 the branch a hostile report_hash always takes, since the recomputed hash is 64 hex ASCII characters."
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
      "description": "`&hash[..12]` panics with 'end byte index 12 is not a char boundary' when the field's byte 12 falls inside a multi-byte UTF-8 character: stdout stays empty, no check line and no verdict is produced, exit code 101 (observed panic location main.rs:762:29)."
    }
  ],
  "evidence": [
    {
      "file": "crates/report/src/main.rs",
      "line": 762,
      "description": "The failing expression `&hash[..12]` in short(); used by verify (main.rs:545), build (main.rs:629) and verify-anchor (main.rs:392). Reproduced panics at 762:29 in the verify, build and verify-anchor runs."
    },
    {
      "file": "crates/report/src/main.rs",
      "line": 287,
      "description": "short_tail()'s `&value[..8]` and `&value[value.len() - 8..]`, reached from verify-anchor with an operator --root that disagrees with the report; reproduced panic at 287:28."
    },
    {
      "file": "crates/report/src/verify.rs",
      "line": 851,
      "description": "Second copy of the same helper, used at verify.rs:673 on report.report_hash in the hash-mismatch branch; reproduced panic at 851:29."
    },
    {
      "file": "crates/report/src/matrix.rs",
      "line": 109,
      "description": "`&column.commit[..8.min(column.commit.len())]`; commit comes straight from the parsed run.json at matrix.rs:69; reproduced panic at 109:27."
    },
    {
      "file": "crates/report/src/lib.rs",
      "line": 33,
      "description": "Report is a plain derive (struct at line 33; flow_commit at 36, report_hash at 54) with both fields unvalidated Strings \u2014 the untrusted source of both fields."
    },
    {
      "file": "crates/report/src/main.rs",
      "line": 49,
      "description": "main() documents the exit contract: Ok(code) -> exit(code), Err -> exit(2); a panic bypasses it with exit 101 (controls: benign verify 1, missing file 2, benign build/matrix 0; hostile inputs 101)."
    },
    {
      "file": "crates/report/src/index.rs",
      "line": 136,
      "description": "Third copy of the same helper shape (`&hash[..12]` at 138), applied to report.flow_commit and report_hash while rendering the index (index.rs:298/314/326). Source-verified, not executed here."
    },
    {
      "file": "crates/report/src/receipt.rs",
      "line": 188,
      "description": "Fourth copy (`&hex[..8]`/`&hex[hex.len() - 8..]` at 190), applied to receipt attestation key strings (receipt.rs:145-153). Source-verified, not executed here."
    }
  ],
  "conditions": [
    {
      "kind": "data_state",
      "description": "a report/run JSON field (flow_commit or report_hash) contains a multi-byte UTF-8 character straddling byte 12 for short(), byte 8 for the matrix/8-byte slices, or bytes 8 / len-8 for short_tail(); the rest of the file only has to parse (reproduced with U+00E9: 'aaaaaaaaaaa\u00e9' 13 bytes, 'abcdefg\u00e9' 9 bytes, 'bbbbbbbbbbb\u00e9' 13 bytes)"
    },
    {
      "kind": "user_interaction",
      "description": "the operator runs zanary-report verify/build/matrix/verify-anchor over a report or run directory written by someone else \u2014 the documented stranger-checks-a-claim usage (fixtures README: 'try editing one hex digit of a txid in a copy and re-running')"
    }
  ],
  "execution": {
    "attacker_perspective": "Whoever publishes the claim sets flow_commit (or report_hash) to a string with a non-ASCII character at the truncation offset. Nothing else in the report needs to be valid: the verifier, the builder and the matrix renderer all abort while formatting their first output. The verifier needs no node, no indexer and no keys for the crash, so the trigger works against an offline check.",
    "payloads": [
      "Report JSON: {\"version\":1,\"flow_id\":\"regtest-checkout\",\"flow_commit\":\"aaaaaaaaaaa\\u00e9\",\"chain\":\"regtest\",\"verified_at_height\":100,\"findings\":[],\"evidence\":[],\"merchant_is_simulated\":true,\"report_hash\":\"<64 zeros>\"} -> zanary-report verify v1_hostile_commit_report.json",
      "Report JSON, same shape, with \"report_hash\":\"bbbbbbbbbbb\\u00e9\" and an ASCII flow_commit -> zanary-report verify v2_hostile_hash_report.json",
      "FlowRun JSON: \"flow_commit\":\"aaaaaaaaaaa\\u00e9\" -> zanary-report build --run hostile_run_build.json --out out/b1_report.json",
      "FlowRun JSON in a run directory (with findings.json []): \"flow_commit\":\"abcdefg\\u00e9\" -> zanary-report matrix --runs runs2-hostile --out out2/m1.md",
      "CLI arguments only: zanary-report verify-anchor --root \"ccccccccccc\\u00e9\" --beacon-ufvk uview1invalid ; and with a parseable report beside it: zanary-report verify-anchor --report benign_report_for_anchor.json --root \"aaaaaaa\\u00e9zzzzzzzzzzzzzzzz\" --beacon-ufvk uview1invalid"
    ],
    "instructions": [
      "Use the approved disposable copy /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 (git archive HEAD with rust-toolchain.toml removed); before running, `diff -r` against a fresh git archive HEAD confirmed every production file identical \u2014 the only deltas in the copy are #[cfg(test)]-only appended modules added by other hunters (crates/probe/src/main.rs, crates/report/src/receipt.rs), which `cargo build` does not compile.",
      "Rebuild offline inside the sandbox: docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 -v /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2:/t -v /home/kiter/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim sh -c 'CARGO_BUILD_JOBS=6 cargo build --offline -p zanary-report; sh /t/verifier3-checks/run.sh' (build exit 0, Finished in 40.38s; binary sha256 2e903494733440be53a656846c954ccee551f5a740773cf6f585c2b19b534e60).",
      "Write the fixtures above into /t/verifier3-checks/ (e-acute stored as JSON \\u00e9 so the files are ASCII on disk; the --root arguments built with printf octal bytes), one benign control per command included.",
      "Capture stdout, stderr and the exit code per command under /t/verifier3-checks/out/; no node, indexer, network or wallet is involved (the panics happen before any connect)."
    ],
    "observed_result": "Independent sandbox reproduction. verify with hostile flow_commit: exit 101, stdout EMPTY, stderr \"panicked at crates/report/src/main.rs:762:29: end byte index 12 is not a char boundary; it is inside '\u00e9' (bytes 11..13 of string)\". verify with hostile report_hash: exit 101, stdout banner + chain line only, stderr \"panicked at crates/report/src/verify.rs:851:29\" (same boundary message). build: stdout \"built out/b1_report.json\", out/b1_report.json exists (4122 bytes), exit 101, stderr \"panicked at crates/report/src/main.rs:762:29\". verify-anchor --root: exit 101, stdout empty, \"panicked at crates/report/src/main.rs:762:29\"; with --report and mismatching --root: exit 101, stdout empty, \"panicked at crates/report/src/main.rs:287:28: end byte index 8 is not a char boundary\". matrix: exit 101, stdout empty, \"panicked at crates/report/src/matrix.rs:109:27\", out2/m1.md not written. Benign controls: verify exit 1 with the full check list and NOT VERIFIED; missing-file verify exit 2; build exit 0 (artifact 4133 bytes, flow line printed); matrix exit 0 with out2/m0.md rendered (654 bytes). No memory-safety, confidentiality or privilege effect observed in any run."
  },
  "remediation": {
    "strategy": "Make the display truncation character-safe and stop accepting non-hash text in hash fields. Either reject at intake (validate that flow_commit/report_hash are ASCII hex before anything else runs, exiting 2 with the existing error style) or truncate on character boundaries; add one regression case per command that feeds a multi-byte field and asserts the documented exit code. The same two-line helper shape is duplicated in the verify, index and receipt modules; the fix should replace every copy (main.rs, verify.rs, matrix.rs including the CLI summary at main.rs:226, index.rs, receipt.rs) so a later caller cannot reintroduce the byte slice.",
    "code_changes": [
      {
        "file_name": "crates/report/src/main.rs",
        "fixed_code": "fn short(hash: &str) -> String {\n    // Char-safe: never split a UTF-8 character. Validate: reject non-ASCII in hash fields.\n    let head: String = hash.chars().take(12).collect();\n    if hash.chars().count() > 12 {\n        format!(\"{head}\u2026\")\n    } else {\n        hash.to_string()\n    }\n}\n\nfn short_tail(value: &str) -> String {\n    if value.chars().count() <= 20 {\n        return value.to_string();\n    }\n    let chars: Vec<char> = value.chars().collect();\n    let head: String = chars[..8].iter().collect();\n    let end: String = chars[chars.len() - 8..].iter().collect();\n    format!(\"{head}\u2026{end}\")\n}\n"
      },
      {
        "file_name": "crates/report/src/matrix.rs",
        "fixed_code": "let short_commit: String = column.commit.chars().take(8).collect();\nout.push_str(&format!(\" {} `{}` |\", column.flow_id, short_commit));\n// same for matrix.rs:148 (and the CLI summary at main.rs:226)\n"
      },
      {
        "file_name": "crates/report/src/verify.rs",
        "fixed_code": "fn short(hash: &str) -> String {\n    let head: String = hash.chars().take(12).collect();\n    if hash.chars().count() > 12 { format!(\"{head}\u2026\") } else { hash.to_string() }\n}\n\n// plus a regression test:\n//   let report = serde_json::from_str(r#\"{\"version\":1,\"flow_id\":\"x\",\"flow_commit\":\"aaaaaaaaaaa\u00e9\",\n//     \"chain\":\"regtest\",\"verified_at_height\":1,\"findings\":[],\"evidence\":[],\n//     \"merchant_is_simulated\":true,\"report_hash\":\"0\".repeat(64)}\"#).unwrap();\n//   assert!(hash_report(&report).is_ok()); // verify_report must return a verdict, not abort\n"
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "low",
      "reason": "The trigger is a few bytes in a JSON field the claimant fully controls and no layer validates them, but this rig's own pipeline always writes ASCII hex hashes, so it takes an adversarial or hand-authored claimant rather than ordinary operation."
    },
    "impact": {
      "score": "low",
      "reason": "Observable effect is safe process termination: the checking process aborts (exit 101) and produces no verdict, replacing the documented 0/1/2 contract; build additionally exits 101 after writing its artifact, so automation sees a failure with output on disk. No memory-safety, confidentiality or privilege effect was observed, and the failure is fail-closed for a verifier."
    },
    "overall_severity": "low"
  },
  "confidence": {
    "score": "high",
    "reason": "Complete source trace from the parsed JSON field to the exact panicking expressions, re-derived line by line, then reproduced end-to-end in the approved sandbox: verify/build/matrix/verify-anchor all exit 101 with the predicted char-boundary panics at main.rs:762:29, verify.rs:851:29, main.rs:287:28 and matrix.rs:109:27, with a benign control for every command showing the documented behaviour."
  }
}
```

## Linked ledger unit

- coverage_id: `crates%2Freport%2Fsrc%2Fmain.rs%23run-and-report-intake::crates%2Freport%2Fsrc%2Flib.rs%23renderers-and-property-evaluation::published-evidence::ATTACK-CLASSES.md%23Resource%20and%20file%20handling`
- surface: `crates/report/src/main.rs#run-and-report-intake` · subsystem `published-evidence`
- boundary: `crates/report/src/lib.rs#renderers-and-property-evaluation`
- assignment owner: `hunter-intake`

The unit's checks (each `artifact` path is relative to the run directory; read the promoted files
under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/`):

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

## What to check for this record

1. Every repository-relative trace/evidence path, line, scope and described operation — re-read them in the target at `7058406`.
2. Real entry interface and the exact local input shape the record claims.
3. Every condition, parser/policy step, source-visible preventing layer and the observed local result; reproduce the minimum observed result yourself under the sandbox when you can.
4. Affected principal/resource and the demonstrated impact (nothing stronger).
5. Severity separation: realistic likelihood, demonstrated impact, overall no greater than impact.
6. Remediation strategy and any `code_changes`: does the proposed fix enforce the invariant at the last trusted decision point without merely moving trust?

## Boundaries

Target-controlled code may run only inside the approved sandbox. The approved disposable copy is
`/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` (`git archive HEAD`, `rust-toolchain.toml` removed so the image's own toolchain is used).
Rebuild it if needed: `rm -rf /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && mkdir -p /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && cd /home/kiter/hackathon/zanary && git archive HEAD | tar -x -C /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && rm -f /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2/rust-toolchain.toml`

```bash
docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 \
  -v /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2:/t -v /home/kiter/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim \
  sh -c 'CARGO_BUILD_JOBS=6 <command>'
```

No external network, no image pulls, no `sudo`, no host paths besides that copy and the read-only
registry; writes only inside that copy and your own scratch. Out of bounds: the regtest rig's
containers and host ports (18232/18233/9067/9068), any network service, any public URL, and anything
that mutates state outside your scratch. If a decisive fact lives behind one of those, say so in your
`replace` reason rather than reaching past the boundary.

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-22/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-22/artifacts/`. You never write retained artifacts, another agent's directory, or a
shared run file.

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

## Your result

Return exactly one JSON object, no prose and no code fences:

- `{"decision": "verified", "fingerprint": "crates/report/src/main.rs:short-helper-byte-slice-on-untrusted-json-fields"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
