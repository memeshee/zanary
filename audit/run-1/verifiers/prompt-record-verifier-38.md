# Phase-5 record verifier — verifier-38 on `zanary/unbounded-jitter-block-mining`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`needs_validation`)

```json
{
  "verdict": "needs_validation",
  "fingerprint": "zanary/unbounded-jitter-block-mining",
  "title": "flow definition jitter_blocks has no upper bound: one accepted line names a mining request of up to 4,294,967,295 blocks, drawn per payment and sent as a single uncancellable `generate` RPC",
  "description": "`jitter_blocks: [min, max]` is accepted for any 1 <= min <= max, so a three-field flow definition can name a mining request of up to u32::MAX blocks (flow.rs:133, 166-176). Every draw is the argument to `ops::mine_and_resync(payer, jitter.next())` (flow.rs:645 per invoice, flow.rs:883 for a batch) and reaches zcashd's `generate` RPC as one call (ops.rs:436 -> node.rs:232); the only client-side bound is the 300 s response-read timeout (node.rs:122-125), which abandons the response and sends nothing that could cancel the work. Re-established independently in the approved sandbox: a serde-accepted definition `{\"jitter_blocks\":[1,4294967295],\"jitter_seed\":7}` drew 32 values with mean 1,791,207,489 and maximum 4,085,077,839 blocks in a single call, the first draw alone 1,789,459,390 \u2014 so the oversized request is the first payment's, not a tail event (matches the hunter's artifact draw-for-draw). What remains undecided from source or bounded local evidence is entirely node-side: zcashd's `generate` is not part of this repository, so whether it honours a count above the 32-bit signed range, whether it keeps generating after the client's socket closes, and what each block costs in time and disk on the shared host are unknown \u2014 and they decide the severity.",
  "claimed_root_cause": "Jitter::from_definition (flow.rs:166-176) bounds only the shape of the jitter range (1 <= min <= max; malformed ranges fall back to a fixed 3) and never bounds the work the range can name; the drawn value is passed unscaled to the node's `generate` RPC (ops.rs:436 -> node.rs:232), whose execution is not cancellable from the probe's side (the 300 s client read timeout at node.rs:122-125 abandons the response, not the server-side work; no cancel/shutdown path exists anywhere in the probe).",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/flow.rs",
      "line": 133,
      "scope": "FlowDefinition::jitter_blocks",
      "description": "the jitter range arrives as caller-controlled JSON (Option<[u32; 2]>, serde default, no validator); no upper bound is applied at deserialization, and u32::MAX is the largest count a definition can name"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/flow.rs",
      "line": 166,
      "scope": "Jitter::from_definition",
      "description": "`Some([min, max]) if min <= max && min >= 1 => (min, max, None)` accepts the range verbatim (line 167); only malformed ranges are bounded, by the fixed-3 fallback at lines 168-176"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/flow.rs",
      "line": 192,
      "scope": "Jitter::next",
      "description": "`self.rng.gen_range(self.min..=self.max)` yields a count up to 4,294,967,295; recorded in jitter.drawn (line 195) and cast to u64 (line 196), never clamped"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/flow.rs",
      "line": 645,
      "scope": "settle_per_invoice",
      "description": "`ops::mine_and_resync(payer, jitter.next())` runs after every payment (batch path: one call at flow.rs:883 in settle_batch)"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/ops.rs",
      "line": 436,
      "scope": "mine_and_resync",
      "description": "the drawn count becomes the one argument of `node.generate(blocks)` (mine_and_resync, ops.rs:433)"
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/node.rs",
      "line": 232,
      "scope": "NodeRpc::generate",
      "description": "the count is the single parameter of the zcashd `generate` RPC, sent as one raw HTTP request (body at node.rs:80-92); this client's only bound is the 300 s read timeout at node.rs:122-125, which abandons the response without sending any cancellation"
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/flow.rs",
      "line": 166,
      "description": "the acceptance rule for jitter_blocks: min <= max and min >= 1 \u2014 no maximum magnitude, no relationship to chain height or run length; the definition-level validation block (flow.rs:253-297) never inspects the field"
    },
    {
      "file": "crates/probe/src/flow.rs",
      "line": 645,
      "description": "per-invoice mining call (batch equivalent flow.rs:883); the draw is the only thing controlling the number of blocks"
    },
    {
      "file": "crates/probe/src/node.rs",
      "line": 122,
      "description": "the 300 s response-read timeout (connect is bounded at 30 s, node.rs:94): on timeout the client errors and closes the request whose server-side work continues by construction; no cancel/shutdown path exists in the probe (grep: the only 'kill' is the payer grant's unrelated --kill-file)"
    },
    {
      "file": "crates/probe/src/flow.rs",
      "line": 192,
      "description": "observed locally (verifier-19, approved sandbox, disposable copy restored to HEAD sha afterwards): definition jitter_blocks [1,4294967295] + jitter_seed 7 is accepted and 32 draws give max 4,085,077,839, mean 1,791,207,489, first draw 1,789,459,390 \u2014 matching agents/hunter-inputs/artifacts/check-jitter-draw.txt draw-for-draw; raw run: agents/verifier-19/artifacts/check-jitter-independent.txt. Correction to the hunter's arithmetic: the 32-draw mean is 1.79e9, not ~1.9e9. Also: [4294967295,4294967295] is accepted and draws exactly 4,294,967,295; malformed [2,1] falls back to fixed 3"
    },
    {
      "file": "action.yml",
      "line": 113,
      "description": "the composite action wraps the gate in `timeout \"$GATE_TIMEOUT\"` (default 2700 s; 124 -> exit 2 at action.yml:122-124), and the gate reaches this same Jitter path via gate.rs:133 flow::run_recording \u2014 it bounds the probe's own exposure but sends nothing to the node, so the mining it asked for is unaffected by killing the probe"
    }
  ],
  "blockers": [
    "zcashd's `generate` implementation is not part of this repository (harness/docker-compose.yml pins the electriccoinco/zcashd image), so whether it honours a block count above the 32-bit signed range, and whether generation continues after the client's 300 s read timeout closes the socket, cannot be decided from source or from bounded local evidence \u2014 and that answer decides whether the effect is a rejected request, a hung RPC, or a host-level drain.",
    "Per-block mining cost and disk growth are runtime facts of the running node: this run's protocol puts the rig's containers and host ports (zanary-zcashd, 18232/18233) out of bounds, the approved sandbox is --network=none with rust:1-slim only, and image pulls are not permitted \u2014 so no node can be started or reached to measure them.",
    "The named work is finite (<= 4,294,967,295 blocks per call, one call per payment), so the maximum is bounded in principle; what stays unknown is the node's actual rate and whether that work survives client abandonment, which is what a severity statement would need."
  ],
  "validation_plan": {
    "local": "Runnable only where a disposable node is allowed (not this run's sandbox, which has no node and no network). Create a NEW throwaway container from the locally pinned zcashd image \u2014 never the harness's compose project, no published host ports, its own volume \u2014 on a fresh container-internal network, and point ZANARY_NODE_RPC_URL at it from a client on that same network. Then through the target's own code: (a) call NodeRpc::generate with a small count (e.g. 2,000) and confirm one request is emitted and honoured (getblockcount advances by 2,000); (b) start one generate with a count large enough to outlast a forced abort (or SIGKILL the probe process mid-call) and poll getblockcount from inside the node container to see whether height keeps climbing after the client socket closes; (c) record the node volume's size before and after. Bound the whole check to a couple of minutes, never request the drawn ~4.29e9, and delete the container and its volume afterwards.",
    "deployment": "The rig owner (not this verifier) repeats (b)/(c) on a DISPOSABLE node built from the harness image to measure per-block time and disk growth for a bounded count, then extrapolates the cost of 4,294,967,295 blocks arithmetically instead of letting the real request run \u2014 a giant `generate` against a retained rig volume could fill the disk, so the full draw must never be executed against shared rig state. If the end-to-end path must be shown, run the gate once with a definition jitter_blocks [1, 4294967295] against a throwaway node whose volume is discarded, guarded by the node's disk headroom and the action's 2700 s timeout, and watch getblockcount in a bounded window afterwards."
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-38/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-38/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "zanary/unbounded-jitter-block-mining"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
