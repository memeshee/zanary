# Phase-5 record verifier — verifier-39 on `zanary/unchecked-zat-sum-overflow`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`confirmed`)

```json
{
  "verdict": "confirmed",
  "fingerprint": "zanary/unchecked-zat-sum-overflow",
  "title": "Flow-definition amounts are summed with unchecked u64 arithmetic: a two-invoice definition aborts the dev build and wraps silently in the release build",
  "description": "Loading a flow definition validates its shape but never its invoice amounts. The total is computed with `sum::<u64>()` in the run header (flow.rs:336), the same unchecked sum decides the funding shortfall (flow.rs:1182 \u2014 the fee term is added with saturating_add at 1183, the sum itself is not), and funding_target sums the amounts again and adds the fee budget with plain `+` (flow.rs:1296-1297), so amounts totalling more than u64::MAX overflow: in a dev build (overflow checks on \u2014 the build docs/FLOW-RUNNER-NOTES.md:477 and scripts/agent-pay-loop.sh:20 use for `flow`) the process panics before any wallet or node work; in a release build (no [profile] override exists in any of the 5 workspace manifests; the build the CI, the composite action and the demo scripts use) the same input wraps to a smaller total and the run proceeds on it. The only amount bounds are per-amount Zatoshis::from_u64 checks (ops.rs:912 in propose_shielded_payments, ops.rs:983 in pay --invoice), which run later and reject one too-large amount, never the total. Independently verified from a pristine copy of the reviewed commit: dev exits 101 with `attempt to add with overflow` (backtrace frame flow.rs:336:18); the release build prints `184467440737.09551614 ZEC total` for the two-u64::MAX fixture and continues to the node step.",
  "root_cause": "No load-time bound on invoice amounts plus unchecked additive arithmetic over definition-controlled u64 values: `invoices.iter().map(|i| i.amount_zat).sum::<u64>()` (flow.rs:336, 1182) and `sum::<u64>() + fee_budget_zat(...)` (flow.rs:1296-1297). The same function guards its fee term with saturating_add (flow.rs:1183), so the unchecked sum and add are outliers inside the code that computes them.",
  "intended_behavior": "A definition whose amounts are not payable is refused at load time with a diagnostic naming the invoice \u2014 the same place and style as the neighbouring checks \u2014 and the arithmetic that consumes amounts cannot overflow for any accepted definition.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/flow.rs",
      "line": 251,
      "scope": "run_recording (--definition load)",
      "description": "the --definition file is read and deserialized into FlowDefinition; every amount is caller-controlled (InvoiceSpec.amount_zat: u64 at flow.rs:62, no serde bound and no custom deserializer)"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/flow.rs",
      "line": 253,
      "scope": "run_recording definition validation",
      "description": "the validation block (lines 253-297) checks the invoice list, receiver_strategy, receipt_account vs merchant_account and settlement/batch uniqueness \u2014 amounts are never looked at"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/flow.rs",
      "line": 336,
      "scope": "run_recording header println",
      "description": "`.map(|i| i.amount_zat).sum::<u64>() as i128` overflows: reproduced panic `attempt to add with overflow` (exit 101) with the backtrace frame `run_recording` at flow.rs:336:18, before any wallet or node work"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/flow.rs",
      "line": 1182,
      "scope": "shortfall",
      "description": "the same unchecked sum decides whether the payer covers the invoices (the fee term is added with saturating_add at 1183; the sum is not)"
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/flow.rs",
      "line": 1296,
      "scope": "funding_target",
      "description": "`sum::<u64>()` (1296) `+ fee_budget_zat(...)` (1297) adds to the already-wrapped total without saturating; its result is the top-up target the run acts on (consumed at flow.rs:424)"
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/flow.rs",
      "line": 336,
      "description": "the sum that panics in a dev build for two u64::MAX amounts \u2014 independently reproduced: exit 101, `attempt to add with overflow`, backtrace frame at ./crates/probe/src/flow.rs:336:18, before NodeRpc::from_env()"
    },
    {
      "file": "crates/probe/src/flow.rs",
      "line": 1183,
      "description": "sibling control in the same file: `needed.saturating_add(fee_budget_zat(...))` \u2014 the author bounded the fee term deliberately, but not the invoice sum it is added to"
    },
    {
      "file": "crates/probe/src/flow.rs",
      "line": 1296,
      "description": "funding_target (lines 1291-1298) sums the same amounts (1296) and adds the fee budget with plain `+` (1297); the result is the payer top-up target (used at flow.rs:424)"
    },
    {
      "file": "crates/probe/src/ops.rs",
      "line": 912,
      "description": "the amount bound, per payment later: Zatoshis::from_u64(*zats) inside propose_shielded_payments rejects one amount above MAX_MONEY; the sibling site ops.rs:983 (`pay --invoice`) is the same per-amount shape \u2014 neither bounds a total"
    },
    {
      "file": "Cargo.toml",
      "line": 1,
      "description": "no [profile.dev]/[profile.release] section in the workspace or any of the 5 manifests (checked); release builds therefore wrap. CI/action/demo build --release (action.yml:80, .github/workflows/privacy-gate.yml:135/236); verified by building and running both profiles from the reviewed commit"
    },
    {
      "file": "docs/FLOW-RUNNER-NOTES.md",
      "line": 477,
      "description": "the documented way to run a definition is `./target/debug/zanary-probe flow --definition ...` \u2014 the profile in which the overflow panics"
    }
  ],
  "conditions": [
    {
      "kind": "data_state",
      "description": "a definition whose invoice amounts total more than u64::MAX: two u64::MAX amounts in a 144-byte file (the verified fixture), or \u2014 with every amount individually within the consensus money range \u2014 at least 8,785 invoices at MAX_MONEY (2,100,000,000,000,000 zatoshis), a few hundred KB of JSON"
    },
    {
      "kind": "system_configuration",
      "description": "dev profile (overflow checks on \u2014 the build the local docs and scripts use) panics with exit 101; release profile (the CI/action/demo build) wraps silently and the run proceeds on the wrong total. Both observed from binaries built off the reviewed commit: dev exit 101 panic; release printed `184467440737.09551614 ZEC total` for the two-u64::MAX fixture and continued to the node step"
    }
  ],
  "execution": {
    "attacker_perspective": "The flow definition is the untrusted file this CLI takes (caller- or CI-supplied JSON). A definition of a hundred-odd bytes (or a careless one carrying absurd amounts) crashes the dev-build runner where it should have said which invoice is not payable, and in the release build it makes the runner act on a total that is not the sum of the amounts it was given.",
    "payloads": [
      "{\"flow_id\":\"hostile-sum\",\"invoices\":[{\"invoice_id\":\"1\",\"amount_zat\":18446744073709551615},{\"invoice_id\":\"2\",\"amount_zat\":18446744073709551615}]} (evil-sum.json, 144 bytes, sha256 6bf0d8114fbb4420359bc81dc498e7a5006ef469f1ec84e91a003e7fded89b5b)",
      "flow --definition evil-sum.json"
    ],
    "instructions": [
      "In an OS-enforced sandbox with no network, build the reviewed commit from a pristine `git archive HEAD` copy with rust-toolchain.toml removed: `cargo build --offline -p zanary-probe --bin zanary-probe` and `cargo build --offline --release -p zanary-probe --bin zanary-probe` (rustc 1.99.0).",
      "Write the payload JSON to evil-sum.json and run `./target/debug/zanary-probe flow --definition evil-sum.json`; read the exit code and the header output. The run reaches neither the node nor the indexer, so no rig state is touched.",
      "Raw-sum contrast with the same toolchain: `rustc v20-wrap.rs` over `let v = std::hint::black_box([u64::MAX, u64::MAX]); let total: u64 = v.iter().copied().sum::<u64>();` panics with overflow checks on, and `rustc -O v20-wrap.rs` prints 18446744073709551614 without panicking.",
      "Run the release binary on the same fixture: `./target/release/zanary-probe flow --definition evil-sum.json` \u2014 the header shows the wrapped total and the run proceeds to the node step."
    ],
    "observed_result": "Dev build: exit 101; stdout shows the run header (flow id hostile-sum, flow commit 51b8f57faad49badfeb9a0c963fc86b6229ceed6910a74ee08f12e529b23f02c \u2014 identical to the hunter's recorded value, plus settlement/chain lines) and then stderr `thread 'main' panicked at .../library/core/src/iter/traits/accum.rs:206:1: attempt to add with overflow`; with RUST_BACKTRACE=1 the frame is `zanary_probe::flow::run_recording` at ./crates/probe/src/flow.rs:336:18 <- flow::run (flow.rs:230:38) <- main (main.rs:46:41) \u2014 the panic is raised evaluating the header sum at flow.rs:336, before NodeRpc::from_env(). Release build of the same source: no panic; the header prints `invoices : 2 (184467440737.09551614 ZEC total)` (the wrapped 2^64-2 total; the true total would be 368934881474.19103230 ZEC) and the run continues to the node step (exit 1 there with `Connection refused`, since the sandbox has no node) instead of rejecting the definition. A benign control (two 100,000-zat invoices) prints 0.00200000 ZEC total and reaches the same node step in both builds. The optimised raw-sum snippet printed 18446744073709551614 (exit 0); the unoptimised one panicked identically. Transcripts: agents/verifier-20/artifacts/check-panics-repro.txt, agents/verifier-20/artifacts/check-release-binary.txt, agents/verifier-20/artifacts/check-release-wrap.txt."
  },
  "remediation": {
    "strategy": "Validate amounts where the definition is loaded (each within the consensus money range, and the total via checked_add) and make every later sum checked or saturating, so an unpayable definition is a diagnostic instead of a panic or a wrapped number.",
    "code_changes": [
      {
        "file_name": "crates/probe/src/flow.rs",
        "fixed_code": "// in run_recording, beside the existing definition checks (flow.rs:253-297):\nconst MAX_MONEY_ZAT: u64 = 2_100_000_000_000_000; // 21,000,000 ZEC\nlet mut total_zat: u64 = 0;\nfor invoice in &definition.invoices {\n    if invoice.amount_zat == 0 || invoice.amount_zat > MAX_MONEY_ZAT {\n        bail!(\n            \"invoice {} names {} zatoshis \u2014 an invoice amount must be between 1 and {MAX_MONEY_ZAT}\",\n            invoice.invoice_id,\n            invoice.amount_zat\n        );\n    }\n    total_zat = total_zat.checked_add(invoice.amount_zat).ok_or_else(|| {\n        anyhow!(\n            \"the definition's invoice amounts total more than {} zatoshis (u64), so the run cannot account for them\",\n            u64::MAX\n        )\n    })?;\n}\n\n// and at the three sum sites consume the checked total instead of re-summing:\n//   header (flow.rs:336):             core::format_zats_signed(total_zat as i128)\n//   shortfall (flow.rs:1182):         let needed = total_zat.saturating_add(fee_budget_zat(invoices.len(), settlement));\n//   funding_target (flow.rs:1291-98): pass total_zat in and use\n//                                     definition.funding_zat.max(missing).max(total_zat.saturating_add(fee_budget_zat(definition.invoices.len(), settlement_of(definition))))"
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "low",
      "reason": "The overflow needs amounts totalling above u64::MAX; a 144-byte two-invoice file is enough to reach it, but no realistic definition gets there by accident and the file is the caller's own. The release-mode wrap is reachable with individually-plausible amounts only through 8,785+ invoices at MAX_MONEY."
    },
    "impact": {
      "score": "low",
      "reason": "Dev build: the run aborts before any wallet or node work (exit 101, no diagnostic naming the invoice). Release build: the run proceeds on a wrapped total and fails later on its own per-amount checks (ops.rs:912/983 reject amounts above MAX_MONEY) or an unrelated node error. No funds, keys or authority are exposed \u2014 the wallet involved is the operator's own throwaway regtest wallet."
    },
    "overall_severity": "low"
  },
  "confidence": {
    "score": "high",
    "reason": "Both halves were reproduced independently in the network-off sandbox from a pristine copy of commit 7058406 (all tracked files verified byte-identical to the read-only target): the dev binary exits 101 with the panic backtrace frame at flow.rs:336:18, and the release binary of the same source prints the wrapped total for two u64::MAX amounts and continues instead of rejecting the definition. The unchecked sites and the saturating sibling control are visible in the reviewed source. Nothing stronger than the claim was observed \u2014 the downstream funding-target consequence stays source-established because it needs a live node."
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-39/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-39/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "zanary/unchecked-zat-sum-overflow"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
