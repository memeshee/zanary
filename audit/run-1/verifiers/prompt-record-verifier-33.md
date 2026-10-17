# Phase-5 record verifier — verifier-33 on `zanary/payer-expiry-text-compare`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`confirmed`)

```json
{
  "verdict": "confirmed",
  "fingerprint": "zanary/payer-expiry-text-compare",
  "title": "Bounded payer enforces grant expiry as a text comparison, so an expiry instant written with a UTC offset (or an unparsable value) still pays",
  "description": "decide() refuses an invoice only when the ASCII comparison `now >= grant.expires_at` holds (crates/probe/src/payer.rs:91). Both sides are unvalidated strings: `now` comes from the workspace clock (RFC3339 Z, fractional seconds) and a grant file's `expires_at` is used verbatim (payer.rs:173-184 parses only the *default* path, when the field is absent). Equal instants written differently therefore get opposite verdicts, and any text that sorts after the clock never expires. The lower-trust principal is whoever supplies the grant JSON to `zanary-probe payer --grant` (architecture.md boundary 3: caller-supplied JSON and paths into the probe CLI). No attacker is even needed: an operator or generic tool that writes the expiry from local time (a legal RFC3339 offset) or with a typo silently widens the authority window, and the control that should have caught it is the grant's own expiry rule ('the instant after which the authority is void', docs/BOUNDED-PAYER.md:31). The failure is invisible after the fact: PayerDecision records no decision time (observation/src/lib.rs:202-214) and payer-policy has no expiry rule (payer_policy.rs:114 only echoes the expiry; a paid row under a grant that expired in 2020 returns Pass).",
  "root_cause": "Text comparison of two RFC3339 strings instead of an instant comparison, plus no validation or canonicalisation of `expires_at` on the grant-file path. `if now >= grant.expires_at.as_str()` (payer.rs:91) is only correct when both strings are the same encoding \u2014 canonical UTC 'Z', whole seconds, fixed width. The clock producer and the grant author are different components with different conventions: record.rs:132 formats fractional seconds (data/runs/payer-pass/report.json: verified_at \"2026-10-16T10:57:34.300715717Z\"), data/grants/*.json and the fixtures are whole-second Z, and RFC3339 permits UTC offsets that the comparison misorders ('.' < 'Z', '0' < '1').",
  "intended_behavior": "An invoice shown at or after the grant's expiry instant must be refused ('expiry \u2014 the instant after which the authority is void', docs/BOUNDED-PAYER.md:31; 'now >= expires_at? -> refuse (authority expired)', docs/BOUNDED-PAYER.md:43), and that refusal must hold for any valid RFC3339 representation of that instant; a value that is not a usable instant should refuse the run rather than read as unexpired.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/payer.rs",
      "line": 149,
      "scope": "zanary-probe payer --grant",
      "description": "the grant path is taken from argv (`args.require(\"--grant\")`); the file is read at 165-166 and parsed into Grant at 167-168, so expires_at enters as a caller-supplied string"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 173,
      "scope": "Grant -> PayerGrant",
      "description": "a file-supplied expires_at is used verbatim; only the absent (None) case parses and re-formats a timestamp (176-183)"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 250,
      "scope": "decision loop",
      "description": "decide(&grant, spent, invoice, &now) is called once per invoice with the startup clock string (169-172)"
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/payer.rs",
      "line": 91,
      "scope": "decide() expiry rule",
      "description": "lexicographic comparison: a past instant written with a positive offset sorts after the clock and is treated as unexpired; the invoice is then counted and pushed to to_pay (258-259) and paid by the loop at 291-308"
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/payer.rs",
      "line": 91,
      "description": "`if now >= grant.expires_at.as_str()` \u2014 the expiry rule is a &str comparison with no parse on either side"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 173,
      "description": "a grant file's expires_at is taken verbatim (`Some(ts) => ts`); the RFC3339 parser (178) runs only for the None case, and Grant (49-62) deserialises expires_at as an opaque Option<String> with no validator"
    },
    {
      "file": "crates/probe/src/record.rs",
      "line": 132,
      "description": "utc_now_rfc3339 is the workspace clock and formats fractional seconds; data/runs/payer-pass/report.json carries verified_at \"2026-10-16T10:57:34.300715717Z\" while data/grants/*.json are whole-second Z"
    },
    {
      "file": "crates/observation/src/lib.rs",
      "line": 194,
      "description": "PayerGrant.expires_at is a String (documented as an RFC3339 UTC instant) \u2014 the observation model never derives an instant from it"
    },
    {
      "file": "crates/observation/src/lib.rs",
      "line": 202,
      "description": "PayerDecision carries invoice_id/amount/receiver/decision/reason/txid/height and no decision timestamp, so an expiry violation cannot be re-derived from the ledger"
    },
    {
      "file": "crates/properties/src/properties/payer_policy.rs",
      "line": 114,
      "description": "payer-policy echoes grant.expires_at into its summary and evidence but never compares it to anything: the rules (78-102 cap/budget/allowlist/revoked, 68 the \"error:\" prefix) contain no expiry check, so a paid row under a long-expired grant is a Pass"
    }
  ],
  "conditions": [
    {
      "kind": "data_state",
      "description": "the grant's expires_at is a valid RFC3339 instant written with a positive UTC offset (a local-time writer), whose literal text is later than its instant, so enforcement lapses for up to the offset window (<= ~14 h); or the value is unparsable text that sorts after the clock ('never'), which never expires at all"
    },
    {
      "kind": "timing_dependency",
      "description": "at the exact expiry second the payer's own clock text carries fractional seconds while a whole-second grant text is still greater, so the refusal lapses for the remainder of that second"
    },
    {
      "kind": "system_configuration",
      "description": "the payer is invoked with that grant file; nothing rejects a non-canonical or malformed expires_at \u2014 no grant schema exists under schemas/, and payer.rs:91 is the only grant-expiry comparison in the probe crate"
    }
  ],
  "execution": {
    "attacker_perspective": "whoever writes the grant JSON handed to `zanary-probe payer --grant` (architecture boundary 3: caller-supplied JSON and paths into the probe CLI); no privileged access is needed and no deliberate attacker is required \u2014 a local-time writer or one typo is enough",
    "payloads": [
      "grant.json: {\"budget_zat\":1000,\"per_payment_cap_zat\":400,\"allowlist\":[\"uregtest1merchant\"],\"expires_at\":\"2026-11-02T13:00:00+14:00\"} (the same instant as the canonical 2026-11-01T23:00:00Z the demo grants use)",
      "grant.json: \"expires_at\": \"never\" (or any text sorting after the clock, e.g. \"9999-01-01T00:00:00Z\")",
      "--now 2026-11-02T00:00:00Z with the two encodings of one instant: 2026-11-01T23:00:00Z (refused) vs 2026-11-02T13:00:00+14:00 (paid)"
    ],
    "instructions": [
      "git archive HEAD the reviewed revision into a disposable /t copy (the reviewed repo stays untouched); confirm crates/probe/src/payer.rs and payer_policy.rs match the repo md5 before editing (54d121450a61b6fb35395a40c94ee710, 0000a14d897c48f6fec7c39dd58929bc)",
      "append the bounded unit module (check-decide-src.txt) to /t/crates/probe/src/payer.rs",
      "docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 -v <sandbox>:/t -v <cargo registry>:ro -w /t rust:1-slim sh -c 'CARGO_BUILD_JOBS=6 cargo test --offline -p zanary-probe --bin zanary-probe -- verifier14 --nocapture --test-threads=1'",
      "cargo build --offline -p zanary-probe --bin zanary-probe, then run /t/target/debug/zanary-probe payer --grant /t/verifier14-cli/grants/grant-<canonical|offset|never>.json --invoices /t/verifier14-cli/invoices --now 2026-11-02T00:00:00Z for each grant",
      "append the payer-policy module (check-property-src.txt) and run cargo test --offline -p zanary-properties --lib -- verifier14 --nocapture --test-threads=1",
      "append the fail-closed fix module (check-fix-src.txt) and run the same probe-bin test command with filter verifier14_fixcheck"
    ],
    "observed_result": "Sandbox (rust:1-slim, --network=none, disposable git archive copy; no rig contact). (a) Unit, real decide(): 5/5 passed \u2014 now 2026-11-02T00:00:00Z with expires_at 2026-11-01T23:00:00Z -> allowed=false \"refused: the grant expired at 2026-11-01T23:00:00Z (this invoice was shown at 2026-11-02T00:00:00Z)\"; the SAME instant written 2026-11-02T13:00:00+14:00 -> allowed=true \"paid: within the cap (400 zat) and the budget (10 of 1000 zat spent after this)\" (the test asserts both strings parse to one instant and that the clock is at/past it); now 2026-11-01T00:00:00.300715717Z vs 2026-11-01T00:00:00Z -> allowed=true; \"never\"/\"2030-99-99T00:00:00Z\"/\"9999-01-01T00:00:00Z\" -> allowed=true while \"\" -> allowed=false (control: the rule is string ordering). (b) Real binary, real grant files: `zanary-probe payer --grant ... --invoices ... --now 2026-11-02T00:00:00Z` printed \"REFUSE v14-8301 \u2014 refused: the grant expired at 2026-11-01T23:00:00Z ...\" for the canonical grant and \"ALLOW v14-8301 \u2014 paid: within the cap (400 zat) ...\" for the offset and \"never\" grants, failing afterwards only at NodeRpc \"Connection refused\" because the sandbox has no network (decisions precede wallet use, payer.rs:246-271 vs 273-277). (c) Real payer-policy property: a paid row under a grant whose expires_at is 2020-01-01T00:00:00Z returns Pass, the expiry appearing only in the evidence text. Not exercised: the actual shielded broadcast (payer.rs:306-308), which needs the regtest rig."
  },
  "remediation": {
    "strategy": "Compare instants, not text: parse both `now` and the grant's expires_at with time::OffsetDateTime::parse (the crate already depends on `time` with `parsing`, payer.rs:176-179) and fail closed \u2014 refuse the invoices when expires_at is not a usable instant instead of reading it as unexpired. Additionally record a `decided_at` timestamp on every PayerDecision so the payer-policy property can re-derive the expiry rule after the fact (today PayerDecision has no decision time, observation/src/lib.rs:202-214). The snippet below keeps decide()'s Decision return type and was compile- and behaviour-checked in the sandbox: all four bad encodings refuse, a genuine future canonical expiry still pays.",
    "code_changes": [
      {
        "file_name": "crates/probe/src/payer.rs",
        "fixed_code": "// decide(): compare instants, and fail closed when either side is not a usable instant.\nlet rfc3339 = time::format_description::well_known::Rfc3339;\nlet now_dt = match time::OffsetDateTime::parse(now, &rfc3339) {\n    Ok(dt) => dt,\n    Err(e) => {\n        return Decision {\n            allowed: false,\n            reason: format!(\"refused: the clock {now} is not RFC3339: {e}\"),\n        }\n    }\n};\nlet expires_dt = match time::OffsetDateTime::parse(&grant.expires_at, &rfc3339) {\n    Ok(dt) => dt,\n    Err(e) => {\n        return Decision {\n            allowed: false,\n            reason: format!(\n                \"refused: the grant's expires_at {} is not RFC3339 ({e}); refusing rather than treating it as unexpired\",\n                grant.expires_at\n            ),\n        }\n    }\n};\nif now_dt >= expires_dt {\n    return Decision {\n        allowed: false,\n        reason: format!(\n            \"refused: the grant expired at {} (this invoice was shown at {now})\",\n            grant.expires_at\n        ),\n    };\n}"
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "medium",
      "reason": "needs an expires_at that is not canonical whole-second UTC-Z text: a local-time writer emitting a legal RFC3339 offset, or a typo/malformed value. The repo's own grants, fixtures and shipped demo data are all canonical Z, so the demo path does not trigger it, but a generic tool that writes its local time is a routine source of offset-form timestamps."
    },
    "impact": {
      "score": "medium",
      "reason": "a payment made after the authority was void, up to the offset window (<= ~14 h), and it cannot be found afterwards: the ledger has no decision time and payer-policy returns Pass under an expired grant (both reproduced/source-confirmed). Bounded by the offset window and by the chain here being regtest-only."
    },
    "overall_severity": "medium"
  },
  "confidence": {
    "score": "high",
    "reason": "the path was re-established from source line by line and the minimum result was reproduced twice in the sandbox \u2014 the real decide() (5/5, equal instants with opposite verdicts, verified with time::OffsetDateTime::parse) and the real zanary-probe binary refusing the canonical grant and paying the offset/'never' ones \u2014 plus the payer-policy blind spot; the fix is local and was itself compiled and behaviour-checked."
  }
}
```

## Linked ledger unit

- coverage_id: `crates%2Fprobe%2Fsrc%2Fpayer.rs%23decide::crates%2Fproperties%2Fsrc%2Fproperties%2Fpayer_policy.rs%23evaluate::bounded-payer::ATTACK-CLASSES.md%23Business%20logic`
- surface: `crates/probe/src/payer.rs#decide` · subsystem `bounded-payer`
- boundary: `crates/properties/src/properties/payer_policy.rs#evaluate`
- assignment owner: `hunter-payer`

The unit's checks (each `artifact` path is relative to the run directory; read the promoted files
under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/`):

```json
[
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/observation/src/lib.rs"
    ],
    "invariant": "decide() is the single decision point for an invoice and applies the five grant rules in the documented order (revoked -> expired -> allowlist -> cap -> budget) over the grant actually recorded for the run",
    "method": "source",
    "result": "verified for the reference payer: decide is called once per invoice before any payment (payer.rs:246-270), the grant recorded in the run is the same object decide used (payer.rs:193-199, 388), and every refusal/allowance carries the first rule that fired. The rule bodies themselves are where the authority model leaks: the expiry rule is a text comparison (payer.rs:91), the allowlist rule never validates the receiver (100), the budget rule counts invoice principal only (119-128).",
    "artifact": null
  },
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/properties/src/properties/payer_policy.rs",
      "crates/observation/src/lib.rs",
      "crates/probe/src/record.rs"
    ],
    "invariant": "the ledger a payer records carries everything the payer-policy boundary needs to re-derive every grant rule, and the boundary re-derives the same verdict as the payer's own decide()",
    "method": "source",
    "result": "not re-derivable: PayerDecision has no decision time (observation:200-214), so the expiry rule cannot be checked at all (payer_policy only echoes expires_at into its summary, 105-116); the property never cross-checks the ledger against run.payments; and its internal-error rule keys on the literal reason prefix \"error:\" (68) which the reference payer never writes (it aborts, payer.rs:298-309).",
    "artifact": null
  },
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/probe/src/flow.rs"
    ],
    "invariant": "the payer's own gate holds: an expired grant cannot pay (as an instant), a repeated invoice_id is decided once, the budget bounds what the run spends, a misspelled grant key cannot widen authority, and an invoice the payment loop cannot pay is not decided payable",
    "method": "local",
    "result": "all five fail against the reviewed decide() (7/7 tests passed, i.e. the failures are the observed behaviour): H1a same expiry instant two encodings -> refused (canonical Z) vs ALLOWED (offset +14:00); H1b fractional clock against a whole-second expiry -> ALLOWED; H1c \"never\"/\"2030-99-99T00:00:00Z\" -> ALLOWED; H2 invoice dup-8301 decided ALLOW twice (spent 300 then 600 of 1000); H3 1-zat invoices -> 10,000,000 zat of fees for a 1,000-zat budget; H4 typo'd allow_list/expiry -> allowlist [] (any receiver), expires_at None; H8 allowlisted non-address receiver and 600-byte memo -> ALLOWED",
    "artifact": "agents/hunter-payer/artifacts/check-payer-decide.txt"
  },
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/properties/src/properties/payer_policy.rs",
      "crates/observation/src/lib.rs",
      "crates/probe/src/flow.rs"
    ],
    "invariant": "the payer-policy boundary re-derives the grant verdict from the run record for invoice identity, for whether a paid row is backed by a payment, and for the grant's expiry",
    "method": "local",
    "result": "passes in all three blind spots (3/3 tests passed): H5 two paid rows sharing one invoice_id (the second with no txid) -> PASS; H6 a paid row with zero run.payments entries -> PASS; H7 a ledger whose recorded grant expired in 2020 -> PASS, with the expiry echoed in the summary text only",
    "artifact": "agents/hunter-payer/artifacts/check-payer-policy.txt"
  },
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/probe/src/ops.rs"
    ],
    "invariant": "what the payer spends and does per allowed invoice is what the grant's budget accounts for (the cost of paying at the shared wallet layer is visible to the authority model)",
    "method": "source",
    "result": "the budget comparison never sees the payment layer's cost: every allowed invoice is one shielded transaction plus mine_and_resync(11 blocks) (payer.rs:291-310; ops.rs:59) and the payer's own funding model budgets 10,000 zat of ZIP-317 fee per payment (payer.rs:530), none of which the budget rule (119-128) or payer_policy (77-90) counts \u2014 the fee the payer knows it will pay is invisible to the authority it enforces.",
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-33/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-33/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "zanary/payer-expiry-text-compare"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
