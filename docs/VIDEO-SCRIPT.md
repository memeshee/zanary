# The demo video — beats, timings, and what is on screen

Seven minutes, one take, driven by one command. The recording is not a tour of the repository: it is
four stages that produce artifacts, each one allowed to fail.

```sh
harness/up.sh && scripts/demo-run.sh        # the single take
```

Recording notes (kept out of the repository on purpose — no recorders, renderers or fonts ship here):

* terminal at 14–16pt, dark background, `data/` and the terminal *not* cleared between stages; the
  scrolling output is the point;
* the run is unattended after `up.sh`, so the operator's job in the video is to talk, not to type;
* every number spoken below appears in the artifact on screen. If a stage fails, say so and keep going —
  `demo-run.sh` reports each stage and exits non-zero instead of hiding it.

---

## 0:00 — the opener (before anything is run)

**On screen:** the README's component list, scrolled once, then the empty terminal.

> "Zcash already has a till: gateways and facilitators that accept shielded payments on behalf of a
> merchant. What nobody hands you is the *payer's* side — an agent with a budget instead of a key:
> bounded authority, repeat payments that don't cluster, and a receipt that verifies without trusting
> the tool that made it.
>
> ZANARY is the rig that proves that property, on a throwaway chain, as a test result that can fail."

**Do not** open with architecture. This is the whole first 20 seconds: the payer's side, bounded
authority, unlinkable repeats, verifiable receipts.

## 0:20 — the problem, in one operator's terms

**On screen:** the grant file, opened once (`data/payer-demo/grants/allowed.json`).

> "An operator who wants an agent to pay for things has two options today: no budget, or a public rail
> that publishes everything the agent buys. Here is the third: a grant — a budget, a per-payment cap,
> a list of receivers, an expiry, and a kill switch. Authority, not a key: there is nothing in this
> file that can move money on its own."

## 1:00 — stage 1: a flow that does it right

**On screen:** `scripts/demo-run.sh` starts; stage 1 prints the flow definition, then the payments.

> "Two invoices, a fresh diversified receiver for each, one ZIP-302 memo per payment, the wallet asked
> for full privacy. The rig watches the chain while it happens — not the app's logs."

**On screen:** the property table prints, then the anchor, then the claim block.

> "Every property passes, and the evidence is re-derived from the chain afterwards — the verification
> lines you can see. Then the evidence root goes into a shielded memo, which gives the claim a
> timestamp nobody has to be trusted for. That is what 'verified at height 135' means."

## 2:15 — stage 2: the same rig, on a flow that leaks

**On screen:** stage 2 runs the control flow; four `FAIL` lines print.

> "Same rig, same property library, a checkout that is deliberately careless: one receiver reused for
> every invoice, no memos, the wallet told it may reveal amounts, round amounts. Four properties bite,
> and the gate says *caught* — because a test suite that only ever passes is decoration. This is the
> hardest thing to fake in a demo: the property library has to demonstrably still bite."

## 3:15 — stage 3: the bounded payer

**On screen:** stage 3 — four invoices issued, the grant, then the payer's decisions.

> "Now the subject. Four invoices: one fits the grant, one would pass the budget, one is over the cap,
> one comes from a receiver the grant never named. The payer decides *all four before it pays
> anything* — so a refusal can't be an artifact of a payment that went wrong earlier."

**On screen:** `ALLOW 8301`, `REFUSE 8302 … budget`, `REFUSE 8303 … per-payment cap`, `REFUSE 8304 …
allowlist`, then the ledger.

> "One payment. Three refusals, each naming the rule that refused it, all of it in the ledger — which is
> the evidence, not the payer's console. The rig then checks that ledger against the grant:
> `payer-policy` reads the amount, the receiver and the reason, and fails if a payment went past the cap,
> past the budget, to a stranger, or through after the switch was thrown."

**On screen:** the kill-switch run — four `REFUSE … kill switch`.

> "And the kill switch: the same four invoices, authority withdrawn. Nothing is paid, and the refusal
> says why. `--expect-refused` makes that a test, not a screenshot."

## 5:00 — stage 4: the claims are public, and they expire

**On screen:** stage 4 renders the index, then the browser opens the live page.

> "Every one of those claims is a file with a hash, and the page is a rendering of them — you can
> download the claim itself and re-verify it against the node. Each row has a clock: *live*, then
> *stale*, then *dead*, and a weekly CI run is what re-issues it. A privacy claim is not a fact, it is a
> subscription."

**On screen:** the GitHub Actions page — the four jobs green, the artifacts, the badge.

> "The same four stages run on every push, from a clean checkout on a fresh chain: the private flow must
> pass, the control flow must be caught, the payer's ledger must show the refusals, and the index gets
> republished."

## 6:00 — what this does not prove (say it before a judge finds it)

> "Three honest limits. The chain is regtest and local — the rig tests property classes, not a mainnet
> anonymity set. The merchant side is the rig's own simulation, and the report says so on the line where
> a verifier skips it. And the flow you watch passing is a flow in this repository: testing *your*
> implementation directly needs a driver for your endpoint — that is the next thing, and the control
> app is the reference for what such a driver has to record."

## 6:30 — close

> "So: the payer gets bounded authority, the receipts verify without trusting the tool, the profiler
> tries to cluster repeat payments and finds nothing, and the claim expires unless it is re-proved.
> Everything here is one command away — `harness/up.sh && scripts/demo-run.sh`, and the docs next to it
> say exactly what each stage proves and what it does not."

---

## The exact commands on screen (for the recording checklist)

| time | command | artifact to point at |
|---|---|---|
| 1:00 | `scripts/demo-run.sh` (stage 1) | `data/runs/demo/demo-private/report.json` (PASS, anchored) |
| 2:15 | stage 2 | `…/demo-control/report.json` (4 FAILs, caught) |
| 3:15 | stage 3 (`scripts/bounded-payer-demo.sh`) | `data/demo-payer/runs/allowed/ledger.jsonl` |
| 4:40 | stage 3, second run | `data/demo-payer/runs/killed/ledger.jsonl` |
| 5:00 | stage 4 (`zanary-report index`) | <https://memeshee.github.io/zanary/> |
| 5:40 | browser → Actions | the four green jobs, `zanary-*-evidence` artifacts |

## Voice notes for the narration

* The three sentences that must land: *a budget instead of a key* · *a privacy claim is a subscription* ·
  *the confessions are first-class* (do not say "we couldn't do X": say what is out of scope and why).
* Never say "we built an AI agent" — the agent is the *subject*, the rig is the product of this build.
* Numbers are spoken the way the artifact prints them (`verified at height 135`, `9 pass · 0 fail`).
  No rounded-up numbers, no counts that are not on screen.
