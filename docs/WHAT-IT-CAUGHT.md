# What the rig caught, and what only reading could

This repository makes privacy claims falsifiable. In October it applied the same standard to itself:
an independent, read-only audit of revision `7058406` (two hunter waves, one verifier per candidate,
a second record verifier per retained finding) returned **19 findings — 11 confirmed, 8 leads**.

Nineteen findings invites the wrong question (*"is this a list of defects?"*). The useful question is
the one this page answers: **which of them could a leak class in this library have caught in a flow
run?** — measured, not asserted, because the answer is the boundary of what a flow prober can prove.

Of the 19: **7 sit inside a class the library names**, and **12 do not**. The 12 are not a gap in the
coverage ledger. They are the shape of the product: a checker that watches a payment flow is blind, by
construction, to the tool around the flow.

## The numbers, and how to re-derive them

| what | number | re-derive with |
| --- | --- | --- |
| findings returned by the audit | 19 — 11 confirmed, 8 leads, 0 rejected | `python3 -c "import json,collections;d=json.load(open('audit/run-1/findings.json'));print(len(d),collections.Counter(x['verdict'] for x in d))"` |
| findings inside a class this library names | 7 | the table in §2 |
| findings outside it | 12 | §4 |
| `#[test]` count at `7058406` → now | 67 → 89 | `git grep -c '#\[test\]' 7058406 -- '*.rs'` — and without the revision for the current tree |
| of which added since the audit | 22 — 20 came with the fixes (named after the finding each one guards), 2 later, on the claim index | `git diff 7058406..HEAD -- '*.rs' \| grep -cE '^\+\s*#\[test\]'` |
| passes on `main` today | `cargo test --workspace` — 89 tests, 0 failures | the same command |

## 1. Inside the library's sight (7)

Counted as *inside* only where a class named in [`docs/PROPERTIES.md`](PROPERTIES.md) could have flagged
the finding in a flow run — in most of these, the finding names the class itself. The line is a
judgment call, so the rule is stated: relabel a row and the count changes, which is why every row also
carries the fix and its test.

| finding | class | what the check could not see | what changed | the guard |
| --- | --- | --- | --- | --- |
| `zanary/payer-budget-excludes-fees` | `payer-policy` | the *rule* and the *check* shared one assumption — "budget" meant invoice principal — so a ledger that under-counted what paying costs was internally consistent | the per-payment fee is part of the budget rule **and** of the property; the grant carries a cardinality bound | `the_budget_counts_the_fee_of_paying`, `the_budget_counts_the_fee_the_payer_also_spends` |
| `zanary/payer-duplicate-invoice-id-replay` | `payer-policy` | invoice identity was a label, not a key: two files carrying one `invoice_id` were both allowed, and the duplicated ledger passed the property | ids deduped in decision order, txids attributed by position, ledger replay refused | `a_repeated_invoice_id_is_decided_once_and_paid_once`, `an_invoice_the_ledger_already_paid_is_refused` |
| `zanary/payer-expiry-text-compare` | `payer-policy` | the property compared the same text the rule did: an expiry written with a UTC offset still paid | instants compared, an unreadable expiry fails closed, `decided_at` recorded | `the_expiry_is_an_instant_not_a_string`, `a_fractional_clock_does_not_beat_a_whole_second_expiry`, `an_unreadable_expiry_is_a_void_authority_not_an_unlimited_one` |
| `probe/payer.rs:run:unbounded-payment-count` | `payer-policy` | the grant bounded the sum of amounts, not how many payments the authority covers | the cardinality bound is in the grant and in the property; intake is capped too (1024 files, 64 KiB each) | `invoice_intake_is_type_checked_and_bounded` |
| `zanary/payer-ledger-not-durable` | `payer-policy` | a run killed mid-flight left on-chain payments with no ledger, and a re-run would pay them again | the ledger is appended and flushed per decision; a replay is refused, not paid again | rig-verified under SIGKILL (`audit/README.md` §*Verified against the running rig*) |
| `payment-record-pool-legs-and-policy-are-constants-not-derived` | `payer-policy`, `policy-downgrade`, `unshielding` | the record the properties read said `Orchard → Orchard · Shielded · FullPrivacy` for every payment, because those fields were literals | `pool_out`, `kind` and `policy` are derived from the receiver address; a receiver whose value would land outside Orchard is refused before any money moves, with a ledger line naming the pool | rig-verified: a transparent receiver allowlisted in the grant is refused, the Orchard invoice under the same grant is paid |
| `zanary/published-summary-carries-invocation-paths` | `report-leak` | the published claim files carried the gate's absolute invocation paths — the class whose job is "an exported report reveals more than the claim it supports", applied to us | the gate's summary records workspace-relative paths | `published_paths_are_relative_to_the_working_directory` |

**The pattern worth naming:** three of those rows (`budget-excludes-fees`, `expiry-text-compare`,
`record-pool-legs`) are one failure mode wearing three hats — *the rule and the check encoded the same
belief*, so a run that should have failed passed, with evidence attached. No amount of additional
property coverage finds that: the property is only as right as its definition of the leak. What finds
it is a second implementation, fixtures taken from real failures, and reading.

## 2. The finding that says the observation can lie

`payment-record-pool-legs-and-policy-are-constants-not-derived` is the sharpest thing the audit found,
because it attacks the thing this repository sells: *evidence*. The payer wrote its payment record from
literals — every payment was recorded as a same-pool, fully shielded, `FullPrivacy` payment **because
those strings were typed in the code**. `policy-downgrade` and `unshielding` read that record. So a
payment to a Sapling receiver would have been recorded as Orchard→Orchard and scored as clean.

The property was right about the record. The record was wrong.

Two things followed, and the second is the one this project cares about:

1. The fix derives the pool legs from the receiver address and refuses, before any money moves, a
   receiver whose value would land outside Orchard — naming the pool it would have crossed in the
   ledger line. Verified against the running rig, not argued.
2. The **flow runner's** records were deliberately left exactly as they were. Its own definitions are
   Orchard→Orchard and it gates every recipient through `ops::check_policy`, so the finding does not
   apply there — but rewriting how it records would change the bytes of reports whose hashes are in
   claims already published. It is flagged in `audit/README.md` rather than quietly changed. A tidier
   record is worth less than a re-derivable one.

## 3. Outside the library's sight (12)

Grouped by the kind of check that actually catches them. None of these needs a leak class; all of them
needed someone reading the code as a tool rather than as a flow.

| group | n | findings | what catches it |
| --- | --- | --- | --- |
| a tool reading a file someone else wrote | 4 | report CLI truncating untrusted JSON by byte index (panic, no verdict) · verify-receipt aborting on non-ASCII hex · verify-invoice panicking on a multi-byte txid · flow-definition amounts summed with unchecked `u64` arithmetic | hostile-input handling, per call site | `shortening_never_splits_a_character`, `non_ascii_key_fields_are_refused_not_fatal`, byte-wise `hex_decode`, `unpayable_amounts_are_refused_not_wrapped` |
| a verifier certifying what it did not check | 1 | verify-receipt certifying claim/body agreement by substring containment — a receipt whose claims state *different figures* passes | the verifier's own test suite | `claims_may_not_merely_embed_the_bodys_figures` |
| resources a stranger controls | 3 | a FIFO or symlink in the merchant-supplied invoice directory blocking the run · unbounded intake memory before any authority check · a definition requesting up to 2³²−1 mined blocks per payment | bounds taken from metadata, before the work | `invoice_intake_is_type_checked_and_bounded`, `MAX_JITTER_BLOCKS`, `an_unbounded_jitter_range_is_ignored` |
| the plumbing around the rig | 3 | composite-action inputs expanded into shell script text · the rig that produces every published claim running from mutable `:latest` images · the control plane and indexer published on every interface, gated by credentials the public repository carries | reading the plumbing | inputs travel via `env:`; both images pinned by digest; both services bound to `127.0.0.1` |
| still open | 1 | the published video's frames are produced by a recorded run but nothing machine-checks what the pixels say | a human, for now | `audit/run-1/NEEDS-VALIDATION.md` |

## 4. What this says about the product

- **What the rig answers:** did this flow leak; can a stranger re-derive that answer from the chain;
  is that answer still true today. **What it does not answer:** whether the tool is honest with hostile
  input, whether a merchant-supplied directory can stall the run, whether the plumbing around the rig is
  injectable, or whether a published asset says what it claims to say.
- **A checker that shares a definition with the thing it checks cannot find a mistake in the
  definition.** The three rows in §1 and the record finding in §2 are exactly that mistake, which is an
  argument for audits — including of your own repository — rather than against properties.
- **Every row above is re-checkable.** `audit/run-1/findings.json` carries all 19 records with their
  evidence; `audit/run-1/coverage-ledger.json` names the 20 coverage units, and the 7 the budget did
  not fund, with reasons; the fixes are in `1eb0675` and `c4f78e5`; the tests are named after the
  finding they guard. Nothing here asks to be taken on trust, including the count of findings that a
  property could not have caught.
