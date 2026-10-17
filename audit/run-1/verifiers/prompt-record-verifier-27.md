# Phase-5 record verifier — verifier-27 on `probe/payer.rs:run:unbounded-payment-count`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`needs_validation`)

```json
{
  "verdict": "needs_validation",
  "fingerprint": "probe/payer.rs:run:unbounded-payment-count",
  "title": "Bounded payer queues an unbounded number of payments: the grant bounds the sum of amounts, not the count of heavy per-invoice work",
  "description": "run() appends every invoice decide() allows to to_pay with no cardinality cap (payer.rs:249-260) and then pays them one at a time (payer.rs:291-309). Each queued entry costs fixed heavy work: a proposal (ops.rs:884-891), an Orchard build/prove/sign and broadcast through the indexer (ops.rs:360-428), then 11 mined blocks plus a wallet resync (payer.rs:309 -> ops.rs:433-451; CONFIRM_BLOCKS = 11 at ops.rs:59). The grant bounds only each amount and the running sum of amount_zat (payer.rs:110-128), so zero-amount invoices never consume the 50,000,000-zat default budget (payer.rs:64-66) and one-zat invoices fit up to 50,000,000 payments, while the attacker's per-payment input is one small JSON file (~140 B). Independently reproduced from a fresh build of the pinned revision inside the sandbox: 5,000 zero-amount invoices were all read, decided ALLOW and queued in one run ('invoices : 5000', 5,000 ALLOW lines, ~1 s), and a 300-file one-zat run with budget 150 admitted exactly 150 (the sum-of-amounts budget was the only count-limiting check). The run then requires the harness node (payer.rs:275-276) and stops with 'Connection refused' in --network=none, so the payment work itself, its per-payment cost and the effect of N sequential payments on the shared regtest node are owner-rig observations this audit cannot make.",
  "claimed_root_cause": "No bound on the number of invoices accepted into payment work: run() queues every allowed invoice (payer.rs:249-260) and pays them in a serial loop with no per-run payment-count cap (payer.rs:291-309); the grant bounds only amounts (payer.rs:110-128), and each queued entry is fixed heavy work (proposal, Orchard proof, broadcast, 11 mined blocks + wallet resync; ops.rs:59, 360-428, 433-451).",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/payer.rs",
      "line": 150,
      "scope": "payer::run",
      "description": "--invoices directory supplies the invoice count; load_invoices (payer.rs:495-520) collects every *.json entry, so the count is caller/merchant-chosen"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 250,
      "scope": "payer::run",
      "description": "decide() is the only gate: per-payment cap (110-117) and sum-of-amounts budget (119-128) are amount checks only"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 259,
      "scope": "payer::run",
      "description": "every ALLOWed invoice is appended to to_pay with no cardinality check (loop 249-260)"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 307,
      "scope": "payer::run",
      "description": "payment loop (291-309) is serial over the queue: propose_shielded_payment (306-307), build_sign_broadcast (308), then mine_and_resync(CONFIRM_BLOCKS) (309) per queued entry; no payment-count bound"
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/ops.rs",
      "line": 436,
      "scope": "ops::mine_and_resync (called per payment from payer.rs:309)",
      "description": "node.generate(CONFIRM_BLOCKS=11) mines 11 blocks on the harness node then core::sync_wallet re-syncs (ops.rs:433-451) before the next payment starts - serial heavy work per tiny attacker file"
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/payer.rs",
      "line": 259,
      "description": "single append site into the payment queue (loop 249-260); no cap on to_pay length exists anywhere in crates/probe/src (no max/take/limit applies to the invoice queue)"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 119,
      "description": "budget check is amount-only: after = spent.saturating_add(invoice.amount_zat); after > budget; amount_zat 0 keeps after at 0 forever (unbounded count), amount_zat 1 allows up to the 50,000,000-zat default budget (payer.rs:64-66)"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 291,
      "description": "payment loop over to_pay: one proposal + build/prove/broadcast + mine_and_resync per queued invoice, serially, with no count bound"
    },
    {
      "file": "crates/probe/src/ops.rs",
      "line": 59,
      "description": "CONFIRM_BLOCKS = 11: each payment mines 11 blocks and re-syncs the wallet (core::sync_wallet at ops.rs:444); proof/sign/broadcast at ops.rs:360-428"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 503,
      "description": "load_invoices selects by extension only and collects every *.json entry (498-506) with no file-count cap, so the queue length is attacker-chosen"
    }
  ],
  "blockers": [
    "Payment-loop execution requires the harness regtest rig, which is out of bounds for this run: the first payment work is reached only after the node connection (NodeRpc::from_env -> node.block_count(), payer.rs:275-276) and needs the harness zcashd (127.0.0.1:18232) plus the lightwalletd indexer (9067) for wallet bootstrap and broadcast; the sandbox is --network=none and the rig's containers/host ports are shared host state explicitly excluded, so per-payment cost (proposal + Orchard proof + broadcast + 11 mined blocks + wallet resync) and the effect of N sequential payments on the shared node could not be measured.",
    "No offline substitute exercises ops::build_sign_broadcast / ops::mine_and_resync: both need a synced probe wallet and node JSON-RPC; running a mock service is itself out of bounds (any network service) and could not reproduce real proving/broadcast/mine semantics, so the queued payment work remains unvalidated locally; the local evidence covers intake, decisions and queue construction only."
  ],
  "validation_plan": {
    "local": "On the owner's own rig (harness up: zcashd 127.0.0.1:18232, lightwalletd 9067), pinned revision built, disposable run directory and throwaway wallet state: (1) create ~20 one-zat invoices (zanary-probe invoice --out <dir>/<n>.json --amount-zec 0.00000001, or hand-written JSON with amount_zat 1) and a default grant (budget_zat 50000000, per_payment_cap_zat 10000000, empty allowlist, future expiry); (2) record getblockcount and wall clock, run 'zanary-probe payer --grant <grant.json> --invoices <dir> --out <disposable-run> --now <now> --expect-refused 0', then record wall clock, the height delta, ledger.jsonl line count and the report's payment count; (3) repeat with a single invoice for the per-payment cost and with zero-amount invoices to show the budget never binds in count; (4) note the number of payments actually settled versus queued (each payment consumes the ZIP-317 fee; shortfall() tops up by mining when the amount sum is nonzero, payer.rs:523-533) and extrapolate per-payment cost x N for plausible N. Non-destructive: disposable regtest rig and run dirs only (the chain advances ~11 blocks per payment), no repo or shared-state changes.",
    "deployment": "Observe the shared-node blast radius and recovery on the owner's running rig: with another rig workload (flow runner/demo) active, feed a medium directory (e.g. 200 tiny invoices) through the payer and observe contention on zcashd (block generation, wallet sync latency, other runs failing or timing out) and chain growth (11 blocks per payment); then check recovery: the kill switch is read only at startup (payer.rs:185-191), so touching --kill-file after the payment loop starts should not stop it - confirm whether only killing the process ends the run, and what the rig needs afterwards (fresh wallet resync or shortfall top-up). Non-destructive: disposable rig instance, disposable run dirs, no repo or shared-state changes."
  }
}
```

## Linked ledger unit

- coverage_id: `crates%2Fprobe%2Fsrc%2Fpayer.rs%23load-invoices::merchant-supplied%20invoice%20directory%20-%3E%20payer%20memory%20and%20payment%20work::bounded-payer::RESOURCE-EXHAUSTION-AND-AVAILABILITY.md%23Resource%20accumulation%20attack%20classes`
- surface: `crates/probe/src/payer.rs#load-invoices` · subsystem `bounded-payer`
- boundary: `merchant-supplied invoice directory -> payer memory and payment work`
- assignment owner: `hunter-invoices`

The unit's checks (each `artifact` path is relative to the run directory; read the promoted files
under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/`):

```json
[
  {
    "agent_id": "hunter-invoices",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/probe/src/ops.rs"
    ],
    "invariant": "load_invoices bounds what a merchant-supplied directory can make the payer read (file count, file size) and the payment loop bounds its own cardinality; grant budget must bound work as well as money",
    "method": "source",
    "result": "No count cap (read_dir collects every *.json entry), no size cap (read_to_string per file, no metadata check), no payment-count cap (to_pay grows per allowed invoice; budget bounds only sum(amount_zat)); intake (payer.rs:201) runs before the first decide() (payer.rs:250) and is not conditional on revoked/kill-file state",
    "artifact": null
  },
  {
    "agent_id": "hunter-invoices",
    "reviewed_paths": [
      "crates/probe/src/payer.rs"
    ],
    "invariant": "invoice entries are regular files inside the supplied directory: no symlink following, no blocking on non-regular files, read has a deadline",
    "method": "source",
    "result": "Selection is by extension only (payer.rs:503); read_to_string (payer.rs:513) follows symlinks and blocks in open() on a FIFO; no symlink_metadata/is_file check, no O_NOFOLLOW, no timeout in the intake",
    "artifact": null
  },
  {
    "agent_id": "hunter-invoices",
    "reviewed_paths": [
      "crates/probe/src/payer.rs"
    ],
    "invariant": "memory intake scales with an operator-chosen bound, not with merchant-controlled file bytes; intake must not run for void authority",
    "method": "local",
    "result": "256 MiB invoice -> peak RSS 526 MB (revoked grant: intake still completed, REFUSE printed) / 527 MB (full grant); 768 MiB invoice under ulimit -v 1200000 -> exit 1, 'Error: reading the invoice ... Caused by: out of memory', zero decisions; same limit with tiny and 256 MiB fixtures completes intake",
    "artifact": "agents/hunter-invoices/artifacts/check-memory-intake.txt"
  },
  {
    "agent_id": "hunter-invoices",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/probe/src/ops.rs"
    ],
    "invariant": "the number of invoices accepted into payment work is bounded (queue cardinality / per-run payment cap)",
    "method": "local",
    "result": "5000 zero-amount invoice files all read, decided ALLOW and queued in one run (5000 ALLOW lines, 'invoices : 5000'); each queued entry costs a proposal + broadcast + 11 mined blocks + resync (payer.rs:307-309, ops.rs:59); the payment phase itself could not run offline",
    "artifact": "agents/hunter-invoices/artifacts/check-count-queue.txt"
  },
  {
    "agent_id": "hunter-invoices",
    "reviewed_paths": [
      "crates/probe/src/payer.rs"
    ],
    "invariant": "non-regular and symlinked *.json entries cannot hang the payer or be read from outside the supplied directory",
    "method": "local",
    "result": "FIFO named hang.json (with a revoked grant): 0 bytes of output for 15 s until the harness killed it (exit 137) - blocked in intake before any decision; symlink link.json -> file outside --invoices: the outside invoice was loaded and ALLOWed",
    "artifact": "agents/hunter-invoices/artifacts/check-filetype.txt"
  }
]
```

## What to check for this record

1. The source path is real and supports only the `claimed_root_cause` stated.
2. Every listed blocker is decisive and not already answerable locally.
3. The candidate names a boundary and a possible concrete result rather than a generic concern.
4. At least one `validation_plan` field is present and exact: `local` uses a bounded fixture; `deployment` asks the owner to observe a configuration, identity, route, policy or runtime fact. Do not invent a plan for an inapplicable context, and never send audit traffic to a deployment.
5. The fingerprint matches prior/current records for the same root cause.

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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-27/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-27/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "probe/payer.rs:run:unbounded-payment-count"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
