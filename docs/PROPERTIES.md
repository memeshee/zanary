# CANARY — the leak classes the rig tests

This is the intellectual core of the rig: a named, evidence-producing test for each way a Zcash
payment flow can leak. Every property is written so that it can be **executed** (real shielded
transactions on a throwaway chain), **observed** (what the chain, the merchant, and a hypothetical
observer can see), and **re-run** (deterministic, minutes, no mainnet funds).

Severity is about consequence, not style: **D** = disqualifying-class (identity or amounts become
linkable), **W** = warning (weakens an anonymity set), **N** = note (hygiene).

A flow passes only if it is clean on every `D` property. The rig's job is to be the adversary a
project never gets for free: the one that says "you did the right thing in the UI and the wrong
thing in the bytes".

| id | property | severity |
|---|---|---|
| `transparent-fallback` | a transparent address is accepted, required, or produced anywhere in the flow | **D** |
| `unshielding` | value leaves the shielded pool, or the flow routes through transparent | **D** |
| `receiver-reuse` | the same receiver is used for two distinct payments/invoices | **D** |
| `payer-linkability` | repeat payments by the same payer are clusterable by an observer | **D** |
| `policy-downgrade` | the wallet is instructed to relax its privacy policy (`AllowRevealedAmounts`, weaker) | **D** |
| `memo-absent` | the flow claims invoice reconciliation but binds no memo to the payment | W |
| `memo-content` | the memo carries PII in cleartext, or anything that must not be permanent | W |
| `amount-fingerprint` | amounts are round, repeated, or echo a funding amount so deposits and withdrawals link | W |
| `timing-correlation` | a payment follows its funding within a window that identifies the payer | W |
| `same-block-roundtrip` | funding and spending land in the same block | W |
| `server-metadata` | the flow hands a third party (facilitator, API, merchant) information about the payer | W |
| `report-leak` | an exported receipt/report reveals more than the claim it supports | N |

---

## D-class properties

### `transparent-fallback`
**Leaks:** everything — with a transparent address, amounts, both parties and timing are public and
permanent, and (worse for a privacy product) the convenience path becomes the default for every user.
**Detect:** execute the flow's happy path and its error paths; watch every address the flow emits or
accepts. Test with a transparent recipient, a TEX-form unified address, a cross-pool request, and a
malformed-but-parseable address. Any success, or any silent downgrade, is a finding.
**Evidence:** the address, the txid, the code path name the flow reported.
**Why it matters:** this is the single most common way a "privacy" payment product is not private.

### `unshielding`
**Leaks:** the moment funds leave the pool, the payout is public; a merchant that accepts a shielded
deposit and unshields it daily has published its own revenue.
**Detect:** trace value-pool movement across the flow's transactions (`getblockchaininfo` value pools
before/after, per-transaction pool deltas). Also detect *routes*: a transparent hop between two
shielded legs.
**Evidence:** the txid that moved value out of the pool, with the before/after pool totals.

### `receiver-reuse`
**Leaks:** two invoices paid to the same receiver are linkable by the merchant, by the payer's own
wallet provider, and by anyone the merchant shares data with — address reuse is the substrate of every
customer graph.
**Detect:** run N distinct payments (N ≥ 5, two different payers, two different amounts). Collect every
recipient receiver observed per payment. Any repeat across *distinct* payments/invoices is a finding,
including repeats within one diversified-UA family that the flow reuses on purpose.
**Evidence:** the two payments' txids and the shared receiver.

### `payer-linkability`
**Leaks:** even with fresh receivers, a payer whose payments share a distinguishable pattern is
fingerprintable: the same "agent" buying twice is a profile.
**Detect:** the rig acts as an adversary with full chain visibility and the merchant's logs, then tries
to cluster. Signals scored: exact amount repeats, memo structure (prefix/structure, not content),
intra-hour timing buckets, common change behaviour, and the payer's own funding transaction if the
flow exposes it. The property fails if clustering links ≥ 2 payments to one payer with confidence.
**Evidence:** the cluster (txids), the signals that produced it, and the control: the same analysis on
a flow that is *designed* not to leak, where the cluster must not form.

### `policy-downgrade`
**Leaks:** a wallet told to reveal amounts reveals amounts — the UI can still say "shielded".
**Detect:** capture every privacy-policy parameter the flow passes to the wallet/node layer
(`AllowRevealedAmounts`, `AllowRevealedSenders`, `AllowLinkingAccountAddresses`, `AllowFullyTransparent`,
or their equivalents at the library level), and every implicit downgrade a node performs on the flow's
behalf. Real-world confirmation: zcashd refuses a cross-pool spend by default with
*"…would reveal transaction amounts. THIS MAY AFFECT YOUR PRIVACY…"*, so an app that stops getting that
error either fixed its flow or turned the guard off.
**Evidence:** the call, the parameter, the resulting txid.

## W-class properties

### `memo-absent`
**Leaks:** not directly — but it breaks the only reconciliation channel a shielded payment has, which
in practice pushes the merchant back to transparent rails to reconcile. A flow that claims invoice
matching must bind the invoice to the payment in a memo.
**Detect:** for every payment the flow makes in a reconciliation-shaped flow, assert that a memo exists
and that a fresh verifier can recompute the binding (invoice → note) from the chain alone.
**Evidence:** txid, memo bytes, and the verifier's recomputation.

### `memo-content`
**Leaks:** memos are permanently encrypted to a recipient's key, but they are also read by anyone who
gets the viewing key — a support desk, an accountant, a subpoena, a future employee — and Zcash's
memos cannot be rotated or deleted. PII that must not be permanent belongs nowhere near a memo.
**Detect:** extract memo bytes from every payment, decode, and flag: cleartext email/phone/name
patterns, government-id-shaped strings, unstructured free text, and any memo whose plaintext is not
needed to make the payment's claim verifiable.
**Evidence:** the memo bytes, the flagged span, and why the claim can be supported without it.

### `amount-fingerprint`
**Leaks:** Quesnelle (2017) linked 31.5% of coins entering the pool to later exits using amount and
timing alone. Round amounts, echoed amounts (pay exactly what you received), and deposit-shaped
amounts are the same attack.
**Detect:** compare the multiset of amounts across the flow's funding and spending events; flag exact
matches, round numbers, and amounts within a small epsilon of a funding amount, weighted by how few
candidates exist in the anonymity set.
**Evidence:** the two events, the match type, and the candidate count.

### `timing-correlation`
**Leaks:** the interval between funding and spending is a partial identifier when the pool is small or
the amounts are unusual.
**Detect:** for each spend, compute the delay to the nearest preceding inflow to the same flow's
wallets; flag delays inside a fixed window (default: same block or < 10 minutes, configurable), and
report the pool-wide candidate count at that height.
**Evidence:** heights, delays, and candidate counts.

### `same-block-roundtrip`
**Leaks:** funding and spending in one block is a hard link for any observer with block-level data.
**Detect:** height comparison across the flow's txids. Always D-adjacent when combined with
`amount-fingerprint`; reported as W alone only if the amounts differ and the anonymity set is large.
**Evidence:** the two txids and their shared height.

### `server-metadata`
**Leaks:** a payment rail that learns who pays, from where, and for what has rebuilt the card-network
surveillance model on a privacy chain. This is the failure mode of today's agent-payment rails:
public settlement plus PII metadata handed to a facilitator *before* settlement.
**Detect:** the rig runs the merchant/facilitator side of the flow itself and logs exactly what it
receives: headers, IP, user agent, invoice↔payer correlation, timing, any identity field, any
webhook. The flow fails if the merchant side can link two payments from the same payer, or receives
any identity field it does not need to serve the request.
**Evidence:** the recorded requests (redacted), and the field that made the link possible.

## N-class properties

### `report-leak`
**Leaks:** the receipt or report a flow emits to *prove* something (payment made, invoice settled,
totals for an auditor) can leak more than the claim: payee identity, other payments, wallet history,
or a viewing key that unlocks everything.
**Detect:** inspect every artifact the flow exports; assert it is verifiable for its stated claim and
useless for anything else (no IVK/OVK, no address list beyond the claim, no amounts not required).
**Evidence:** the artifact, the claim it supports, and the excess it carries.

---

## What the rig deliberately does *not* test

- **Network-layer deanonymisation** (IP exposure to the indexer, birthday inference from scan ranges).
  Real, documented, and being addressed by other projects in the ecosystem; out of scope here.
- **Protocol-level soundness** (pool turnstiles, counterfeiting, consensus bugs). Not a test suite's job.
- **Anything requiring mainnet funds.** The rig's chain is local and disposable by design; the
  properties above are about the flow's *structure*, and structure is chain-independent.

## Report shape

Each run emits `report.json`: flow id, chain + height, per-property `pass`/`fail`/`skip` with severity,
and an `evidence` array (txids, receivers, memos as hex + decoded form, heights). The report is then
hashed, and the root is anchored in a shielded memo to a beacon address, so the evidence is
timestamped publicly without revealing who ran it. Claims built from a report carry a TTL: re-proved
or expired (§ the index), never "true forever".
