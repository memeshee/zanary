# An independent audit of this repository, and what came of it

The rig makes privacy claims falsifiable. This directory applies the same standard to the rest of
the repository: an audit of revision `7058406` (what `main` was on 2026-10-16), with every finding
reproduced before it was reported, and every confirmed finding fixed with a regression test.

Method: the *security-audit* skill's protocol — two hunter waves (13 seat-hunts), a coverage ledger
built from the skill's unit definitions, post-wave coverage critics, one fresh candidate verifier per
candidate, then a second, independent record verifier per retained finding. Nothing was reported on
reading alone: each record carries the probe output that reproduced it.

## Result

- **19 findings** — **11 confirmed**, **8 leads**, 0 rejected.
- **Where each one landed:** 7 sit inside a leak class this library names, 12 live in the tool around the
  flow, 1 is still open — [`../docs/WHAT-IT-CAUGHT.md`](../docs/WHAT-IT-CAUGHT.md) sorts all 19 by what
  could actually have caught them, with each fix's guarding test.
- **20 coverage units** (2 covered, 11 candidate, 7 deferred by budget), **113 recorded checks**.
- **56 of 60 agent invocations** spent; the units that budget could not fund are named with their reason in `run-1/REPORT.md` §7.
- Every retained finding was reproduced by a verifier that did not write it, and re-verified by a
  second record verifier that read neither. No verifier asked for a replacement.
- The audited revision was never modified: the worktree stayed clean and every target-controlled
  command ran inside a disposable copy with `--network=none` and a read-only cargo registry.
- The run was resumed once, after a gateway restart dropped a hunter wave's results; the takeover
  and the budget changes are recorded in `run-1/run-metadata.json`.

## Confirmed findings, and where each one stands

| finding | what it is | status |
| --- | --- | --- |
| `crates/report/src/main.rs:short-helper-byte-slice-on-untrusted-json-fields` | Report CLI truncates untrusted report/run JSON fields by byte index and panics (exit 101) instead of returning a verdict | fixed — one char-safe display helper (`head_chars`/`short_chars`/`short_tail`) serves every call site |
| `crates/report/src/receipt.rs:hex_decode+short:utf8-byte-slice-of-untrusted-receipt-fields` | verify-receipt aborts with a Rust panic (exit 101, no verdict) on a receipt whose hex fields or attestation key are not ASCII | fixed — hex validated before decoding; display char-safe |
| `crates/report/src/receipt.rs:validate_claim:claims-body-agreement-is-substring-containment` | verify-receipt certifies claim/body agreement for a receipt whose claims state different figures (substring containment) | fixed — agreement is token-boundary aware |
| `probe/payer.rs:load_invoices:nonregular-entry` | Bounded payer follows symlinks and blocks forever on non-regular *.json entries in the merchant-supplied invoice directory | fixed — symlinks and non-regular entries are refused before the open, so a FIFO cannot block the run |
| `probe/payer.rs:load_invoices:unbounded-intake-memory` | Bounded payer ingests a merchant-supplied invoice directory into memory with no size or count cap, before and independent of grant authority | fixed — intake bounded from file metadata before any read (count, size, regular-file check) |
| `zanary/composite-action-input-shell-expansion` | Composite action expands caller-supplied with: inputs into shell script text | fixed — every input and derived step output travels via `env:`, never into script text |
| `zanary/hex-decode-str-slice-panic` | verify-invoice panics on a txid containing a multi-byte UTF-8 character instead of rejecting it | fixed — `flow::hex_decode` decodes byte-wise |
| `zanary/payer-budget-excludes-fees` | The grant's budget bounds invoice principal, not the cost of paying: per-payment fees and the invoice count sit outside the authority | fixed — `PAYMENT_FEE_ZAT` counted in the budget rule and the property; grant carries a cardinality bound |
| `zanary/payer-duplicate-invoice-id-replay` | Invoice identity is a label, not a key: two invoice files carrying one invoice_id are both decided ALLOW and queued for payment in a single run, the run record puts the second payment's txid on the first row, and payer-policy passes the duplicated ledger | fixed — invoice ids deduped in decision order, txids attributed by position, ledger replay refused |
| `zanary/payer-expiry-text-compare` | Bounded payer enforces grant expiry as a text comparison, so an expiry instant written with a UTC offset (or an unparsable value) still pays | fixed — instants compared, unreadable expiry fails closed, `decided_at` recorded |
| `zanary/unchecked-zat-sum-overflow` | Flow-definition amounts are summed with unchecked u64 arithmetic: a two-invoice definition aborts the dev build and wraps silently in the release build | fixed — amounts validated where the definition loads, totals via `checked_add` |

## Leads: what only a fact from outside this repository can settle

`run-1/NEEDS-VALIDATION.md` carries each one with its exact blocker and an owner-observable plan.
Three of them were settled from the running rig after the fixes landed, and one remains open:

| finding | what it is | status |
| --- | --- | --- |
| `crates/probe/src/payer.rs:payment-record-pool-legs-and-policy-are-constants-not-derived-from-the-built-transaction` | The bounded payer writes every payment record from literals (Orchard -> Orchard, Shielded, FullPrivacy) and never applies the recipient-policy guard, so a payment whose transaction crosses a shielded pool boundary (a Sapling receiver) would be recorded and scored as a same-pool, fully-shielded, FullPrivacy payment | fixed on the payer's path, rig-verified — `pool_out`/`kind`/`policy` are derived from the receiver address, and a receiver whose value would land outside Orchard is refused with a ledger line naming the pool. The flow runner's records are deliberately unchanged: its own definitions are Orchard→Orchard and it does gate recipients through `check_policy` (`flow.rs:633`, `:826`); rewriting how it records would invalidate the hashes of the claims already published from those runs, so this is flagged rather than silently changed |
| `probe/payer.rs:run:unbounded-payment-count` | Bounded payer queues an unbounded number of payments: the grant bounds the sum of amounts, not the count of heavy per-invoice work | fixed — the grant's cardinality bound caps payments per run; intake is capped too (1024 files, 64 KiB each) |
| `video/zanary-demo.mp4:publish-video-unverifiable-media-content` | Publish-video step serves a media asset whose disclosed content no repository artifact constrains or verifies | open — the published asset's metadata is clean (no device/GPS/creation tags) and its frames come from a recorded exit-0 run, but no machine check reads the frames themselves; a human review is the remaining step |
| `zanary/payer-ledger-not-durable` | Payments are recorded only after all of them are broadcast: an aborted or repeated run leaves on-chain payments with no ledger and re-pays them | fixed, rig-verified — the ledger is appended and flushed per decision; a SIGKILL mid-run keeps the payments it made and the re-run refuses the replay |
| `zanary/published-summary-carries-invocation-paths` | Published claim files carry the gate's raw invocation-path arguments verbatim (${{ github.workspace }}-prefixed in the documented CI usage) | fixed — the gate's summary records workspace-relative paths (`publishable_path`) |
| `zanary/rig-image-tags-unpinned` | The rig that produces and re-verifies every published privacy claim runs from two mutable :latest image tags | fixed — `harness/docker-compose.yml` pins both images by digest (the digests the audit ran against) |
| `zanary/rig-ports-published-all-interfaces` | Rig control plane (zcashd JSON-RPC) and the credential-less lightwalletd indexer are published without a host-IP qualifier, so the documented setup inherits the Docker engine's every-interface default, gated only by RPC credentials the public repository itself carries | fixed — both services publish on `127.0.0.1` only; remote use forwards the port |
| `zanary/unbounded-jitter-block-mining` | flow definition jitter_blocks has no upper bound: one accepted line names a mining request of up to 4,294,967,295 blocks, drawn per payment and sent as a single uncancellable `generate` RPC | fixed — `MAX_JITTER_BLOCKS` bounds a definition's settlement jitter |

## Verified against the running rig, after the fixes

The audit could not touch the rig (out of bounds for a read-only run), so its rig-dependent leads
were recorded as `needs_validation`. They were re-run against the live regtest harness on the fixed
code:

1. **The ledger survives an abort.** The payer was SIGKILLed the moment its first payment reached the
   ledger. The payment's line survived (append + flush per decision, not a summary written at the
   end); the run's own summary did not. A re-run into the same directory refused that invoice —
   *"invoice 9201 was already paid into this run directory (the ledger is the payer's memory; a replay
   is refused, not paid again)"* — and paid only the remaining invoices: no invoice paid twice, one
   txid per payment, confirmed independently by the merchant's own UFVK trial-decrypting the chain.
2. **The payment record's pool legs are derived, not asserted.** An invoice naming a transparent
   receiver, allowlisted in the grant so no other rule could refuse it, is refused by the shape
   pre-flight with a ledger line naming the pool it would have crossed; an Orchard invoice under the
   same grant is paid. `pool_out`, `kind` and `policy` in the record come from the address.
3. **The rig's own images and ports.** `docker ps` showed both services published on every interface
   and `docker image inspect` showed both images resolved from `:latest`; `harness/docker-compose.yml`
   now pins both digests and binds both services to `127.0.0.1`.

## Scope: what this audit did not do

- It is a hunter-based audit: broad, but not exhaustive. The coverage ledger names what was planned,
  hunted and deferred — a deferred row is a statement, not an omission.
- No fuzzing beyond the in-repo suite; no review of the upstream Zcash crates; no mainnet path
  (there is none).
- It reviewed `7058406`. The fixes it produced are themselves unreviewed by a second party — the
  regression tests and CI are the evidence for those.

## How to read this directory

| path | what it is |
| --- | --- |
| `run-1/REPORT.md` | the audit's own report: method, coverage, findings, deferred units, budget |
| `run-1/FINDINGS-DETAIL.md` | each finding with its trace, reproduction and suggested remedy |
| `run-1/NEEDS-VALIDATION.md` | the leads, each with its blocker and an owner-observable plan |
| `run-1/findings.json` | the machine-readable records the report is generated from |
| `run-1/coverage-ledger.json` | the 20 units, their checks, and the fingerprints they produced |
| `run-1/agents/`, `run-1/verifiers/` | each seat's prompt, record and evidence artifacts |
