# The bounded payer — a budget instead of a key

An autonomous payer is the case the rest of ZANARY exists to test: someone hands a program the ability
to spend, and the only thing standing between it and a stranger is the authority it was given. This is
the reference implementation of that authority — small on purpose, because its job here is to be
*tested*, not sold.

```sh
zanary-probe payer --grant data/grants/payer-pass.json --invoices data/invoices/payer-pass \
    --out data/runs/payer-pass --expect-refused 3 --ttl-days 30
```

## The grant

A grant is authority, not a key. It says what the payer may do, and nothing in it can move money on its
own — no spending key, no seed, no viewing key:

```json
{
  "budget_zat": 12000000,
  "per_payment_cap_zat": 10000000,
  "allowlist": ["uregtest1…", "uregtest1…", "uregtest1…"],
  "expires_at": "2026-11-15T00:00:00Z",
  "revoked": false
}
```

* **budget** — the most the run may spend in total — invoice principal **plus the per-payment fee**
  (`PAYMENT_FEE_ZAT`, the ZIP-317 fee the wallet also pays to move value at all) — checked cumulatively
  in ledger order;
* **per-payment cap** — the most any single invoice may be;
* **allowlist** — the receivers it may pay. An invoice from anyone else is a stranger;
* **expiry** — the instant after which the authority is void, compared as an **instant**: any RFC3339
  encoding (an offset, fractional seconds, `Z`) names the same instant and gets the same verdict, and an
  expiry the payer cannot read voids the grant instead of extending it. `--now` makes this testable
  without waiting for a date;
* **kill switch** — `revoked: true`, or a `--kill-file <path>` that the operator can `touch` to withdraw
  authority while the grant file itself stays immutable evidence.

## What the payer does with an invoice

The decision is one pure function, asked in this order, so the ledger reason always names the *first*
rule that applied:

```text
revoked?                    → refuse (kill switch)
now ≥ expires_at?           → refuse (authority expired, instants compared)
invoice id already decided? → refuse (one invoice is one decision)
invoice id already paid?    → refuse (the ledger at --out is the payer's memory)
receiver not on allowlist?  → refuse (stranger)
amount > per_payment_cap?   → refuse (per-payment cap)
spent + amount + fee > budget? → refuse (budget)
otherwise                   → pay: shielded, ZIP-302 memo from the invoice, fresh invoice receiver
```

Every invoice produces exactly one ledger line — `paid` or `refused` — with the rule in it, the amount,
the receiver, and the txid when there is one. The ledger is **append-only and written as the run
progresses** (refusals as they are decided, a payment the moment it broadcasts), so a crash or an
interrupt cannot leave a payment on chain with no record of it; the same file is read back at startup and
an invoice it already paid is refused, so a re-run cannot pay it twice. It is written to `ledger.jsonl`
and recorded in the run as `payer`, which is where `payer-policy` reads it (see below). A refusal is a
*result*: `--expect-refused N` is how a caller says "this run was supposed to refuse exactly N invoices",
and the exit code says whether that happened.

## How the rig tests it

The payer is a subject, and the rig judges it two ways that do not depend on the payer's own summary:

1. **`payer-policy`** (`docs/PROPERTIES.md`) reads the ledger plus the grant and fails if any *paid*
   decision breaks a rule — over the cap, past the budget, to a stranger, or while revoked — or if an
   invoice that should have been paid was refused because of an internal error rather than a rule. A run
   with no payer skips it: nothing held a grant, so there is no authority to check.
2. **The chain.** The payments the payer did make are ordinary ZANARY evidence: `report.json` carries
   their txids, receivers, memos and heights, and `zanary-report verify` re-derives every line from the
   node. The refusal has no txid — refusing is not a transaction.

Two honest notes about the boundary. The payer shares the rig's wallet code (`ops::propose_shielded_payment`,
`ops::build_sign_broadcast`), which is why its receipts and the rig's evidence agree by construction —
what makes it a subject is that it decides under a grant. And the merchant side is the rig's own
simulation (`merchant_is_simulated: true`), stated in the report rather than implied.

## Recorded runs (2026-10-16, the harness chain)

**The reference run** — `data/runs/payer-pass`, 4 invoices, budget 0.12 ZEC, cap 0.1 ZEC, three receivers
allowlisted, the fourth a stranger:

| invoice | amount | verdict | rule in the ledger |
|---|---|---|---|
| 8201 | 0.0987654 | **paid** | within the cap (10,000,000 zat) and the budget (9,876,540 of 12,000,000 spent) |
| 8202 | 0.0432109 | refused | would take the run to 14,197,630 of a 12,000,000 budget |
| 8203 | 0.5 | refused | asks 50,000,000, above the per-payment cap of 10,000,000 |
| 8204 | 0.02 | refused | not on the allowlist (3 entries) |

Report `aaaedbba00544d83…`, one payment at height 867 (tx `76a84f50d2e903ff…`), properties **9 pass ·
0 fail · 4 skip** with `payer-policy` PASS, `--expect-refused 3` met.

**The kill switch** — `data/runs/payer-killed`, the same invoices and the same grant with
`revoked: true`: all four refused with `the grant was revoked (kill switch)`, `--expect-refused 4` met,
nothing spent.

**A finding on our own first draft** — the first run (`data/runs/payer-ref`) used round invoice amounts
(0.1, 0.04, 0.5, 0.02) and `amount-fingerprint` FAILED: *"payment amount echoes a public funding
amount"*. The property was right and the demo data was wrong — the same failure mode the library exists
to catch in real integrations, and a fair reminder that the first draft of a payment flow usually leaks.
The re-run above uses distinct non-round amounts, as the flow runner's private flow does.

## Limits, stated

* **Allowlist by receiver, not by identity.** A receiver is what a payment can be checked against; a
  merchant that rotates receivers needs its new receiver added (or an allowlist by merchant id, which
  needs a merchant registry this project does not have).
* **No revocation of in-flight payments.** The kill switch stops the next decision, not one already
  broadcast — the chain does not have an undo.
* **One wallet, one profile.** The payer spends from the rig's dev account; a real deployment would keep
  the payer's wallet distinct from anything an operator uses, and the report's evidence would then lead
  back to the payer's own chain footprint rather than a shared one.
