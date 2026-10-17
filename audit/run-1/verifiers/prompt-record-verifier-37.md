# Phase-5 record verifier — verifier-37 on `zanary/rig-ports-published-all-interfaces`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`needs_validation`)

```json
{
  "verdict": "needs_validation",
  "fingerprint": "zanary/rig-ports-published-all-interfaces",
  "title": "Rig control plane (zcashd JSON-RPC) and the credential-less lightwalletd indexer are published without a host-IP qualifier, so the documented setup inherits the Docker engine's every-interface default, gated only by RPC credentials the public repository itself carries",
  "description": "harness/docker-compose.yml publishes the regtest node's JSON-RPC (host port 18232), its unused P2P port (18233) and lightwalletd's gRPC/HTTP (9067/9068) as unqualified PORT:PORT specs; no host interface is named anywhere in that file (mechanically re-derived: 4 publish specs, 0 host-IP-qualified, no loopback qualifier, no host_ip key), so the host-side binding falls to the Docker engine's default for an unqualified publish - every interface on a standard daemon - to be confirmed or refuted on the operator's engine. Nothing on the node then restricts who may authenticate: harness/zcash.conf - seeded verbatim onto a fresh checkout by harness/up.sh - sets rpcbind=0.0.0.0 and rpcallowip=0.0.0.0/0, and the accepted credential is the fixed regtest pair the same repository publishes (compose command line, conf seed, probe client default; equality re-derived by comparison, values never printed). A peer that can reach the port and has read the public repository is therefore the node operator: mine blocks at will (generate), stop the node, read node-wallet state, and invoke file-writing wallet RPCs whose outputs land in the datadir bind-mounted from the host checkout. lightwalletd is started --no-tls-very-insecure on 0.0.0.0 with no credential at all, giving the same peer a channel to the chain. The project's own model assumes the opposite (docs/THREAT-MODEL.md:113 'the harness runs everything on localhost'; harness/HARNESS.md:3 'no network beyond this machine'; HARNESS.md:55 'never expose it' for the no-TLS indexer), while README.md:111-114, docs/CI.md:66 and the composite action (action.yml:85) direct any reader to create the binding, including inside third-party CI jobs on infrastructure the authors do not control. Impact is bounded by the target: regtest only, no real value, no mainnet key material, and the only host path a rig container can write is data/zcashd (docker-compose.yml:18) - which is why this stays a validation lead rather than a confirmed finding: the decisive remaining facts are the engine's effective binding and off-host reachability (host firewall and network), both unobserved here.",
  "claimed_root_cause": "The documented one-command local rig publishes its control-plane and indexer ports without a loopback (host-IP) qualifier, and the node is configured to accept RPC from any source (rpcbind=0.0.0.0 / rpcallowip=0.0.0.0/0) under a credential committed to the public repository; no in-repo consumer needs off-host reachability, so the all-interface default binding is an unintended network exposure created wherever the documented setup runs.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "harness/docker-compose.yml",
      "line": 20,
      "scope": "zcashd service ports",
      "description": "The node's JSON-RPC host port is published as an unqualified scalar ('18232:18232'); the repository constrains no host interface anywhere (no host-IP port spec and no 127.0.0.1 in the file), so the host-side binding falls to the Docker engine's default for an unqualified publish - every host interface (to be confirmed on the operator's engine)."
    },
    {
      "kind": "propagation",
      "file": "harness/zcash.conf",
      "line": 15,
      "scope": "node RPC listener",
      "description": "rpcbind=0.0.0.0 makes the node listen on every container interface, so a connection forwarded through the published port is not refused inside the container."
    },
    {
      "kind": "propagation",
      "file": "harness/zcash.conf",
      "line": 16,
      "scope": "node RPC allowlist",
      "description": "rpcallowip=0.0.0.0/0 accepts RPC from every source address - no source-side restriction backs up the missing interface restriction."
    },
    {
      "kind": "propagation",
      "file": "harness/docker-compose.yml",
      "line": 24,
      "scope": "zcashd container command",
      "description": "The node is started with the fixed repository-published RPC username/password pair on its command line (lines 24-25; the healthcheck at line 28 and lightwalletd at lines 54-55 reuse the same pair), so authentication is a public constant, not a secret a peer must obtain."
    },
    {
      "kind": "propagation",
      "file": "harness/docker-compose.yml",
      "line": 49,
      "scope": "lightwalletd entrypoint",
      "description": "The indexer is started with --no-tls-very-insecure and 0.0.0.0 gRPC/HTTP binds (lines 49-51) with its ports published (lines 46-47): a second, credential-less service on the same host interfaces."
    },
    {
      "kind": "propagation",
      "file": "README.md",
      "line": 111,
      "scope": "documented usage",
      "description": "The README's one-command setup (also docs/CI.md:66 and, for third-party CI, action.yml:85) instructs anyone to run harness/up.sh, so the compose binding is created on machines the authors do not control."
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/node.rs",
      "line": 64,
      "scope": "node RPC client identity",
      "description": "The credential the node accepts is the same value the in-repo client defaults to (lines 64-66) and the client's entire authentication is HTTP Basic over plain HTTP (line 79; module doc lines 9-12): a network peer holding the published repository credential is indistinguishable from the rig's own tooling and, wherever the published port is reachable from that peer, reaches the node's full RPC surface, whose datadir is the host bind mount at docker-compose.yml:18."
    }
  ],
  "evidence": [
    {
      "file": "harness/docker-compose.yml",
      "line": 20,
      "description": "Host port 18232 (JSON-RPC) published with no host-IP qualifier; re-derived mechanically: 4 publish specs, 0 host-IP-qualified specs, no loopback qualifier and no host_ip key anywhere in the file."
    },
    {
      "file": "harness/docker-compose.yml",
      "line": 21,
      "description": "Port 18233 published too, described by the file itself as unused ('regtest has no peers') - surface with no in-repo consumer."
    },
    {
      "file": "harness/docker-compose.yml",
      "line": 46,
      "description": "lightwalletd's 9067/9068 published the same way; 9068 has no in-repo consumer - its only tracked references are this spec (lines 47, 51) and the HARNESS.md port table (line 55)."
    },
    {
      "file": "harness/zcash.conf",
      "line": 15,
      "description": "rpcbind=0.0.0.0 in the tracked conf that up.sh seeds onto a fresh checkout."
    },
    {
      "file": "harness/zcash.conf",
      "line": 16,
      "description": "rpcallowip=0.0.0.0/0 - the node's own allowlist admits every source address."
    },
    {
      "file": "harness/up.sh",
      "line": 50,
      "description": "The seed step copies the tracked conf (with its regtest RPC credential pair; values not reproduced here) to data/zcashd/.zcash/zcash.conf when absent, and never overwrites an existing one."
    },
    {
      "file": "harness/docker-compose.yml",
      "line": 24,
      "description": "The node's command line carries the credential pair (lines 24-25); the healthcheck (line 28) and lightwalletd (lines 54-55) reuse the same pair - equality with the conf seed and the probe default re-derived by comparison."
    },
    {
      "file": "harness/HARNESS.md",
      "line": 55,
      "description": "The project's own instruction for the no-TLS indexer: 'never expose it' - while the compose file publishes it on every interface by default."
    },
    {
      "file": "harness/HARNESS.md",
      "line": 60,
      "description": "The harness doc describes the seeded conf as 'regtest, RPC open to the docker bridge' - the openness is known; the missing control is the host-side interface restriction."
    },
    {
      "file": "harness/HARNESS.md",
      "line": 3,
      "description": "The harness's own stated model: 'Everything here is a test chain. No real funds, no faucet, no sync, no network beyond this machine' - the documented premise the unqualified publishes leave unenforced."
    },
    {
      "file": "docs/THREAT-MODEL.md",
      "line": 113,
      "description": "Stated premise used to scope network observers out: 'the harness runs everything on localhost'."
    },
    {
      "file": "crates/probe/src/node.rs",
      "line": 26,
      "description": "Every in-repo consumer of the node RPC defaults to loopback (this line; the indexer in crates/probe/src/core.rs:101; both in harness/up.sh:25-26 and crates/report/src/node.rs:186, crates/report/src/verify.rs:127) - no documented use needs off-host reachability."
    },
    {
      "file": "action.yml",
      "line": 85,
      "description": "The composite action runs harness/up.sh inside a third-party caller's CI job, so the binding is created on a stranger's runner."
    },
    {
      "file": "crates/probe/src/node.rs",
      "line": 79,
      "description": "The probe client's entire authentication is an HTTP Basic header over plain HTTP (module doc, lines 9-12: the harness endpoint is plain http with HTTP basic auth) - the same single-factor scheme any network peer would use."
    },
    {
      "file": "harness/docker-compose.yml",
      "line": 18,
      "description": "The only host path the stack mounts is ../data/zcashd (lightwalletd mounts nothing), which bounds any file effect reachable through the exposed RPC surface."
    }
  ],
  "blockers": [
    "Effective host-side binding is unobserved: the reviewed revision qualifies no publish with a host IP (docker-compose.yml:20, 21, 46, 47), so the bind falls to the engine's default for an unqualified publish - all host interfaces on a standard Docker daemon - but `docker port`/`docker inspect` on the running rig and any connection to the rig's ports are out of bounds for this validation, and an engine or runtime that deviates from the default would change it.",
    "Off-host reachability is a deployment fact: the host firewall/security group and the host's network position (developer LAN, shared or CI network, cloud address) decide whether a second host can reach 18232/9067; the repository's own model assumes localhost, and the one documented path that runs the rig on infrastructure the authors do not control is the composite action (action.yml:85) on a caller's runner.",
    "zcashd v6.20.0 option precedence between the container command line (docker-compose.yml:24-25) and the seeded conf gates only the sub-claim that the credential cannot be rotated through documented knobs - a node-behaviour fact outside this repository; every value in play is the same committed pair, so nothing in the exposure turns on it."
  ],
  "validation_plan": {
    "local": "Completed and repeatable offline (re-derived by this verifier in the sandbox on a disposable copy of the reviewed revision - both trees hash-pinned: harness/docker-compose.yml sha256 7d6e6470..., harness/zcash.conf sha256 b635374b...): all four publish specs are host-IP-unqualified (lines 20, 21, 46, 47); no 127.0.0.1 and no host_ip key anywhere in the compose file; harness/zcash.conf:15-16 carry rpcbind=0.0.0.0 and rpcallowip=0.0.0.0/0; the conf credential equals the compose command-line value and the probe default (compared, never printed); every consumer default is loopback (crates/probe/src/node.rs:26, crates/probe/src/core.rs:101, harness/up.sh:25-26, crates/report/src/node.rs:186, crates/report/src/verify.rs:127). Nothing further about the exposure is observable locally - the effective bind requires the engine - so the only remaining local step is to re-run this parse on a fresh git-archive copy; it cannot settle the claim.",
    "deployment": "Operator-observed, on hardware the owner controls: (1) run `docker port zanary-zcashd` and `docker port zanary-lwd` on the rig host - a 0.0.0.0/:: mapping confirms the every-interface bind, a 127.0.0.1 mapping refutes it; (2) from a second host the owner controls on the same network (never a network the owner does not control) make one bounded getblockcount against port 18232 with the repository credential and one bare TCP connect to 9067 - a JSON reply / accept confirms off-host reachability including the host firewall, a refusal refutes it; (3) apply the smallest fix - prefix 127.0.0.1: to the four port specs and optionally drop 18233 and 9068, which have no in-repo consumer - then repeat step 2 to show the path closes; every documented use keeps working because every in-repo consumer already targets 127.0.0.1."
  }
}
```

## Linked ledger unit

- coverage_id: `harness%2Fdocker-compose.yml%23services::harness%2Fzcash.conf%23rpc-credentials::local-rig::ATTACK-CLASSES.md%23Resource%20and%20file%20handling`
- surface: `harness/docker-compose.yml#services` · subsystem `local-rig`
- boundary: `harness/zcash.conf#rpc-credentials`
- assignment owner: `hunter-rig`

The unit's checks (each `artifact` path is relative to the run directory; read the promoted files
under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/`):

```json
[
  {
    "agent_id": "hunter-rig",
    "reviewed_paths": [
      "harness/docker-compose.yml"
    ],
    "invariant": "the compose file constrains the host interface of every published rig port",
    "method": "source",
    "result": "refuted: all four publishes are bare PORT:PORT scalars (docker-compose.yml:20, 21, 46, 47); the file contains no host-IP port spec and no loopback qualifier anywhere, so nothing in the repository restricts the host-side binding the documented setup creates.",
    "artifact": null
  },
  {
    "agent_id": "hunter-rig",
    "reviewed_paths": [
      "harness/zcash.conf",
      "harness/docker-compose.yml"
    ],
    "invariant": "the node refuses RPC from unexpected sources, or binds only the interface it needs",
    "method": "source",
    "result": "refuted: rpcbind=0.0.0.0 (zcash.conf:15) and rpcallowip=0.0.0.0/0 (zcash.conf:16) admit every address, and the container command (docker-compose.yml:22-26) starts the node with the same credential pair the tracked conf seed carries, so a connection forwarded through the published port is accepted and the shared credential is the whole gate.",
    "artifact": null
  },
  {
    "agent_id": "hunter-rig",
    "reviewed_paths": [
      "harness/docker-compose.yml",
      "harness/zcash.conf",
      "harness/up.sh",
      "crates/probe/src/node.rs"
    ],
    "invariant": "the credential the node accepts is rotatable by the person running the documented setup",
    "method": "source",
    "result": "not rotatable through the documented knobs: up.sh's ZANARY_NODE_USER/ZANARY_NODE_PASS (up.sh:23-24) feed only the docker-exec CLI calls (up.sh:33); the probe reads ZANARY_NODE_RPC_USER/PASSWORD but defaults to the same repo-published pair (node.rs:47-48, 64-66); the compose command line re-passes the pair at every container start (docker-compose.yml:24-25) and the healthcheck reuses it (line 28). The local check confirmed conf seed = command-line value = probe default; zcashd command-line-vs-conf precedence stays a node-side fact (blocker 3).",
    "artifact": null
  },
  {
    "agent_id": "hunter-rig",
    "reviewed_paths": [
      "crates/probe/src/node.rs",
      "harness/docker-compose.yml",
      "harness/zcash.conf"
    ],
    "invariant": "RPC peer identity is bound to something beyond one shared constant (per-peer, channel, or source restriction)",
    "method": "source",
    "result": "refuted: the only identity on the node RPC is HTTP Basic with the single shared pair over plain HTTP (node.rs:79; module doc lines 9-12), rpcallowip=0.0.0.0/0 attaches no source restriction, and the indexer hop carries no credential and no TLS at all (docker-compose.yml:48-57) - anything that can reach the ports and has read the public repository is indistinguishable from the rig's own tooling.",
    "artifact": null
  },
  {
    "agent_id": "hunter-rig",
    "reviewed_paths": [
      "docs/THREAT-MODEL.md",
      "harness/HARNESS.md",
      "crates/probe/src/node.rs",
      "crates/probe/src/core.rs",
      "harness/up.sh",
      "crates/report/src/node.rs",
      "crates/report/src/verify.rs"
    ],
    "invariant": "the repository's stated boundary (rig is localhost-only) matches what the compose file binds",
    "method": "source",
    "result": "contradicted: docs/THREAT-MODEL.md:113 asserts 'the harness runs everything on localhost' (network observers scoped out on that premise), HARNESS.md:55 says of the no-TLS indexer 'never expose it', and every consumer default is loopback (node.rs:26, core.rs:101, up.sh:25-26, report/src/node.rs:186, report/src/verify.rs:127) - while the compose publishes are interface-unqualified. The smallest fix (loopback-qualified publish specs) breaks no documented use.",
    "artifact": null
  },
  {
    "agent_id": "hunter-rig",
    "reviewed_paths": [
      "README.md",
      "docs/CI.md",
      "action.yml",
      ".github/workflows/privacy-gate.yml",
      "harness/seed.sh"
    ],
    "invariant": "any exposure created by the compose binding stays on the authors' own host",
    "method": "source",
    "result": "refuted: README.md:111-114 and docs/CI.md:66 tell any reader to run harness/up.sh; action.yml:85 runs it in a third-party caller's job, .github/workflows/privacy-gate.yml:239 in this repository's own CI, and harness/seed.sh:4 documents the same raw 'docker compose up -d' - the binding is created wherever the documented setup runs, including machines the authors do not control.",
    "artifact": null
  },
  {
    "agent_id": "hunter-rig",
    "reviewed_paths": [
      "harness/up.sh",
      "harness/seed.sh",
      "harness/docker-compose.yml",
      "crates/probe/src/node.rs"
    ],
    "invariant": "no lower-trust input selects a file path across this boundary (the assigned Resource-and-file-handling traversal/SSRF/temp-file shapes have a sink here)",
    "method": "source",
    "result": "held: up.sh writes only the fixed $ROOT/data/zcashd/.zcash/zcash.conf (up.sh:27-28, 45-52), seed.sh only under $ROOT/data (seed.sh:18), the compose stack mounts one fixed relative path (docker-compose.yml:17-18), and the probe's node client takes env-supplied host/port/credential from the same user and writes no files. The only file-adjacent effect a lower-trust principal can reach in this unit is through the exposed RPC surface (see the candidate); up.sh's check-then-copy seed race is same-principal and recorded as hardening.",
    "artifact": null
  },
  {
    "agent_id": "hunter-rig",
    "reviewed_paths": [
      "harness/docker-compose.yml",
      "harness/zcash.conf",
      "harness/up.sh",
      "crates/probe/src/node.rs",
      "crates/probe/src/core.rs",
      "crates/report/src/node.rs",
      "crates/report/src/verify.rs",
      "docs/THREAT-MODEL.md",
      "harness/HARNESS.md",
      "README.md",
      "docs/CI.md",
      "action.yml",
      ".github/workflows/privacy-gate.yml"
    ],
    "invariant": "the repo-side facts hold under mechanical re-derivation on a disposable copy",
    "method": "local",
    "result": "held: no host-IP port spec and no loopback qualifier in the compose file; rpcbind/rpcallowip present in the conf; the conf credential equals the compose command-line value and the probe default (compared, never printed); every consumer of both ports defaults to 127.0.0.1; the documented entry paths that create the binding were enumerated in the same sandbox pass. The live host binding and external reachability were deliberately not observed (out of bounds; deployment step).",
    "artifact": "agents/hunter-rig/artifacts/check-rig-ports.json"
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-37/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-37/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "zanary/rig-ports-published-all-interfaces"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
