# Phase-5 record verifier — verifier-25 on `probe/payer.rs:load_invoices:nonregular-entry`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`confirmed`)

```json
{
  "verdict": "confirmed",
  "fingerprint": "probe/payer.rs:load_invoices:nonregular-entry",
  "title": "Bounded payer follows symlinks and blocks forever on non-regular *.json entries in the merchant-supplied invoice directory",
  "description": "load_invoices (crates/probe/src/payer.rs:495-520) selects entries by the json extension alone (line 503) and reads each with std::fs::read_to_string (line 513). A symlinked *.json is followed and read from outside the supplied directory (verifier-reproduced: an invoice living outside --invoices was loaded, decided and queued), and a FIFO named *.json makes the process block in open() indefinitely (verifier-reproduced: the process was alive and sleeping with kernel wchan wait_for_partner after 5 s and 10 s, zero bytes of output, SIGKILL at ~10 s and exit 137) - there is no regular-file check, no O_NOFOLLOW and no timeout anywhere in intake. A party able to create one entry in the directory can therefore hang the payer before any authority evaluation (even a revoked grant never reaches its refusal) and can make the payer read and queue a file outside the directory it was given. Calibration: the report crate applies is_file() to its input paths (crates/report/src/main.rs:716) while the payer intake does not, and the repo's CI payer job bound (timeout-minutes: 60, .github/workflows/privacy-gate.yml:223) limits an unattended CI hang without preventing it.",
  "root_cause": "The entry type is never validated: the extension filter (payer.rs:503) is the only selection rule, and read_to_string (payer.rs:513) both follows symlinks and blocks in open() on FIFOs, with no read deadline.",
  "intended_behavior": "Only regular files inside the supplied directory should be read: reject symlinks and non-regular entries (symlink_metadata/file_type check, or an O_NOFOLLOW open) as ledger-able refusals, and put a deadline on intake reads so one blocking entry cannot hold the payer hostage.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/payer.rs",
      "line": 150,
      "scope": "payer::run",
      "description": "--invoices names the merchant-supplied invoice directory; its entries are processed as invoices with no file-type check"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 503,
      "scope": "load_invoices",
      "description": "entries are selected by extension only; entry.path() is never type-checked"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 513,
      "scope": "load_invoices",
      "description": "read_to_string opens the path: a symlink is followed outside the directory, a FIFO blocks the process in open() with no timeout"
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/payer.rs",
      "line": 515,
      "scope": "load_invoices",
      "description": "with a symlink, the outside file is parsed and retained like any invoice; with a FIFO the process never reaches the decide loop at payer.rs:249 - no refusal, no ledger, nothing"
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/payer.rs",
      "line": 498,
      "description": "std::fs::read_dir enumerates every entry type (regular file, symlink, FIFO); the only filter in the loop is the extension test at line 503 - no file-type discrimination"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 513,
      "description": "read_to_string: whole-file read with no size cap, following symlinks; on a FIFO it blocks in File::open() (observed kernel wchan wait_for_partner)"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 201,
      "description": "the (potentially blocking) intake runs before any decision and does not consult the grant or kill-switch state; the banner and the decide loop come later (lines 210 and 249-250)"
    }
  ],
  "conditions": [
    {
      "kind": "system_configuration",
      "description": "merchant (or any untrusted producer) write access to the --invoices directory - creating a FIFO or symlink needs the same directory write permission as creating an invoice file"
    },
    {
      "kind": "data_state",
      "description": "the directory contains a FIFO or other blocking object named *.json, or a symlink named *.json pointing at a file outside the directory"
    }
  ],
  "execution": {
    "attacker_perspective": "A party who can create one directory entry: mkfifo hang.json for a permanent hang, or ln -s /target link.json to have the payer read and act on a file outside the directory it was given.",
    "payloads": [
      "mkfifo verifier6-fx/inv-fifo/hang.json (with one valid invoice afterfifo.json beside it; mkfifo needs only directory write, the same permission an invoice file needs)",
      "ln -s ../outside/outside.json verifier6-fx/inv-link/link.json (outside/outside.json is a valid invoice, invoice_id \"outside\", 7 zat, living outside the --invoices directory)",
      "docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 -v /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2:/t -v /home/kiter/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim sh -c 'sh verifier6-fx/check.sh'"
    ],
    "instructions": [
      "In the disposable copy, create the FIFO and the symlink fixtures above (plain filesystem operations, no privileges beyond writing the directory); keep a valid invoice in the FIFO directory and a control directory holding the same bytes as a plain file.",
      "Build zanary-probe from the copy's sources (touch crates/probe/src/payer.rs; cargo build --offline -p zanary-probe) so no prebuilt artifact is trusted.",
      "Run the payer against each fixture with --now 2026-10-17T00:00:00Z: the FIFO case under a revoked grant with a bounded harness kill (10 s), the symlink case under a permissive grant; run the plain-file controls under both grants.",
      "Record: binary sha256, exit code, elapsed time, output bytes, the printed decision lines, and the blocked process's state, wchan and open file descriptors."
    ],
    "observed_result": "FIFO case (revoked grant, afterfifo.json sorts before hang.json): the process was alive and sleeping after 5 s and 10 s (state=S, wchan=wait_for_partner - blocked in open() on the FIFO, no fd returned for hang.json), 0 bytes of output, SIGKILL at ~10 s gives exit 137, elapsed 11 s. Control: the same directory without the FIFO prints the full banner and 'REFUSE inplace - refused: the grant was revoked (kill switch)', then 'Error: connecting to the zcashd RPC at 127.0.0.1:18232 ... Connection refused' (offline sandbox), exit 1. Symlink case (permissive grant): 'ALLOW outside - paid: within the cap (10000000 zat) and the budget (7 of 50000000 zat spent after this)' and 'invoices : 1 in verifier6-fx/inv-link' - a file living in verifier6-fx/outside was read through the link, decided and queued; the same bytes copied inside a directory produce the identical ALLOW line; no payment was attempted (the run stops at the offline node connect). Both effects match the hunter's independently recorded run."
  },
  "remediation": {
    "strategy": "Validate the entry type during selection and read with a deadline: skip or refuse entries whose symlink_metadata file_type is not a regular file, open reads with O_NOFOLLOW (or refuse symlinks outright), and bound the intake with a timeout so a blocking file cannot hold the process.",
    "code_changes": [
      {
        "file_name": "crates/probe/src/payer.rs",
        "fixed_code": "// inside the read_dir loop, before pushing a candidate:\n            let entry = entry?;\n            let candidate = entry.path();\n            if candidate.extension().is_some_and(|e| e == \"json\")\n                && entry.file_type().is_ok_and(|t| t.is_file())\n            {\n                files.push(candidate);\n            }\n\n// and in the read loop, refuse anything that is not a regular file inside the directory:\n        let meta = std::fs::symlink_metadata(file)\n            .with_context(|| format!(\"statting the invoice {}\", file.display()))?;\n        if !meta.file_type().is_file() {\n            bail!(\n                \"{} is not a regular file (symlink or special file) - refusing to read it as an \\\n                 invoice\",\n                file.display()\n            );\n        }"
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "low",
      "reason": "One directory entry is enough and it needs no privilege beyond directory write (mkfifo and ln -s need the same permission as writing an invoice file). The precondition is merchant write access to the --invoices directory - the modeled input channel of this unit; the repo's own demo and CI populate the directory themselves and the merchant side is simulated (docs/BOUNDED-PAYER.md), so the deployed demo path is not attacker-written."
    },
    "impact": {
      "score": "low",
      "reason": "Availability of the operator's payer run: intake blocks indefinitely before any decision, so even a revoked grant never reaches its refusal or the ledger, and only killing the process recovers; plus a directory-scoping bypass - a symlinked entry makes the payer read and queue an invoice file outside the supplied directory (weak: a writer can already place an invoice inside it). No money moves while blocked, nothing persists, recovery is removing the entry; the repo's CI payer job's timeout-minutes: 60 bounds an unattended CI hang to the job timeout."
    },
    "overall_severity": "low"
  },
  "confidence": {
    "score": "high",
    "reason": "The verifier independently rebuilt zanary-probe from the reviewed revision inside the sandbox (payer.rs sha256 22799a3b155aa1e92d96e07650f8155122e56924b423d15195bf93684aac9720 identical to HEAD; only cfg(test) additions elsewhere in the copy) and reproduced both effects: a FIFO entry blocked the process in open() (kernel wchan wait_for_partner) with zero output until killed (exit 137), and a symlinked entry had an outside invoice file ALLOWed and queued, matching the hunter's record. Source shows no type check, no O_NOFOLLOW and no intake timeout, and payer.rs:201 is the only caller."
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-25/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-25/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "probe/payer.rs:load_invoices:nonregular-entry"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
