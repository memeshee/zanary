# Phase-5 record verifier — verifier-36 on `zanary/rig-image-tags-unpinned`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`needs_validation`)

```json
{
  "verdict": "needs_validation",
  "fingerprint": "zanary/rig-image-tags-unpinned",
  "title": "The rig that produces and re-verifies every published privacy claim runs from two mutable :latest image tags",
  "description": "Both rig containers are selected by floating tags (harness/docker-compose.yml:9 electriccoinco/zcashd:latest, :34 electriccoinco/lightwalletd:latest). Nothing in the reviewed revision pins a digest, sets pull_policy or platform, or checks a repo digest (git grep '@sha256:' / 'pull_policy' / 'repoDigests' over the tracked tree: 0 hits each; '@sha256:' 0 hits across every commit), so the content that runs is chosen by whoever can re-point those Docker Hub tags, not by this repository. That content is not marginal: zcashd is the chain and the miner, and lightwalletd is the CompactTxStreamer the wallets sync against, so the pair produces every block, transaction and balance the flow runner observes, the gate's chain verification re-reads against the same endpoints (crates/probe/src/gate.rs:165, verify at :156), and the anchor timestamp is a shielded memo on that chain. The resolution point is reached by (a) any third-party workflow that uses the composite action (action.yml:85 runs harness/up.sh on the caller's runner; the action is documented as uses: memeshee/zanary@main), (b) this repo's own gate and payer jobs (.github/workflows/privacy-gate.yml:87 via uses: ./ and :239 directly) including the weekly cron (:23) on runners with no image cache, and (c) the judge-facing one command (README.md:111/114, docs/SUBMISSION.md:36). A substituted image therefore both executes on those machines and can fabricate the data a 'privacy claim' is made of, while the recorded claim carries no image, digest or container identity at all (crates/report/src/lib.rs:33-54 binds version/flow_id/flow_commit/chain/heights/findings/evidence/report_hash only), and the strongest source-visible chain control is a string equality between the node's self-reported getblockchaininfo.chain and the report's own self-asserted 'regtest' (crates/report/src/verify.rs:697) - so a drift is invisible in the evidence, the artifacts and the published page. The docs assert fixed identities that no mechanism enforces (harness/HARNESS.md:54-55 and README.md:75 name v6.20.0 / v0.5.5 next to the mutable tags). Whether the tags have in fact been re-pointed, and who can re-point them, is a registry fact this run cannot observe - hence needs_validation rather than confirmed.",
  "claimed_root_cause": "harness/docker-compose.yml selects both rig services by mutable tag (:9, :34) and the repository never binds the running container to a reviewed artifact: no image digest, no pull_policy, no platform pin, no repo-digest comparison anywhere in the tracked tree or history (git grep '@sha256:' / 'pull_policy' / 'repoDigests': 0 hits; the only docker inspect in the tree is harness/up.sh:116 reading .State.Running; the only docker pull is scripts/phase0-warmup.sh:31, recording no digest). The one step that could record what resolved - the bare `docker compose up -d` at harness/up.sh:60 (default pull policy `missing`, so any machine without the image cached pulls it) - records nothing, and the claim schema (crates/report/src/lib.rs:33-54) has no field for it, so the gate verdict and the published evidence are produced by an unbound third-party component.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "harness/docker-compose.yml",
      "line": 9,
      "scope": "services.zcashd.image (and services.lightwalletd.image at :34)",
      "description": "The chain/miner container and the indexer container are selected by floating tags (electriccoinco/zcashd:latest, electriccoinco/lightwalletd:latest); the file pins no digest, platform or pull policy, so the satisfying artifact is decided outside this repository."
    },
    {
      "kind": "propagation",
      "file": "harness/up.sh",
      "line": 60,
      "scope": "the documented one-command bring-up",
      "description": "`docker compose up -d` resolves those tags on whichever machine runs the harness - compose's default pull policy is `missing`, so a machine with no cached image (a fresh CI runner, a judge) pulls from the registry - and the script records nothing about what it resolved; its only identity-adjacent call (:116) reads .State.Running."
    },
    {
      "kind": "propagation",
      "file": "action.yml",
      "line": 85,
      "scope": "composite action step 'start the throwaway chain'",
      "description": "A third-party workflow that consumes `uses: memeshee/zanary@main` executes this step, so the current content of both tags is pulled and run inside the caller's runner/job, which is the execution boundary the tag controller reaches."
    },
    {
      "kind": "propagation",
      "file": ".github/workflows/privacy-gate.yml",
      "line": 87,
      "scope": "jobs.gate (uses: ./ at :87, job payer at :239)",
      "description": "This repository's own gate and payer jobs run the same harness/up.sh - the schedule (:23, cron 17 4 * * 1) re-runs it weekly on fresh runners - and the publish job (:110-163) renders the resulting runs to Pages, so the artifact published as the project's public privacy evidence is decided downstream of the unpinned containers."
    },
    {
      "kind": "propagation",
      "file": "crates/report/src/lib.rs",
      "line": 33,
      "scope": "Report / claim schema",
      "description": "What a claim binds: version, flow_id, flow_commit, chain ('regtest' as a self-asserted string), verified_at_height, verified_at, ttl_seconds, findings, evidence, merchant_is_simulated, report_hash - no image, digest, container or environment identity, so the container content behind a claim is unrecorded and uncheckable after the fact."
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/gate.rs",
      "line": 165,
      "scope": "gate verdict (chain_verified) and its published summary",
      "description": "The pass/fail verdict is re-derived from the same unbound node+indexer pair (verify_report at :156 reads the live node and indexer; the strongest chain check is the self-reported string at verify.rs:697), and the anchor it may publish (:181-206) is a shielded memo on the chain those containers produce, so the value that reaches the build result, the uploaded artifacts and the public claim index is produced by a component whose identity this repository never fixed or recorded."
    }
  ],
  "evidence": [
    {
      "file": "harness/docker-compose.yml",
      "line": 9,
      "description": "image: electriccoinco/zcashd:latest - floating tag, no @sha256 digest, no pull_policy, no platform"
    },
    {
      "file": "harness/docker-compose.yml",
      "line": 34,
      "description": "image: electriccoinco/lightwalletd:latest - same for the indexer; the second service mounts nothing and adds no identity requirement"
    },
    {
      "file": "harness/up.sh",
      "line": 60,
      "description": "the only resolution point: bare `docker compose up -d` (the compose header documents the same command for the user)"
    },
    {
      "file": "harness/up.sh",
      "line": 116,
      "description": "the rig's only docker inspect call (git grep 'docker inspect': 1 hit) reads .State.Running - liveness, not image identity"
    },
    {
      "file": "action.yml",
      "line": 85,
      "description": "run: harness/up.sh - the composite action third parties are told to use brings up the unpinned rig in their runner; its own diagnostics (:157) print {{.Image}} only, a name:tag, never a digest"
    },
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 239,
      "description": "run: harness/up.sh in the payer job; the gate job reaches the same step through uses: ./ at :87"
    },
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 23,
      "description": "weekly cron (17 4 * * 1): the gate re-probes on a fresh runner with no cached image, so a re-pointed tag is consumed by this repo's own CI with no repo change"
    },
    {
      "file": "harness/HARNESS.md",
      "line": 54,
      "description": "documents `electriccoinco/zcashd:latest` (v6.20.0) - a fixed version asserted next to a mutable tag, with no mechanism enforcing either the version or the identity"
    },
    {
      "file": "README.md",
      "line": 75,
      "description": "'zcashd v6.20.0 regtest + lightwalletd v0.5.5, one `docker compose up -d`' - the reproducibility the README sells is not bound to the pulled artifact"
    },
    {
      "file": "docs/SUBMISSION.md",
      "line": 36,
      "description": "'One command: harness/up.sh && scripts/demo-run.sh' - the judge-facing instruction resolves the unpinned tags on the judge's machine"
    },
    {
      "file": "docs/CI.md",
      "line": 113,
      "description": "'the run has a timestamp nobody had to be trusted for' - on this rig the timestamp (an anchor memo) and the run share the unpinned containers as their producer"
    },
    {
      "file": "crates/report/src/lib.rs",
      "line": 33,
      "description": "Report struct: the published claim carries no container/image/digest identity, so drift between what produced it and what the docs name is undetectable from the evidence"
    },
    {
      "file": "crates/report/src/verify.rs",
      "line": 697,
      "description": "the strongest source-visible chain control: equality of the node's self-reported getblockchaininfo.chain with the report's own 'regtest' string - a substituted node answers it trivially"
    },
    {
      "file": "scripts/phase0-warmup.sh",
      "line": 29,
      "description": "even the repo's own warm-up pulls the same floating tag (`electriccoinco/zcashd:latest`) with a bare docker pull, recording no digest"
    }
  ],
  "blockers": [
    "Registry fact, not observable from source and out of bounds for this run (no registry access, no pull or inspect of the rig's images): which manifest digest docker.io/electriccoinco/zcashd:latest and docker.io/electriccoinco/lightwalletd:latest resolve to, and whether either tag has been re-pointed since the reviewed revision (introduced with the tags in e79f4e6, 2026-10-11; `git log -S':latest' -- harness/docker-compose.yml` returns only that commit).",
    "Hosted-control fact: who holds write access to the two Docker Hub repositories (the electriccoinco org) and whether tag immutability/protection is enabled there - this decides whether a lower-trust writer of the tag exists at all, which is the precondition of the handoff.",
    "Consumer fact: whether any run (this repo's weekly cron, a push, or a third-party workflow using the action) executed between the reviewed commit and now against container content that differs from the reviewed-era image; decidable only from the registry plus Actions logs and artifact digests that are outside this unit's reach.",
    "No bounded local check can settle it: the decisive experiment needs a substituted node/indexer pair (a hostile lightwalletd answering CompactTxStreamer plus a node answering JSON-RPC) reachable by the probe, i.e. the rig's own images and host ports (18232/18233/9067/9068) - both explicitly out of bounds - and the approved sandbox (--network=none, no image pulls) cannot build or run them."
  ],
  "validation_plan": {
    "local": "On a machine allowed to reach the registry (read-only; nothing pushed or published): (1) at two times at least a day apart, resolve both tags without pulling - `docker buildx imagetools inspect electriccoinco/zcashd:latest` and the same for lightwalletd:latest (fallback `docker manifest inspect --verbose <tag>`) - recording the per-platform manifest digest; (2) in a disposable checkout of the reviewed commit run `harness/up.sh` once and immediately capture `docker inspect --format '{{index .RepoDigests 0}}' zanary-zcashd` and the same for zanary-lwd; a digest that changes between the two observations while harness/docker-compose.yml is unchanged is the missing drift observation, and if the machine that produced the recorded/golden evidence still holds those images, `docker image inspect --format '{{.RepoDigests}}'` it and compare (the recorded evidence itself cannot supply the baseline - that is the finding). A check needing no registry at all: `docker compose -f harness/docker-compose.yml config --images` shows the resolved references carry no @sha256.",
    "deployment": "Owner-observed, no probing: on the CI runner used by the privacy-gate jobs (and for one third-party caller of the action), add an always-run step alongside the existing rig-diagnostics step (action.yml:149-170) printing `docker inspect --format '{{.Config.Image}} {{index .RepoDigests 0}}' zanary-zcashd zanary-lwd` into the job summary, and read it across two runs a week apart; if the digests differ on an unchanged harness/docker-compose.yml, the tag drifted under a claim that is still published. Closure (repo-side, non-destructive): pin both services to the observed digests, have harness/up.sh compare the running containers' RepoDigests against the pins before the flow runs, and record the observed digest in the claim/summary so a claim names the container content that produced it."
  }
}
```

## Linked ledger unit

- coverage_id: `harness%2Fdocker-compose.yml%23services::electriccoinco%2Fzcashd%3Alatest%20and%20electriccoinco%2Flightwalletd%3Alatest%20image%20references::local-rig::SUPPLY-CHAIN-AND-RELEASE.md%23Dependency%20and%20build-input%20attack%20classes`
- surface: `harness/docker-compose.yml#services` · subsystem `local-rig`
- boundary: `electriccoinco/zcashd:latest and electriccoinco/lightwalletd:latest image references`
- assignment owner: `hunter-images`

The unit's checks (each `artifact` path is relative to the run directory; read the promoted files
under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/`):

```json
[
  {
    "agent_id": "hunter-images",
    "reviewed_paths": [
      "harness/docker-compose.yml",
      "harness/up.sh",
      "harness/HARNESS.md",
      "action.yml",
      ".github/workflows/privacy-gate.yml",
      "README.md",
      "docs/CI.md",
      "scripts/phase0-warmup.sh"
    ],
    "invariant": "both rig images are selected by an identity bound to the reviewed revision, and every consumer of the rig (the composite action, this repo's gate/payer jobs, the documented one-command setup) resolves exactly that artifact",
    "method": "source",
    "result": "refuted: docker-compose.yml:9 and :34 use floating :latest tags with no digest, no pull_policy and no platform pin; the tracked tree contains zero occurrences of @sha256: / pull_policy / repoDigests; resolution happens at the bare `docker compose up -d` (up.sh:60) and the rig's only inspect (up.sh:116) reads .State.Running; the callers are action.yml:85 (third-party consumers), .github/workflows/privacy-gate.yml:87/:239 (this repo's jobs); docs assert v6.20.0 / v0.5.5 (HARNESS.md:54-55, README.md:75) with nothing enforcing them.",
    "artifact": null
  },
  {
    "agent_id": "hunter-images",
    "reviewed_paths": [
      "harness/docker-compose.yml",
      "harness/up.sh",
      "action.yml",
      ".github/workflows/privacy-gate.yml",
      "crates/report/src/lib.rs",
      "harness/HARNESS.md"
    ],
    "invariant": "the repo-side facts above hold under mechanical re-derivation on a disposable copy, with no registry contact, and the recorded claim format really carries no environment identity",
    "method": "local",
    "result": "re-derived inside the approved sandbox (docker run --network=none, rust:1-slim, script on stdin, no target code executed, no image pulled or inspected): two image references and nothing else; 0 files matching @sha256:/pull_policy/repoDigests; callers action.yml:85 + privacy-gate.yml:239 (gate job via uses: ./ at :87); tracked claim fixtures mention no image/version identity (files=0) and the Report fields (crates/report/src/lib.rs:33-54) bind flow/chain/height/hash only; cited-file sha256 values match the target checkout (harness/docker-compose.yml 7d6e6470..., harness/up.sh 0958b05d...). Full transcript in the artifact.",
    "artifact": "agents/hunter-images/artifacts/check-images-refs.txt"
  },
  {
    "agent_id": "hunter-images",
    "reviewed_paths": [
      "crates/probe/src/main.rs",
      "crates/probe/src/core.rs",
      "crates/probe/src/gate.rs"
    ],
    "invariant": "the target code that consumes the rig can distinguish a substituted node/indexer from the reviewed one (any version, digest or chain-identity check on the client side, including at the gate's re-derivation)",
    "method": "source",
    "result": "no such control exists: crates/probe/src/main.rs:97-107 prints the indexer's self-reported chain_name/version/vendor and compares them to nothing; core.rs connects to the endpoint and waits for a tip height only (wait_for_indexer_height, core.rs:128-160); the flow runner's 'chain: regtest' is a self-asserted string in the artifact (flow.rs:396, payer.rs:394); the gate's chain_verified (gate.rs:165, summary fields :342-348) re-reads evidence from that same unbound pair. A substituted pair is therefore indistinguishable to the client, which is why the impact rides entirely on image identity - this unit's root cause.",
    "artifact": null
  },
  {
    "agent_id": "hunter-images",
    "reviewed_paths": [
      "rust-toolchain.toml",
      "docs/SUBMISSION.md"
    ],
    "invariant": "the entry points a judge or third-party consumer follows are bound to reviewed artifacts - the images the one command resolves, and the toolchain the build step resolves",
    "method": "source",
    "result": "refuted on both: docs/SUBMISSION.md:36 (and README.md:111) tells a judge to run `harness/up.sh && scripts/demo-run.sh`, which resolves the unpinned images with no digest check, and rust-toolchain.toml:4 sets channel = \"stable\" (its own comment :1-2 presents it as the pin against a pre-1.88 rustc) which action.yml:59 and privacy-gate.yml:50/:130/:231 install with a bare `rustup toolchain install` - a second unbound build input, recorded in this result as an uncovered variant because no current unit owns toolchain identity.",
    "artifact": null
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-36/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-36/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "zanary/rig-image-tags-unpinned"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
