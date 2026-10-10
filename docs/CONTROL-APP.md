# The control app — the thing the rig is allowed to bite

A privacy tool that has never found anything is indistinguishable from a privacy tool that does nothing.
So the rig ships with three flows, and the first one is **designed to leak**:

| flow | what it is | what the demo shows |
|---|---|---|
| **`control-leaky`** | a small, plausible ZEC checkout: one receiver for everything, paid by a wallet that takes the easy path | the rig fails it on 8 properties, with txids |
| **`control-fixed`** | the same application after the fixes the rig demanded | the same rig passes it |
| **`reference-payer`** | our own bounded payer (budget · caps · allowlist · receipts), built correctly from the start | it passes, and pays for something real |

The leaky control is not a strawman built to fail. Every one of its behaviours is something a real
integration does in its first week, and each is a single line of application code.

## `control-leaky` — concrete behaviour, and the property it trips

| # | behaviour (one line of app code each) | trips |
|---|---|---|
| 1 | one static receiver address is generated at startup and printed in the README as "the shop address" | `receiver-reuse` |
| 2 | invoices are matched by **amount** instead of by memo (no memo is ever set) | `memo-absent`, `amount-fingerprint` |
| 3 | prices are round numbers (1, 5, 0.5 ZEC) | `amount-fingerprint` |
| 4 | the wallet is constructed with `AllowRevealedAmounts` "so cross-pool sends just work" | `policy-downgrade` |
| 5 | the settlement step pays out to a transparent address (the payout account) | `unshielding` |
| 6 | refunds go to whatever address the customer typed, including a T-address | `transparent-fallback` |
| 7 | the app funds its own wallet and settles in the same block, the moment a request arrives | `same-block-roundtrip`, `timing-correlation` |
| 8 | the checkout posts `{order_id, buyer_email, amount, ip}` to its payment endpoint | `server-metadata` |
| 9 | the receipt it emails contains the receiver and the txid but is asserted in prose ("paid") instead of being verifiable | `report-leak`, `memo-absent` |

**Expected rig output for `control-leaky`:** 8 fails (5 `D`, 3 `W`) with, for each, the txid(s),
the receiver(s) and the exact observation that produced it — plus the profiler successfully clustering
all of one payer's payments into a single customer with high confidence.

### `control-fixed` — the diff the rig should force

1. a **fresh diversified receiver per invoice**, derived from one unified address per merchant
2. the invoice id in a **ZIP-302 memo**, and matching by memo (+amount as a sanity check only)
3. **non-round prices** with per-invoice jitter in the low digits
4. the strict default privacy policy — the rig's finding is a build failure, not a warning to silence
5. settlement stays **inside the shielded pool**; payouts use fresh receivers
6. transparent recipients are **refused**, with the reason shown to the user, not silently upgraded
7. funding and settlement are separated by blocks (the wallet waits for confirmations before paying)
8. the payment endpoint receives `{invoice_id}` and nothing else; no IP logging, no buyer identity
9. the receipt is a **verifiable artifact** (note commitment + memo + amount) that a third party can
   check against the chain without trusting the merchant

**Expected rig output for `control-fixed`:** all `D` pass; the profiler's clustering confidence on the
same player data falls to "not linkable"; W-class items are listed as residual notes where they are
inherent to Zcash (fees, pool sizes at that height) rather than to the app.

## Why ship the control at all

- **Evidence that the tool works.** A finding on code nobody wrote is a finding nobody can dismiss.
- **A regression suite for the rig itself.** If a property stops firing on `control-leaky`, the property
  is broken — that failure is caught by the rig's own CI, which is the honest version of "trust us".
- **The teaching artifact.** Most of `control-leaky`'s nine behaviours are the *default* outcome of
  writing a ZEC integration quickly, so the fixed diff is the shortest useful document the rig could
  hand a team: nine lines of application code stand between a checkout and a customer graph.

## Implementation notes (so the control stays small and honest)

- The leaky and fixed variants share one codebase with a `LEAKY=1` switch; the diff between them is the
  document, so it must stay readable in one screen.
- Both talk to the chain through the same indexer interface the rig uses — no special paths, no mocking
  of the payment itself. Only the *merchant endpoint* is ours (and the report says so).
- The control app never touches mainnet and never holds real value; it is a test fixture with a UI.
- Every behaviour above is written into the app with a comment naming the property it trips, so a
  reader can jump from the failing report line straight to the line of code responsible.
