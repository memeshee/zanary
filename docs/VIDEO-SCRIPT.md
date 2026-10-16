# The demo video — a ≤2-minute cut

The submission form asks for a **≤2-minute video URL**, and the plan's note is that *profiler-vs-control is
the closing shot*. So this is a 120-second cut of one real run, not a tour — and the run is longer than
the video: **6m 04s** of execution (140s private flow · 145s control flow · 77s payer · instant index).

The edit compresses the waiting and **says so on screen** (`6 min of run → 2 min`) instead of pretending
the rig is that fast. Nothing is staged: every line below is verbatim from
`~/hackathon/zanary-rehearsal.log`, a real take that exited 0.

## Narration, word-counted to the second

~2.2 words/second with room to breathe — **275 words**, landing at about 1:58. The numbers are the only
part that must not be rushed.

### 0:00–0:15 · the gap (38 words)

> "Zcash already has tills — gateways that take shielded payments for a merchant. What nobody hands you
> is the *payer's* side: an agent with a budget instead of a key, repeat payments that don't cluster, and
> receipts that verify without trusting the tool that made them. This is ZANARY."

**On screen:** title card only — `ZANARY — privacy claims that expire`, `github.com/memeshee/zanary`,
`ZECATHON · Core & Tooling`. No architecture, no repository tour.

### 0:15–0:50 · the run, and the claim (80 words)

> "One command runs the whole story on a throwaway chain: `harness/up.sh`, then `scripts/demo-run.sh`.
> Two invoices, a fresh diversified receiver for each, one ZIP-302 memo per payment, full privacy. While
> it runs, the rig watches the chain — not the app's logs. All thirteen leak properties hold. Then the
> evidence — txids, receivers, memos, pool totals — is re-derived from the chain by a second
> implementation, and its root goes into a shielded memo, so the claim carries a timestamp nobody has to
> be trusted for."

**On screen, cut on the phrases:** the command being typed, then ~10 seconds real-time of the run (the
`6 min of run → 2 min` label appears here and stays up) → payment lines with txids and heights → the
property table → `verification summary PASS 12 pass · 0 fail · 6 skip` → the claim block:

```
  --- claim ---
  flow              : zanary-ci-checkout @ 4d8279bb
  verified at       : height … · 2026-10-16T…Z
  properties        : 12 pass · 0 fail · 1 skip
  report hash       : 17742795dc9c — recomputed over the report's own bytes
  anchor            : the evidence root in a shielded memo to the public beacon
  expires           : … — re-probe to re-issue the claim, or it goes stale then dead
```

### 0:50–1:15 · the subject: a budget instead of a key (55 words)

> "The subject that has to pass is a bounded payer — a budget, a per-payment cap, a receiver allowlist,
> an expiry, a kill switch. Four invoices: one fits the grant, one would pass the budget, one is over the
> cap, one is a stranger. One paid, three refused, each naming the rule that refused it. Throw the switch
> and nothing is paid at all."

**On screen:**
```
  ALLOW    8301 — paid: within the cap (10000000 zat) and the budget (9876540 of 12000000 zat spent)
  REFUSE   8302 — refused: paying 4321090 zat would take this run to 14197630 of a 12000000 zat budget
  REFUSE   8303 — refused: invoice 8303 asks 50000000 zat, above the per-payment cap of 10000000
  REFUSE   8304 — refused: uregtest13dam4gdg… is not on the allowlist (3 entries)
  --- ledger ---
  1 paid (9876540 zat spent) · 3 refused [allowlist, budget, cap]
```
then a hard cut to the same four invoices with the switch thrown:
```
  kill switch       : TRIPPED — every invoice will be refused
  REFUSE 8301 · REFUSE 8302 · REFUSE 8303 · REFUSE 8304 — the grant was revoked (kill switch)
```

### 1:15–1:35 · the contrast — the closing shot (45 words)

> "Now the same rig, pointed at a careless checkout: one receiver reused, no memos, the wallet told it may
> reveal amounts, round amounts. *Caught* — four properties, by name. That contrast is the product: a
> clean run passes, a leaky run fails, and a detector that goes quiet fails the build too."

**On screen:** a two-up frame held for a full beat — the control's
```
  caught            : policy-downgrade, receiver-reuse, amount-fingerprint, memo-absent
  verdict           : CAUGHT — the property library found the leaks this flow plants
```
beside the private flow's `12 pass · 0 fail`. This is the money shot the plan named, so it gets the
longest hold in the cut.

### 1:35–1:50 · the clock (35 words)

> "Every claim is published next to the file it came from, with a clock: live, then stale, then dead,
> unless CI re-proves it. Four jobs: the private flow must pass, the control must be caught, the payer's
> ledger must show its refusals, and the index is republished."

**On screen:** the live page — claim rows and their countdowns, the one `regression` row — then the
Actions page with the four green jobs.

### 1:50–2:00 · the honest close (22 words)

> "Regtest, local, our own flow — and the claim expires unless someone re-proves it. A privacy claim is
> not a fact; it's a subscription."

**On screen:** `github.com/memeshee/zanary` · `memeshee.github.io/zanary`, above one line:
`regtest · merchant side simulated · driving your own implementation needs a driver`.

## Recording checklist

```sh
harness/up.sh                                    # the rig (once)
ZANARY_DEMO_RUNS=data/runs/demo \
ZANARY_PAYER_DEMO_DIR=data/demo-payer scripts/demo-run.sh    # the take
```

* terminal at 14–16pt, dark, nothing cleared between stages — the scrolling output is the point;
* the demo scripts already export `RUST_LOG=warn`, so there is no wallet-migration noise to cut;
* the `6 min of run → 2 min` label stays up from 0:15. State the compression; never hide it;
* **if a stage fails on the take:** say so and keep it. The summary names the stage and the script exits
  non-zero — a rig that reports its own failure *is* the claim, and cutting it would contradict the whole
  submission;
* if the video must be fully real-time (no compression label), the lever is smaller flows: one invoice per
  flow and three payer invoices bring the run to about three minutes, at the cost of the control's
  `receiver-reuse` (which needs two payments to compare). Decide before recording, not in the edit.

## Voice notes

* Three sentences have to land: *a budget instead of a key* · *a privacy claim is a subscription* · the
  contrast (*clean passes, leaky is caught*).
* Never say "we built an AI agent" — the agent is the *subject*; the rig is what this build ships.
* No number in the voiceover that is not on screen, including heights and hashes. Read H off the screen.
* Do not narrate the making-of (no "we first tried…", no recorder/renderer talk).

## A longer walkthrough, if a judge asks

The two-minute cut is the submission. For a live walkthrough where there is no time limit, the honest
answer is one command and a scroll: `harness/up.sh && scripts/demo-run.sh` prints all four stages with
their artifacts, and each stage's failure mode is reported rather than hidden.
