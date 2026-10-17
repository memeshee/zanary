# Phase-5 record verifier — verifier-30 on `zanary/hex-decode-str-slice-panic`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`confirmed`)

```json
{
  "verdict": "confirmed",
  "fingerprint": "zanary/hex-decode-str-slice-panic",
  "title": "verify-invoice panics on a txid containing a multi-byte UTF-8 character instead of rejecting it",
  "description": "flow::hex_decode checks only that the txid's byte length is even, then slices the string with `&hex[i..i + 2]`. Rust &str indices are byte offsets, so any input whose even UTF-8 byte length lets a 2-byte slice boundary land inside a multi-byte character aborts the process (exit 101, `end byte index N is not a char boundary`) instead of returning the normal `invalid hex at byte N` error (exit 1). verify-invoice decodes --txid before it opens a wallet or touches the chain, so the crash is reachable offline with a four-byte argument. The same workspace decodes hex safely in two other places (crates/report/src/node.rs:193-203 and crates/probe/src/bin/zanary-seed.rs:714-723 both step over `hex.as_bytes().chunks(2)` and map a bad chunk to an error), and the same function's other rejections are clean (odd length and non-hex input both exit 1), so the panicking shape is an outlier rather than a deliberate fail-loudly choice.",
  "root_cause": "flow::hex_decode (crates/probe/src/flow.rs:1778-1789) byte-indexes a &str \u2014 `u8::from_str_radix(&hex[i..i + 2], 16)` at flow.rs:1785 \u2014 after validating only that the byte length is even; a multi-byte UTF-8 character straddling a 2-byte slice boundary makes the slice panic.",
  "intended_behavior": "A --txid that is not hex is refused with `Error: decoding the txid ... invalid hex at byte N` and exit 1 (the odd-length case with `hex string has an odd length`), as every other bad-txid shape already is.",
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
      "description": "the --txid value is captured verbatim from argv (`txid_hex = Some(value(i, \"--txid\")?)`), with no character-level validation"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/flow.rs",
      "line": 1687,
      "scope": "verify_invoice",
      "description": "`hex_decode(&txid_hex)` runs before `core::bootstrap_with(WalletProfile::merchant(...))` at flow.rs:1699-1700, so no wallet, node or indexer is involved"
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/flow.rs",
      "line": 1785,
      "scope": "hex_decode",
      "description": "`&hex[i..i + 2]` slices at byte offsets; an offset inside a multi-byte character panics (verifier-11 repro: --txid \"a\u00e9b\" (bytes 61 c3 a9 62) -> exit 101, `panicked at crates/probe/src/flow.rs:1785:36: end byte index 2 is not a char boundary; it is inside '\u00e9' (bytes 1..3 of string)`)"
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
      "description": "the panicking expression; independently reproduced by verifier-11 with the binary built from a fresh `git archive HEAD` copy of commit 7058406 in a --network=none sandbox: payload \"a\u00e9b\" -> exit 101, panic message names flow.rs:1785:36"
    },
    {
      "file": "crates/report/src/node.rs",
      "line": 198,
      "description": "sibling control: the report crate's hex_decode iterates `hex.as_bytes().chunks(2)` and converts each chunk with std::str::from_utf8, returning an error instead of panicking"
    },
    {
      "file": "crates/probe/src/bin/zanary-seed.rs",
      "line": 718,
      "description": "sibling control: zanary-seed's hex_decode_into (lines 714-723) iterates `hex.as_bytes().chunks(2)` and returns an error via std::str::from_utf8"
    },
    {
      "file": "crates/probe/src/flow.rs",
      "line": 1687,
      "description": "call site: `hex_decode(&txid_hex)` in verify_invoice, before any wallet or network work (bootstrap_with at flow.rs:1699-1700); a valid 64-hex txid proceeds past this point to the node connection attempt, a non-hex multi-byte one panics here"
    }
  ],
  "conditions": [
    {
      "kind": "user_interaction",
      "description": "the caller (an operator, or a script/agent that passes a txid it was handed) supplies a --txid value that is not hex ASCII, e.g. a txid pasted with an accented character or a typographic mark"
    },
    {
      "kind": "data_state",
      "description": "the value's UTF-8 byte length is even and a 2-byte slice boundary falls inside a multi-byte character \u2014 \"a\u00e9b\" (61 c3 a9 62), \"a\u00abb\" (61 c2 ab 62), \"\u2014a\" (e2 80 94 61) and \"a\ud83e\udd80b\" (61 f0 9f a6 80 62) all panicked, while the odd-length mixed input \"a\u2014b\" (61 e2 80 94 62) was instead refused by the length guard with exit 1"
    }
  ],
  "execution": {
    "attacker_perspective": "Anyone who can hand a payer or an agent the text that becomes --txid (a settlement claim, a chat message, a re-encoded log line) makes `zanary-probe verify-invoice` abort with a Rust panic instead of reporting a bad txid; in a scripted check the 101 exit also stands in for the exit 1 the CLI's other argument errors produce, turning a validation failure into an unexplained crash.",
    "payloads": [
      "verify-invoice --invoice invoice-fixture.json --txid \"$(printf 'a\\303\\251b')\"   # the 4 UTF-8 bytes 61 c3 a9 62 (\"a\u00e9b\")",
      "verify-invoice --invoice invoice-fixture.json --txid \"$(printf '\\342\\200\\224a')\"   # the 4 UTF-8 bytes e2 80 94 61 (\"\u2014a\")",
      "{\"invoice_id\":\"9001\",\"amount_zat\":1,\"receiver\":\"not-a-real-address\",\"memo\":\"\",\"uri\":\"\",\"issued_at_height\":1,\"merchant_account\":0}"
    ],
    "instructions": [
      "In the OS-enforced sandbox (docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024, host cargo registry mounted read-only) build a fresh disposable `git archive HEAD` copy of the reviewed commit 7058406 with rust-toolchain.toml removed so the image's own toolchain is used: `cargo build --offline -p zanary-probe --bin zanary-probe` (rustc 1.99.0, dev profile, finished in 8m03s).",
      "Confirm the copy's crates/probe/src/flow.rs is byte-identical to the target's (sha256 6fe40483fc367ef5df33b89bca1cfe383af13c259ff5144562d5279a34970782).",
      "Write the shape-valid Invoice fixture above to invoice-fixture.json (verify_invoice only serde-parses it \u2014 no field is validated on this path).",
      "Run `./target/debug/zanary-probe verify-invoice --invoice invoice-fixture.json --txid \"$(printf 'a\\303\\251b')\"` and read the exit code; nothing is contacted (no node, indexer or wallet).",
      "Controls, same command and fixture: --txid = the 2-byte \u00e9 (even length, non-hex), 64 ASCII 'z' and 4 ASCII 'z' all exit 1 with `invalid hex at byte 0: invalid digit found in string`; --txid = \"a\u2014b\" (odd length) exits 1 with `hex string has an odd length`; a valid 64-hex txid passes hex_decode and then fails at `connecting to the zcashd RPC at 127.0.0.1:18232` (Connection refused under --network=none)."
    ],
    "observed_result": "exit 101; stderr `thread 'main' (10) panicked at crates/probe/src/flow.rs:1785:36: end byte index 2 is not a char boundary; it is inside '\u00e9' (bytes 1..3 of string)`. The same panic reproduces with --txid \"a\u00abb\" (exit 101, '\u00ab' bytes 1..3), --txid \"\u2014a\" (exit 101, '\u2014' bytes 0..3) and --txid \"a\ud83e\udd80b\" (exit 101, bytes 1..5). All controls exited 1: `Error: decoding the txid / Caused by: invalid hex at byte 0: invalid digit found in string` (\u00e9, 64\u00d7'z', 4\u00d7'z') and `hex string has an odd length` (a\u2014b). Transcripts: verifier-11 scratch check-hex-decode-panic-runs.txt, check-hex-decode-panic-build.txt, check-hex-decode-panic-fix.txt."
  },
  "remediation": {
    "strategy": "Decode the txid through bytes instead of slicing the &str, mirroring what the same workspace already ships in crates/report/src/node.rs and crates/probe/src/bin/zanary-seed.rs, and keep the `invalid hex at byte N` error shape so callers see the same diagnostic. Verifier-11 applied this exact change to the sandbox copy: it compiles (incremental dev build 1m08s), the added unit test passes, and the payloads above then exit 1 with `invalid hex at byte 0: incomplete utf-8 byte sequence...` instead of panicking.",
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
      "reason": "The trigger is a malformed txid: unusual, but one stray non-ASCII character of even byte length suffices, no privilege is needed, and no other validation step rejects it first."
    },
    "impact": {
      "score": "low",
      "reason": "The caller's own command aborts (Rust panic, exit 101) instead of printing `invalid hex`; no data, funds or authority are affected and nothing is written."
    },
    "overall_severity": "low"
  },
  "confidence": {
    "score": "high",
    "reason": "The panic was reproduced end-to-end by verifier-11 with the reviewed commit's source built in a network-off sandbox, the message names the exact source line (flow.rs:1785:36), every control behaved as claimed, and two sibling implementations show the intended error-returning behaviour; the proposed fix was also compiled and tested in the sandbox copy."
  }
}
```

## Linked ledger unit

- coverage_id: `crates%2Fprobe%2Fsrc%2Fflow.rs%23parse-args-and-flow-load::crates%2Fprobe%2Fsrc%2Frecord.rs%23write-artifacts::cli-inputs::ATTACK-CLASSES.md%23Resource%20and%20file%20handling`
- surface: `crates/probe/src/flow.rs#parse-args-and-flow-load` · subsystem `cli-inputs`
- boundary: `crates/probe/src/record.rs#write-artifacts`
- assignment owner: `hunter-inputs`

The unit's checks (each `artifact` path is relative to the run directory; read the promoted files
under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/`):

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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-30/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-30/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "zanary/hex-decode-str-slice-panic"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
