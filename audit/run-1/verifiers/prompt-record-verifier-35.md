# Phase-5 record verifier — verifier-35 on `zanary/published-summary-carries-invocation-paths`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`needs_validation`)

```json
{
  "verdict": "needs_validation",
  "fingerprint": "zanary/published-summary-carries-invocation-paths",
  "title": "Published claim files carry the gate's raw invocation-path arguments verbatim (${{ github.workspace }}-prefixed in the documented CI usage)",
  "description": "The publish job republishes each gate run's test.json into public/claims/<artifact>/ (privacy-gate.yml:161, served by upload-pages-artifact at 209-211 and deploy-pages at 213-215) and the same file is uploaded as the gate job's 30-day artifact (privacy-gate.yml:95-104, world-downloadable on this public repository). That summary stores the raw path arguments the gate was invoked with - flow_definition, run_dir and report_file (crates/probe/src/gate.rs:332-364, written verbatim at 365 from PathBufs built straight from the CLI arguments at gate.rs:92/102/105-108; no canonicalisation or relativisation exists anywhere on the path, record.rs:76-79 stores out_dir as given). In the documented CI usage those arguments are built from ${{ github.workspace }} (action.yml:96, 103, 113-119), so the published summary would carry the runner's workspace-prefixed paths - on a GitHub-hosted runner the absolute workspace root /home/runner/work/<repo>/<repo>/... Nothing that consumes a claim needs the prefix: the page renderer never reads test.json (no .rs file in crates/ references it; the index reads report.json and anchor.json, main.rs:682-711), zanary-report verify takes the report path as an argument, and the five recorded data/runs/*/test.json instances only replay the relative arguments their local invocations passed. The same strings also reach the job summary (scripts/ci-summary.py:50, 68) and gate.log (gate.rs:114-115, tee'd at action.yml:120).",
  "claimed_root_cause": "the summary writer records the CLI path arguments verbatim (definition.display(), outcome.out_dir.display(), outcome.report_path.display(); crates/probe/src/gate.rs:336/337/343) with no normalisation to workspace-relative paths, and the publish step copies that summary into the published claims directory (privacy-gate.yml:161)",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "action.yml",
      "line": 103,
      "scope": "composite action (any caller; the caller's workspace value enters the argument strings here)",
      "description": "OUT is built as \"${{ github.workspace }}/${{ inputs.out }}\" and the flow candidate is resolved against ${{ github.workspace }} first (line 96), so both path arguments are workspace-prefixed wherever workspace points"
    },
    {
      "kind": "propagation",
      "file": "action.yml",
      "line": 113,
      "scope": "composite action, gate step",
      "description": "zanary-probe test is invoked with --definition \"$FLOW\" (114), --out \"$OUT\" (115) and --summary \"$OUT/test.json\" (119), so the prefixed strings are the argv values"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/gate.rs",
      "line": 336,
      "scope": "summary writer",
      "description": "the machine-readable summary stores flow_definition = definition.display() (336), run_dir = outcome.out_dir.display() (337) and report_file = outcome.report_path.display() (343) - the raw argument strings, no normalisation"
    },
    {
      "kind": "sink",
      "file": ".github/workflows/privacy-gate.yml",
      "line": 161,
      "scope": "publish job",
      "description": "cp \"$dir\"report.json \"$dir\"anchor.json \"$dir\"test.json \"public/claims/$name/\" republishes the summary verbatim as public/claims/zanary-<matrix id>-evidence/test.json, and upload-pages-artifact (209-211) + deploy-pages (213-215) serve it publicly; the same file is also in the job's 30-day artifact (97-104)"
    }
  ],
  "evidence": [
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 154,
      "description": "render step: zanary-report index --reports claim-input --out public/index.html (154) consumes the run directories; the loop at 158-162 then copies each run's report.json/anchor.json/test.json into public/claims/<artifact>/ (161) - test.json included"
    },
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 97,
      "description": "gate job: actions/upload-artifact@v4 uploads path: runs/${{ matrix.id }}/ (100) with retention-days: 30 (103) as zanary-${{ matrix.id }}-evidence, so test.json and gate.log are downloadable independently of the Pages site"
    },
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 138,
      "description": "publish job: actions/download-artifact@v4 with pattern: zanary-*-evidence (140) lands each run's files under claim-input/<artifact-name>/ (141), so the bytes copied at 161 are exactly the gate job's files"
    },
    {
      "file": "data/runs/ci-checkout/test.json",
      "line": 17,
      "description": "recorded instance of the summary: flow_definition \"flows/ci-checkout.json\" (17), report_file \"data/runs/ci-checkout/report.json\" (20), run_dir \"data/runs/ci-checkout\" (22) - the raw arguments this local invocation passed; the file is untracked run output (/data is gitignored, .gitignore:6), and the CI invocation passes the same three fields ${{ github.workspace }}-prefixed"
    },
    {
      "file": "crates/probe/src/gate.rs",
      "line": 365,
      "description": "the summary is written to the path given by --summary with the three path fields exactly as displayed (std::fs::write(path, serde_json::to_string_pretty(&summary)?)); the CLI takes them verbatim (92 PathBuf::from(args.require(\"--definition\")?), 102/105-108 for --out) and record.rs:76-79 stores out_dir.to_path_buf() / out_dir.join(\"report.json\") - no canonicalisation, relativisation or workspace check anywhere in crates/probe/src"
    },
    {
      "file": "scripts/ci-summary.py",
      "line": 50,
      "description": "the job-summary renderer prints the recorded flow_definition (50) and run_dir (68) into $GITHUB_STEP_SUMMARY (action.yml:139-141), so the same raw strings are also in the job summary and, via the probe's own output (gate.rs:114-115, tee'd at action.yml:120), in the step log and gate.log"
    }
  ],
  "blockers": [
    "The decisive fact is the runtime value of ${{ github.workspace }}: the action always builds OUT and the flow candidate with that prefix (action.yml:96, 103), but whether it is the absolute hosted-runner workspace (/home/runner/work/<repo>/<repo>) rather than a relative path is GitHub provider behaviour, not repository source. Nothing in the tree settles it - a case-sensitive grep for 'home/runner' over tracked and untracked files returns zero hits, and all five recorded data/runs/*/test.json instances replay relative arguments from local invocations (scratch/ci-private.log:2-3 shows the local recording's 'flow definition : flows/ci-checkout.json' / 'run directory : data/runs/ci-checkout').",
    "The published bytes themselves - the Pages claim file and the 30-day uploaded artifact - are outside this read-only review (no live URL or artifact access; the project's published copy is out of bounds), so the end-to-end observation of the published field values was not made. The copy step is source-verified as verbatim (privacy-gate.yml:161), so what is unresolved is exactly the workspace value above, not the publication path.",
    "No offline execution path reaches the summary writer: zanary-probe test writes test.json only after the flow has run and been verified against the chain (gate.rs:133, 156, 331-368), so replaying the writer with absolute arguments needs the local regtest rig (harness/up.sh - docker plus loopback ports 18232/18233/9067/9068), which a --network=none sandbox and this review may not use."
  ],
  "validation_plan": {
    "local": "Part 1 (no rig, read-only, already done here): 'grep -rn canonicalize crates/' is empty, gate.rs:92/102/105-108 take the arguments verbatim, record.rs:76-79 stores out_dir as given, and gate.rs:332-364 serialises the three .display() strings - which is why every recorded data/runs/*/test.json replays the invocation strings it was given. Part 2 (needs the documented local rig; non-destructive): with harness/up.sh running (docker + loopback regtest), run './target/release/zanary-probe test --definition \"$PWD/flows/ci-checkout.json\" --out \"$PWD/runs/abs-path-probe\" --summary \"$PWD/runs/abs-path-probe/test.json\" --expect pass' and read runs/abs-path-probe/test.json: flow_definition/run_dir/report_file are expected to be the absolute strings passed in, which settles the writer (not the provider's workspace value). It writes only into the fresh runs/abs-path-probe directory and touches no existing run, claim or wallet state. A unit test in crates/probe/src/gate.rs asserting the three fields are workspace-relative closes the issue permanently.",
    "deployment": "Owner-observed, read-only: (1) open one recent 'privacy gate' run on main in the GitHub UI and read the gate step's log or the job summary - both print the recorded fields ('flow definition : ...', 'run directory : ...'; scripts/ci-summary.py:50, 68) - and check whether they start with /home/runner/work/; (2) open the published claim file (for example https://memeshee.github.io/zanary/claims/zanary-private-evidence/test.json) or download the zanary-private-evidence artifact and read flow_definition, run_dir and report_file in it. If those strings are absolute runner paths, the disclosure is confirmed (the same strings are also in the 30-day artifact, the step log and the job summary); if they are relative, this candidate is refuted. Fix: write the three fields relative to the workspace (or to the run directory) in crates/probe/src/gate.rs - that clears every channel - or, at minimum, drop test.json from the public/claims/ copy at privacy-gate.yml:161 and keep it only in the artifact, which only narrows the disclosure."
  }
}
```

## Linked ledger unit

- coverage_id: `crates%2Freport%2Fsrc%2Flib.rs%23index-renderer::.github%2Fworkflows%2Fprivacy-gate.yml%23publish::published-evidence::ATTACK-CLASSES.md%23Cryptography%20and%20secrets`
- surface: `crates/report/src/lib.rs#index-renderer` · subsystem `published-evidence`
- boundary: `.github/workflows/privacy-gate.yml#publish`
- assignment owner: `hunter-evidence`

The unit's checks (each `artifact` path is relative to the run directory; read the promoted files
under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/`):

```json
[
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/report/src/index.rs",
      "crates/report/src/main.rs",
      "crates/report/src/lib.rs",
      "crates/report/fixtures/index.golden.html"
    ],
    "invariant": "the published claim page shows the claim (flow id, clock, hashes, anchor txid, verdicts) and must never render the evidence payload or the anchor record's key material",
    "method": "source",
    "result": "render_index_with_anchors formats only flow_id/flow_commit(short)/chain/verified_at/height/ttl/countdown/evidence-root+report-hash (full only in title attributes)/anchor txid+height/evidence count/merchant_is_simulated/finding property-severity-verdict-title-note/regression links; no Evidence value (txid, receiver, memo, pool totals, merchant field) is ever formatted; every interpolated value passes esc(); the golden fixture and the four recorded renderings (crates/report/fixtures/index.golden.html, data/claims/index.html, data/runs/index.html, data/runs/demo/index.html) contain zero occurrences of any receiver address or memo hex",
    "artifact": null
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      ".github/workflows/privacy-gate.yml",
      "action.yml",
      ".gitignore"
    ],
    "invariant": "the publish job must publish a fixed allowlist of claim files and never widen to the rest of the run directory",
    "method": "source",
    "result": "publish (privacy-gate.yml:110-215) copies only index.html plus per-run report.json/anchor.json/test.json (line 161) and the tracked fixture reports (lines 143-148, 163); run.json, findings.json, gate.log and claim-input/ are not republished; the job is main-only and reads artifacts of its own workflow run; .gitignore excludes /data, claim-input/, public/, *.env, *.key",
    "artifact": null
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/report/src/anchor.rs",
      "crates/probe/src/ops.rs",
      "crates/probe/src/core.rs",
      "docs/THREAT-MODEL.md",
      "docs/ANCHOR-NOTES.md",
      "docs/CI.md"
    ],
    "invariant": "the only key material the pipeline publishes is the beacon viewing key its documented verifier needs; no seed, spending key or wallet credential reaches published output",
    "method": "source",
    "result": "anchor.json/test.json publish the beacon account-0 UFVK; the beacon wallet (WalletProfile::beacon) is instantiated only in the anchor path (ops.rs:540) and is paid only the 1000-zat anchor notes, so that key decrypts the anchor log and nothing else; no mnemonic, seed, USK, transparent key or node credential is written into report.json/anchor.json/test.json or the page (the docs state the beacon key must be public: THREAT-MODEL.md:48-56, ANCHOR-NOTES.md:32-36, CI.md:99)",
    "artifact": null
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/observation/src/lib.rs",
      "crates/report/src/verify.rs",
      "crates/properties/src/properties/report_leak.rs",
      "docs/CLAIMS.md",
      "docs/LEAK-MATRIX.md"
    ],
    "invariant": "published report evidence is a whitelist that excludes payer-side and receipt material, and every published evidence entry is re-checkable",
    "method": "source",
    "result": "collect_evidence (lib.rs:209-254) whitelists transaction/batch_size, recipient receivers, memo bytes, pool totals and merchant fields; Payment.sender_receivers, funding receivers, receipts (with signer ak and wallet attestation), merchant headers, policy strings and free-form observed values are never copied into a report; verify re-checks tx/receiver/memo/pool entries against the chain and deliberately SKIPs merchant fields as not-chain facts (verify.rs:774-781)",
    "artifact": null
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "docs/SUBMISSION.md",
      "data/claims/index.html",
      "data/claims/2026-10-11-checkout-private.json",
      "crates/report/fixtures/reports/report-2026-10-10.json",
      "flows/ci-checkout.json",
      "scripts/bounded-payer-demo.sh"
    ],
    "invariant": "what the documented publish shape actually contains for a recorded claim",
    "method": "source",
    "result": "the recorded claim file and page carry the full evidence array (txids, regtest receivers, 20-byte invoice memos, merchant webhook fields, pool totals) next to the page - intended and requirable by 'zanary-report verify'; the flat fixture report published as history is already tracked in the repo, and the bounded-payer demo dir that the publish job also downloads has no top-level report.json/anchor.json/test.json, so no payer data reaches public/",
    "artifact": null
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/probe/src/gate.rs",
      "data/runs/demo/demo-private/report.json",
      "data/runs/demo/demo-private/run.json",
      "data/runs/demo/demo-private/anchor.json",
      "data/runs/demo/demo-private/test.json",
      "data/runs/demo/demo-control/report.json",
      "data/runs/ci-checkout/test.json",
      "data/runs/ci-checkout/report.json",
      "data/demo-payer/invoices/8301.json",
      "scratch/ci-private.log"
    ],
    "invariant": "recorded run-dir contents vs what the publish job copies (does any un-copied file carry material that would be excess if it were published)",
    "method": "source",
    "result": "run.json carries receipts with signer+attestation (merchant wallet ak), merchant-side headers, invoice ids and observed values, and gate.log carries the payer/merchant AccountUuid lines (scratch/ci-private.log:21-22) - the publish job copies none of them, but the gate/payer jobs upload those files as world-downloadable artifacts; test.json carries the raw flow_definition/run_dir/report_file argument strings plus the anchor's beacon_ufvk",
    "artifact": null
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/report/src/index.rs",
      "data/runs/demo/demo-private/report.json",
      "data/runs/demo/demo-private/anchor.json"
    ],
    "invariant": "what the page actually renders when the claim input carries full evidence and an anchor record",
    "method": "local",
    "result": "rendering the built tool at commit 7058406 over a claim-input copy of the recorded private-flow report (32 evidence entries) plus its anchor.json produced a page that contains the flow id, short commit, report hash, evidence root, anchor txid+height, evidence count and finding text, and none of the 32 evidence entries' values, no anchor-record field except the txid, and no unescaped markup from a hostile flow_id/note/title (LEAKMARKER values absent)",
    "artifact": "agents/hunter-evidence/artifacts/check-r1-render.json"
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/report/src/index.rs"
    ],
    "invariant": "the index must not abort on a report whose hash-like field is not pure ASCII at byte 12",
    "method": "local",
    "result": "a report whose flow_commit is 11 ASCII chars followed by U+00E9 panics at crates/report/src/index.rs:138 ('end byte index 12 is not a char boundary'), exit 101, no page written - byte-index truncation in short()",
    "artifact": "agents/hunter-evidence/artifacts/check-r2-slice.txt"
  },
  {
    "agent_id": "hunter-evidence",
    "reviewed_paths": [
      "crates/report/tests/golden.rs",
      "crates/report/src/index.rs"
    ],
    "invariant": "the published page's bytes are pinned by tests, so an accidental widening of what the page shows fails CI",
    "method": "local",
    "result": "cargo test --offline -p zanary-report in the offline sandbox at commit 7058406: exit 0, 13 unit tests and 4 golden tests pass, including index_matches_golden_html which compares the renderer's output byte-for-byte against crates/report/fixtures/index.golden.html",
    "artifact": "agents/hunter-evidence/artifacts/check-r3-tests.txt"
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-35/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-35/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "zanary/published-summary-carries-invocation-paths"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
