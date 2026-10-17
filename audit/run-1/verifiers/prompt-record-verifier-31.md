# Phase-5 record verifier — verifier-31 on `zanary/payer-budget-excludes-fees`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`confirmed`)

```json
{
  "verdict": "confirmed",
  "fingerprint": "zanary/payer-budget-excludes-fees",
  "title": "The grant's budget bounds invoice principal, not the cost of paying: per-payment fees and the invoice count sit outside the authority",
  "description": "docs/BOUNDED-PAYER.md defines budget as \"the most the run may spend in total\" (line 28), and observation::PayerRun::spent_zat() labels its sum \"Zatoshis actually spent under this grant\" (observation/src/lib.rs:233-234). decide()'s budget rule (payer.rs:119-128) adds only invoice amounts, and the payer-policy property re-derives exactly the same number (payer_policy.rs:77-90), while the payer's own funding model budgets one ZIP-317 fee per payment at 10_000 zat (payer.rs:528-530; the runner's fee budget agrees, flow.rs:1165-1177) \u2014 a cost the grant never counts. With no minimum invoice amount (Invoice.amount_zat is a bare u64, flow.rs:1544) and no cap on how many invoices a run may load, decide and pay (load_invoices, payer.rs:495-520), a merchant can make the run's real spend dwarf the granted budget: 1-zat invoices fill a 1,000-zat budget with 1,000 payments and ~10,000,000 zat of fees, each payment also costing 11 mined blocks and a wallet sync (payer.rs:309, ops.rs:59). The ledger and payer-policy report 1,000 of 1,000 zat spent. Re-verified by execution: the real decide() admits the full thousand and the real payer-policy property passes the resulting ledger. Calibration: the flow runner funds fees the same way but holds no grant, so only the payer's authority claim is contradicted.",
  "root_cause": "The budget and per-payment cap are denominated in invoice principal only: neither decide()'s budget rule (payer.rs:119-128) nor the payer-policy re-derivation (payer_policy.rs:77-90) counts the per-payment fee the payer's own funding model budgets (payer.rs:530), and the grant schema has no cap on the number of payments (or a minimum amount) even though each payment carries a fixed-cost floor for the operator (one ZIP-317 fee, one 11-block mining round, one wallet sync).",
  "intended_behavior": "The grant's budget is the operator's bound on what the run spends (\"the most the run may spend in total\", docs/BOUNDED-PAYER.md:28; ledger/property summary \"N of M zat spent\"): a run at its budget must not be able to cost the operator an unbounded multiple of it.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/payer.rs",
      "line": 495,
      "scope": "load_invoices",
      "description": "the merchant-supplied invoice directory is read whole: every *.json in it is loaded (sorted by name) with no count cap, no size cap and no minimum amount per invoice (payer.rs:495-520)"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 119,
      "scope": "decide() budget rule",
      "description": "only invoice principal is accumulated (spent_zat.saturating_add(invoice.amount_zat), 119; refuse only when after > budget_zat, 120): a 1-zat invoice is always affordable until the nominal budget is exhausted"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 278,
      "scope": "funding",
      "description": "shortfall(&payer, &to_pay) (523-533) budgets sum(amounts) + 10_000 zat per payment (530) and flow::top_up_for_target (286; flow.rs:1202) mines/shields that headroom from outside the grant \u2014 the fee spend is real to the operator but invisible to the authority"
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/payer.rs",
      "line": 291,
      "scope": "payment loop",
      "description": "one shielded transaction (307-308) plus mine_and_resync(CONFIRM_BLOCKS = 11) (309; ops.rs:59) per allowed invoice: the run's real cost multiplies with the invoice count while the ledger reports only principal"
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/payer.rs",
      "line": 530,
      "description": "the payer's own fee model: `needed = needed.saturating_add(10_000 * to_pay.len() as u64)` \u2014 comment: one ZIP-317 fee per payment at 2 actions; the runner's fee_budget_zat models per-invoice settlement the same way (flow.rs:1165-1177, 10_000 zatoshis each)"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 119,
      "description": "the budget rule compares only spent+amount (120) against budget_zat \u2014 fees never enter the comparison"
    },
    {
      "file": "crates/properties/src/properties/payer_policy.rs",
      "line": 77,
      "description": "payer-policy re-derives `spent` the same way (77-90), so a run that pays 1,000 x 1-zat invoices passes with '1000 of 1000 zat spent' (re-executed on the real property)"
    },
    {
      "file": "crates/observation/src/lib.rs",
      "line": 234,
      "description": "PayerRun::spent_zat() (doc line 233: 'Zatoshis actually spent under this grant') sums paid decision amounts only \u2014 the model has no field for the run's real outflow (the ledger summary at payer.rs:428-440 computes the same sum inline)"
    },
    {
      "file": "crates/probe/src/flow.rs",
      "line": 1544,
      "description": "Invoice.amount_zat: u64 with no minimum, so 1-zat invoices (the maximum-fee-leverage shape) are valid invoices"
    },
    {
      "file": "crates/probe/src/ops.rs",
      "line": 59,
      "description": "CONFIRM_BLOCKS = 11: every invoice also costs a mined-block round plus a wallet sync"
    }
  ],
  "conditions": [
    {
      "kind": "data_state",
      "description": "the invoice directory holds many small invoices (e.g. 1 zat each) from a receiver on the allowlist, each within the per-payment cap"
    },
    {
      "kind": "system_configuration",
      "description": "the grant's budget and cap are the run's only spend bounds; shortfall/top_up_for_target fund the fee headroom from outside the grant"
    },
    {
      "kind": "environmental_dependency",
      "description": "each payment costs a ZIP-317 fee (10,000 zat per payment in the payer's own budget model) and one 11-block mining round"
    }
  ],
  "execution": {
    "attacker_perspective": "the merchant issuing invoices: many tiny allowlisted invoices turn a 1,000-zat budget into ~10,000,000 zat of operator fees plus one broadcast and 11 mined blocks each",
    "payloads": [
      "1,000 invoice files of 1 zat each to an allowlisted receiver under grant {budget_zat:1000, per_payment_cap_zat:400}",
      "12,000,000 one-zat invoices under the demo grant defaults (budget 12,000,000 / cap 10,000,000 in scripts/bounded-payer-demo.sh)"
    ],
    "instructions": [
      "In the disposable sandbox copy, add crates/probe/src/verifier12_check.rs and `#[cfg(test)] mod verifier12_check;` to crates/probe/src/main.rs (the module only calls the target's own payer::decide and PayerPolicy::check; no production line changed), then run: cargo test --offline -p zanary-probe --bin zanary-probe -- verifier12 --nocapture --test-threads=1",
      "The same shape as the hunter's H3 run: cargo test --offline -p zanary-probe --bin zanary-probe -- hunter_payer --nocapture --test-threads=1",
      "Measuring the chain-level fee itself needs the regtest rig (out of sandbox bounds); this record does not claim that measurement"
    ],
    "observed_result": "Reproduced by the verifier in the sandbox (rust:1-slim, --network=none): the real payer::decide admitted exactly 1,000 payments of 1 zat under a 1,000-zat / 400-cap grant (spent 1,000) and refused the 1,001st with 'refused: paying 1 zat would take this run to 1001 of a 1000 zat budget'; the real payer-policy property PASSED a 1,000-row paid ledger with summary '1000 paid \u00b7 0 refused \u00b7 1000 of 1000 zat spent'; PayerRun::spent_zat() = 1,000. With the payer's own funding model (10,000 zat per payment, payer.rs:530, matching flow.rs:1170) that run tops up 10,000,000 zat of fees from outside the grant. The decision-layer result was executed; the fee magnitude is the payer's own model, not a chain observation."
  },
  "remediation": {
    "strategy": "denominate the budget in what the run actually spends: add the per-payment cost (ZIP-317 fee estimate) to the budget comparison in decide() and to payer-policy's re-derivation, and give the grant a bounded cardinality (a `max_payments`/`max_invoices` field or a minimum invoice amount) so a merchant cannot multiply the operator's fixed per-payment cost (fee + 11 mined blocks + a wallet sync) arbitrarily.",
    "code_changes": [
      {
        "file_name": "crates/probe/src/payer.rs",
        "fixed_code": "/// ZIP-317 fee budget for one 2-action payment (the same per-payment figure shortfall() uses at 530).\npub const PAYMENT_FEE_ZAT: u64 = 10_000;\n// in decide():\nlet after = spent_zat\n    .saturating_add(invoice.amount_zat)\n    .saturating_add(PAYMENT_FEE_ZAT);\nif after > grant.budget_zat {\n    return Decision { allowed: false, reason: format!(\n        \"refused: paying {} zat (+ {} zat fee) would take this run to {} of a {} zat budget\",\n        invoice.amount_zat, PAYMENT_FEE_ZAT, after, grant.budget_zat) };\n}\n// and bound the cardinality: e.g. refuse once `paid >= grant.max_payments`,\n// with a `max_payments` field (serde default = unlimited) added to PayerGrant."
      },
      {
        "file_name": "crates/properties/src/properties/payer_policy.rs",
        "fixed_code": "// keep the detector in step with the rule it judges:\nspent = spent\n    .saturating_add(decision.amount_zat)\n    .saturating_add(PAYMENT_FEE_ZAT);\n// then the existing `if spent > grant.budget_zat { violations.push(...) }`\n// fires for the fee-multiplied ledger, and a `paid > grant.max_payments`\n// check covers the count bound."
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "high",
      "reason": "trivially reachable by the merchant's own invoice files; every allowed invoice independently adds a fixed cost the grant never counts, and no source-visible bound limits the count, size or amount of invoices"
    },
    "impact": {
      "score": "medium",
      "reason": "operator-owned spend (fees, funding mined to cover them) and node work far beyond the granted budget while ledger, report and payer-policy all show the grant respected; on this target the chain is regtest and the money is minted, so the concrete loss is bounded to the demo"
    },
    "overall_severity": "medium"
  },
  "confidence": {
    "score": "medium",
    "reason": "the decision-layer half (1,000 x 1-zat invoices allowed; ledger/property report 1,000 of 1,000) was re-executed by the verifier on the real decide() and the real payer-policy property; the fee magnitude is the payer's own funding constant (payer.rs:530) corroborated by flow.rs:1170 rather than a chain observation, which a rerun on the rig could measure exactly"
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-31/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-31/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "zanary/payer-budget-excludes-fees"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
