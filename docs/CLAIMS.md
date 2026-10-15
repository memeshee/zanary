# The claim index — privacy claims that expire

A scan is a moment; a claim is a commitment. The index is the part of ZANARY that turns a run into
something a stranger can rely on *and* falsify later.

## A claim

```
claim = (flow_id, flow_commit, property_id, verdict, evidence_root, chain, verified_at_height, ttl)
```

- `flow_commit` — the hash of the flow definition, so a claim is about an exact revision, not a vibe.
- `verdict` — `pass` | `fail` | `skip`, with the severity the property carries.
- `evidence_root` — the Merkle root of the run's evidence (txids, receivers, memo bytes, heights).
- `verified_at_height` — the chain height the run was made against; the clock starts here.
- `ttl` — the claim's lifetime (default 30 days, or 3× the flow's change rate).

A claim is **never** "the app is private". It is *"this exact revision of this flow passed these
properties on this date, and here is the evidence"* — which is the strongest statement a test can make
and the only one it can defend.

## Expiry, by construction

- Every claim carries `expires_at = verified_at + ttl`.
- A scheduled re-probe re-runs the flow; a passing re-run **re-issues** the claim with a new
  `verified_at_height` and a new evidence root (the old one is kept, append-only — history is evidence
  too).
- Miss the TTL and the claim moves to **stale** (visibly, with the countdown); miss two and it goes
  **dead**. A dead claim is not deleted: the page keeps saying "this was true in October and has not
  been re-proved since", because that is the honest rendering of an unmaintained guarantee.
- A future `fail` on a previously passing property is shown as a **regression** with both runs linked.

The rule the whole design serves: an unverified privacy claim decays to a liability, so nobody has to
guess which of the ecosystem's privacy claims are current.

## Publishing without doxxing

- **Anonymised by default.** A claim page identifies the flow by its commit hash and a
  conforming-or-refusing label (`flow-less 3.1.0`), not by the project's name. Maintainers can attach
  their name, logo and a link with an opt-in toggle; the tool does not out anyone, including by
  implication.
- **Findings are framed as defaults, not verdicts on a team.** Every FAIL line names the line of
  application code that produced it and the shortest known fix. Most findings are the *default*
  outcome of writing a ZEC integration quickly — including in our own first draft.
- **Evidence, not scores.** No "privacy score" out of ten: a composite number invites gaming and hides
  which property broke. The page lists properties, severities and evidence links.

## Anchoring: making evidence tamper-evident and anonymous

Each run hashes its `report.json`, and the Merkle root is written into a **shielded memo** to a public
beacon address (CAₙ beacons). That gives three things no ordinary test artifact has:

1. **Timestamping without an authority** — anyone can prove the evidence existed at that height, and
   nobody can retro-fit a passing run.
2. **Anonymity for the runner** — the anchor reveals nothing about who ran the probe, only that
   evidence with this root exists; the memo carries no identity, and the spending side of the deposit
   is unlinkable, which is exactly the property the rig is asserting elsewhere.
3. **A verifiable boundary for the claim's TTL** — `verified_at_height` is a chain fact, not a server's
   word.

A verifier therefore needs no trust in ZANARY's servers to check a claim: the report hash, the
properties, the evidence array, and the anchor memo are all checkable against the chain. The index is
the *convenience*, not the authority.

**As built** (`docs/ANCHOR-NOTES.md`): `zanary-probe anchor --report <report.json>` writes the payload
into a shielded memo paid to the beacon (`8f760aa8…`, mined at height 630 for the batched run's report),
and `zanary-report verify-anchor --report <report.json> --beacon-ufvk <the published key> --node <node>`
finds the **earliest** anchor carrying that root — 640 blocks and 693 transactions scanned in ~3 s, and
exit 1 for an unanchored report, a flipped root, a `--root` that disagrees with the report, or a report
whose hash no longer recomputes. Anchoring is permissionless, which cannot forge a timestamp (earliest
hit wins) but does mean the log can be spammed and that the verifier must name the hit it accepted.

## What the page shows a judge in ten seconds

```
flow         checkout-control-fixed @ 4f2a9c1        verified at height 1,482  (expires in 29d)
privacy      PASS  no transparent fallback · no unshielding · fresh receiver per invoice
cluster      FAIL  mock: payer cluster reconstructed with 0.31 confidence (threshold 0.70)
residual     W  fees (5000 zat/action) make sub-dollar invoices uneconomic — not an app defect
evidence     report hash 9c1f…  ·  anchor memo tx 3d0e…  ·  re-run: zanary test ./flow.toml
```

## Where this sits in the submission

The index is the difference between "we wrote a test suite" and "we built infrastructure a community
can keep using": a test suite answers a question once, an index with expiry keeps answering it, and an
ecosystem that funds privacy work gets a way to ask for reproducible evidence instead of prose.
