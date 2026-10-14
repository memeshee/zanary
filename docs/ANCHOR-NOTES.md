# ANCHOR-NOTES — turning a report into a timestamped claim

What this documents: `canary-probe anchor` and `canary-report verify-anchor`, the two halves of the
one thing that makes a CANARY report more than a file someone could have edited last night. A report's
`evidence_root` and `report_hash` go into a **shielded memo** paid to a public beacon address, and
from then on the chain says for everyone, without any authority, that this evidence existed by that
height.

```
canary-probe anchor --report data/runs/<run>/report.json     # → data/runs/<run>/anchor.json
canary-report verify-anchor --report data/runs/<run>/report.json \
    --beacon-ufvk <the beacon's published UFVK> --node http://127.0.0.1:18232
```

## What goes on chain

The memo is `canary-anchor:v1:` followed by a **closed** JSON payload — a field the format does not
define is refused, the same rule receipts follow:

```json
{"v":1,"chain":"regtest",
 "flow_commit":"f2f15593be078285500f3352fa7be3e96de386891042c0993f72fc89e105f970",
 "report_hash":"ffa951496701533a024edfb34dd1691256f327e71fb188b018fa4fdad6860fe0",
 "evidence_root":"48fc1ee47429a2bf90ae2a8921ba9a1d14656923d81867074c6de6baccd07468",
 "verified_at_height":629}
```

That is 269 bytes of a 512-byte memo field: hashes, a chain name, and the height the report was made
at. No identity, no amount, no finding text, nothing about the payer or the flow's owner — an anchor
says *"evidence with this root existed by then"* and nothing else.

The anchor rides on a **payment** (1 000 zatoshis — the smallest thing that gives a memo a note to
sit on), paid to the beacon's next fresh receiver, from a third wallet of its own seed
(`core::BEACON_MNEMONIC`, `data/wallet-beacon/canary-beacon.db`). The beacon's *viewing key* is
published in the anchor record, on purpose: it can only ever read this public log of hashes, and
publishing it is what lets a stranger do the check below with no trust in CANARY.

## What was actually run

The batched flow's report — the one with the designed `payer-linkability` failure — anchored at the
end of the same session:

```
  report hash       : ffa951496701533a024edfb34dd1691256f327e71fb188b018fa4fdad6860fe0
  evidence root     : 48fc1ee47429a2bf90ae2a8921ba9a1d14656923d81867074c6de6baccd07468
  verified height   : 629
  anchor amount     : 0.00001000 ZEC (the note the memo rides on, not a payment)
  beacon            : uregtest1zh65e4se5ncjfsfxvjg9qaj2yttzjw9jge0qxdgypcc7lr6r5gtkxs28v60gz95hj84tam3ll0qf7qf8f8nfttr3ejmkl9wnvc8lf4xk
  beacon tag        : DiversifierIndex([12, 207, 186, 18, 1, 0, 0, 0, 0, 0, 0])
  anchor txid       : 8f760aa8e22002e0cda9b1bdf54329c329095aab5f11a4d5d192f537a266ef9f
  anchored at       : height 630
```

A second anchor followed, for the **control** run's report (the flow that fails four properties by
design — its evidence deserves a timestamp as much as a clean run's, and a log with one entry is weak
evidence for a log):

```
  report hash       : f0bf4917001b86b49086961dab0f76b3c746e26a350a61702d69d78cea38dd0f
  evidence root     : 7b3813d1f1e4530f69da4c664225bdf088dd7accdcae2ae95f32d452d5fd01a2
  beacon tag        : DiversifierIndex([79, 221, 186, 18, 1, 0, 0, 0, 0, 0, 0])
  anchor txid       : adaa09371753994add6f0281b6f299c47340657fa3372d848c521e9c1b5428db
  anchored at       : height 641
```

The beacon's log therefore holds two anchors at two heights, and each report's root resolves to its
own earliest hit.

## The stranger's check, and the four ways it can fail

`verify-anchor` needs a node, the chain, and the beacon's viewing key. It recomputes the report's hash
and evidence root *from the report*, scans the chain for a memo carrying that root, and accepts the
**earliest** one. On the harness that scan is 640 blocks and 693 transactions in ~3 seconds —
`getblock <hash> 2` hands back each block's transactions with their raw bytes, so the whole walk is
two RPC calls per block and none per transaction.

```
  scan              : 651 block(s) 1..=651 read · 705 transaction(s) trial-decrypted
  anchor memos      : 2 found in that range
  anchor            PASS first anchored at height 641 in tx adaa0937… (orchard output 0)
  anchor height     PASS anchored at height 641 ≥ the report's height 474
  anchor payload    PASS every field the report can re-derive agrees with the memo

  summary: 4 pass · 0 fail → ANCHORED          exit 0
```

Every one of those four verdicts was exercised against a case built to break it:

| case | result |
|---|---|
| an anchored report (the batched flow's, then the control's) | `4 pass · 0 fail → ANCHORED`, **exit 0** |
| a different, genuinely unanchored report (the jittered run) against the same beacon | `anchor FAIL — no anchor memo in blocks 1..=651 carries this evidence root (2 anchor memo(s) were readable, none of them this one)`, **exit 1** |
| a root with one hex digit flipped, no report | `0 pass · 1 fail → NOT ANCHORED`, **exit 1** |
| `--root` disagreeing with the report it is passed with | `root argument FAIL --root 48fc1ee4…ccd07460 disagrees with the report's evidence root 48fc1ee4…ccd07468`, **exit 1** |
| an anchored report edited after the fact (a failing verdict flipped to `pass`) | `report hash FAIL … refusing to search for the anchor of a report that does not hash to itself`, **exit 1** |

The last row is the one that matters: the anchor is a claim *about an exact artifact*, so the check
refuses to even look for an anchor when the file no longer hashes to itself. An edited report does not
inherit the anchor — it loses it.

The second row is the one that keeps the check honest: **two** anchor memos were readable in that scan
and the verdict was still `NOT ANCHORED`, because neither carries the root under test. The check is
root-specific — "the beacon has anchors" is not the same statement as "this evidence is anchored", and
only the second one is what the command reports.

## What an anchor does and does not buy

- **A timestamp nobody controls.** The height that mined the memo is the moment the evidence existed;
  nobody can backdate a passing run, and nobody can remove an anchor once mined (short of a reorg,
  below).
- **Anonymity for the runner.** The memo carries hashes; the payment's spending side is shielded, so
  an observer learns that *some* wallet anchored a root, not which one. This is the same
  unlinkability the rig asserts about payments elsewhere, applied to its own publishing.
- **A permissionless log — with the caveat that implies.** Anyone can pay the beacon and add an anchor
  (it is a payment to a published address). That cannot forge a timestamp: a root's *earliest* anchor
  is still the moment it first existed on chain. It does mean the log can be spammed, which is exactly
  why the verifier reports which hit it accepted instead of "an anchor exists".
- **Not a proof of the claim's content.** The anchor says these bytes existed by then; `canary-report
  verify` is what says the bytes are what the report claims. The two checks are independent on purpose.

## The claim page

`canary-report index` reads an `anchor.json` beside each `report.json` (either layout: a run
directory or a flat directory of reports) and puts the timestamp on the page:

```html
<p class="meta">anchor memo <code>8f760aa8e220…</code> @ height 630 — anchored evidence: the chain
says this root existed by then</p>
```

A report with no anchor record says so instead (`no anchor record beside this report — unanchored
evidence is only as old as the file that carries it`), which is the honest rendering: the index will
not present an unanchored claim as if it had a timestamp. Rendering the index over reports with no
anchor records at all produces the same HTML as before anchoring existed — the golden fixture in
`crates/report/tests/golden.rs` is unchanged, which is the proof of that.

## Traps hit

1. **A scan that finds nothing must never look like a scan that did not run.** The verdict line
   carries what was read (`640 block(s) 1..=640 · 693 transaction(s) trial-decrypted`) and how many
   anchor memos were readable in that range, because "no anchor" and "no scan" would otherwise print
   the same thing.
2. **Diversifier indices are drawn, not counted.** The first version of the probe's output called the
   beacon receiver's index an "anchor number"; a fresh wallet's first receiver came back at
   `DiversifierIndex([12, 207, 186, 18, 1, 0, …])`, not 0 — this stack draws indices rather than
   walking them. It is recorded as a *tag* for the anchor, and the docs say so, because a number that
   looks like a sequence invites "where are anchors 1–11?".
3. **A prefix match that does not parse is an error, not an absent anchor.** `canary-anchor:v1:` that
   is not valid JSON, or carries a field v1 does not define, fails loudly — silently skipping it
   would turn a corrupted anchor into "this run was never anchored".
4. **`short()` hides the difference in the tail.** A root that differs only in its last digit printed
   identically on both sides of a `FAIL` line, which reads as "these agree". The comparison messages
   now print `first8…last8`.

## Reproduce

```sh
cd /home/kiter/hackathon/canary
cargo build --bin canary-probe --bin canary-report
export CANARY_LWD_URL=http://127.0.0.1:9067

./target/debug/canary-probe anchor --report data/runs/checkout-private-batch/report.json
UFVK=$(python3 -c "import json;print(json.load(open('data/runs/checkout-private-batch/anchor.json'))['beacon_ufvk'])")
./target/debug/canary-report verify-anchor --report data/runs/checkout-private-batch/report.json \
    --beacon-ufvk "$UFVK" --node http://127.0.0.1:18232

# the negative case: a report nobody anchored
./target/debug/canary-report verify-anchor --report data/runs/checkout-private/report.json \
    --beacon-ufvk "$UFVK" --node http://127.0.0.1:18232      # exit 1
```

## Known gaps

- **One anchor per run, and only for the runs we anchored by hand.** Anchoring is not yet wired into
  the flow runner (a run ends, then you anchor its report) nor into a scheduler; the claim index's
  re-probe story (`docs/CLAIMS.md`) still needs an anchor per re-run.
- **Reorgs.** On a throwaway regtest chain a reorg is theoretical; on a real chain the earliest anchor
  could move by a block or two. The verifier reports the height it found, so a moving anchor is
  visible rather than silent — but the TTL arithmetic should treat a young anchor as soft.
- **The beacon wallet is local state.** Its `data/wallet-beacon` database holds the addressing
  bookkeeping; the *memory* of anchors is the chain plus the published viewing key, which is why
  losing that database would not lose the log, only the ability to derive the next tag.
- **The beacon's log is unauthenticated by design** (permissionless) and has no size limit: a spammer
  can make the scan slower. A production beacon would want a small fee or a rotating set of beacons —
  the format already carries the chain and the height, so rotating beacons stay compatible.
