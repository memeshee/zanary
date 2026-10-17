# Phase-5 record verifier — verifier-32 on `zanary/payer-duplicate-invoice-id-replay`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`confirmed`)

```json
{
  "verdict": "confirmed",
  "fingerprint": "zanary/payer-duplicate-invoice-id-replay",
  "title": "Invoice identity is a label, not a key: two invoice files carrying one invoice_id are both decided ALLOW and queued for payment in a single run, the run record puts the second payment's txid on the first row, and payer-policy passes the duplicated ledger",
  "description": "Nothing in the payer keys on invoice_id: load_invoices loads every *.json in the merchant-supplied directory (payer.rs:495-520; verifier-reproduced: two files carrying one id load as two invoices), decide() never consults it (84-136, it appears only inside reason text), the decision loop stores it as a label (262) and the repo has no dedupe, seen-set or uniqueness rule anywhere (grep over crates/). Two invoice files carrying the same invoice_id - a re-issued, corrected or re-sent invoice, which the merchant writes as an ordinary file ('issue an invoice the payer can act on with no channel other than the file', flow.rs:1554-1555) - are therefore two ordinary invoices: the production loop allows both while the cumulative budget holds and pushes both to to_pay, charging the budget twice (verifier-reproduced through the real CLI: two ALLOW lines for v13-8301, '300 of 1000' then '600 of 1000'; and with the loop's own statements: to_pay 2 entries, spent 600). The payment loop (291-363) pays every to_pay entry with no dedupe and the funding covers every entry including the duplicate (523-533), so the same invoice is paid twice in one run; the two shielded broadcasts themselves need the regtest rig and were not executed here. The attribution at 320-326 finds the FIRST row with that invoice_id, so the second payment's txid/height land on the first row while the second row is persisted with none (verifier-reproduced on the real PayerDecision type; a control with distinct ids attributes correctly) - ledger.jsonl, written per payment at 327-338, keeps its own two rows. payer-policy then passes the duplicated ledger because it has no notion of invoice identity (payer_policy.rs:63; verifier-reproduced: Pass, '2 paid - 0 refused - 600 of 1000 zat spent'), and the only property that reacts to this shape at all is amount-fingerprint's amount-repeat warning when the duplicate carries the same amount (it names equal amounts, never the repeated id, and fires only after both payments and the artifacts are written). The merchant is paid twice for one invoice and the run's payer ledger cannot be reconciled row-for-row against the chain.",
  "root_cause": "invoice_id is never treated as an identity key anywhere in the authority model: no dedupe in load_invoices (495-520) or decide (84-136), no persisted set of decided/paid ids in PayerRun (observation:224-230 holds only label + grant + decisions), no uniqueness rule in payer-policy (payer_policy.rs:63-103); the only lookup that uses the id as a key is decisions.iter_mut().find() for txid attribution (320-326), which is also why a repeat corrupts the attribution. A fresh `spent = 0` per invocation (247) compounds it across runs.",
  "intended_behavior": "One invoice is one payment: 'Every invoice produces exactly one ledger line - paid or refused - with the rule in it, the amount, the receiver, and the txid when there is one' (docs/BOUNDED-PAYER.md:50-52). A row that is paid must name the txid of the payment that settled it.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/payer.rs",
      "line": 495,
      "scope": "load_invoices",
      "description": "every *.json entry in the merchant-supplied directory is parsed into an Invoice with no count cap and no invoice_id uniqueness check (503 selects by extension only, 511-518 parses); verifier-reproduced: two files carrying v13-8301 load as two invoices"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 250,
      "scope": "decision loop",
      "description": "each invoice - including a second file repeating an id - is decided independently by decide(); the id is stored as a label at 262; verifier-reproduced through the real CLI: two ALLOW lines for v13-8301, budget 300 then 600 of 1000"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 258,
      "scope": "to_pay",
      "description": "the allowed amount is added to `spent` and the invoice is pushed to to_pay (259), so a repeated id consumes the budget twice; verifier-reproduced with the loop's statements: to_pay 2 entries, spent 600, both decisions 'paid'"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 291,
      "scope": "payment loop",
      "description": "every to_pay entry is proposed, signed and broadcast as its own shielded payment with no dedupe or per-entry guard; this is the unexecuted step (the wallet path needs the regtest rig) and it follows unconditionally from the loop plus the funding at 523-533"
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/payer.rs",
      "line": 322,
      "scope": "txid attribution",
      "description": "`decisions.iter_mut().find(|d| d.invoice_id == invoice.invoice_id)` returns the FIRST row with that id, so the later payment's txid/height overwrite the earlier row and the later row keeps none; verifier-reproduced on the real PayerDecision type (control with distinct ids attributes correctly)"
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/payer.rs",
      "line": 262,
      "description": "invoice_id is copied into each decision as a label; the file's only use of it as a lookup key is the find() at 320-326 (the other mentions are prints, the ledger row and the payment label)"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 322,
      "description": "attribution by find() on invoice_id - the exact predicate; over two payments sharing v13-8301 it leaves row 0 holding the second txid/height and row 1 without any (verifier-reproduced)"
    },
    {
      "file": "crates/properties/src/properties/payer_policy.rs",
      "line": 63,
      "description": "the property iterates decisions with no notion of invoice identity; the real check returns Pass for two paid rows sharing v13-8301 ('2 paid - 0 refused - 600 of 1000 zat spent', both rows echoed, one 'no txid')"
    },
    {
      "file": "crates/properties/src/properties/amount_fingerprint.rs",
      "line": 70,
      "description": "the only property that reacts to this shape flags the equal amounts (W: 'exact repeat across payments'), never the repeated id; evaluated over the duplicated ledger it is the run's only failure, and the run would abort on it only after both payments and after the artifacts are written (payer.rs:406-411, 479-486)"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 247,
      "description": "`let mut spent: u64 = 0;` - a fresh invocation re-decides the whole directory on a zero budget; the second identical CLI run printed the same 300 then 600 sequence"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 530,
      "description": "the payer funds for every to_pay entry including a duplicate (10,000 zat ZIP-317 fee per entry; shortfall over to_pay, 523-533), so both payments are funded by design"
    },
    {
      "file": "crates/observation/src/lib.rs",
      "line": 224,
      "description": "PayerRun holds label + grant + decisions only (224-230): no persisted set of decided or paid invoice ids to dedupe against across runs"
    },
    {
      "file": "crates/probe/src/flow.rs",
      "line": 1554,
      "description": "the merchant side issues invoices with no channel other than the file ('issue an invoice the payer can act on with no channel other than the file', 1554-1555), so a re-issued invoice is a natural second file"
    }
  ],
  "conditions": [
    {
      "kind": "data_state",
      "description": "two invoice JSON files in the --invoices directory carry the same invoice_id (a re-issued, corrected or re-sent invoice); each amount is within the per-payment cap and their sum within the budget"
    },
    {
      "kind": "system_configuration",
      "description": "the payer is pointed at that directory; nothing rejects duplicates (no dedupe in intake, decide or the payment loop) and a second invocation starts from spent = 0, so the same directory is decided - and paid - again"
    }
  ],
  "execution": {
    "attacker_perspective": "the merchant, who writes the invoice files the payer is pointed at: it needs only the ability to place two *.json files in the --invoices directory. No access to the payer, its wallet or the grant is needed; the duplicate is an ordinary re-issued or re-sent invoice.",
    "payloads": [
      "inv-a.json and inv-b.json, byte-identical: {\"invoice_id\":\"v13-8301\",\"amount_zat\":300,\"receiver\":\"uregtest1merchant\",\"memo\":\"zanary: invoice v13-8301\",\"uri\":\"zcash:?address=uregtest1merchant&amount=0.000003\",\"issued_at_height\":1,\"merchant_account\":0}",
      "the same directory settled again by a second, identical invocation (fresh spent = 0; nothing consults the first run's ledger)"
    ],
    "instructions": [
      "rebuild the disposable copy of the reviewed revision (git archive HEAD | tar -x, rust-toolchain.toml removed) and add the verifier's module crates/probe/src/payer.rs#mod verifier13_checks; production lines 1-655 of that file stay byte-identical to the worktree (diff empty)",
      "docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 -v <copy>:/t -v ~/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim sh -c 'CARGO_BUILD_JOBS=6 cargo test --offline -p zanary-probe --bin zanary-probe verifier13 -- --nocapture --test-threads=1'  (real load_invoices, the production loop's statements, the loop's attribution predicate, the real PayerPolicy::check and the full real property report; 5/5 passed)",
      "in the same sandbox: cargo build --offline -p zanary-probe --bin zanary-probe, then ./target/debug/zanary-probe payer --grant /t/v13-scratch/grant.json --invoices /t/v13-scratch/invoices --out /t/v13-scratch/out (grant: budget 1000, cap 400, allowlist [uregtest1merchant], expires 2030-01-01T00:00:00Z) - run it twice"
    ],
    "observed_result": "In the approved sandbox (--network=none, disposable copy, production lines byte-identical to the target): (a) real load_invoices reads 2 invoices, ids [\"v13-8301\",\"v13-8301\"] (V13-1); (b) the real CLI payer over that directory prints ALLOW twice for v13-8301 - 'paid: within the cap (400 zat) and the budget (300 of 1000 zat spent after this)' then '600 of 1000' - then aborts at the node-RPC bootstrap (exit 1, --out never created); a second identical invocation prints the same 300 then 600 sequence (V13-CLI); (c) the production loop's statements queue both (to_pay 2 entries, spent 600, both decisions 'paid') (V13-2); (d) the loop's own attribution predicate over the real PayerDecision vector puts the second payment's txid/height on row 0 and leaves row 1 with none (control with distinct ids correct) (V13-3); (e) the real PayerPolicy::check returns Pass for the duplicated ledger ('2 paid - 0 refused - 600 of 1000 zat spent') (V13-4); (f) the full real property report over the duplicated ledger passes payer-policy, receiver-reuse, memo-absent, memo-content and the pool/policy properties, and FAILs only amount-fingerprint (W, 'exact repeat across payments' on the equal amounts) (V13-5). NOT executed: the two shielded broadcasts and the persisted run record of a real run - the payer's wallet path needs the regtest rig's funded synced wallet plus node/indexer, which is out of bounds; they are the unconditional continuation of the loop at 291-363 with funding for every entry (523-533)."
  },
  "remediation": {
    "strategy": "Treat invoice_id as a key in the authority model: refuse - and record - a second decision for an id already decided in the run, persist the decided/paid ids next to the ledger so a re-run over the same --out refuses ids already paid, and attribute txid/height by position (or a map keyed by (invoice_id, occurrence)) instead of find(), so a repeat cannot overwrite the first row.",
    "code_changes": [
      {
        "file_name": "crates/probe/src/payer.rs",
        "fixed_code": "let mut decided: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();\nlet mut to_pay: Vec<(usize, Invoice, String)> = Vec::new(); // decision index first\nfor (index, invoice) in invoices.iter().enumerate() {\n    if !decided.insert(invoice.invoice_id.clone()) {\n        decisions.push(PayerDecision {\n            invoice_id: invoice.invoice_id.clone(),\n            amount_zat: invoice.amount_zat,\n            receiver: invoice.receiver.clone(),\n            decision: \"refused\".to_string(),\n            reason: format!(\"refused: invoice {} was already decided this run\", invoice.invoice_id),\n            txid: None,\n            height: None,\n        });\n        continue;\n    }\n    // ... the existing decide()/to_pay path, now carrying `index` ...\n}\n// in the payment loop: for (index, (invoice, allowed_reason)) in to_pay.iter().enumerate() {\n//     ... propose/sign/broadcast ...\n//     decisions[*index].txid = Some(txid.to_string());\n//     decisions[*index].height = height;\n// }\n// plus: before deciding, read the id set persisted next to --out and refuse ids already paid."
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "medium",
      "reason": "the merchant writes the invoice files the payer is pointed at, and a re-issued, corrected or re-sent invoice is ordinary; nothing in intake, decide, the payment loop or payer-policy keys on the id, and a second invocation re-decides the same directory from spent = 0. The one incidental signal in this shape (amount-fingerprint's amount-repeat warning, W) fires only when the duplicate carries the same amount, names the amounts rather than the duplicate id, and arrives after the payments."
    },
    "impact": {
      "score": "medium",
      "reason": "the merchant is paid twice for one invoice, bounded by the grant's budget; the run record's payer ledger shows two paid rows for one id with the second payment's txid on the first row and none on the second, so it cannot be reconciled row-for-row against the chain (ledger.jsonl keeps its own per-payment txids), and payer-policy - the boundary meant to judge the payer - passes."
    },
    "overall_severity": "medium"
  },
  "confidence": {
    "score": "high",
    "reason": "every layer short of the shielded broadcast was reproduced with the target's own code (real intake via the real CLI, the production loop's statements, the loop's own attribution predicate, real PayerPolicy::check, the full real property report), and the broadcast is the unconditional per-entry continuation of payer.rs:291-363; the regtest rig is the only place the two transactions and the persisted run record can be observed end to end."
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-32/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-32/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "zanary/payer-duplicate-invoice-id-replay"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
