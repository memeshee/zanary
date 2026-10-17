# Candidate verifier prompt — verifier-18 on `zanary/rig-ports-published-all-interfaces`

You did not write this candidate. Try to refute it from repository source and bounded
local evidence. Do not contact deployed endpoints or external/shared services. Run
target-controlled code only inside the approved OS-enforced sandbox: no external
network, empty allowlisted environment, read-only target and tools, scratch-only
writes, and explicit low resource and wall-clock limits. If any control is unavailable,
do not execute; retain the exact missing capability as a needs_validation blocker.
Treat every scratch entry as target-controlled after execution. After the sandbox and
all its processes terminate, only trusted parent-side code may promote a predeclared
scratch-relative file, following the promotion procedure block included verbatim in
this prompt. You and target code never write retained artifacts. If promotion is
unavailable or fails, do not use that file as evidence.

1. Verify every trace and evidence file, positive line number, scope, and description.
   Confirm the first entry is a real lower-trust entrypoint and the last is the
   claimed sink or boundary effect.
2. Reconstruct the strongest source-visible validation, identity, authorization,
   normalization, lifecycle, framework, and containment controls on the path.
   Where the architecture summary names a comparable baseline, note whether it
   shares the pattern — as calibration, never as grounds to dismiss.
3. For a proposed confirmed candidate, independently reproduce the minimum observed
   result when possible. Verify inputs, interface shape, conditions, and affected
   dummy principal/resource. Do not infer a stronger result or continue after it.
4. Verify that likelihood, impact, confidence, and the proposed source fix match only
   what the evidence establishes.
5. For a proposed needs_validation candidate, decide whether the blocker is genuinely
   outside source/local observation. If source refutes the trace, reject it. If the
   missing fact remains decisive, keep needs_validation and make the local and
   owner-observed plans exact and non-destructive.
6. Preserve the fingerprint for the same source-derived root cause across every state.

Return exactly one JSON object and no surrounding prose:
{"decision": "confirmed|needs_validation|rejected", "record": { ... }}
where record exactly matches the decision's verdict branch of the schema included
in this prompt. A corrected record replaces the hunter's wording.

## The candidate you must try to refute

Hunter: `hunter-rig` · proposed verdict: `needs_validation` · fingerprint: `zanary/rig-ports-published-all-interfaces`

```json
{
  "proposed_verdict": "needs_validation",
  "fingerprint": "zanary/rig-ports-published-all-interfaces",
  "title": "Rig control plane (zcashd JSON-RPC) and lightwalletd indexer are published on every host interface, protected only by RPC credentials the public repository itself carries",
  "description": "harness/docker-compose.yml publishes the regtest node's JSON-RPC (host port 18232), its P2P port (18233) and lightwalletd's gRPC/HTTP (9067/9068) as unqualified PORT:PORT specs; the repository never names a host interface, so the binding falls to the Docker daemon's default for an unqualified publish, which is every interface (to be confirmed on the operator's engine). Nothing on the node restricts who may then authenticate: harness/zcash.conf - seeded verbatim onto a fresh checkout by harness/up.sh - sets rpcbind=0.0.0.0 and rpcallowip=0.0.0.0/0, and the port's only authentication is HTTP Basic with the fixed regtest RPC credential pair that the same repository publishes in the compose command line, in the conf seed, and as the probe client's default. A peer on any network the host is attached to - a LAN, a shared or CI network, a cloud VM's public address - that can reach the port is therefore the node operator: mine blocks at will (generate), stop the node, read node-wallet state, and invoke file-writing wallet RPCs whose outputs land in the datadir bind-mounted from the host checkout (bounded to data/zcashd). lightwalletd is started --no-tls-very-insecure on 0.0.0.0 and gives the same peer an unauthenticated channel to the chain. The project's own model assumes the opposite: docs/THREAT-MODEL.md:113 states 'the harness runs everything on localhost' and harness/HARNESS.md:55 warns of the no-TLS indexer 'never expose it', while README.md:111-114, docs/CI.md:66 and the composite action (action.yml:85) tell strangers to run harness/up.sh on their own machine or runner. Impact is bounded by the target: regtest only, no real value, no mainnet key material, and the only host path any container can write is data/zcashd. Because the decisive question - what the host-side binding actually is and whether a second host can reach it - is an operator-visible deployment fact (host firewall and network included), this is filed for validation rather than confirmed.",
  "claimed_root_cause": "The documented one-command local rig publishes its control-plane and indexer ports without a loopback (host-IP) qualifier, and the node is configured to accept RPC from any source (rpcbind=0.0.0.0 / rpcallowip=0.0.0.0/0) under a credential committed to the public repository; no in-repo consumer needs off-host reachability, so the all-interface default binding is an unintended network exposure created wherever the documented setup runs.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "harness/docker-compose.yml",
      "line": 20,
      "scope": "zcashd service ports",
      "description": "The node's JSON-RPC host port is published as an unqualified scalar ('18232:18232'); the repository constrains no host interface anywhere (no host-IP port spec and no 127.0.0.1 in the file), so the host-side binding is the Docker daemon's default for an unqualified publish - every interface."
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
      "description": "The credential the node accepts is the same value the in-repo client defaults to (lines 64-66) and the client's entire authentication is HTTP Basic over plain HTTP (line 79; module doc lines 9-12): a network peer holding the published repository credential is indistinguishable from the rig's own tooling and reaches the node's full RPC surface, whose datadir is the host bind mount at docker-compose.yml:17-18."
    }
  ],
  "evidence": [
    {
      "file": "harness/docker-compose.yml",
      "line": 20,
      "description": "Host port 18232 (JSON-RPC) published with no host-IP qualifier; verified mechanically that the file contains no host-IP port spec and no loopback qualifier."
    },
    {
      "file": "harness/docker-compose.yml",
      "line": 21,
      "description": "Port 18233 published too, described by the file itself as unused ('regtest has no peers') - surface with no in-repo consumer."
    },
    {
      "file": "harness/docker-compose.yml",
      "line": 46,
      "description": "lightwalletd's 9067/9068 published the same way; 9068 is referenced nowhere else in the repository."
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
      "description": "The node's command line carries the credential pair (lines 24-25); the healthcheck (line 28) and lightwalletd (lines 54-55) reuse the same pair."
    },
    {
      "file": "harness/HARNESS.md",
      "line": 55,
      "description": "The project's own instruction for the no-TLS indexer: 'never expose it' - while the compose file publishes it on every interface."
    },
    {
      "file": "harness/HARNESS.md",
      "line": 60,
      "description": "The harness doc describes the seeded conf as 'regtest, RPC open to the docker bridge' - the openness is known; the missing control is the host-side interface restriction."
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
    }
  ],
  "blockers": [
    "Host-side binding and reachability were not observable from this hunter: the unit's guardrail forbids docker inspect/exec on the rig's containers and any connection to the rig's ports, so the daemon's effective publish ('0.0.0.0:18232' vs '127.0.0.1:18232') is unobserved. Docker's default for an unqualified publish is every interface, but that is a daemon-version fact to confirm on the operator's engine, not a repository fact.",
    "Whether any peer can actually reach the host's ports is a deployment fact: the host's firewall/security group and its network (LAN, NAT, public cloud address) decide it, and neither is visible from source.",
    "zcashd v6.20.0's option precedence between the container command line (docker-compose.yml:24-25) and the seeded conf file gates only the sub-claim that a user cannot rotate the credential in place - a node-behaviour fact outside this repository."
  ],
  "validation_plan": {
    "local": "Done once in the sandbox and recorded as agents/hunter-rig/artifacts/check-rig-ports.json: on a disposable copy, assert that every published port spec in harness/docker-compose.yml is host-IP-unqualified, that the file contains no loopback qualifier, that the credential seeded from harness/zcash.conf equals the compose command-line value and the probe default (compared, never printed), and that every consumer default is 127.0.0.1. A verifier can repeat the same offline parse; nothing above needs a live service.",
    "deployment": "Operator-observed, on hardware the owner controls: (1) run 'docker port zanary-zcashd' and 'docker port zanary-lwd' on the rig host - a 0.0.0.0/:: binding confirms the exposure, a 127.0.0.1 binding refutes it; (2) from a second host on the same network (or the host's public address where it has one) make one bounded RPC call with the repository credential (for example getblockcount) and a bare TCP connect to 9067 - a JSON reply confirms the off-host control path, a refusal refutes it (host firewall included); (3) repeat step 2 after adding a 127.0.0.1 host-IP qualifier to the compose port specs to show that the smallest fix closes the path. Never run any of this against hosts the owner does not control."
  }
}
```

## Its linked coverage unit and the hunter's checks

- coverage_id: `harness%2Fdocker-compose.yml%23services::harness%2Fzcash.conf%23rpc-credentials::local-rig::ATTACK-CLASSES.md%23Resource%20and%20file%20handling`
- surface: `harness/docker-compose.yml#services` (subsystem `local-rig`)
- boundary: `harness/zcash.conf#rpc-credentials`
- attack class: `ATTACK-CLASSES.md#Resource and file handling` · assignment owner: `hunter-rig`
- unit reviewed paths: [".github/workflows/privacy-gate.yml", "README.md", "action.yml", "crates/probe/src/core.rs", "crates/probe/src/node.rs", "crates/report/src/node.rs", "crates/report/src/verify.rs", "docs/CI.md", "docs/THREAT-MODEL.md", "harness/HARNESS.md", "harness/docker-compose.yml", "harness/seed.sh", "harness/up.sh", "harness/zcash.conf"]

The unit's checks, including the local checks whose promoted artifacts you may read
(paths are relative to the run directory `/home/kiter/security-audit-skill/zanary/run-1`):

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

Read the promoted files under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/` — those are the hunter's retained
local evidence. You may read a hunter's artifact, but every decisive claim must be re-established by you.

## Architecture facts (verbatim, written by the parent)

# Architecture — zanary (Phase 1 recon, run in the parent)

Target: `/home/kiter/hackathon/zanary` · repo `memeshee/zanary` · commit `7058406` · worktree clean at recon.
Profile `standard`, budget 14 agent invocations. Run dir outside the target: `~/security-audit-skill/zanary/run-1`.

## What the system is

A Rust workspace (38 `.rs` files, 14,465 lines, 100 tracked files) that runs a Zcash **regtest** payment
flow on a throwaway chain, observes what the chain/merchant/observer can see, scores it against 13 named
leak properties, hashes the evidence, and publishes claims that expire unless re-proved.

Crates: `probe` (CLI + rig driver + flow runner + bounded payer + CI gate), `properties` (the 13
properties + fixtures), `report` (report/anchor/verify/index/matrix + the renderer for the published page),
`observation` (the observation model, grants, ledger), plus fixtures/tests and `bin/zanary-seed`.

## Entry points and who can reach them

| entry point | reachable by | what it trusts |
|---|---|---|
| `zanary-probe {wallet,address,fund,shield,pay,anchor,flow,test,payer,invoice,verify-invoice}` | whoever runs the binary | CLI args, flow/grant/invoice JSON, env (`ZANARY_LWD_URL`, `ZANARY_UID/GID`, timeouts) |
| `zanary-report {verify,verify-receipt,build,index,matrix,verify-anchor}` | local user, CI | report/run/receipt JSON + a node/indexer URL |
| `harness/up.sh` + `harness/docker-compose.yml` | local user, CI, **any consumer of the composite action** | docker/podman availability, `ZANARY_UID/GID` |
| `action.yml` (composite GitHub Action, `uses: memeshee/zanary@main`) | **any third-party workflow** | action `inputs:` — interpolated into `run:` steps |
| `.github/workflows/privacy-gate.yml` (5 jobs: tests, private, control, payer, publish) | push/PR/dispatch/**weekly cron** | repo contents; publishes Pages + uploads artifacts |
| `scripts/{demo-run.sh,bounded-payer-demo.sh,ci-summary.py}` | local user, CI | env knobs `ZANARY_DEMO_RUNS`, `ZANARY_PAYER_DEMO_DIR`, `ZANARY_LOG` |
| published Pages site (`/`, `/video/`) + uploaded CI artifacts | **the public internet** (public repo, public Pages) | the evidence files it renders/copies |

## Trust boundaries

1. **Third-party CI ↔ this repo's automation.** `action.yml` is designed so a stranger's workflow runs it
   (`uses: memeshee/zanary@main` is the whole setup). Anything it interpolates from `inputs:` into a shell
   is a boundary between that consumer's workflow and their runner.
2. **Public repo/artifacts ↔ the rig's secrets.** Actions artifacts on a public repo are world-downloadable;
   the Pages deploy is world-readable. Wallet material, viewing keys, or RPC credentials reaching either is
   a disclosure boundary crossing.
3. **Untrusted data ↔ the probe CLI.** Flow definitions, invoices, grants and reports are JSON files that a
   user (or a judge, or CI) supplies; `--out`/`--definition`/`--grant` are caller-controlled paths.
4. **Host network ↔ the local rig.** `harness/docker-compose.yml` publishes zcashd JSON-RPC (18232/18233)
   and lightwalletd gRPC/HTTP (9067/9068) on the host.
5. **Authority model ↔ money movement.** The bounded payer decides whether a payment happens, from a grant
   (budget, per-payment cap, receiver allowlist, expiry, kill switch). Its ledger is the audit trail; the
   `payer-policy` property is the detector.
6. **Claim ↔ observer.** The published evidence (receivers, txids, memos, pool totals) is *intended* to be
   public; the tool's promise is that it never publishes more than the claim needs.

## Secrets and sensitive state

- `harness/zcash.conf` carries `rpcuser`/`rpcpassword` for the regtest node — **committed on purpose** and
  documented as throwaway, in a repo whose README tells people to run it locally. Not a secret in the usual
  sense; it *is* the credential that protects the JSON-RPC control plane (boundary 4).
- Wallet DBs, chain data, per-run evidence and keys live under `data/` and are gitignored (`/data`).
- No API keys, tokens, or `.env` in the tree (checked: no tracked file matches `*.env|secret|key|token|pem`).
- `video/zanary-demo.mp4` (3.6 MB) is a committed artifact, deliberately.

## What the tests need

`cargo test --workspace` is offline (fixtures + golden reports). The demo scripts, the flow runner and the
gate need **Docker plus the local regtest node/indexer over loopback** (`ZANARY_LWD_URL=http://127.0.0.1:9067`).
Anything requiring that rig is outside a `--network=none` sandbox and can only ever land as
`needs_validation` unless an offline path exists.

## Sandbox available

The methodology requires OS-enforced sandboxing for target-controlled execution: no external network,
read-only target, scratch-only writes, bounded resources.

Available: Docker with locally-present images (`rust:1-slim`, `node:22-slim`, `rust:1-slim` verified for
`cargo --version`), `--network=none`, a disposable `git archive HEAD` copy at
`/home/kiter/.hermes/cache/scratch/zanary-sandbox`, and the host cargo registry mounted read-only at
`/usr/local/cargo/registry` so `cargo test --offline` needs no network. Smoke-tested before dispatch —
see `hunter-protocol.md`.

**Sandbox proof (run before the hunters' results were collected).** From the disposable
`git archive HEAD` copy with `rust-toolchain.toml` removed (the pin names `channel = "stable"`, which rustup
can only resolve with network — the sandbox correctly refused: `dns error: Temporary failure in name
resolution`, no silent fetch) and the host registry mounted read-only:

```
docker run --rm --network=none -v $SB:/t -v ~/.cargo/registry:/usr/local/cargo/registry:ro -w /t \
  rust:1-slim cargo test --offline -p zanary-properties --lib
→ test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured
```

So: the unit suite runs offline inside the sandbox, target code executes with no network, and a local check
is available to hunters for this unit class.

Not available: any sandbox that can reach the regtest chain (the rig's containers are shared host state and
out of bounds for hunters), and any image that is not already local (no pulls, by policy).

## Parent-side scans performed (coverage, recorded here)

Run in the parent before wave 1, read-only, over the whole history (`git rev-list --all`) rather than the
working tree only:

| scan | result |
|---|---|
| secret-shaped patterns across every commit (`uview1`, `upriv1`, `secret-extended-key`, `ghp_`, `AKIA`, `AIza`, `BEGIN … PRIVATE KEY`, `lightwalletd.*password`) | **0 hits** |
| files ever added or deleted on secret-shaped paths | only source/docs names (`wallet.rs`, `WALLET-NOTES.md`, `harness/seed.sh`, the pre-rename `canary-seed.rs`) |
| was `data/` (wallet DB, chain data, per-run evidence) ever committed? | **never** — 0 paths in history |
| anything sensitive still tracked today (`wallet`, `*.key`, `seed`, `*.env`, `*.pem`) | only source files whose *names* match; no key material |
| viewing keys inside recorded evidence (`data/runs/demo/demo-private/{report,run}.json`) | 0 hits for `ufvk`/`uview`/`unified_full_viewing` |

One observation belongs to the published-evidence unit rather than here, and the parent is not the unit's
hunter: `docs/WALLET-NOTES.md` commits a dev **seed phrase** and dev **UFVK**. The phrase is the canonical
BIP-39 test vector (`abandon ×11 · about`) and the wallet is a throwaway regtest wallet, so nothing private
is disclosed — no key material of value exists to disclose. If a finding, it is a *hardening* note about the
habit (a doc-level pattern that would be a disclosure if repeated on a wallet with real history), not a
crossed boundary. Recorded here so the report does not have to rediscover it.

## Coverage note

No prior `run-*` directory exists for this target, so this is the first pass: one wave of 5 units, a
post-wave coverage critic and a distinct final-clean critic, and one fresh verifier per candidate. Absence
of findings on a unit means that unit's assignment was covered, never that the repo is clean.

## Companion validation blocks for this unit (verbatim)

### [companion validation block] CLOUD-AND-DEPLOYMENT.md#Validation rules — verbatim

## Validation rules (apply before reporting ANY finding here)

1. Establish the active source path and effective deployment object; otherwise use `needs_validation` and state which rendered manifest or owner-observed attachment is missing.
2. Name the lower-trust caller/workload, cloud or application identity, controllable selector, affected resource, and unauthorized operation or disclosure.
3. Verify provider and orchestrator defaults at the pinned version. Do not assume a public IP, reachable metadata service, permissive firewall, or absent admission attachment.
4. Local validation may render templates, evaluate policy, inspect container/user namespaces in an isolated fixture, or run an emulator with dummy identities. Do not probe live endpoints or alter shared cloud resources.
5. Return `confirmed` only with a complete active source trace and concrete boundary result. Return `needs_validation` with the exact deployed policy, identity attachment, overlay, network, or drift observation needed.

## Promotion procedure (verbatim; reference only — the parent performs it, you never do)

```text
Artifact promotion procedure (trusted parent-side code only):
Reference only for you: the parent performs these steps; you never perform them.

Before execution, the parent opens and retains trusted, non-inheritable directory
descriptors for the agent's scratch/ and artifacts/ roots, and records an allowlist
of expected scratch-relative artifact files plus explicit per-file and cumulative
byte limits. Never pass those descriptors to the agent or sandbox. After the sandbox
and all its processes terminate, trusted parent-side code promotes each allowlisted
file separately:

1. Validate the declared relative path: reject absolute, empty, `.`, `..`, or
   symlinked components.
2. Walk each parent component from the retained scratch-root descriptor with
   no-follow directory-relative operations; never reopen by path.
3. Open the leaf no-follow and nonblocking.
4. Verify with `fstat` that it is a regular file with link count exactly one and
   within the recorded per-file and cumulative byte limits.
5. Enforce those limits again while reading from that descriptor.
6. Copy exactly the verified size, repeat `fstat`, and reject a changed identity,
   type, link count, or size.
7. For the destination, walk every parent component from the retained
   artifacts-root descriptor with no-follow directory-relative operations; require
   each existing component to be a real directory, and create any missing directory
   exclusively before reopening and verifying it no-follow.
8. Create the leaf exclusively without following links, verify that the opened
   destination is a regular file with link count exactly one, and copy from the
   verified source descriptor without reopening either path.
9. Use equivalent race-safe APIs on non-POSIX systems.
10. Never recursively copy or glob scratch, extract an archive into artifacts, or
    open or promote a symlink, FIFO, socket, device, directory, hard-linked file,
    changing file, or file that exceeds its bound.
11. If any check is unavailable, cannot be enforced, or fails, discard the scratch
    entry; if it is decisive evidence, retain `needs_validation` with the exact
    promotion blocker.
```

## Source/local execution boundary and the approved sandbox

Target source is read-only: `/home/kiter/hackathon/zanary` (commit `7058406`, worktree clean — do not modify it).
Target-controlled code may run only inside the OS-enforced sandbox. The approved disposable copy of the
reviewed revision is `/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` (`git archive HEAD` copy with `rust-toolchain.toml` removed so the image's own
toolchain is used — do not restore the pin). Rebuild it yourself if needed:
`rm -rf /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && mkdir -p /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && cd /home/kiter/hackathon/zanary && git archive HEAD | tar -x -C /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && rm -f /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2/rust-toolchain.toml`

```bash
docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 \
  -v /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2:/t -v /home/kiter/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim \
  sh -c 'CARGO_BUILD_JOBS=6 cargo test --offline -p <crate> <filter> 2>&1 | tail -30'
```

No external network, no image pulls, no `sudo`, no host paths except `/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` and the read-only registry,
writes only inside `/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` and your own scratch. Out of bounds: the regtest rig's containers and host ports
(18232/18233/9067/9068), any network service, any public URL, and anything that mutates state outside your
scratch. If a decisive fact lives behind one of those, it stays a `needs_validation` blocker — name it exactly.

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-18/scratch` (scratch-relative files only, `check-*.txt` / `check-*.json` /
`evidence.txt`, 64 KiB per file, 256 KiB total); the parent promotes into `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-18/artifacts`. You never write
retained artifacts, another agent's directory, or any shared run file.

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

## Prior records with the same fingerprint

None. This target has no prior audit run, and no other current candidate carries this fingerprint.

## Your result

Return exactly one JSON object, no prose and no code fences:
`{"decision": "confirmed|needs_validation|rejected", "record": { ... }}` where `record` exactly matches
the decision's verdict branch above (`confirmed` records carry `root_cause`, `intended_behavior`,
`conditions`, `execution`, `remediation`, `severity`, `confidence`; `needs_validation` records carry
`claimed_root_cause`, `blockers`, a nonempty `validation_plan` with at least one applicable `local` or
`deployment` step and no severity or execution; `rejected` records carry `claimed_root_cause` and `reason`).
A corrected record replaces the hunter's wording. Keep the fingerprint unchanged unless you found a
genuinely different root cause — and then say so in the record.
