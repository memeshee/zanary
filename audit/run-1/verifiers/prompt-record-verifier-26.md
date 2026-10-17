# Phase-5 record verifier — verifier-26 on `probe/payer.rs:load_invoices:unbounded-intake-memory`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`confirmed`)

```json
{
  "verdict": "confirmed",
  "fingerprint": "probe/payer.rs:load_invoices:unbounded-intake-memory",
  "title": "Bounded payer ingests a merchant-supplied invoice directory into memory with no size or count cap, before and independent of grant authority",
  "description": "load_invoices (crates/probe/src/payer.rs:495-520) enumerates every *.json entry under the --invoices path, reads each file whole with std::fs::read_to_string (payer.rs:513) and parses it with serde_json::from_str (payer.rs:515), retaining every parsed Invoice in one Vec (payer.rs:517). There is no per-file size cap, no invoice-count cap and no memory budget anywhere in 495-520; the only accounting in the payer is decide()'s monetary budget over amount_zat (payer.rs:119-128). The whole intake runs before the first decide() call (payer.rs:201 vs 249-250) and is not conditional on the grant state computed at payer.rs:187-199, so it is performed even when the grant is revoked or the kill file exists: the kill switch limits payments, not intake work. Re-established locally against the reviewed revision in the approved sandbox: a 256 MiB invoice completes intake at about 525 MiB peak RSS (read buffer plus parsed copy, about 2x the file) under a revoked grant, which prints 'REFUSE v7big - refused: the grant was revoked (kill switch)'; the same file under a 256 MiB or a 700 MB address-space budget aborts inside intake with 'Error: reading the invoice ... Caused by: out of memory' and zero decision lines; 5000 tiny invoice files are all read, accepted (ALLOW) and queued in one run. Each allowed invoice is additionally cloned into to_pay (payer.rs:259) for the payment phase.",
  "root_cause": "The invoice directory is an untrusted merchant channel, but intake is unbounded and unconditional: read_dir collects an unlimited number of *.json entries (payer.rs:498-506), read_to_string reads each one whole with the file's own size hint deciding the allocation (payer.rs:513), every parsed Invoice is retained in one Vec (payer.rs:511-517) and each allowed one is cloned into the payment queue (payer.rs:259) - with no size cap, no count cap, no metadata size or regular-file check, and no consideration of the grant or kill-switch state (payer.rs:187-199) before the work.",
  "intended_behavior": "A payer given 'authority, not a key' should bound what an untrusted merchant channel can make it ingest: a per-invoice byte cap and an invoice-count cap (or streaming/deferred parsing under a total memory budget) enforced from file metadata before any read, and enforced for a revoked grant and a tripped kill file too, so the kill switch also caps intake work. An entry over a cap should become a named refusal line rather than a read.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/payer.rs",
      "line": 498,
      "scope": "load_invoices",
      "description": "read_dir enumerates every *.json entry under the operator-designated --invoices path (payer.rs:150, handed to load_invoices at payer.rs:201): the first touch of merchant-controlled entries, with no cap on how many (payer.rs:498-506)"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 513,
      "scope": "load_invoices",
      "description": "std::fs::read_to_string reads each entry whole - the merchant-controlled file size sizes the allocation, with no size check and no regular-file check before the read"
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/payer.rs",
      "line": 515,
      "scope": "load_invoices",
      "description": "serde_json::from_str materialises the whole Invoice (a second copy of every String field, memo included), payer.rs:517 retains it in one Vec that lives for the run, and the entire intake completes before the first decide() call (payer.rs:249-250) unconditionally, because the revoked/kill-switch state from payer.rs:187-199 is never consulted here; each allowed invoice is then cloned again into to_pay at payer.rs:259"
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/payer.rs",
      "line": 150,
      "description": "the merchant channel enters as the operator-supplied --invoices path (args.require) whose contents the merchant controls"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 187,
      "description": "the revoked/kill-switch state is computed here (187-191) and is never consulted by load_invoices (201) or by anything in 495-520"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 201,
      "description": "load_invoices runs before the decide loop (249-250) and before all wallet/node work (275-277): a revoked or killed grant still pays the full intake cost"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 244,
      "description": "comment 'Decide everything first, then pay': the complete invoice directory is materialised in memory before any decision or payment"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 259,
      "description": "every allowed invoice is cloned (memo String included) into to_pay before payments begin, so the payment queue holds a second copy of each allowed invoice"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 495,
      "description": "load_invoices: static extension filter only, files.sort(), read_to_string + serde_json::from_str per file, Vec<Invoice> returned - no count cap, no size cap, no memory budget in 495-520"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 513,
      "description": "std::fs::read_to_string reads each file whole; nothing checks the file's size or type before the read"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 119,
      "description": "decide()'s only accounting is the monetary budget over amount_zat (119-128): no notion of invoice count or intake bytes"
    }
  ],
  "conditions": [
    {
      "kind": "system_configuration",
      "description": "the operator points --invoices at a directory a merchant or any less-trusted party can write to - the channel the bounded payer is designed around ('the merchant side of an agent payment ... no channel other than the file', crates/probe/src/flow.rs:1554-1555). The repo's own recorded runs, scripts/bounded-payer-demo.sh and the CI payer job author the directory themselves, so the shipped demo does not trigger it."
    },
    {
      "kind": "environmental_dependency",
      "description": "the payer process has a finite memory budget (systemd MemoryMax, container memory limit, small host) or the directory's files approach the available memory: measured, a 256 MiB invoice aborts intake under RLIMIT_AS 262144 kB or 700000 kB ('out of memory', zero decisions) and completes with no limit at about 525-532 MiB peak RSS; the debug binary's own VmPeak is already about 547 MiB on a 3-invoice run, so intake spends the operator's budget before any decision."
    }
  ],
  "execution": {
    "attacker_perspective": "A merchant who can write files into the directory the operator passes as --invoices; no wallet, key, network access or payment participation is needed - one file is enough. Because intake precedes every decision, that one file also denies every other invoice in the run and the run's ledger.",
    "payloads": [
      "grant-revoked.json (129 B): {\"budget_zat\":50000000,\"per_payment_cap_zat\":10000000,\"allowlist\":[],\"expires_at\":\"2030-01-01T00:00:00Z\",\"revoked\":true}",
      "inv-big/big.json (268435546 B): {\"invoice_id\":\"v7big\",\"amount_zat\":1,\"receiver\":\"uregtest1noreceivercheck\",\"memo\":\"<268435400 bytes of the byte a>\",\"uri\":\"zcash:test\",\"issued_at_height\":1,\"merchant_account\":0}",
      "inv-count/: 5000 files c00001.json..c05000.json, 161 B each, amount_zat 0, receiver uregtest1noreceivercheck",
      "inv-mix/: big.json plus the 5000 files copied together (5001 entries, 282284 KiB)",
      "kill-file variant: any existing path passed as --kill-file (path.exists() at payer.rs:188-191)",
      "docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 -v <disposable copy of the reviewed revision>:/t -v ~/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim sh -c 'env -i PATH=/usr/bin:/bin sh /t/v7-fx/checks.sh'"
    ],
    "instructions": [
      "Rebuild the disposable copy from the reviewed revision (git archive HEAD | tar -x, rust-toolchain.toml removed) and build inside the approved sandbox: docker run ... sh -c 'CARGO_BUILD_JOBS=6 cargo build --offline -p zanary-probe --bin zanary-probe'. The resulting binary hashes 30435f8bf1d5962ecd2bc964aa015dd1cd567807616c21682146eeea9d8b6416, identical to the hash the hunter recorded.",
      "Write the fixtures under /t/v7-fx (two grants, 3 tiny invoices, the one 256 MiB invoice, the 5000-file directory); the memo padding is what sizes the big file.",
      "Run the payer with --now 2026-10-17T00:00:00Z and --out under /t, sampling /proc/<pid>/status VmHWM every 50 ms and counting ALLOW/REFUSE lines; the fatal cases add 'ulimit -v 262144' or 'ulimit -v 700000' in the child subshell so the memory effect stays bounded.",
      "No payment path is exercised and nothing leaves the sandbox: --network=none and no zcashd/lightwalletd, so every run stops after intake at 'connecting to the zcashd RPC at 127.0.0.1:18232 ... Connection refused'. No wallet, key, external endpoint or host state is touched.",
      "Stop at the first observed effect: the fatal intake is shown with the 256 MiB fixture under a 256 MiB address-space cap; no larger fixture is needed."
    ],
    "observed_result": "Sandbox, --network=none: control (3 tiny invoices, full grant) prints 'invoices : 3' and 3 ALLOW lines then exits 1 at the node connect - intake and decisions precede all wallet/node work. Revoked grant + 256 MiB invoice: intake completes (peak RSS 525 MiB; header 'invoices : 1') and prints 'REFUSE v7big - refused: the grant was revoked (kill switch)' before the same node error; the kill-file run refuses all 3 tiny invoices the same way, after reading them. Same 256 MiB invoice under RLIMIT_AS 262144 kB: exit 1 in under 1 s with the whole log 'Error: reading the invoice /t/v7-fx/inv-big/big.json / Caused by: out of memory' - zero decisions, no header: the run aborts inside intake; under RLIMIT_AS 700000 kB the read of the same file also fails, while with no limit it completes, so the merchant file size decides the required memory. 5000 zero-amount invoices: 5000 ALLOW lines and 'invoices : 5000' in 9 s, peak RSS 19 MiB - nothing caps count; the mixed directory (one 256 MiB + 5000 tiny) ingests 5001 invoices, peak RSS 532 MiB. My independent rebuild of the probe reproduced the hunter's recorded binary sha256."
  },
  "remediation": {
    "strategy": "Bound the intake at the boundary, from file metadata and before any read, regardless of authority state: cap the number of invoice files and each file's size, reject non-regular entries and turn an over-cap entry into a named refusal/error line instead of a read. If large invoices must be supported, stream or defer parsing under a total memory budget. Apply the caps for a revoked grant and a tripped kill file too, so the kill switch also caps intake work.",
    "code_changes": [
      {
        "file_name": "crates/probe/src/payer.rs",
        "fixed_code": "/// Invoices from a directory (`*.json`) or from a single file, sorted by name so a run is reproducible.\n/// Bounded: a merchant channel may not decide how much this process reads.\nconst MAX_INVOICE_FILES: usize = 1000;\nconst MAX_INVOICE_BYTES: u64 = 262_144; // far above any real invoice\n\nfn load_invoices(path: &Path) -> Result<Vec<Invoice>> {\n    let mut files: Vec<PathBuf> = Vec::new();\n    if path.is_dir() {\n        for entry in std::fs::read_dir(path)\n            .with_context(|| format!(\"reading the invoice directory {}\", path.display()))?\n        {\n            let entry = entry?;\n            let candidate = entry.path();\n            if candidate.extension().is_some_and(|e| e == \"json\") {\n                files.push(candidate);\n            }\n        }\n    } else {\n        files.push(path.to_path_buf());\n    }\n    files.sort();\n    if files.len() > MAX_INVOICE_FILES {\n        bail!(\n            \"{} invoice files in {} exceeds the cap of {MAX_INVOICE_FILES} - refusing to read a merchant directory this large\",\n            files.len(),\n            path.display()\n        );\n    }\n    let mut invoices = Vec::new();\n    for file in &files {\n        let meta = std::fs::symlink_metadata(file)\n            .with_context(|| format!(\"statting the invoice {}\", file.display()))?;\n        if !meta.file_type().is_file() {\n            bail!(\n                \"{} is not a regular file - refusing to read it as an invoice\",\n                file.display()\n            );\n        }\n        if meta.len() > MAX_INVOICE_BYTES {\n            bail!(\n                \"invoice {} is {} bytes, above the {MAX_INVOICE_BYTES}-byte cap\",\n                file.display(),\n                meta.len()\n            );\n        }\n        let text = std::fs::read_to_string(file)\n            .with_context(|| format!(\"reading the invoice {}\", file.display()))?;\n        let invoice: Invoice = serde_json::from_str(&text)\n            .with_context(|| format!(\"parsing the invoice {}\", file.display()))?;\n        invoices.push(invoice);\n    }\n    Ok(invoices)\n}"
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "low",
      "reason": "Requires the operator to point --invoices at a directory a hostile or buggy party can write to (the documented merchant-invoice channel): the repo's own recorded runs, scripts/bounded-payer-demo.sh and the CI payer job author the directory themselves, so the shipped and demo configurations do not trigger it."
    },
    "impact": {
      "score": "low",
      "reason": "Availability of the operator's own payer run: under a finite memory budget the intake aborts with exit 1 ('out of memory') before any decision, or the process is OOM-killed by whatever enforces the budget; without one it merely consumes about 2x the merchant file size of RSS (measured 525-532 MiB for a 256 MiB invoice). No payment is made, nothing is corrupted, one run and its ledger are lost, and recovery is deleting the entry or raising the limit. The blast radius inside the run is the whole run - one merchant's file denies every other invoice in it - but the affected principal is the operator's single local process, so there is no cross-tenant or network-visible effect."
    },
    "overall_severity": "low"
  },
  "confidence": {
    "score": "high",
    "reason": "The missing bound is fully source-visible (495-520 has no cap of any kind and no knob anywhere in the tree caps it) and was independently reproduced against the reviewed revision in the approved sandbox: about 2x-file-size intake RSS, completion under a revoked grant and under a tripped kill file, fatal ENOMEM at intake under a bounded address space, and 5000 files accepted in one run; no preventing layer exists on the path (the only accounting, the monetary budget at 119-128, does not bound intake)."
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-26/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-26/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "probe/payer.rs:load_invoices:unbounded-intake-memory"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
