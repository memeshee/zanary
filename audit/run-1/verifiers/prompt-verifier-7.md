# Candidate verifier prompt — verifier-7 on `probe/payer.rs:load_invoices:unbounded-intake-memory`

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

Hunter: `hunter-invoices` · proposed verdict: `confirmed` · fingerprint: `probe/payer.rs:load_invoices:unbounded-intake-memory`

```json
{
  "proposed_verdict": "confirmed",
  "fingerprint": "probe/payer.rs:load_invoices:unbounded-intake-memory",
  "title": "Bounded payer reads a merchant-supplied invoice directory into memory with no size or count cap, before and independent of grant authority",
  "description": "load_invoices (crates/probe/src/payer.rs:495-520) enumerates every *.json entry in the --invoices path and reads each file whole with std::fs::read_to_string plus serde_json::from_str, retaining every parsed Invoice in one Vec and (for invoices decide() allows) a second full clone in to_pay (payer.rs:259). There is no per-file size cap, no invoice-count cap and no memory budget, and the whole intake runs before the first decide() call (payer.rs:201 vs 250), so it is performed even when the grant is revoked or the kill file exists: the kill switch limits payments, not intake work. Observed locally against the reviewed revision: a 256 MiB invoice costs about 526-527 MB peak RSS (read text plus serde String, roughly 2x the file) both with a revoked and a full grant; 5000 small invoice files are all accepted; and a 768 MiB invoice under a 1.2 GB address-space limit stops the run at intake with 'out of memory' before a single decision is printed, while the same limit with tiny and 256 MiB fixtures completes intake normally.",
  "root_cause": "The invoice directory is an untrusted merchant channel, but intake is unbounded and unconditional: read_dir collects an unlimited number of *.json entries (payer.rs:498-506), read_to_string reads each one whole (payer.rs:513), every parsed Invoice is retained (payer.rs:511-517) and each allowed one is cloned into the payment queue (payer.rs:259) - with no size cap, no count cap, and no consideration of the grant or kill-switch state before the work.",
  "intended_behavior": "A payer given 'authority, not a key' should bound what an untrusted merchant channel can make it ingest: a per-invoice byte cap and an invoice-count cap (or a streaming/deferred parse under a total memory budget) enforced from file metadata before any read, and enforced for a revoked grant too, so the kill switch also caps intake work. An entry over a cap should become a named refusal line rather than a read.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/payer.rs",
      "line": 150,
      "scope": "payer::run",
      "description": "the operator-supplied --invoices directory (the merchant's invoice channel) becomes invoices_path"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 201,
      "scope": "payer::run",
      "description": "load_invoices(&invoices_path) is called after the grant/kill-switch state is computed and before any decision, so intake is unconditional on authority"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 498,
      "scope": "load_invoices",
      "description": "read_dir collects every entry whose extension is json into files - no count cap"
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 513,
      "scope": "load_invoices",
      "description": "std::fs::read_to_string reads each file whole; the merchant-controlled file size decides the allocation, with no cap"
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/payer.rs",
      "line": 515,
      "scope": "load_invoices",
      "description": "serde_json::from_str parses the whole text into an Invoice (another full copy of every String field), payer.rs:517 retains it, and payer.rs:259 clones each allowed invoice into to_pay before any wallet or node work"
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/payer.rs",
      "line": 495,
      "description": "load_invoices: extension filter only, files.sort(), read_to_string + from_str per file, Vec<Invoice> returned - no cap of any kind"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 201,
      "description": "load_invoices runs before the decide loop (payer.rs:249-270) and before any wallet/node work (payer.rs:276)"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 259,
      "description": "every allowed invoice is cloned into to_pay before payments begin"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 244,
      "description": "comment 'Decide everything first, then pay': the whole directory is held in memory before any payment can move money"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 119,
      "description": "decide() has no notion of invoice count or intake bytes; the budget check only accumulates amount_zat"
    }
  ],
  "conditions": [
    {
      "kind": "system_configuration",
      "description": "the operator points --invoices at a directory a merchant or any less-trusted party can write to - the channel the bounded payer is designed around ('the merchant side of an agent payment ... no channel other than the file')"
    },
    {
      "kind": "environmental_dependency",
      "description": "the payer process has a memory budget (systemd MemoryMax, container memory limit, small host) or the directory holds files whose combined size approaches it; the observed stop used ulimit -v 1200000 with a 768 MiB invoice"
    }
  ],
  "execution": {
    "attacker_perspective": "A merchant who can write files into the invoice directory the payer is pointed at; no wallet, key or network access is needed - one file is enough.",
    "payloads": [
      "inv-huge/huge.json (805306513 bytes): {\"invoice_id\":\"huge\",\"amount_zat\":1,\"receiver\":\"uregtest1noreceivercheck\",\"memo\":\"<768 MiB of the byte 'a'>\",\"uri\":\"zcash:test\",\"issued_at_height\":1,\"merchant_account\":0}",
      "grant-ok.json: {\"budget_zat\":50000000,\"per_payment_cap_zat\":10000000,\"allowlist\":[],\"expires_at\":\"2030-01-01T00:00:00Z\",\"revoked\":false}",
      "docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 -v <freeze copy>:/t -v ~/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim sh -c \"ulimit -v 1200000; exec /t/target/debug/zanary-probe payer --grant /t/w2-invoices-fx/grant-ok.json --invoices /t/w2-invoices-fx/inv-huge --out /t/w2-invoices-fx/out/chk2 --now 2026-10-17T00:00:00Z\""
    ],
    "instructions": [
      "In a disposable copy of the reviewed revision (no network), build the probe: cargo build --offline -p zanary-probe --bin zanary-probe.",
      "Write grant-ok.json next to a directory containing one invoice file of the stated shape and size (the memo padding is what sizes the file).",
      "Run the payer against that directory under a strict address-space limit (ulimit -v 1200000) so the effect stays bounded; repeat with 3 tiny invoices and with a 256 MiB invoice under the same limit as controls.",
      "Record exit code, stderr, and peak RSS (VmHWM sampling) only; no payment path is exercised, nothing is sent anywhere."
    ],
    "observed_result": "768 MiB invoice: exit=1 in <1 s, stderr 'Error: reading the invoice /t/w2-invoices-fx/inv-huge/huge.json' followed by 'Caused by: out of memory'; no decision line was printed. Control 3 tiny invoices under the same limit: 3 ALLOW lines and 'invoices : 3 in ...' (peak RSS 8 MB), then the expected node connection error. Control 256 MiB invoice under the same limit: intake completed (peak RSS 527 MB). Revoked grant with the 256 MiB invoice: intake still completed and printed 'REFUSE big - refused: the grant was revoked (kill switch)'."
  },
  "remediation": {
    "strategy": "Bound the intake at the boundary: cap the number of invoice files and each file's size from metadata before reading, turn over-limit entries into named refusal decisions instead of reads, and apply the caps before any parse work and regardless of authority state. If large invoices must be supported, stream or defer parsing under a total memory budget.",
    "code_changes": [
      {
        "file_name": "crates/probe/src/payer.rs",
        "fixed_code": "/// Invoices from a directory (`*.json`) or from a single file, sorted by name so a run is reproducible.\n/// Bounded: a merchant channel may not decide how much this process reads.\nconst MAX_INVOICE_FILES: usize = 1000;\nconst MAX_INVOICE_BYTES: u64 = 262_144; // far above any real invoice\n\nfn load_invoices(path: &Path) -> Result<Vec<Invoice>> {\n    let mut files: Vec<PathBuf> = Vec::new();\n    if path.is_dir() {\n        for entry in std::fs::read_dir(path)\n            .with_context(|| format!(\"reading the invoice directory {}\", path.display()))?\n        {\n            let entry = entry?;\n            let candidate = entry.path();\n            if candidate.extension().is_some_and(|e| e == \"json\") {\n                files.push(candidate);\n            }\n        }\n    } else {\n        files.push(path.to_path_buf());\n    }\n    files.sort();\n    if files.len() > MAX_INVOICE_FILES {\n        bail!(\n            \"{} invoice files in {} exceeds the cap of {MAX_INVOICE_FILES} - refusing to read a \\\n             merchant directory this large\",\n            files.len(),\n            path.display()\n        );\n    }\n    let mut invoices = Vec::new();\n    for file in &files {\n        let meta = std::fs::symlink_metadata(file)\n            .with_context(|| format!(\"statting the invoice {}\", file.display()))?;\n        if !meta.file_type().is_file() {\n            bail!(// ledger-able refusal instead of a read\n                \"{} is not a regular file - refusing to read it as an invoice\",\n                file.display()\n            );\n        }\n        if meta.len() > MAX_INVOICE_BYTES {\n            bail!(\n                \"invoice {} is {} bytes, above the {MAX_INVOICE_BYTES}-byte cap\",\n                file.display(),\n                meta.len()\n            );\n        }\n        let text = std::fs::read_to_string(file)\n            .with_context(|| format!(\"reading the invoice {}\", file.display()))?;\n        let invoice: Invoice = serde_json::from_str(&text)\n            .with_context(|| format!(\"parsing the invoice {}\", file.display()))?;\n        invoices.push(invoice);\n    }\n    Ok(invoices)\n}"
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "low",
      "reason": "Requires the operator to run the payer over a directory a hostile party can write to (the documented merchant-invoice channel); the repo's own recorded runs author the directory themselves, so the demo setup does not trigger it."
    },
    "impact": {
      "score": "low",
      "reason": "Availability of the operator's own payer process: intake aborts with 'out of memory' (exit 1) or the process is OOM-killed; no payment is made, no data is corrupted, recovery is deleting the file or raising the limit. No cross-tenant or network-visible effect."
    },
    "overall_severity": "low"
  },
  "confidence": {
    "score": "high",
    "reason": "The missing bound is fully source-visible and the effect was reproduced locally with the reviewed revision (allocation stop at intake, ~2x memory per file, revoked grant no protection); no preventing layer exists in the code path."
  }
}
```

## Its linked coverage unit and the hunter's checks

- coverage_id: `crates%2Fprobe%2Fsrc%2Fpayer.rs%23load-invoices::merchant-supplied%20invoice%20directory%20-%3E%20payer%20memory%20and%20payment%20work::bounded-payer::RESOURCE-EXHAUSTION-AND-AVAILABILITY.md%23Resource%20accumulation%20attack%20classes`
- surface: `crates/probe/src/payer.rs#load-invoices` (subsystem `bounded-payer`)
- boundary: `merchant-supplied invoice directory -> payer memory and payment work`
- attack class: `RESOURCE-EXHAUSTION-AND-AVAILABILITY.md#Resource accumulation attack classes` · assignment owner: `hunter-invoices`
- unit reviewed paths: ["crates/probe/src/ops.rs", "crates/probe/src/payer.rs"]

The unit's checks, including the local checks whose promoted artifacts you may read
(paths are relative to the run directory `/home/kiter/security-audit-skill/zanary/run-1`):

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

### [companion validation block] RESOURCE-EXHAUSTION-AND-AVAILABILITY.md#Validation rules — verbatim

## Validation rules (apply before reporting ANY finding here)

1. Name untrusted input, requester work, service amplification or retained resource, shared blast radius, and recovery. Missing limits without concrete shared impact are hardening.
2. Confirm no source-visible upstream, parser, queue, tenant, or framework bound prevents the path. Unknown deployed controls require `needs_validation`.
3. For superlinear behavior, establish the accepted complexity and bounded local growth. For leaks, show repeatable retention after cleanup should occur. For fatal paths, identify process/supervisor isolation.
4. Prioritize by low requester work, unauthenticated reachability, cross-tenant scope, persistence, and poor recovery; do not validate with availability impact.
5. Return `confirmed` only with safe local proof and meaningful shared effect. Return `needs_validation` with the exact upstream limit, topology, quota, or recovery observation an owner must check.

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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-7/scratch` (scratch-relative files only, `check-*.txt` / `check-*.json` /
`evidence.txt`, 64 KiB per file, 256 KiB total); the parent promotes into `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-7/artifacts`. You never write
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
