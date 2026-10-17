# Phase-5 record verifier — verifier-29 on `zanary/composite-action-input-shell-expansion`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`confirmed`)

```json
{
  "verdict": "confirmed",
  "fingerprint": "zanary/composite-action-input-shell-expansion",
  "title": "Composite action expands caller-supplied with: inputs into shell script text",
  "description": "action.yml is published as a composite action that third-party workflows run (`uses: memeshee/zanary@main` is the documented whole setup: action.yml:7, docs/CI.md:49). Its five inputs (flow, expect, anchor, ttl-days, out; action.yml:23-46) are free-form strings with no validation, and the action places their `${{ inputs.* }}` values in the script text of its `run:` steps -- the gate step (:95, :100, :103, :106, :116, :117), the always-run job summary step (:140) and the verdict step (:177-178) -- instead of receiving them through env: or argv. The runner substitutes those expressions into the script before bash parses it, so a value carrying shell syntax (a quote plus ;, $( ), backticks, #) is executed by the calling job's shell. The gate step also writes the value derived from inputs.out into GITHUB_OUTPUT as one raw, unescaped line (:107), so the value leaves the step as a step output and is re-interpolated into script text at :136 (job summary, `if: always()`) and :153 (rig diagnostics); the verdict step interpolates the step's exit_code output the same way (:175, numeric in this action's flow). Nothing in this repository triggers it: the documented consumer setup passes literal values (docs/CI.md:49-55, the flow literal at :51) and this repository's own gate passes static matrix entries defined in the same file (.github/workflows/privacy-gate.yml:87-93 from the matrix at :71-80). The defect is in the action as a library: it hands every consumer that templates one of the five inputs into `with:` a code-execution sink in that consumer's own job.",
  "root_cause": "The action treats caller-supplied input values as script fragments rather than data. `${{ inputs.* }}` values are placed in `run:` script text at eight sites (:95, :100, :103, :106, :116, :117, :140, :177-178, counting the two verdict lines as one site), and no value is passed through env: or argv anywhere in the file (action.yml has no env: block at all). Nothing validates or rejects shell syntax, so the double quotes around the substitutions are the sole barrier and $( ) / backticks defeat them. Values derived from the inputs are then written to GITHUB_OUTPUT on one raw, unescaped line (:107), which both lets them leave the step as a step output and re-enter script text in later steps (:136, :153); the exit_code output is interpolated the same way at :175.",
  "intended_behavior": "with: inputs are data, not code: each step should take them through env: (or argv) so a value cannot change which commands run; out should be written to GITHUB_OUTPUT with the multiline-safe key<<DELIM form; and steps.gate.outputs.*, which are derived from the inputs, must be passed the same way instead of being re-inserted into script text.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "action.yml",
      "line": 23,
      "scope": "composite action inputs",
      "description": "The action accepts five caller-supplied strings (flow, expect, anchor, ttl-days, out; lines 23-46) with no format validation; a third-party workflow supplies them in its own repository via with:."
    },
    {
      "kind": "propagation",
      "file": "action.yml",
      "line": 95,
      "scope": "step run the gate - run: script text",
      "description": "The runner substitutes ${{ inputs.flow }} into the script text (a double-quoted assignment) before bash parses it; the identical pattern sits at lines 100, 103, 106, 116, 117 and in the job summary / verdict steps (140, 177-178). Reproduced locally: with a payload in flow the injection executed here, and also at :100 when the value was invalid."
    },
    {
      "kind": "propagation",
      "file": "action.yml",
      "line": 107,
      "scope": "step run the gate - step output write",
      "description": "The out value is appended raw to GITHUB_OUTPUT (no key/newline escaping), so it also leaves the step as a step output and re-enters later scripts at 136, 153 and 175. Reproduced locally: the recorded value kept its $( ) text verbatim and executed again when the job summary step was rendered with it."
    },
    {
      "kind": "sink",
      "file": "action.yml",
      "line": 103,
      "scope": "step run the gate - command context",
      "description": "OUT is assigned from ${{ github.workspace }} and ${{ inputs.out }}, so the expansion lands in a double-quoted assignment in command position: a value with $( ), backticks, a closing quote plus ;, or # is executed by the caller shell (independently reproduced on this line and on line 95 with marker payloads in the offline sandbox)."
    }
  ],
  "evidence": [
    {
      "file": "action.yml",
      "line": 95,
      "description": "FLOW is assigned from ${{ inputs.flow }} inside the run the gate step - flow is expanded into script text."
    },
    {
      "file": "action.yml",
      "line": 100,
      "description": "second expansion of flow in the same step (the no-flow-definition error path) - a payload runs here as well (reproduced: two executions for one invalid value)."
    },
    {
      "file": "action.yml",
      "line": 103,
      "description": "OUT is assigned from ${{ github.workspace }} and ${{ inputs.out }} - out expanded into a command context (the verified local run B)."
    },
    {
      "file": "action.yml",
      "line": 106,
      "description": "the anchor input is expanded into script text in the [ anchor = true ] test."
    },
    {
      "file": "action.yml",
      "line": 116,
      "description": "expect is expanded inside the probe command line (--expect argument)."
    },
    {
      "file": "action.yml",
      "line": 117,
      "description": "ttl-days is expanded the same way (--ttl-days argument)."
    },
    {
      "file": "action.yml",
      "line": 107,
      "description": "the echo of out=$OUT into $GITHUB_OUTPUT - raw write, no newline escaping; the value leaves the step as a step output and a newline in it can forge further output lines (reproduced: an injected payload=owned key)."
    },
    {
      "file": "action.yml",
      "line": 136,
      "description": "OUT is assigned from ${{ steps.gate.outputs.out }} in the always-run job summary step - the stored value re-enters script text (same at 153 in rig diagnostics); verified by execution."
    },
    {
      "file": "action.yml",
      "line": 175,
      "description": "code is assigned from ${{ steps.gate.outputs.exit_code }} in the verdict step - the output that decides the build result is also script text (numeric in this action's flow)."
    },
    {
      "file": "action.yml",
      "line": 140,
      "description": "the title argument of the job summary step expands inputs.flow and inputs.expect; that step runs with if: always()."
    },
    {
      "file": "action.yml",
      "line": 177,
      "description": "inputs.expect is expanded inside the verdict step case branches (also line 178)."
    },
    {
      "file": "docs/CI.md",
      "line": 51,
      "description": "the documented consumer setup passes literal values (flow: flows/my-checkout.json), which is why no in-repo caller triggers the injection."
    }
  ],
  "conditions": [
    {
      "kind": "third_party_dependency",
      "description": "A calling workflow passes a non-constant value (a reusable-workflow parameter, a dispatch or matrix value, or an event-derived string) to one of flow, expect, anchor, ttl-days, out. No caller in this repository does that: the documented setup passes literals (docs/CI.md:49-55) and .github/workflows/privacy-gate.yml:87-93 passes static matrix entries defined in the same file at :71-80, so nothing fires here."
    }
  ],
  "execution": {
    "attacker_perspective": "Whoever controls a value that a consumer workflow passes into the action inputs -- a contributor-influenced branch/PR-derived string, a reusable-workflow parameter, a dispatch input, or the consumer's own templated value -- gets the calling job's shell: the payload runs in the action's steps with that job environment (GITHUB_* variables, the step outputs, any job/step environment the step can see). A consumer that passes a literal cannot be attacked; the defect is that the action makes the templating pattern dangerous.",
    "payloads": [
      "$(printf x >> /t/verifier10/count-flow.txt; printf 'flows/stub.json')",
      "$(printf x >> /t/verifier10/count-out.txt; printf 'x$(printf y >> /t/verifier10/count-roundtrip.txt)')",
      "$(printf 'x\\npayload=owned')"
    ],
    "instructions": [
      "Read action.yml from the read-only target (commit 7058406, worktree clean) and assert the verbatim text of every interpolation site: :95, :96, :100, :103, :106, :107, :116, :117, :136, :140, :153, :175, :177, :178.",
      "Render the gate step's run-block (verbatim lines 92-129) with the literal expression substitution the Actions runner documents (plus ${{ github.workspace }} and ${{ github.action_path }}), one payload value per run: flow kept valid, flow made invalid (so :100 expands it again), out, and out with a newline; all other inputs benign literals.",
      "Run each rendered script with `bash --noprofile --norc -eo pipefail` inside `docker run --rm --network=none --cpus=6 --memory=3500m --pids-limit=1024 --user 1000:1000 -v <disposable copy>:/t -w /t/verifier10/action rust:1-slim`, with GITHUB_OUTPUT / GITHUB_WORKSPACE / GITHUB_ACTION_PATH / GITHUB_STEP_SUMMARY set and a placeholder ./target/release/zanary-probe (the real rig is out of bounds); each payload only appends one byte to a counter file so the execution count is visible.",
      "Clear the counters and the output file first, and run a control pass with every input a literal.",
      "Then render the always-run job summary step (action.yml:136-147) with the exact out= value the gate step recorded in GITHUB_OUTPUT and execute it the same way (the step-output round trip), with a benign-value render as the control; stop at the first observed boundary result."
    ],
    "observed_result": "Independently reproduced in the offline sandbox with command-substitution payloads (no quote-breaking needed) against the verbatim gate run-block, rendered with literal expression substitution as the runner documents it and with a placeholder probe. Control (all inputs literal): the step ran normally, wrote out=/t/verifier10/ws/z and exit_code=0, and no counter file appeared. Flow payload (valid value): executed at :95 (counter x) while the step still finished normally, exit 0, with its own outputs. Flow payload (invalid value): executed twice (counter xx), at :95 and in the :100 error echo, then exited 2. Out payload: executed at :103 (counter x) and the step exited 0 having written the raw value out=/t/verifier10/ws/x$(printf y >> /t/verifier10/count-roundtrip.txt) and exit_code=0 to GITHUB_OUTPUT. Rendering the always-run job summary step with that recorded value executed the embedded fragment at :136 (counter y), while the benign-value control executed nothing. A newline in the out value made GITHUB_OUTPUT contain out=/t/verifier10/ws/x, payload=owned and exit_code=0, i.e. an injected output key. Nothing outside the disposable copy was written; --network=none, placeholder probe, no rig and no live runner. The only element not executed here is the runner's own substitution step (documented behavior; a live runner is out of bounds)."
  },
  "remediation": {
    "strategy": "Keep inputs as data on both sides of every step: declare them under env: and reference the environment variables inside the scripts; validate the shapes the action actually needs (flow/out caller-workspace-relative paths; expect in {pass, fail}; anchor in {true, false}; ttl-days a positive integer) and fail early with a clear error; keep the existing three-candidate flow search operating on the env value; write out to GITHUB_OUTPUT with the multiline-safe key<<DELIM form; and in the job summary, rig diagnostics and verdict steps pass steps.gate.outputs.out / steps.gate.outputs.exit_code through env: instead of re-interpolating them.",
    "code_changes": [
      {
        "file_name": "action.yml",
        "fixed_code": "    - name: run the gate\n      id: gate\n      shell: bash\n      working-directory: ${{ github.action_path }}\n      env:                                    # every input arrives as data\n        INPUT_FLOW: ${{ inputs.flow }}\n        INPUT_OUT: ${{ inputs.out }}\n        INPUT_EXPECT: ${{ inputs.expect }}\n        INPUT_ANCHOR: ${{ inputs.anchor }}\n        INPUT_TTL_DAYS: ${{ inputs.ttl-days }}\n      run: |\n        set -o pipefail\n        case \"$INPUT_EXPECT\" in pass|fail) ;; *) echo \"::error::expect must be pass or fail\"; exit 2 ;; esac\n        case \"$INPUT_TTL_DAYS\" in ''|*[!0-9]*) echo \"::error::ttl-days must be a positive integer\"; exit 2 ;; esac\n        FLOW=\"$INPUT_FLOW\"\n        for candidate in \"$GITHUB_WORKSPACE/$FLOW\" \"$GITHUB_ACTION_PATH/$FLOW\" \"$FLOW\"; do\n          if [ -f \"$candidate\" ]; then FLOW=\"$candidate\"; break; fi\n        done\n        if [ ! -f \"$FLOW\" ]; then echo \"::error::ZANARY gate: no flow definition at $INPUT_FLOW\"; exit 2; fi\n        OUT=\"$GITHUB_WORKSPACE/$INPUT_OUT\"; mkdir -p \"$OUT\"\n        anchor_flag=\"\"\n        if [ \"$INPUT_ANCHOR\" = \"true\" ]; then anchor_flag=\"--anchor\"; fi\n        { echo \"out<<ZANARY_OUT\"; printf '%s\\n' \"$OUT\"; echo \"ZANARY_OUT\"; } >> \"$GITHUB_OUTPUT\"\n        set +e\n        timeout \"${ZANARY_GATE_TIMEOUT_SECONDS:-2700}\" ./target/release/zanary-probe test \\\n          --definition \"$FLOW\" --out \"$OUT\" --expect \"$INPUT_EXPECT\" \\\n          --ttl-days \"$INPUT_TTL_DAYS\" $anchor_flag --summary \"$OUT/test.json\" 2>&1 | tee \"$OUT/gate.log\"\n        code=${PIPESTATUS[0]}\n        if [ \"$code\" = \"124\" ]; then\n          echo \"::warning::the gate did not finish within ${ZANARY_GATE_TIMEOUT_SECONDS:-2700}s - treating it as a rig failure\"\n          code=2\n        fi\n        echo \"exit_code=$code\" >> \"$GITHUB_OUTPUT\"\n        exit 0\n    # Later steps (job summary, rig diagnostics, verdict) take GATE_OUT / GATE_EXIT\n    # through env: (env: GATE_OUT: ${{ steps.gate.outputs.out }}) and never\n    # re-interpolate ${{ }} inside script text.\n"
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "low",
      "reason": "No caller in this repository passes a non-constant value (docs/CI.md:49-55 and privacy-gate.yml:87-93 both pass constants), so exploitation requires a consumer that templates one of the five inputs from a reusable-workflow parameter, a dispatch input, a branch/PR-derived string or similar -- a caller-side pattern the defect makes dangerous but cannot guarantee."
    },
    "impact": {
      "score": "high",
      "reason": "When it fires, arbitrary commands run inside the consumer's CI job as the runner user: the action's own steps, their step outputs (including the value the later steps trust) and any job or step environment the step can see. No in-repo caller is templated, so the impact lands on consumers of memeshee/zanary@main rather than on this repository's own CI."
    },
    "overall_severity": "medium"
  },
  "confidence": {
    "score": "high",
    "reason": "Every interpolation site, the raw GITHUB_OUTPUT write and the step-output round trip were re-read line-by-line at commit 7058406 (worktree clean; action.yml sha256 1c58ef4994e322ff0b222943132de8744dbdb35f6af59c842c595f8db43acde3 -- the hunter's recorded 6e58f6bb... is the same content without its trailing newline), and the expansion was re-executed independently in the offline sandbox with my own command-substitution payloads (flow at :95 and :100, out at :103, and the recorded value executing again at :136) against a benign-value control that ran nothing. The only step not executed here is the runner's own expression substitution (documented behavior, and a live runner is out of bounds for this run), and exploitation still requires a consumer that templates a value into with:, which the low likelihood records."
  }
}
```

## Linked ledger unit

- coverage_id: `action.yml%23inputs-and-steps::.github%2Fworkflows%2Fprivacy-gate.yml%23jobs::ci-and-release::ATTACK-CLASSES.md%23Injection`
- surface: `action.yml#inputs-and-steps` · subsystem `ci-and-release`
- boundary: `.github/workflows/privacy-gate.yml#jobs`
- assignment owner: `hunter-ci`

The unit's checks (each `artifact` path is relative to the run directory; read the promoted files
under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/`):

```json
[
  {
    "agent_id": "hunter-ci",
    "reviewed_paths": [
      "action.yml"
    ],
    "invariant": "composite-action inputs must stay data: no value from `${{ inputs.* }}` may be placed in the script text of a run: step (pass through env:/argv instead)",
    "method": "source",
    "result": "violated at action.yml:95, 100, 103, 106, 116, 117 (step run the gate), :140 (step job summary) and :177-178 (step verdict); the same class re-enters through `${{ steps.gate.outputs.out }}` at :136 and :153 and `${{ steps.gate.outputs.exit_code }}` at :175 because :107 stores the input-derived value raw in `$GITHUB_OUTPUT`",
    "artifact": null
  },
  {
    "agent_id": "hunter-ci",
    "reviewed_paths": [
      "action.yml"
    ],
    "invariant": "an input value containing shell syntax must not execute in the action steps",
    "method": "local",
    "result": "violated: the run the gate run-block (lines 92-129) was rendered with literal expression substitution (as the runner does) and executed under bash in `docker run --rm --network=none`; payloads in inputs.flow and inputs.out both executed (marker files created), and run B still finished with the step normal outputs `out=...` and `exit_code=0`",
    "artifact": "agents/hunter-ci/artifacts/check-injection-rce.txt"
  },
  {
    "agent_id": "hunter-ci",
    "reviewed_paths": [
      ".github/workflows/privacy-gate.yml"
    ],
    "invariant": "no event- or contributor-controlled value may reach shell script text or command arguments in the workflow jobs",
    "method": "source",
    "result": "covered, no sink: all run: scripts are built from static matrix values and constants; github.ref appears only in concurrency.group and in an if: predicate, never in script text; artifact names and run directories derive from static matrix ids; the sed-derived rust-toolchain channel is passed quoted to rustup",
    "artifact": null
  },
  {
    "agent_id": "hunter-ci",
    "reviewed_paths": [
      "scripts/demo-run.sh",
      "scripts/bounded-payer-demo.sh",
      "harness/up.sh",
      "scripts/ci-summary.py"
    ],
    "invariant": "caller-tunable env values and file contents are consumed as data (quoted expansions, no eval, no re-entry into shell text)",
    "method": "source",
    "result": "covered, no sink: `set -euo pipefail` in the shell scripts, every ZANARY_* caller value used inside quotes and confined to paths/timeouts; quoted python heredocs; ci-summary.py prints gate JSON fields as argv/Markdown into the caller own job summary only",
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-29/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-29/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "zanary/composite-action-input-shell-expansion"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
