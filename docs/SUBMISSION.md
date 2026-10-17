# Submission — what goes in the form, and what was actually shipped

ZECATHON 2026 · track: **Core & Tooling** · deadline **Oct 28, 23:59 UTC** (Oct 29, 06:59 ICT).
Everything below is either in the repository or live at a URL, and every number is one this rig printed.

## Form fields (ready to paste)

**Project name:** ZANARY — the privacy gate for Zcash payment flows

**One line:** An independent prober for Zcash payment flows: it runs a flow with real shielded
transactions on a throwaway chain, proves or breaks each privacy claim with reproducible evidence, and
publishes claims that expire unless someone re-proves them.

**Repo:** <https://github.com/memeshee/zanary> (public) · **Live claims:**
<https://memeshee.github.io/zanary/> · **CI:** the `privacy gate` workflow, five jobs per push (fmt + clippy + the 87-test unit suite; private flow must pass; control flow must be caught; the bounded payer's ledger must show its refusals; the index and the demo video are republished)

**Track:** Core & Tooling (one submission, one track)

## Description (the form's field is ≤2000 characters)

**Paste this — 1987 characters, including the markdown:**

```
**What it does.** A privacy gate for Zcash payment flows: it runs a checkout on a throwaway local chain with real shielded transactions, observes what the chain, a merchant and an observer can see, scores it against 13 named leak classes, hashes the evidence — and lets the claim expire. One command: `harness/up.sh && scripts/demo-run.sh`.

**Why it does not leak.** The verdict is re-derived, not reported: `zanary-report verify` is a second implementation that re-checks every line against the chain (tx exists, receiver decrypts, memo decrypts, pool totals match getblock) — flip a byte and it exits 1. Claims are flow x commit x height with a clock, dead unless CI re-proves them weekly. The gate fails builds on itself too: one flow must pass, a deliberately leaky control must be CAUGHT; exit 2 is "could not check", never a verdict. The evidence root also rides in a shielded memo to a beacon whose viewing key is published, so its time is not ours to fake. Nothing of the payer's escapes either: runs are regtest-local (no mainnet keys, no funds), and anchors carry hashes, a chain name and a height.

**The subject that passes.** A bounded payer — an agent given a budget instead of a key: budget, per-payment cap, receiver allowlist, expiry, kill switch. It decides every invoice before paying any, writes one ledger line per decision naming the deciding rule, and the rig checks that ledger against the grant: 4 invoices → 1 paid, 3 refused by three different rules; the same four under a tripped kill switch → all refused.

**Real vs mocked.** Real: the node, the shielded transactions, the memos, the proofs, the evidence hashes, and the second implementation that re-derives them. Ours: the wallets, built on librustzcash. Simulated and labelled as such: the merchant endpoint the rig runs itself — the verifier skips those fields instead of pretending. Scope: regtest and local, so network-level deanonymisation is out of scope and the flows tested are this repository's.
```

<details>
<summary>The long form, for the README or a judge who wants the full argument (do not paste: 3,178 characters)</summary>

**The problem.** Privacy claims about Zcash payment flows are un-falsifiable prose. "We don't leak your
identity" is either untested or tested by the same code that makes the claim. Anyone building a checkout
knows the failure mode: the UI is right and the *bytes* are wrong — one receiver reused, a memo left
out, the wallet told it may reveal amounts, round amounts that fingerprint a deposit.

**What ZANARY is.** A rig that runs a payment flow on a throwaway regtest chain with real shielded
transactions, observes what the chain, the merchant and an observer can see, and scores it against a
library of **13 named leak classes** — then hashes the evidence and lets the claim expire.

Four things make it more than a test suite:

1. **The evidence is re-derived, not reported.** `report.json` carries a hash of its own bytes and an
   evidence root; `zanary-report verify` is a *second implementation* that re-checks every line against
   the chain (transaction exists, receiver decrypts, memo decrypts, pool totals match `getblock`). A
   flipped byte exits 1.
2. **The claims have a clock.** Each claim is `flow × commit × height` with a TTL: *live*, then *stale*,
   then *dead* unless CI re-proves it weekly. *A privacy claim is not a fact, it is a subscription.*
3. **The gate fails builds — including on itself.** `zanary-probe test` exits `0`/`1`/`2` (passed / did
   not match expectation / rig could not check). CI runs two flows: a private one that must pass and a
   deliberately leaky control that must be **caught**, so a property library that goes quiet fails the
   build too. The control's four planted leaks are printed by name in the job summary.
4. **Timestamps nobody is trusted for.** The evidence root goes into a shielded memo to a public beacon;
   `verify-anchor` finds the *earliest* anchor for that root with a node and the beacon's published
   viewing key — nothing from us.

**The subject that passes.** A bounded payer — an agent given **a budget instead of a key**: budget,
per-payment cap, receiver allowlist, expiry, kill switch. It decides every invoice before it pays any of
them, writes one ledger line per decision with the rule that decided it, and the rig checks that ledger
against the grant (`payer-policy`: over-cap, past-budget, stranger-receiver and post-revocation payments
fail; an "internal error" refusal fails too). Recorded: 4 invoices → 1 paid, 3 refused by three
different rules; the same four under a tripped kill switch → all refused, nothing spent.

**Reproduce it (one command each).**

```sh
git clone https://github.com/memeshee/zanary && cd zanary
harness/up.sh && scripts/demo-run.sh        # private PASS · control CAUGHT · payer ledger · index
```

**What this does not prove** (stated in the reports themselves, not discovered by a judge): the chain is
regtest and local, so pool-specific and network-level behaviour is out of scope; the merchant side is the
rig's own simulation and the verifier *skips* those fields rather than pretending; the flows tested are
flows in this repository — driving a third party's implementation directly needs an endpoint driver, for
which the control app is the reference.

</details>

## The form's other constraints, checked

| field | constraint | state |
|---|---|---|
| track | exactly one | Core & Tooling |
| description | **≤2000 characters** | the version above, **1987** — the long form is folded away and must not be pasted into the field |
| repo URL | public by the deadline | <https://github.com/memeshee/zanary>, public |
| working demo URL | a live URL a judge can open | <https://memeshee.github.io/zanary/> — the published claim index (this product is a CLI rig; the index is its live surface, with the raw claims served beside it) |
| video URL | **≤2 minutes**, closing shot = profiler-vs-control | **<https://memeshee.github.io/zanary/video/>** (player) · direct file: `.../video/zanary-demo.mp4` · **112 s**, 1600×900, 3.7 MB, served by the same Pages deploy as the claims; frames from a real exit-0 run, narration word-counted to 1:52 |
| attestation | repo will be public | true, and it already is |
| profile | Telegram handle required | operator step |

## The ship checklist (memo §5, D17)

| artifact | state |
|---|---|
| README (truthful about what is real) | ✅ |
| architecture diagram | ✅ `docs/rig.svg`, referenced from the README |
| mark | ✅ `docs/zanary-mark.svg` |
| description | ✅ this file |
| video (≤2 min) | ✅ built and served: <https://memeshee.github.io/zanary/video/> — 112 s, frames verbatim from a real run, published by the same workflow as the claims (`docs/VIDEO-SCRIPT.md` is the cut sheet; replace the audio track with your own voice any time) |
| repo public by the deadline | ✅ public since the first commit |
| CI green on `main` | ✅ five jobs: tests (fmt · clippy · 87 unit tests), private flow, control flow, bounded payer, publish |
| claims published | ✅ <https://memeshee.github.io/zanary/> |
| independent audit, in-tree | ✅ [`audit/`](../audit/README.md) — 19 findings (11 confirmed, 8 leads) from a read-only run at `7058406`, every confirmed one fixed with a regression test, three leads re-verified against the running rig |
| GitHub release (not a form field) | ✅ [`zecathon-2026-r2`](https://github.com/memeshee/zanary/releases/tag/zecathon-2026-r2) — cut at `main`, demo mp4 attached; the earlier `zecathon-2026` snapshot is marked superseded |
| submission form sent | ⏳ operator step: Telegram handle, one track (Core & Tooling), before Oct 28 23:59 UTC |

## Numbers a judge can re-derive

| what | where it comes from |
|---|---|
| `12 pass · 0 fail` on the private flow | `data/runs/demo/demo-private/report.json` (or the CI artifact) |
| `4 fail` caught on the control flow | `data/runs/demo/demo-control/report.json` — four properties named |
| `1 paid · 3 refused [allowlist, budget, cap]` | `data/payer-demo/runs/allowed/ledger.jsonl` (the demo scripts' default output dir; the recorded take used `ZANARY_PAYER_DEMO_DIR=data/demo-payer`, so the same file lives under whichever name that command was run with) |
| `0 paid · 4 refused [kill switch]` | `data/payer-demo/runs/killed/ledger.jsonl` — same caveat |
| 13 properties × 7 runs | `docs/LEAK-MATRIX.md` |
| a claim's clock | every row of the published index: `expires in …`, then stale, then dead |
| a claim surviving a moving chain | re-run `zanary-report verify data/runs/demo/demo-private/report.json --indexer $ZANARY_LWD_URL`: the evidence is height-pinned (`@983`, `@987`, `@1000`), so it still reads `12 pass · 0 fail · 6 skip → VERIFIED` after the chain has advanced past it — the report is a check, not a snapshot |
| the audit's own counts | `audit/run-1/findings.json` (19 records: `verdict` per finding) and `audit/run-1/coverage-ledger.json` (20 units, 113 `local_checks`); `audit/README.md` maps every finding to its fix, and `audit/run-1/agents/` holds each seat's evidence |
