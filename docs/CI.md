# The gate — a privacy claim that can fail a build

A test suite that only ever reports "green" is decoration. ZANARY's gate is one command that runs a
payment flow on a throwaway chain, scores it against the property library, re-derives the result
from the chain, publishes a timestamp — and exits non-zero the moment something goes wrong,
*including* when the property library stops detecting a leak it is supposed to detect.

```sh
zanary-probe test --definition flows/ci-checkout.json --expect pass
```

## The exit-code contract

| code | meaning | what a reviewer should do |
|---|---|---|
| `0` | the run matched its expectation **and** the chain agrees with the report | read the claim block; nothing to do |
| `1` | the run did **not** match its expectation: a flow that must not leak leaked, or a control flow the library was supposed to bite was not caught | read the `FAIL`/`caught` lines; the flow or the detector is wrong |
| `2` | the gate itself is broken: chain unreachable, the report no longer hashes to itself, the anchor cannot be found by a scan with the beacon's published key | fix the rig; this says nothing about the flow |

`2` exists because "the rig could not check" and "the flow leaks" are different news, and a CI job
that conflates them teaches people to ignore it.

## Both halves of the pair

The gate is only meaningful because it runs *two* flows against the same rig:

- `flows/ci-checkout.json` — fresh diversified receiver per invoice, a ZIP-302 memo per payment,
  `FullPrivacy` policy, non-round amounts. `--expect pass`: a leak fails the build.
- `flows/ci-checkout-control.json` — one receiver reused for every invoice, no memo,
  `AllowRevealedAmounts`, identical round amounts. `--expect fail`: the property library **must**
  catch it, and a control that stops being caught also fails the build.

The control flow is the reason to believe the property library still works. Both jobs are green
when the rig behaves, and each job summary shows what it actually found — the control's summary
lists the four planted leaks by name.

## Using it in your own repository

The whole setup is one step:

```yaml
permissions:
  contents: read
jobs:
  privacy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: memeshee/zanary@main
        with:
          flow: flows/my-checkout.json   # your flow, in your repository
          expect: pass
          anchor: "true"                 # publish the evidence root in a shielded memo
          ttl-days: "30"                 # how long this claim stays live
          out: zanary-run                # run.json, findings.json, report.json, anchor.json, test.json
```

The rig, the property library, the wallets and the report tooling come from the action's own
checkout; the calling repository provides a flow definition and gets its evidence back as files it
can publish. `harness/up.sh` and `zanary-probe test` are the same two pieces, so the action is a
convenience over the documented commands rather than a second implementation.

Run it locally with the same two commands:

```sh
harness/up.sh                                   # seeds the node config, starts the chain, waits until it can be used
cargo build --release --bin zanary-probe --bin zanary-report
./target/release/zanary-probe test --definition flows/ci-checkout.json --expect pass \
    --anchor --ttl-days 30 --summary data/runs/ci/test.json
```

## Waits are bounded, and readiness is a streak

Two failures from the first real CI runs shaped this, and both are worth knowing before debugging a
red job:

- **A gRPC call with no reply does not fail, it hangs.** The second CI run sat in the harness's
  indexer wait for twelve minutes printing nothing, because the readiness probe itself had no
  timeout. Every wait in `harness/up.sh` is now bounded (`ZANARY_PROBE_TIMEOUT_SECONDS`, default 20),
  the gate itself runs under `timeout` (`ZANARY_GATE_TIMEOUT_SECONDS`, default 45 min) and reports a
  timeout as a rig failure — exit `2`, never a verdict about the flow.
- **The indexer exits on startup and is restarted.** lightwalletd starts before zcashd's RPC answers,
  dies on its first failed block fetch, and a client connecting in that window sees the connection
  close. So the indexer container is `restart: unless-stopped`, readiness is *two consecutive probe
  answers with the container still running* (never a bare port check: a restarting container takes
  its port up before it is serving), and the flow runner waits for the indexer to hold every block
  the node does before a wallet syncs against it. Every wait prints what state it saw.

## Re-checking a CI run without trusting the job

Every claim the gate prints is checkable by someone who has the node and nothing from ZANARY:

```sh
zanary-report verify <run>/report.json --node <node>            # every evidence line against the chain
zanary-report verify-anchor --report <run>/report.json \
    --beacon-ufvk <the UFVK printed in the job summary> --node <node>   # earliest anchor carrying the root
```

The beacon's viewing key is public by design (that is what makes the log permissionless), and the
verifier finds the **earliest** anchor for the root, so a later copy cannot forge a timestamp. A
flipped root, a hand-edited report or an unanchored claim exits `1`.

## What this proves, and what it does not

Proved by a green pair of jobs:

- a payment flow really executed: real shielded transactions, real memos, real confirmations, on a
  chain this job started and threw away;
- the flow's privacy properties were evaluated by a library that demonstrably still catches planted
  leaks (the control job);
- the report's evidence agrees with the chain after the fact, verified by code that shares nothing
  with the runner except the data model;
- the run has a timestamp nobody had to be trusted for.

Not proved, and stated in the report itself:

- **The merchant side is simulated by the rig** (`merchant_is_simulated: true`), so the verifier
  skips merchant fields it cannot re-derive from the chain — the skip lines are the honest boundary,
  not a gap hidden behind greens.
- **The chain is regtest and local.** It has no peers, no network layer and an Orchard/NU6 pool, so
  network-level deanonymisation and pool-specific behaviour are out of scope (`docs/PROPERTIES.md`).
- **The flow is a declared flow, not your code.** Today a flow is executed by ZANARY's own wallets
  against the same definitions your app uses; testing *your* implementation directly needs a driver
  for your endpoint (HTTP, Zallet JSON-RPC) — the control app in this repository is the reference for
  what that driver has to record.

## Cost

The honest number: a cold CI run is dominated by the Rust build of the Zcash stack, and the flow's
Orchard proving runs in release mode for exactly that reason. Block production is instant on
regtest (140 blocks in about two seconds), the flow itself is two invoices, and every run after the
first reuses the cargo cache. Budget tens of minutes for the first run in a new repository, single
digits once the cache is warm; `timeout-minutes: 120` in this repository's workflow is deliberate
headroom, not an expectation.

## The published index

A third job (`publish`, after the gates, `main` only) collects the two run directories, adds the
recorded claims from `crates/report/fixtures/reports/` as history, renders them with
`zanary-report index` and deploys the result to GitHub Pages:
**<https://memeshee.github.io/zanary/>**.

The page is a rendering, not the authority: each claim sits at `claims/<run>/report.json` next to
its `anchor.json`, in the run-directory layout the index reads, and **every card links its own
files** (`zanary-report index --claims-prefix claims`, the default), so a reader downloads the
report and `zanary-report verify` it instead of trusting the markup. The footer carries the
one-command reproduction, so the page is usable by someone who has never seen the repository.

The claim's clock is on the page (`expires in 12d`, then `stale`, then
`dead`), and the weekly `schedule` in the workflow is what re-issues it. A claim that stops being
re-proved keeps its row and its countdown — that is the point of publishing it.

## Where this sits

`docs/CLAIMS.md` describes the claim lifecycle (live → stale → dead, re-probe to re-issue). This
document is how a claim gets *made* on every push: the scheduled run in
`.github/workflows/privacy-gate.yml` is the re-probe, and a claim that stops being re-proved is what
the index renders as stale.
