# Candidate verifier prompt — verifier-11 on `zanary/hex-decode-str-slice-panic`

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

Hunter: `hunter-inputs` · proposed verdict: `confirmed` · fingerprint: `zanary/hex-decode-str-slice-panic`

```json
{
  "proposed_verdict": "confirmed",
  "fingerprint": "zanary/hex-decode-str-slice-panic",
  "title": "verify-invoice panics on a txid containing a multi-byte UTF-8 character instead of rejecting it",
  "description": "flow::hex_decode checks only that the txid's byte length is even, then slices the string with `&hex[i..i + 2]`. Rust &str indices are byte offsets, so any input whose 2-byte step lands inside a multi-byte UTF-8 character aborts the process (exit 101, `byte index N is not a char boundary`) instead of returning the normal `invalid hex at byte N` error (exit 1). verify-invoice decodes --txid before it opens a wallet or touches the chain, so the crash is reachable offline with a four-byte argument. The same workspace decodes hex safely in two other places (crates/report/src/node.rs:193-203, crates/probe/src/bin/zanary-seed.rs:714-722 both step over `hex.as_bytes().chunks(2)` and map a bad chunk to an error), and the same function's other rejections are clean, so the panicking shape is an outlier rather than a deliberate fail-loudly choice.",
  "root_cause": "flow::hex_decode (crates/probe/src/flow.rs:1778-1789) byte-indexes a &str \u2014 `u8::from_str_radix(&hex[i..i + 2], 16)` at flow.rs:1785 \u2014 after validating only that the byte length is even; a multi-byte UTF-8 character straddling a 2-byte boundary makes the slice panic.",
  "intended_behavior": "A --txid that is not hex is refused with `Error: decoding the txid ... invalid hex at byte N` and exit 1, as the odd-length and non-hex-ASCII cases already are.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/main.rs",
      "line": 52,
      "scope": "main dispatch",
      "description": "`Some(\"verify-invoice\") => flow::verify_invoice(rest).await` passes the caller's raw argv to the command"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/flow.rs",
      "line": 1671,
      "scope": "verify_invoice argument loop",
      "description": "the --txid value is captured verbatim from argv, with no character-level validation"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/flow.rs",
      "line": 1687,
      "scope": "verify_invoice",
      "description": "hex_decode(&txid_hex) runs before the merchant wallet is opened, so no node or indexer is involved"
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/flow.rs",
      "line": 1785,
      "scope": "hex_decode",
      "description": "`&hex[i..i + 2]` slices at byte offsets; an offset inside a multi-byte character panics (observed: `end byte index 2 is not a char boundary; it is inside '\u00e9' (bytes 1..3 of string)`)"
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/flow.rs",
      "line": 1779,
      "description": "the only guard is `!hex.len().is_multiple_of(2)` (a byte count); after it the function walks the string with byte indices"
    },
    {
      "file": "crates/probe/src/flow.rs",
      "line": 1785,
      "description": "the panicking expression, reproduced with the sandbox-built binary: exit 101, panic at crates/probe/src/flow.rs:1785:36"
    },
    {
      "file": "crates/report/src/node.rs",
      "line": 198,
      "description": "sibling control: the report crate's hex_decode iterates `hex.as_bytes().chunks(2)` and converts each chunk with std::str::from_utf8, returning an error instead of panicking"
    },
    {
      "file": "crates/probe/src/bin/zanary-seed.rs",
      "line": 718,
      "description": "sibling control: zanary-seed's hex_decode_into uses the same byte-chunk shape"
    },
    {
      "file": "crates/probe/src/flow.rs",
      "line": 1687,
      "description": "call site: hex_decode(&txid_hex) in verify_invoice, before any wallet or network work"
    }
  ],
  "conditions": [
    {
      "kind": "user_interaction",
      "description": "the caller (an operator, or a script/agent that passes a txid it was handed) supplies a --txid value that is not pure ASCII hex, e.g. a txid pasted with a typographic quote, em dash or accented character"
    },
    {
      "kind": "data_state",
      "description": "the value's UTF-8 byte length is even and a multi-byte character starts at an odd byte offset (a\u00e9b, a\u2014b, a\u00bbb), so the even-length guard passes and a 2-byte slice boundary falls inside the character"
    }
  ],
  "execution": {
    "attacker_perspective": "Anyone who can hand a payer or an agent the text that becomes --txid (a settlement claim, a chat message, a re-encoded log line) makes `zanary-probe verify-invoice` abort with a Rust panic instead of reporting a bad txid; in a scripted check the 101 exit also falls outside the tool's documented 0/1/2 contract, turning a validation failure into an unexplained crash.",
    "payloads": [
      "verify-invoice --invoice evil-invoice.json --txid \"$(printf 'a\\303\\251b')\"   # the 4 UTF-8 bytes 61 c3 a9 62 (\"a\u00e9b\")",
      "{\"invoice_id\":\"9001\",\"amount_zat\":1,\"receiver\":\"not-a-real-address\",\"memo\":\"\",\"uri\":\"\",\"issued_at_height\":1,\"merchant_account\":0}"
    ],
    "instructions": [
      "In an OS-enforced sandbox with no network (docker run --rm --network=none, the reviewed commit's disposable `git archive HEAD` copy mounted read-write, cargo registry read-only), build the target: `cargo build --offline -p zanary-probe --bin zanary-probe` (rustc 1.99.0, dev profile, 4m03s).",
      "Write the payload JSON to invoice-fixture.json (any shape-valid Invoice; no field of it is validated on this path).",
      "Run `./target/debug/zanary-probe verify-invoice --invoice invoice-fixture.json --txid \"$(printf 'a\\303\\251b')\"` and read the exit code; no node, indexer or wallet is contacted.",
      "Controls, same command and fixture: --txid = the 2-byte \u00e9 (odd length) and --txid = 64 ASCII 'z' \u2014 both exit 1 with `invalid hex at byte 0: invalid digit found in string`."
    ],
    "observed_result": "exit 101; stderr `thread 'main' (18) panicked at crates/probe/src/flow.rs:1785:36: end byte index 2 is not a char boundary; it is inside '\u00e9' (bytes 1..3 of string)`. Both controls exited 1 with `Error: decoding the txid / Caused by: invalid hex at byte 0: invalid digit found in string`. Transcript: agents/hunter-inputs/artifacts/check-panics.txt"
  },
  "remediation": {
    "strategy": "Decode the txid through bytes instead of slicing the &str, mirroring what the same workspace already ships in crates/report/src/node.rs and crates/probe/src/bin/zanary-seed.rs, and keep the `invalid hex at byte N` error shape so callers see the same diagnostic.",
    "code_changes": [
      {
        "file_name": "crates/probe/src/flow.rs",
        "fixed_code": "/// Decodes lowercase hex into bytes, for txids that arrive as text.\nfn hex_decode(hex: &str) -> Result<Vec<u8>> {\n    if !hex.len().is_multiple_of(2) {\n        bail!(\"hex string has an odd length\");\n    }\n    let mut out = Vec::with_capacity(hex.len() / 2);\n    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {\n        let pair = std::str::from_utf8(chunk)\n            .map_err(|e| anyhow!(\"invalid hex at byte {i}: {e}\"))?;\n        out.push(\n            u8::from_str_radix(pair, 16)\n                .map_err(|e| anyhow!(\"invalid hex at byte {i}: {e}\"))?,\n        );\n    }\n    Ok(out)\n}\n\n#[cfg(test)]\nmod hex_decode_tests {\n    #[test]\n    fn rejects_multibyte_input_without_panicking() {\n        assert!(super::hex_decode(\"a\\u{e9}b\").is_err());\n        assert!(super::hex_decode(\"\\u{e9}\").is_err());\n        assert_eq!(super::hex_decode(\"00ff\").unwrap(), vec![0x00, 0xff]);\n    }\n}"
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "low",
      "reason": "The trigger is a malformed txid: unusual, but one stray non-ASCII character in a pasted or copied value suffices, no privilege is needed, and no other validation step rejects it first."
    },
    "impact": {
      "score": "low",
      "reason": "The caller's own command aborts (Rust panic, exit 101) instead of printing `invalid hex`; no data, funds or authority are affected and nothing is written."
    },
    "overall_severity": "low"
  },
  "confidence": {
    "score": "high",
    "reason": "The panic was reproduced end-to-end with the target binary built unmodified from the reviewed commit in the network-off sandbox, the message names the exact source line (flow.rs:1785), and two sibling implementations show the intended error-returning behaviour."
  }
}
```

## Its linked coverage unit and the hunter's checks

- coverage_id: `crates%2Fprobe%2Fsrc%2Fflow.rs%23parse-args-and-flow-load::crates%2Fprobe%2Fsrc%2Frecord.rs%23write-artifacts::cli-inputs::ATTACK-CLASSES.md%23Resource%20and%20file%20handling`
- surface: `crates/probe/src/flow.rs#parse-args-and-flow-load` (subsystem `cli-inputs`)
- boundary: `crates/probe/src/record.rs#write-artifacts`
- attack class: `ATTACK-CLASSES.md#Resource and file handling` · assignment owner: `hunter-inputs`
- unit reviewed paths: ["crates/probe/src/core.rs", "crates/probe/src/flow.rs", "crates/probe/src/gate.rs", "crates/probe/src/main.rs", "crates/probe/src/node.rs", "crates/probe/src/ops.rs", "crates/probe/src/record.rs", "crates/probe/src/wallet.rs", "crates/properties/src/lib.rs", "crates/properties/src/properties/amount_fingerprint.rs", "crates/properties/src/properties/mod.rs", "crates/properties/src/properties/timing_correlation.rs", "flows/ci-checkout-control.json", "flows/ci-checkout.json"]

The unit's checks, including the local checks whose promoted artifacts you may read
(paths are relative to the run directory `/home/kiter/security-audit-skill/zanary/run-1`):

```json
[
  {
    "agent_id": "hunter-inputs",
    "reviewed_paths": [
      "crates/probe/src/flow.rs"
    ],
    "invariant": "A flow definition read from --definition is validated before its values drive work, and every amount it carries reaches only checked arithmetic",
    "method": "source",
    "result": "Validation is shape-only: non-empty invoice list, receiver_strategy, receipt_account vs merchant_account, settlement mode and batch amount uniqueness (flow.rs:253-297). Amounts are never validated and flow into three unchecked u64 sums (flow.rs:336, 1182, 1294-1297) -> candidate zanary/unchecked-zat-sum-overflow. The same unchecked sum shape also exists at payer.rs:519, inside the peer-owned bounded-payer unit, and is not reported here.",
    "artifact": null
  },
  {
    "agent_id": "hunter-inputs",
    "reviewed_paths": [
      "crates/probe/src/record.rs",
      "crates/probe/src/flow.rs"
    ],
    "invariant": "Artifact writes stay under the caller-named --out directory, no path component derives from file content, and each write is bounded by the recorded run",
    "method": "source",
    "result": "record.rs:35-62 is create_dir_all(out_dir) plus three fixed-name writes (run.json, findings.json, report.json); the only definition-derived path is default_out_dir, which uses the definition's file_stem (flow.rs:1091-1097) and a stem cannot contain a separator; --out being caller-named is the documented contract. Write volume is bounded by what the run recorded (per-payment labels from the definition, memos by ZIP-302's 512 bytes). No traversal, TOCTOU or unbounded write at this privilege level.",
    "artifact": null
  },
  {
    "agent_id": "hunter-inputs",
    "reviewed_paths": [
      "crates/probe/src/flow.rs",
      "crates/probe/src/ops.rs",
      "crates/probe/src/main.rs",
      "crates/probe/src/gate.rs",
      "crates/probe/src/wallet.rs",
      "crates/probe/src/core.rs",
      "crates/probe/src/node.rs"
    ],
    "invariant": "Every parse, decode and arithmetic step on caller-supplied values either rejects bad input with an error or panics only on construction-guaranteed state",
    "method": "source",
    "result": "One input-reachable panic (flow.rs:1785 -> candidate zanary/hex-decode-str-slice-panic) and one unchecked-arithmetic family (candidates zanary/unchecked-zat-sum-overflow and zanary/unbounded-jitter-block-mining). All other expects are construction- or chain-guaranteed: ops.rs:81 (peek), ops.rs:1118 (consensus-validated note), flow.rs:936 (permutation from batch_order), core.rs:639/1077/1118/1149/1191/1199. --ttl-days is parsed as a full u64 (flow.rs:1119-1128) and multiplied unchecked at record.rs:58 (hardening: only the caller's own argument). RUST_LOG reaches tracing_subscriber's EnvFilter, which parses lossily (tracing-subscriber 0.3.23 src/filter/env/mod.rs:350-354). Rig-facing RPC/scan code in core.rs and node.rs beyond the generate/timeout path was left to the local-rig unit.",
    "artifact": null
  },
  {
    "agent_id": "hunter-inputs",
    "reviewed_paths": [
      "crates/probe/src/flow.rs",
      "crates/probe/src/ops.rs",
      "crates/properties/src/properties/mod.rs",
      "crates/properties/src/lib.rs",
      "crates/properties/src/properties/amount_fingerprint.rs",
      "crates/properties/src/properties/timing_correlation.rs",
      "flows/ci-checkout.json",
      "flows/ci-checkout-control.json"
    ],
    "invariant": "A caller-supplied invoice file cannot make the payer act on anything the file does not say, and evaluating a recorded run cannot panic",
    "method": "source",
    "result": "Invoice::from_path (flow.rs:1740-1768) cross-checks the file's receiver, amount and memo against its own ZIP-321 URI and refuses a self-disagreeing invoice; the privacy-policy check precedes any proposal (ops.rs:1019-1020); the amount bound Zatoshis::from_u64 precedes any build (ops.rs:912). The 13 properties index only behind guards and use abs_diff (amount_fingerprint.rs:98 behind len()==1, timing_correlation.rs:87 behind f.height <= h). Both checked-in ci flows parse and pass the load-time validation. No finding.",
    "artifact": null
  },
  {
    "agent_id": "hunter-inputs",
    "reviewed_paths": [
      "crates/probe/src/flow.rs",
      "crates/probe/src/main.rs",
      "crates/probe/src/record.rs"
    ],
    "invariant": "A hostile definition file and a hostile --txid are refused with a diagnostic (exit 1), not a crash, and the CLI contacts no chain resource to reject them",
    "method": "local",
    "result": "Built the unmodified reviewed commit in an OS-enforced sandbox (docker --network=none, rustc 1.99.0, dev profile, 4m03s) and ran the CLI: `flow --definition evil-sum.json` exited 101 with `attempt to add with overflow` while the flow.rs:328-338 header evaluated the amount sum; `verify-invoice --txid \"a\u00e9b\"` exited 101 with the char-boundary panic at flow.rs:1785. Controls with odd-length and non-hex-ASCII txids exited 1 with `invalid hex`. Neither run reached a wallet, node or indexer.",
    "artifact": "agents/hunter-inputs/artifacts/check-panics.txt"
  },
  {
    "agent_id": "hunter-inputs",
    "reviewed_paths": [
      "crates/probe/src/flow.rs",
      "crates/probe/src/node.rs"
    ],
    "invariant": "An accepted flow definition cannot name an unbounded mining request",
    "method": "local",
    "result": "A test appended to the disposable sandbox copy drives Jitter::from_definition with the definition jitter_blocks [1,4294967295], jitter_seed 7: 32 draws averaged ~1.9e9 blocks with a maximum of 4,085,077,839 blocks in one node.generate argument (the value flow.rs:645/883 pass to ops.rs:436 -> node.rs:232). Missing-bound half of candidate zanary/unbounded-jitter-block-mining settled; the node-side cost stays needs_validation.",
    "artifact": "agents/hunter-inputs/artifacts/check-jitter-draw.txt"
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-11/scratch` (scratch-relative files only, `check-*.txt` / `check-*.json` /
`evidence.txt`, 64 KiB per file, 256 KiB total); the parent promotes into `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-11/artifacts`. You never write
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
