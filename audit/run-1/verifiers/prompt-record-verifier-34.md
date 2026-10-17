# Phase-5 record verifier — verifier-34 on `zanary/payer-ledger-not-durable`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`needs_validation`)

```json
{
  "verdict": "needs_validation",
  "fingerprint": "zanary/payer-ledger-not-durable",
  "title": "Payments are recorded only after all of them are broadcast: an aborted or repeated run leaves on-chain payments with no ledger and re-pays them",
  "description": "The payer decides every invoice first and keeps every decision in memory (payer.rs:244-270), then pays the allowed ones in a loop that propagates every step with `?` (291-363): Address::decode (298), MemoBytes::from_bytes (304), propose (306-307), build+broadcast (308), mine_and_resync (309, 11 blocks, ops::CONFIRM_BLOCKS). ledger.jsonl is the only ledger persistence and, with run.json/findings.json/report.json, is written once, after the loop (409; 411 -> record.rs:45/50/61) with unconditional truncating std::fs::write. A run that dies inside the loop - decide admits an invoice the loop cannot pay (a receiver that does not decode under an allowlist containing that literal, or a memo over the ZIP-302 cap; both re-verified ALLOW offline and the payer's own intake accepts the JSON), an indexer failure, a timeout, or SIGINT - exits with no artifact at all, while any earlier iteration's payment is already broadcast and mined. Nothing outside those artifacts remembers a paid invoice (PayerRun is label+grant+decisions, observation/src/lib.rs:222-230; spent_zat derives from the in-memory decisions, 234-240; no file under crates/ reads a ledger), so re-running the same command re-decides from spent=0 and broadcasts the same payments again; and a completed re-run into the same --out (default data/runs/payer, payer.rs:151-153) replaces the earlier ledger/run/report wholesale. The grant budget is a per-run bound, not a lifetime one, and no source-visible control dedupes invoices or bounds re-execution (companion RESOURCE-EXHAUSTION rules: input = caller-supplied invoice JSON directory; amplification = up to budget_zat re-paid per completed re-run plus the payments of every aborted run; retained state = none, which is the defect itself; recovery = manual reconciliation against the chain; the failure is process-fatal with no supervisor). Calibration: the flow runner commits through the same end-of-run record::write_run_artifacts (flow.rs:502), but the re-pay consequence is the payer's. Same source-derived root cause as sibling candidate `crates/probe/src/payer.rs:payment-loop-failure-aborts-before-ledger-and-run-artifacts-are-written` (hunter-broadcast, wave 2); the two records should be deduped/merged. This record consolidates two fingerprints that independent verification found to share one root cause: this one and `crates/probe/src/payer.rs:payment-loop-failure-aborts-before-ledger-and-run-artifacts-are-written` (filed in wave 2 by hunter-broadcast and independently verified by verifier-1). Its facets are folded in here rather than counted twice: the decide-everything-then-pay ordering, the `?` abort after earlier invoices were already broadcast, the missing `error:` refusal reason the payer-policy property looks for, and the re-pay on re-run.",
  "claimed_root_cause": "The spend record is an end-of-run artifact rather than a durable append-only journal written before each broadcast: decisions and refusal lines are buffered in memory (payer.rs:291-379), every payment step returns early through `?` with the chain effect already committed (298-310), the only persistence is after the loop (409, 411) via unconditional truncating std::fs::write, nothing persists a paid-invoice set for the next run to consult (PayerRun = label + grant + decisions), and a reused --out erases the previous evidence. This is the same source-derived root cause as the sibling candidate `crates/probe/src/payer.rs:payment-loop-failure-aborts-before-ledger-and-run-artifacts-are-written` (hunter-broadcast, wave 2): both are the end-of-loop commit plus `?` abort ordering at payer.rs:291-411; the no-persisted-paid-state and overwrite facets here are consequences of the same single end-of-run write design, not a genuinely different defect - dedupe/merge the two records. The same ordering defect was filed independently as `crates/probe/src/payer.rs:payment-loop-failure-aborts-before-ledger-and-run-artifacts-are-written`: one end-of-loop commit plus `?` abort at payer.rs:291-411, reported once here.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/payer.rs",
      "line": 201,
      "scope": "invoice intake (load_invoices) and the decide phase",
      "description": "the CLI loads caller-supplied invoice JSON (201) and decides every invoice before any money moves (244-270), fixing the payable set and `spent` in memory; decide validates only the grant fields - never the receiver's address form or the memo - so an allowlisted non-address receiver and a 600-byte memo are decided ALLOW (re-verified offline by this audit's own checker, A1-A3/A5/B1-B2)."
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 291,
      "scope": "payment loop",
      "description": "each to_pay entry is decoded (298), memo-encoded (304), proposed (306-307), built and broadcast (308) and mined (309); every step returns through `?` with the ledger still only in the in-memory Vec."
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 364,
      "scope": "refusals appended (in memory, after the loop)",
      "description": "refusal lines are buffered in the same in-memory ledger only after the payment loop (364-379), so a run that dies mid-loop has no ledger lines of any kind."
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 409,
      "scope": "artifact write (only persistence)",
      "description": "ledger.jsonl (409) and run.json/findings.json/report.json (411 -> record.rs:45/50/61) are written once, after the loop, with unconditional std::fs::write; nothing is written on any earlier exit and a re-run into the same --out replaces the previous evidence."
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 249,
      "scope": "payer::run, decide loop over the invoice directory (246-270)",
      "description": "decide checks revoked, expiry, allowlist (string compare), per-payment cap and budget only - nothing validates the receiver's address form or the memo length, and the intake is plain serde (511-518) - so an invoice whose receiver is not an address at all is decided allowed=true ('paid: within the cap (10000000 zat) and the budget (5000000 of 50000000 zat spent after this)'; re-observed by the verifier as a unit call and through the real CLI)."
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 298,
      "scope": "payer::run, payment loop",
      "description": "Address::decode returns None for that receiver (re-observed: decode('not-an-address') -> None) and the loop returns Err; every earlier iteration of this loop has already broadcast and mined its payment at 308-310."
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/payer.rs",
      "line": 308,
      "scope": "payer::run, inside the loop",
      "description": "ops::build_sign_broadcast sends the transaction to the indexer (ops.rs:407-423) while the run's only evidence write happens after the loop (406-411), so the chain state and the artifact state diverge on failure."
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/payer.rs",
      "line": 408,
      "scope": "payer::run, after the loop",
      "description": "ledger.jsonl (409) and record::write_run_artifacts (411) are reached only when every iteration succeeded; a failed run leaves --out uncreated (observed: no ledger.jsonl/run.json/report.json after a post-decision failure), so the invoices that were paid get no decision record and the run no report."
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/payer.rs",
      "line": 244,
      "description": "the in-code claim 'the ledger a reviewer reads is complete before any money moves' holds only in memory - the file is written at 409."
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 298,
      "description": "Address::decode returns None for an admitted receiver that is not a decodable address (re-verified, B1); the loop's first fallible step, reached after earlier iterations may already have broadcast."
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 304,
      "description": "MemoBytes::from_bytes rejects a memo over the ZIP-302 cap (512 accepted / 513 and 600 rejected, re-verified B2) at the same point."
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 409,
      "description": "std::fs::write(&ledger_path, ...) is the only ledger persistence and the only std::fs::write in payer.rs (re-verified C27), and it truncates."
    },
    {
      "file": "crates/probe/src/record.rs",
      "line": 45,
      "description": "write_run_artifacts writes run.json (45), findings.json (50) and report.json (61) with unconditional std::fs::write (3 truncating writes, no OpenOptions/append, re-verified C31-C32), so a later run at the same --out erases the earlier one."
    },
    {
      "file": "crates/observation/src/lib.rs",
      "line": 234,
      "description": "PayerRun::spent_zat derives from the in-memory decisions (234-240); PayerRun carries only label + grant + decisions (222-230) - no persisted paid-invoice set exists (re-verified D1-D2)."
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 153,
      "description": "the default --out is data/runs/payer (151-153), so a plain re-run shares the output path with the previous run."
    },
    {
      "file": "crates/properties/src/properties/payer_policy.rs",
      "line": 68,
      "description": "the property flags a refusal whose reason starts with 'error:'; payer.rs contains no 'error:' string (grep: none), and an aborted run writes no report for the property to read"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 247,
      "description": "'spent' starts at 0 on every invocation and no code path reads a previous ledger.jsonl (only demo summary scripts display it), so a re-run cannot know what an aborted run already paid"
    },
    {
      "file": "docs/BOUNDED-PAYER.md",
      "line": 50,
      "description": "the documented contract this breaks: 'Every invoice produces exactly one ledger line - paid or refused - with the rule in it', with the budget defined per run and checked in ledger order"
    }
  ],
  "blockers": [
    "The decisive end-to-end observations need the running regtest rig, which is out of bounds: run() reaches the payment loop only through core::bootstrap_with (a real funded, synced wallet database) and NodeRpc::from_env (ZANARY_LWD_URL -> host port 9067), and every payment mines 11 blocks (ops::CONFIRM_BLOCKS = 11, ops.rs:59; payer.rs:309). In a --network=none sandbox with no access to the zanary-zcashd/zanary-lwd containers or ports 18232/18233/9067/9068 it cannot be observed that (a) a mid-loop abort leaves the earlier payment mined while --out holds no ledger.jsonl/run.json/report.json, and (b) a re-run broadcasts a second payment for the same invoice under the same grant.",
    "No offline harness reaches run(): the loop's first iteration is a wallet-backed payment; only its preconditions are observable offline (this record's local plan and the hunter's check-payer-decide.txt H8 line agree on them).",
    "Duplicate root cause (see claimed_root_cause): the same source-derived defect as `crates/probe/src/payer.rs:payment-loop-failure-aborts-before-ledger-and-run-artifacts-are-written` (hunter-broadcast, wave 2, itself proposed needs_validation) - dedupe/merge; do not count as two independent findings.",
    "The partial run itself needs the regtest rig: the payment loop's first iteration is a wallet-backed payment (funded, synced wallet + node/indexer). Offline the loop cannot pass its first wallet-backed step - re-observed in the sandbox: ops::propose_shielded_payment on an unsynced wallet fails with 'could not propose the payment: Must scan blocks first' - and no offline path to a funded wallet exists (no chain, and the read-only registry holds none of the crates a zcash_client_backend test-wallet setup would need; checked: 0 matches for proptest/ambassador/assert_matches). A run where iteration 1 broadcasts and mines and iteration k>1 aborts is therefore unreachable with --network=none.",
    "The two consequences that make the finding remain owner-observed facts: (a) after such a partial run, --out holds no ledger.jsonl/run.json/report.json while the iteration-1 txid is mined on chain (the sandbox re-observed only the weaker, broadcast-free half: a post-decision failure aborts with exit 1 and --out is never created); (b) re-running the identical command re-pays the already-paid invoice under a new txid. Both need the regtest rig."
  ],
  "validation_plan": {
    "local": "Re-established in the disposable sandbox copy during this verification (this verifier's own checker, 51/51 checks, no network): decide() with an allowlist containing the literal non-address receiver and with an empty allowlist decides it ALLOW (A1/A2); a 600-byte memo is decided ALLOW (A3); Address::decode returns None for that receiver (B1); MemoBytes::from_bytes accepts 512 bytes and rejects 513/600 (B2); the source ordering is exact - payer.rs:201/244/258/291/298/304/307/308/309/364/406/409/411, record.rs:45/50/61, observation/src/lib.rs:222-240 (C01-C36) - with the only std::fs::write in payer.rs at 409 after the loop, no File::create/OpenOptions/append, PayerRun serializing to exactly {decisions,grant,label} and no ledger reader anywhere under crates/ (D1-D7). Re-run: `cargo run --offline -p zanary-probe --bin verifier15_check` inside the sandbox copy (checker source: crates/probe/src/bin/verifier15_check.rs there; promoted output: agents/verifier-15/artifacts/check-verifier15.txt, scratch copy agents/verifier-15/scratch/check-verifier15.txt). The hunter's proposed filter (`cargo test -p zanary-probe -- hunter_payer`) no longer applies - that test module is not present in the sandbox copy; use the checker above. Re-runnable offline in the disposable sandbox copy (each step re-executed by the verifier): (1) call payer::decide with the documented grant shape (empty allowlist, cap 10,000,000 zat, budget 50,000,000 zat, unexpired, not revoked), spent = 0 and an invoice {receiver 'not-an-address', amount 5,000,000} -> allowed=true; Address::decode('not-an-address') -> None; a 600-byte memo -> decide allowed, MemoBytes::from_bytes Err, 512 bytes Ok. (2) Run the real binary (cargo build --offline -p zanary-probe --bin zanary-probe) on a grant file plus an invoices dir holding that invoice: it prints ALLOW for it, then aborts non-zero (in the sandbox: 'connecting to the zcashd RPC at 127.0.0.1:18232 ... Connection refused', EXIT=1) and --out is never created - no ledger.jsonl, run.json or report.json. (3) ops::propose_shielded_payment on an unsynced wallet -> Err('Must scan blocks first'), the reason the loop cannot run offline.",
    "deployment": "Owner-observed on the owner's own throwaway regtest rig, non-destructively (fresh ZANARY_PAYER_DEMO_DIR throughout; never the repo's default data/payer-demo; nothing deleted or edited in place): (1) `ZANARY_PAYER_DEMO_DIR=<fresh>/payer-demo scripts/bounded-payer-demo.sh` to build the layout, then add a fifth invoice $DIR/invoices/8305.json (create with `zanary-probe invoice --id 8305 --amount-zec 0.02` and set its receiver field to the literal not-an-address; the payer ignores uri) and a grant file copied from grants/allowed.json whose allowlist additionally contains that same literal. (2) `./target/release/zanary-probe payer --grant $DIR/grants/garbage.json --invoices $DIR/invoices --out $DIR/runs/abort --ttl-days 30`: expect exit non-zero at 8305 with 'the invoice's receiver is not a valid address on this chain', $DIR/runs/abort containing no ledger.jsonl/run.json/report.json (the directory itself may not exist), and the 'paid : <txid>' line printed for 8301 resolving on chain (zcash-cli -regtest getrawtransaction <txid>). (3) Re-run the identical command unchanged: 8301 receives a second, different txid and again nothing is written - the double payment under one grant file is the decisive fact. (4) Remove 8305.json and re-run once more into the same --out: the completed run's ledger.jsonl holds exactly one paid line for 8301 (the second txid) and payer-policy passes, because the only ledger that exists is that run's own. (5) Overwrite facet, independent of any abort: run the two-invoice demo twice into the same --out and observe the second run's ledger.jsonl/run.json/report.json replace the first at the same paths (new report_hash) while the first run's payments stay on chain. Owner-observed on the regtest rig, non-destructive (fresh --out dir, regtest dev funds only, no other data touched, no --expect-refused needed): (1) with the rig up and the dev wallet funded/synced, write grant.json {budget_zat 50,000,000, per_payment_cap_zat 10,000,000, allowlist [], expires_at in the future, revoked false}; (2) make an invoices dir with 01-payable.json first in name order (a real `zanary-probe invoice` file, amount inside cap/budget) and 02-bad.json (same shape, receiver 'not-an-address', amount 5,000,000); (3) run `zanary-probe payer --grant <grant.json> --invoices <dir> --out <scratch>/partial-check`; expect both invoices ALLOW, 'paid : <txidA>' for invoice 01, then 'Error: the invoice's receiver is not a valid address on this chain: not-an-address' with a non-zero exit; expect <scratch>/partial-check to contain no ledger.jsonl / run.json / report.json / findings.json (it should not exist at all), while <txidA> is mined - confirm with `getrawtransaction <txidA> true` on the node or `zanary-probe verify-invoice --invoice 01-payable.json --txid <txidA>`; (4) re-run the identical command and expect a second payment for invoice 01 under a new txid <txidB> under the same grant file, then the same abort - one grant file, two payments, no ledger from either run; (5) optionally repair or remove 02-bad.json, re-run to completion into a fresh --out, and observe the surviving ledger covering only that run (within budget, no line for <txidA> or <txidB>) - the duplicate is visible only on chain. Stop after observing; the scratch run dir may be deleted."
  }
}
```

## Linked ledger unit

- coverage_id: `crates%2Fprobe%2Fsrc%2Fpayer.rs%23decide::crates%2Fproperties%2Fsrc%2Fproperties%2Fpayer_policy.rs%23evaluate::bounded-payer::ATTACK-CLASSES.md%23Business%20logic`
- surface: `crates/probe/src/payer.rs#decide` · subsystem `bounded-payer`
- boundary: `crates/properties/src/properties/payer_policy.rs#evaluate`
- assignment owner: `hunter-payer`

The unit's checks (each `artifact` path is relative to the run directory; read the promoted files
under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/`):

```json
[
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/observation/src/lib.rs"
    ],
    "invariant": "decide() is the single decision point for an invoice and applies the five grant rules in the documented order (revoked -> expired -> allowlist -> cap -> budget) over the grant actually recorded for the run",
    "method": "source",
    "result": "verified for the reference payer: decide is called once per invoice before any payment (payer.rs:246-270), the grant recorded in the run is the same object decide used (payer.rs:193-199, 388), and every refusal/allowance carries the first rule that fired. The rule bodies themselves are where the authority model leaks: the expiry rule is a text comparison (payer.rs:91), the allowlist rule never validates the receiver (100), the budget rule counts invoice principal only (119-128).",
    "artifact": null
  },
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/properties/src/properties/payer_policy.rs",
      "crates/observation/src/lib.rs",
      "crates/probe/src/record.rs"
    ],
    "invariant": "the ledger a payer records carries everything the payer-policy boundary needs to re-derive every grant rule, and the boundary re-derives the same verdict as the payer's own decide()",
    "method": "source",
    "result": "not re-derivable: PayerDecision has no decision time (observation:200-214), so the expiry rule cannot be checked at all (payer_policy only echoes expires_at into its summary, 105-116); the property never cross-checks the ledger against run.payments; and its internal-error rule keys on the literal reason prefix \"error:\" (68) which the reference payer never writes (it aborts, payer.rs:298-309).",
    "artifact": null
  },
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/probe/src/flow.rs"
    ],
    "invariant": "the payer's own gate holds: an expired grant cannot pay (as an instant), a repeated invoice_id is decided once, the budget bounds what the run spends, a misspelled grant key cannot widen authority, and an invoice the payment loop cannot pay is not decided payable",
    "method": "local",
    "result": "all five fail against the reviewed decide() (7/7 tests passed, i.e. the failures are the observed behaviour): H1a same expiry instant two encodings -> refused (canonical Z) vs ALLOWED (offset +14:00); H1b fractional clock against a whole-second expiry -> ALLOWED; H1c \"never\"/\"2030-99-99T00:00:00Z\" -> ALLOWED; H2 invoice dup-8301 decided ALLOW twice (spent 300 then 600 of 1000); H3 1-zat invoices -> 10,000,000 zat of fees for a 1,000-zat budget; H4 typo'd allow_list/expiry -> allowlist [] (any receiver), expires_at None; H8 allowlisted non-address receiver and 600-byte memo -> ALLOWED",
    "artifact": "agents/hunter-payer/artifacts/check-payer-decide.txt"
  },
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/properties/src/properties/payer_policy.rs",
      "crates/observation/src/lib.rs",
      "crates/probe/src/flow.rs"
    ],
    "invariant": "the payer-policy boundary re-derives the grant verdict from the run record for invoice identity, for whether a paid row is backed by a payment, and for the grant's expiry",
    "method": "local",
    "result": "passes in all three blind spots (3/3 tests passed): H5 two paid rows sharing one invoice_id (the second with no txid) -> PASS; H6 a paid row with zero run.payments entries -> PASS; H7 a ledger whose recorded grant expired in 2020 -> PASS, with the expiry echoed in the summary text only",
    "artifact": "agents/hunter-payer/artifacts/check-payer-policy.txt"
  },
  {
    "agent_id": "hunter-payer",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/probe/src/ops.rs"
    ],
    "invariant": "what the payer spends and does per allowed invoice is what the grant's budget accounts for (the cost of paying at the shared wallet layer is visible to the authority model)",
    "method": "source",
    "result": "the budget comparison never sees the payment layer's cost: every allowed invoice is one shielded transaction plus mine_and_resync(11 blocks) (payer.rs:291-310; ops.rs:59) and the payer's own funding model budgets 10,000 zat of ZIP-317 fee per payment (payer.rs:530), none of which the budget rule (119-128) or payer_policy (77-90) counts \u2014 the fee the payer knows it will pay is invisible to the authority it enforces.",
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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-34/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-34/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "zanary/payer-ledger-not-durable"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
