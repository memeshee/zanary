# ZANARY threat model

What this rig claims, what it does not, and every place it deliberately discloses something. Written
before the features that need the decisions (batching), not after — a trade-off discovered late is a
trade-off nobody agreed to.

## The claim

ZANARY measures **what a payment flow reveals to the parties that can see it** — the chain, the
merchant, and whatever the flow hands to a third party. For a given flow definition it produces a
run, a set of property verdicts, and a claim with an expiry. It does not claim that a flow is private
in any absolute sense, and it cannot see leaks that are not in its twelve properties.

## Who sees what

| party | sees | does not see |
|---|---|---|
| **the chain (anyone)** | transaction structure, heights, amounts in the clear pools, shielded value balances, which pools moved, memo *length*, the fact that a shielded payment happened at all | note contents, the payer, the amount of a fully shielded payment, the memo text |
| **the merchant** | that it was paid, the amount, the ZIP-302 memo, the payer's transaction id, the receiver it was paid to | who the payer is, the payer's other payments, the payer's balance |
| **the payer** | its own notes, its own history | the merchant's other customers, the merchant's balance |
| **the rig** (this software, on this harness) | everything it creates: both wallets' seeds, both databases, the definitions | nothing — it *is* the trust boundary, which is why its claims are checked against the chain by a separate verifier |

## Disclosures each feature makes, and why

**ZIP-302 memos.** The memo travels encrypted to the recipient, but the *chain* sees its length, and a
memo is the one field a payer writes that the merchant reads as text. So `memo-content` refuses
anything that looks like PII or free text: an invoice id is a binding, not a conversation. The memo
is also what makes settlement checkable by the merchant without a shared server, which is the point.

**Signed receipts.** A receipt carries the invoice id, the amount, the transaction id, the SHA-256 of
the memo, and the **receipt key's** Orchard spend-authorizing public key (`ak`) as its signer. The
receipt key is a *dedicated* key — ZIP-32 account `receipt_account` of the merchant seed, default
account 1 — so a receipt never names the key that holds the merchant's money. The receipt can carry
the wallet's **attestation**: a statement, signed with the wallet's own spend-authorizing key, that
this receipt key is its own. A verifier holding the merchant's UFVK checks both signatures and gets
the binding; a receipt published *without* the attestation is bound to no wallet at all, which is
the point — publishing a receipt must not publish a linkable identity for the wallet behind it. The
`ak` is still not a viewing key: it reveals no note contents or history, and the attestation is
opt-in, so both the linked and the unlinked presentation are the merchant's choice. Signing with
the wallet's own key remains accepted (a receipt signed that way binds itself), and a receipt signed
by the wallet directly is what earlier runs produced.

**Receipt bodies.** A receipt's `body` is a closed object and its `claims` must state the body's
figures in words (`schemas/zanary-receipt-v1.json`); an extra field is refused, because a field
beyond the format is a disclosure nobody claimed. Each figure must appear as a whole token: a claim
that only embeds it inside a longer number (`invoice 91207` for a body carrying `9120`) contradicts
the body it is supposed to restate, and `verify-receipt` refuses it rather than printing `claim ok`.
`report-leak` applies the same rule to every artifact the flow exports.

**Anchored reports.** An anchor publishes three hashes (`evidence_root`, `report_hash`,
`flow_commit`) and the height the report was made at, in a shielded memo to a public beacon whose
*viewing key is published*. What that discloses: that evidence with this root exists, and that
somebody ran this flow revision at that height. What it does not: who ran it (the memo carries no
identity and the payment's spending side is shielded), what the evidence says (hashes, not content),
or anything about the flow's users. The log is permissionless — anyone can pay the beacon and add an
anchor — which cannot forge a timestamp, because the *earliest* anchor of a root is when it first
existed on chain; a verifier therefore names the hit it accepted. Cost: the anchor's own amount
(1 000 zatoshis) and one transaction per anchored report.

**Timing jitter.** Jitter draws 1–8 blocks between payments instead of a fixed 3, so the distance
from funding to spend stops being a constant. It buys nothing against an adversary who sees the
whole chain and correlates *other* signals (the payer's other activity, the merchant's other
customers); it removes one specific, cheap correlation.

**Funding on this harness.** The rig's own mining pays the payer's transparent receiver, then the
payer shields it. That transparent hop is the *harness's* plumbing, not the flow's choice, and the
run records funding at the receiver the flow ends up holding value in (Orchard) with the transparent
origin named in the label. A finding about the rig's coinbase would be a finding about the rig.

**The merchant is simulated.** `merchant_is_simulated: true` is stated in every run, and the
verifier skips merchant-side fields rather than pretending a rig-simulated webhook is a chain fact.
The merchant's *decryption* is real (its own seed, its own database, its own trial decryption), and
that part is chain-checkable.

## Batching — decided before it is built

Paying N invoices in **one** transaction with randomised output order:

- **Buys:** one txid, one set of pool movements and one height instead of N; one ZIP-317 fee instead
  of N; and no positional mapping from output order to invoice.
- **Gives up:** the invoices in a batch are **linked to each other by construction**. A single
  transaction is public evidence that those outputs were created together, so any party who can
  identify one output's recipient learns the others were the same payer's batch. Batching trades
  per-payment unlinkability for within-batch linkability.

So batching is a **cost and throughput feature with a stated disclosure**, not a privacy win for one
payer's invoices. A batched flow records the batch on every payment (`batch_id`, `batch_size`,
`batch_order`, `batch_output_index`), and `payer-linkability` failing on a batched run is the
*correct* verdict — the matrix will show it next to the unbatched runs rather than the runner hiding
it.

**As built, and what the run showed** (`flows/checkout-private-batch.json`, five invoices, one
transaction `2b35669d…` at height 616):

- one txid, one height, **one fee**: 30 000 zatoshis for a six-action Orchard bundle, against 50 000
  for the same five invoices settled one at a time — and the pool totals move by exactly the fee the
  run recorded, because the fee is recorded once rather than once per payment;
- the merchant read all five of its payments out of that one transaction with its own keys, matching
  each by amount and confirming the invoice in the memo: a read that matched on amount but named
  another invoice fails the run;
- `payer-linkability` failed, as this page said it would: a cluster of five at
  `confidence=1.00 signals=[shared_transaction(1.00), memo_structure(0.20), same_height(0.15)]`. The
  other eleven properties passed, so the failure names what batching costs rather than everything;
- the report's transaction evidence carries `batch_size`, and `zanary-report verify` re-checks it
  against the chain — *"batch: 5 payments in one transaction, 6 orchard action(s) on chain"* — so the
  batch claim is falsifiable by a stranger with no keys;
- **the request order is not the output order.** The runner drew `[2, 4, 1, 0, 3]`; the chain carries
  those invoices at output indices 3, 4, 0, 2, 5. Shuffling the request is belt-and-braces: the
  wallet's own builder permutes outputs, and both orders are recorded so the difference is visible
  rather than assumed.

## What this model does not cover

- **Network-level observers.** Timing, IP addresses and transaction relay are out of scope; the
  harness runs everything on localhost.
- **Wallet-fingerprinting.** The Orchard note-selection strategy and the exact fee are observable
  and are not modelled.
- **Compromised endpoints.** If the merchant's or payer's keys leak, nothing here helps.
- **Properties not in the library.** Twelve checks are not a proof. The value of the rig is that its
  claims are falsifiable and re-checkable by a stranger, not that the list is complete.
