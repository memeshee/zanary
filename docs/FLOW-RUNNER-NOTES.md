# FLOW-RUNNER-NOTES — `canary-probe flow` (the rig's one command)

What this documents: the piece that turns CANARY from "a wallet that can pay" into "a flow whose
privacy claims can be checked" — one command that executes a payment flow on the throwaway chain
and writes the evidence: `run.json` (the observation record), `findings.json` (the property
verdicts), `report.json` (the claim, hashed, with an expiry clock).

```
canary-probe flow --definition flows/checkout-private.json [--out data/runs/<name>]
  → run.json      canary_observation::FlowRun — funding, payments, merchant events, receipts, pool totals
  → findings.json canary_properties::evaluate(&run)
  → report.json   canary_report::build_with_claim(&run, findings, verified_at, ttl)
  → exit 0 when every property passes, 1 when any FAILs (a control run is a failing run by design)
```

## What the runner does, per invoice

1. **The merchant issues the invoice** — a *fresh diversified Orchard-only receiver* from the
   merchant wallet (`get_next_available_address(account, UnifiedAddressRequest::ORCHARD)`), plus a
   ZIP-302 memo rendered from the definition's template.
2. **The payer pays it** — `ops::propose_shielded_payment` + `ops::build_sign_broadcast`, i.e. the
   same code path `canary-probe pay` uses (extracted, not duplicated): Orchard inputs only, ZIP-317
   fees, Orchard change, our own proving and signing, broadcast through the harness indexer.
3. **The merchant verifies it independently** — the merchant wallet fetches the transaction from the
   indexer, parses it, and trial-decrypts it with *its own* UFVK. Different seed, different database,
   its own sync: this is the third-party read the Phase-1 gate asks for, and it is recorded per
   payment as `observed.merchant_verified`.
4. **The rig records** the payment (txid, the chain's height for it, memo bytes + text, receiver,
   pool in/out, the policy verbatim, the fee), a `MerchantEvent` (what the merchant side received),
   a `Receipt` (whose claims state exactly what its body carries), and pool totals before/after.

The **merchant is the BIP-39 test vector 2 seed** (`core::MERCHANT_MNEMONIC`) in
`data/wallet-merchant/canary-merchant.db` — a genuinely separate party, not a second account of the
payer's wallet. `CANARY_MERCHANT_DATA_DIR` moves it.

## Definitions

| file | what it is |
|---|---|
| `flows/checkout-private.json` | 5 invoices, a fresh receiver per invoice, a memo per payment, `FullPrivacy`, distinct non-round amounts |
| `flows/checkout-control.json` | 5 invoices, ONE reused receiver, no memo, `AllowRevealedAmounts`, identical round amounts — the insecure variant, one definition apart |
| `flows/checkout-private-jitter.json` | the private flow with `jitter_blocks: [1, 8]` and a seed: settlement timing drawn instead of fixed |
| `flows/checkout-private-batch.json` | the private flow with `settlement: "batch"` and a `batch_seed`: all five invoices in one transaction |

```json
{
  "flow_id": "canary-checkout",
  "receiver_strategy": "fresh",          // "fresh" | "reused" (control)
  "memo": true,                          // false = control
  "policy": "FullPrivacy",               // recorded verbatim, enforced by our own code
  "funding_zat": 50000000,
  "payer_account": 0,                    // omit for 0; two definitions with different values are two payers
  "settlement": "per_invoice",           // "batch" = one transaction carries every invoice
  "batch_seed": 20261012,                // seeds the batch's output order; omit for a random draw
  "invoices": [
    { "invoice_id": "4471", "amount_zat": 1234567, "memo_template": "canary: invoice {invoice_id}" }
  ]
}
```

The definition is hashed into `flow_commit` (`sha256` over its canonical JSON), so a claim is about
an exact revision of a flow, not a vibe. Fields that default are omitted from the canonical form, so
adding one does not silently change existing commits.

## Results on the harness

### Private flow — `12 pass · 0 fail · 0 skip`

```
  D pass transparent-fallback — no transparent address anywhere in the flow
  D pass unshielding — no value leaves the shielded pool
  D pass receiver-reuse — every payment went to a fresh receiver
  D pass payer-linkability — repeat payments by the same payer do not cluster
  D pass policy-downgrade — the flow never relaxes the wallet's privacy policy
  W pass amount-fingerprint — amounts do not fingerprint deposits or repeat
  W pass memo-absent — invoice-bearing payments bind their invoice in a memo
  W pass memo-content — memos carry no PII or free text
  W pass same-block-roundtrip — funding and spending do not share a block
  W pass server-metadata — merchant side receives no payer identity or linkage
  W pass timing-correlation — spends are not tightly correlated with their funding
  N pass report-leak — exported artifacts reveal no more than their claim
```

Every payment carried a distinct receiver, the merchant's own wallet decrypted every memo, and the
standalone verifier re-derived every claim from the chain (`21 pass · 0 fail · 15 skip → VERIFIED`;
report hash `01aa5208d3d2f4ddf752805bdd36d39ba792d5e46d59d97873fbfcc51d35e8b5`):

| invoice | txid | height | receiver (fresh, diversified) | merchant read |
|---|---|---|---|---|
| 4471 | `a486be119de911b7560d214c9e84901251524301fc036054e02fcf704aa01c81` | 423 | `uregtest1…` (diversifier 1) | verified |
| 4472 | `eafa90f5081c671c980114dcc29a8b8fb010522f1063612b8fa789f9f0f23ceb` | 426 | `uregtest1…` (diversifier 2) | verified |
| 4473 | `d91daaf67b1e520e3301a55400d6b9a109c871c0c1ee52ab635c0e5230fbdadc` | 429 | `uregtest1…` (diversifier 3) | verified |
| 4474 | `3ab9bf0f4f873559c992495ecd9effcdb3b7e6c7ba83237a56e318f99551b690` | 432 | `uregtest1…` (diversifier 4) | verified |
| 4475 | `bfe6adb14c82d98d0b3f3246ad4db15c22ad385f1e31177525b02393d1590804` | 435 | `uregtest1…` (diversifier 5) | verified |

The verifier's checks that matter here: each claimed txid exists at the claimed height; each
recipient receiver **decrypts** as an Orchard output of that transaction (so the receiver is real and
was paid, not just asserted); each memo's bytes decrypt from the transaction; and both pool-total
snapshots match `getblock` at those heights. The 15 skips are the merchant-side fields, skipped
because the run states `merchant_is_simulated: true` — the verifier refuses to call a rig-simulated
webhook a chain fact.

### Control flow — `7 pass · 4 fail · 1 skip`, exit 1

Same runner, same five invoices, one definition apart (`flows/checkout-control.json`: one reused
receiver, no memo, `AllowRevealedAmounts`, five identical round 0.25 ZEC amounts):

```
  D FAIL policy-downgrade — no privacy-policy downgrade instructed or implied
  D FAIL receiver-reuse — no receiver reused across distinct payments
  D pass payer-linkability — payments are not clusterable by an observer
  D pass transparent-fallback — no transparent address anywhere in the flow
  D pass unshielding — no value leaves the shielded pool
  W FAIL amount-fingerprint — amounts do not fingerprint deposits or repeat
  W FAIL memo-absent — invoice-bearing payments bind their invoice in a memo
  W skip memo-content — memos carry no PII or free text
  W pass same-block-roundtrip — funding and spending do not share a block
  W pass server-metadata — merchant side receives no payer identity or linkage
  W pass timing-correlation — spends are not tightly correlated with their funding
  N pass report-leak — exported artifacts reveal no more than their claim
```

Report hash `f0bf4917001b86b49086961dab0f76b3c746e26a350a61702d69d78cea38dd0f`; the run exits 1,
because a failing property is a result, not a crash.

What makes this a contrast rather than a demo: the control breaks **only** the things it was built to
break. `payer-linkability`, `transparent-fallback` and `unshielding` still pass, so the rig is not
"failing everything it sees" — it names the four properties that this flow actually violates, and the
private flow with the same runner, same wallet, same chain names none.

| | private (`checkout-private`) | control (`checkout-control`) |
|---|---|---|
| receivers | 5 distinct, fresh per invoice | 1, reused |
| memos | one per payment, verified by the merchant | none |
| policy recorded | `FullPrivacy` | `AllowRevealedAmounts` |
| amounts | distinct, non-round | 5 × 0.25 ZEC |
| verdict | **12 pass · 0 fail** | **7 pass · 4 fail** |
| report | `01aa5208…` → VERIFIED (21/0/15) | `f0bf4917…` |

### Jittered settlement — the height deltas equal the draws

The runner mines a fixed three blocks between payments by default, which makes every payment a
constant distance from the one before it. `flows/checkout-private-jitter.json` asks for jitter
instead (`"jitter_blocks": [1, 8]`, `"jitter_seed": 20261011`), and the run reports what it drew:

```
  settlement        : jittered 1-8 blocks between payments, drawn: [7, 6, 4, 2, 3]
  invoice 6141 · e0f58f374c4107f1ad4bd9b7ad926198f2fb33d7f14d863839d41c3f4e2bf624 · height 539
  invoice 6142 · 21af4c2adb663e95577a119b73af5a42a3044ad2a60ba1728492ee2228f72daf · height 546
  invoice 6143 · f898bc8969f395eff43c461e2496c8372a8419c1bd703304b15ed8e46102145c · height 552
  invoice 6144 · f36b256c2d40d5c6287939bf7e7cdc9d3b3b99bca725dfc2db58bf087ff05956 · height 556
  invoice 6145 · 7af2477eb49843bcfd9b030ebcef00202865bf14d9825ef15dda3916724f26e9 · height 558
```

| | draws | height deltas |
|---|---|---|
| private (`checkout-private`) | fixed 3 | +3, +3, +3, +3 |
| jittered (`checkout-private-jitter`) | 7, 6, 4, 2, 3 | **+7, +6, +4, +2** |

The deltas equal the first four draws exactly — the last draw feeds the closing mine — which is also
the check that the node includes a just-broadcast payment in the *first* block it generates: if it
lagged, the deltas would not track the draws. `12 pass · 0 fail`, report
`49d61e58e938ab999810421a1bb18c2a257e09d230ca965a93d1855233b582a8`.

The seed makes a jittered run reproducible; omit `jitter_seed` and each run draws fresh (the heights
are recorded either way, so a run stays auditable). A range the runner cannot honour (`min < 1` or
`min > max`) is reported and falls back to the fixed three blocks rather than silently doing
something else. Both new fields are skipped when absent, so existing definitions still hash to their
old `flow_commit` — verified: `checkout-private.json` still commits to
`40d5f3a798f1fa1300e528ea27d911bc9b423c9c41d676a00219595baf3d0773`, the value recorded in the
run that predates the change.

**The trap that hid this:** the first jittered run reported uniform +3 deltas because the command
that launched it was `cargo build … | grep -E "…" && … && canary-probe flow …`. `grep` exits 0 even
when the build fails, so the run executed the **previous binary** — which mines a fixed three
blocks and has no idea jitter exists. The real compile error underneath was `Jitter::next()`
returning `u32` where `mine_and_resync` takes `u64`. Rule: never gate a run on a *piped* build; use
`cargo build … && <run>` so the compiler's exit status is the one that decides.

### Batched settlement — five invoices, one transaction, one fee

`flows/checkout-private-batch.json` is the private flow **one field apart**: same receiver strategy,
same memos, same `FullPrivacy`, same five distinct non-round amounts — plus `"settlement": "batch"`
and a `batch_seed`. The runner then issues every invoice first, settles all five in **one**
transaction whose output order it shuffles, and reads every payment back out of that one
transaction with the merchant's own wallet and keys.

```
  settlement        : batch — ONE transaction carries every invoice (a stated disclosure: it links them)
  issued            : 5 invoice(s), to be settled in one transaction
  batch order       : [2, 4, 1, 0, 3] — invoice ids in the order the request carries them: 7733, 7735, 7732, 7731, 7734
  batch fee         : 0.00030000 ZEC for all 5 payments (one ZIP-317 fee, not 5)
  batch txid        : 2b35669d869880bd666d0aa57ecea90fe9284ae9a678a9fb448fd29fd1826fe9
  merchant read     : 5 output(s) of tx 2b35669d… decrypted with the merchant's own keys
    7731 · 0.01234567 ZEC · orchard output 2 · memo "canary: invoice 7731"
    7732 · 0.02345678 ZEC · orchard output 0 · memo "canary: invoice 7732"
    7733 · 0.03456789 ZEC · orchard output 3 · memo "canary: invoice 7733"
    7734 · 0.04567890 ZEC · orchard output 5 · memo "canary: invoice 7734"
    7735 · 0.05678901 ZEC · orchard output 4 · memo "canary: invoice 7735"
  batch settled     : 5 invoice(s) in one transaction (2b35669d… @ height 616)
```

Five invoices share **one** txid, **one** height and **one** fee:

| | per-invoice (`checkout-private`) | batch (`checkout-private-batch`) |
|---|---|---|
| transactions | 5 | **1** (`2b35669d…`, 21789 bytes) |
| heights | 423, 426, 429, 432, 435 | **616** |
| fees | 5 × 10 000 = 50 000 zat | **30 000 zat** (one ZIP-317 fee for a 6-action bundle) |
| pool movement | orchard −50 000 zat over the run | orchard −30 000 zat, recorded once |
| verdict | 12 pass · 0 fail | **11 pass · 1 fail** (the fail is the batch itself) |

Two guard rails, both proven before the code existed: the definition **refuses a repeated amount**
(`settlement 'batch' needs a distinct amount per invoice … (invoice 5512 repeats 25000000
zatoshis)` — the merchant identifies its outputs by amount, so duplicates cannot be told apart in
one transaction), and an unknown mode lists the valid ones. Both exit 1 with nothing built.

The fee is recorded **once**. The transaction has one fee, so `observed.fee_zat` sits on exactly one
payment — the first invoice in the definition's order — with `fee_scope` saying which it is
(`transaction (one ZIP-317 fee for all 5 payments, recorded once)`); every payment in the batch also
carries `transaction_fee_zat`, the same number, for readability. Recording it on all five would
count one fee five times and hide real pool movement from the `unshielding` check, which subtracts
the run's own recorded fees. The pool totals above are the check that the arithmetic stayed honest:
orchard fell by exactly the 30 000 the run recorded.

**What the run records, and what it gives up.** Every payment in a batch carries `batch_id`,
`batch_size`, `batch_order` (its position in the request the payer built) and
`batch_output_index` (the position of its output in the chain's own bundle), and the merchant reads
its payments by amount and then confirms the invoice binding in the memo — a read that matched on
amount but named a different invoice fails the run rather than being recorded as settled. A batch is
public evidence that its outputs were created together, so `payer-linkability` **fails** on this run:
cluster of 5, `confidence=1.00 signals=[shared_transaction(1.00)…]`. That is the correct verdict, and
it is the whole point of running the batch next to the unbatched flows — see the four columns in
`docs/LEAK-MATRIX.md`.

The last piece is only visible in a run: **the request order is not the output order.** The runner
drew `[2, 4, 1, 0, 3]` and asked for invoices 7733, 7735, 7732, 7731, 7734 in that order; the chain
carries them at output indices 3, 4, 0, 2, 5 (output 1 is the payer's change). The wallet's own
builder permutes outputs, so the shuffle is belt-and-braces rather than the only thing standing
between the invoice list and the chain — and both orders are recorded, so a reader can see that
neither the definition's order nor the drawn order is the on-chain order.

**The batch claim is checkable from the chain, without keys.** The report's transaction evidence
carries `batch_size`, and `canary-report verify` counts the transaction's Orchard actions against it:

```
  tx 2b35669d869880bd666d0aa57ecea90fe9284ae9a678a9fb448fd29fd1826fe9@616 PASS
     exists at height 616 · indexer agrees · 21789 bytes · batch: 5 payments in one transaction, 6 orchard action(s) on chain
  summary: 21 pass · 0 fail · 20 skip · 0 warn → VERIFIED
```

Fewer actions than the claimed batch is a hard failure; more than the batch plus one change output
is a warning. The field is skipped when absent, so the reports written before batching existed still
hash and verify to the same numbers (`21 pass · 0 fail · 15 skip → VERIFIED`, unchanged) — the
20 skips here are the five new merchant-side `batch_size` fields, which the verifier refuses to call
chain facts because the merchant side is simulated.

### The agent run — three invoice → pay → verify cycles, unassisted

`scripts/agent-pay-loop.sh` runs the whole loop three times with no step in between: the merchant
issues an invoice, the payer pays it, the merchant's own wallet reads it back.

```
canary-probe invoice --out data/invoices/invoice-8101.json --id 8101 --amount-zec 0.11234567
canary-probe pay --invoice data/invoices/invoice-8101.json --policy AllowRevealedAmounts
canary-probe verify-invoice --invoice data/invoices/invoice-8101.json --txid <txid>
```

`invoice` issues a *fresh diversified Orchard receiver* from the merchant wallet, renders the ZIP-302
memo, and writes both the JSON and the **ZIP-321 URI** (`zcash:uregtest1…?amount=…&memo=…`). `pay
--invoice` consumes that invoice: `Invoice::from_path` parses the URI with the `zip321` crate and
cross-checks it against the file's own `receiver`, `amount_zat` and `memo`, refusing an invoice that
disagrees with itself — so the payer acts on the standard, not on this rig's JSON. The payment then
goes through the same `propose_shielded_payment` / `build_sign_broadcast` path the flow runner uses.
`verify-invoice` is the merchant side: its own wallet syncs, fetches the transaction from the
indexer, trial-decrypts it with its own UFVK, and fails unless the memo names that invoice's id.

```
================ all three cycles settled ================
  invoice 8101 · e73f4926f0a3da8a7be7efe70ee9c4fd2742e57156158b0ed50235e741043864
  invoice 8102 · 4ae3f0be086ae0b486604436dac0e04669c9c5dd59c9fba14cb5d1fddf05c812
  invoice 8103 · cf0789eeb369e47571c3ed67fc919ad85a26f8aa56ade421c3b480d18d0e26bb
```

| invoice | amount | payer-side decrypt | merchant read-back |
|---|---|---|---|
| 8101 | 0.11234567 ZEC | `Outgoing · 0.11234567 ZEC · memo: "canary: invoice 8101"` | `amount 0.11234567 ZEC`, `memo: canary: invoice 8101`, **binds invoice id: yes** |
| 8102 | 0.12234567 ZEC | `Outgoing · 0.12234567 ZEC · memo: "canary: invoice 8102"` | `amount 0.12234567 ZEC`, `memo: canary: invoice 8102`, **binds invoice id: yes** |
| 8103 | 0.13234567 ZEC | `Outgoing · 0.13234567 ZEC · memo: "canary: invoice 8103"` | `amount 0.13234567 ZEC`, `memo: canary: invoice 8103`, **binds invoice id: yes** |

Two parties, two seeds, two databases, no shared state: the payer's wallet never reads the
merchant's, and the merchant's verification trusts nothing but the chain and its own keys.

**Trap: a displayed txid is the reverse of the transaction's bytes.** The first loop run failed at
the verification step with

```
Error: GetTransaction from the indexer
    code: 'Some requested entity was not found', message: "GetTransaction: getrawtransaction
    fdb5885090dadbad20d5a7a4199b2ca5f4d4d6bdd4bde2ed94b3d2e2e7117a27 failed: -5: No such mempool
    or blockchain transaction."
```

— for a payment whose printed txid was `277a11e7e2d2b394ede2bdd4bdd6d4f4a52c9b19a4a7d520addbda905088b5fd`.
That hex is the same bytes reversed: `TxId::from_bytes` takes the *internal* order, while the RPC,
the explorer and every log line print the display order. `verify-invoice` now reverses before
building the `TxId`. The proof it was only that: the same command against the payment that already
existed then returned `amount read 0.11234567 ZEC`, `memo read: canary: invoice 8101`,
`binds invoice id: yes`.

**Trap: sub-zatoshi amounts are refused, and that is the correct behaviour.** The loop's first
version used `0.011234567` — nine decimal places — and stopped on cycle 1 with
`` `0.011234567` has more than 8 decimal places (1 zatoshi = 0.00000001 ZEC) ``. A money parser that
silently rounds is a bug generator; this one refuses.

**Rule (hit twice in this work): never read success off a pipeline's last stage.** A run launched as
`cargo build … | grep … && canary-probe flow …` used a stale binary for a whole 20-minute run
because `grep` exits 0 when the build fails, and the loop's own wrapper reported exit 0 while the
script had failed, for the same reason. Gate on the compiler's or the script's own status
(`cargo build … && …`, `script; echo $?`), never on a filter downstream of it.

### Receipt signing — a dedicated receipt key, and a wallet attestation that binds it

A receipt used to be an unsigned JSON blob: anyone could write one. Now each receipt is signed with
a **dedicated receipt key** — ZIP-32 account `receipt_account` of the merchant seed, default 1 — so
a receipt never names the key that holds the merchant's money. The receipt carries that key's
32-byte `ak` as its `signer`, and the wallet **attests** it: a second signature, by the wallet's own
spend-authorizing key, over `{format, receipt_key, wallet_ak}`.

The split is the point. A verifier holding the merchant's UFVK can bind receipt key → wallet, and a
receipt published *without* the attestation is bound to no wallet at all — publishing a receipt
must not publish a linkable identity for the wallet behind it.

```
canary-probe flow --definition flows/checkout-private.json  # receipts: signer + signature + attestation
canary-report verify-receipt data/receipts/invoice-4471.json [--network regtest|test|main]
                             [--fvk <the merchant's ufvk>] [--signer <expected hex>]
```

What was actually run, on the receipts of the private flow that followed this change (report
`f23c0cd0…`; merchant wallet key `3bae3a08…e7bfea1b`, receipt key `eea13067…afc89d39`):

| check | command | result |
|---|---|---|
| as issued, signature only | `verify-receipt data/receipts/invoice-4471.json` | `signature : VALID`, `signer : eea13067…afc89d39`, `attested : … attested by wallet key 3bae3a08…`, **exit 0** |
| as issued, `--fvk` merchant | `… --fvk <merchant.ufvk>` | `from --fvk: 3bae3a08…` then `matches — the receipt key is attested by that wallet (binding verified)`, **exit 0** |
| attestation stripped, no `--fvk` | `verify-receipt …-unattested.json` | `signature : VALID`, `binding : none — … bound to no wallet. That is the design, not a defect…`, **exit 0** |
| attestation stripped, `--fvk` merchant | `… --fvk <merchant.ufvk>` | `FAIL — nothing binds this receipt to that wallet: … carries no attestation`, **exit 1** |
| tampered body (amount +1) | `verify-receipt …-tampered.json --fvk …` | `signature : INVALID — … Invalid signature.`, **exit 1** |
| body missing `txid` | `verify-receipt bad-no-txid.json` | `signature : INVALID` **and** `claim : INVALID — the receipt body has no \`txid\``, **exit 1** |
| body field the format does not define | `verify-receipt bad-extra-field.json` | `claim : INVALID — the receipt body carries 'payer_note', which canary-receipt-v1 does not define`, **exit 1** |
| claims disagree with the body | `verify-receipt bad-claim-mismatch.json` | `claim : INVALID — the claims never state the amount in zatoshis (999) that its body carries`, **exit 1** |
| wrong wallet's key | `verify-receipt … --fvk <payer.ufvk>` | **exit 1** |
| wrong expected signer | `verify-receipt … --signer 0000…` | `MISMATCH`, **exit 1** |
| **node and indexer dead** | `CANARY_NODE_URL=http://127.0.0.1:1 CANARY_LWD_URL=http://127.0.0.1:1 verify-receipt … --fvk <merchant.ufvk>` | `VALID` … `binding verified`, **exit 0** |

The scheme itself is proven both ways by unit tests (`crates/report/src/receipt.rs`): an attestation
binds the receipt key to the wallet; a receipt with no attestation is unbound; an attestation that
names a different receipt key, was made by a different wallet, or has one flipped signature digit is
refused; and attaching an attestation does **not** move the receipt's own signed bytes — which is
why the field could be added to a format that already has receipts in the wild.

The verifier reports **every** failure it finds rather than stopping at the first: the tampered row
names both the broken signature and the claim that no longer matches the body, which are two
independent reasons to reject the same artifact.

The **format** is `schemas/canary-receipt-v1.json` (JSON Schema 2020-12), shipped so a third-party
tool can validate a receipt without this code: the body is a closed object (`invoice_id`,
`amount_zat` as an integer, `txid` and `memo_sha256` as lowercase 64-hex, no extra fields), `claims`
must state the body's figures in words, `discloses` must be empty, `signer`/`signature` are the
receipt key's `ak` and its 64-byte RedPallas signature, and the optional `attestation` is itself a
closed object (`format`, `receipt_key`, `wallet_ak`, `signature`). `verify-receipt` implements
exactly those rules — an extra body field is refused, because a field beyond the format is a
disclosure nobody claimed.

The last row is the Phase-2 gate's "independent verifier validates a receipt **with the engine
stopped**": with the node and the indexer both unreachable, the signature still verifies, because
nothing in that path talks to either. The `--fvk` row is the identity binding — and the two rows
above it are why it can stay optional: with a dedicated key the receipt no longer carries a
linkable identity, so the merchant chooses between being bound (publish the attestation) and being
unlinked (withhold it). The payer's UFVK does not bind, which is the negative control.

What is signed: the receipt's own fields in a fixed order (`format`, `body`, `claims`, `discloses`)
as JSON — a projection of the struct, not the whole thing (the signature cannot cover itself). One
byte of the body changes the payload and the check fails, which is the tampered row above.

`report-leak` still passes: `signer` and `signature` are fields *outside* the receipt body, and the
body carries no key material.

Four API dead ends worth not re-discovering (all resolved by reading the vendored sources):

1. **`ak` comes from the FVK's published encoding.** Orchard's `SpendValidatingKey::to_bytes` is
   `pub(crate)`, and the `From<&SpendValidatingKey> for pallas::Point` impl is not reachable from
   outside the crate. `FullViewingKey::to_bytes()` *is* public and its first 32 bytes are `ak`
   (`ak || nk || rivk`, Zcash Protocol Spec §5.6.4.4) — that is what `ak_of_orchard_fvk` reads.
2. **Signatures serialize through orchard's newtype.** `reddsa::Signature` exposes no bytes, but
   `orchard::primitives::redpallas::Signature<T>` implements `From<&Signature<T>> for [u8; 64]`, so
   the wire form is `r || s` in hex, rebuilt with `Signature::from([u8; 64])` to verify.
3. **`randomize` with the identity scalar is the un-randomized pair.** `SpendAuthorizingKey` has no
   accessor for its inner key, so signing uses `randomize(&Scalar::from(0u64))` — and the verifier
   derives the matching key the same way, which is why the two agree.
4. **There is no `Network::Regtest`.** `zcash_protocol::consensus::Network` has only mainnet and
   testnet, and a regtest UFVK (`uviewregtest1…`) is rejected as "for network Regtest but we
   expected Test" — testnet and regtest do *not* share the unified-key HRP. Regtest keys decode with
   `zcash_protocol::local_consensus::LocalNetwork`, which `canary_report::receipt::regtest_params()`
   supplies with the harness's activation heights.

### The standalone verifier is what caught the runner's own bug

`canary-report verify` re-derives every claim from the chain (no wallet, no rig trust). Run against
the first private run's report it failed 5 checks, all the same bug in *my* runner:

```
  tx ac0a9c0752f6c2c53d44e2908dc4b78e29e703e307b0868bc4f1c80945a609c7@379 FAIL  claimed height 379 but the chain has it at 377
  tx 3b31591597740a42fbbebc8bcf4aa7fce4ccd2182b08d8e7fb29fcc818c2bcc3@382 FAIL  claimed height 382 but the chain has it at 380
  summary: 16 pass · 5 fail · 15 skip · 0 warn → NOT VERIFIED
```

The runner had recorded the *tip after mining* as each payment's height instead of the block that
mined it. Fixed by asking the indexer for the transaction's own height (`mined_height_of`), and the
report then verifies: the runner records what the chain says, and the verifier checks it against the
chain — the loop that makes a claim falsifiable.

## Traps hit (and what they turned out to be)

### 1. Three D/W-class findings on a *clean* flow were bugs in the accounting, not the flow

The first private run scored `9 pass · 3 fail`. All three findings were about the rig's own plumbing:

- **`transparent-fallback` (D) FAIL** — "value entered the flow at a transparent address", because the
  runner recorded the rig's own coinbase mining (`Funding.receiver` = the wallet's transparent
  receiver). The property's own note says what to do: *"re-run with the shielded wallet receiver
  recorded in Funding.receiver to isolate the flow's own choices."* Fixed in the runner: the funding
  record now names the flow's Orchard receiver (`pool: orchard`), with the transparent origin in the
  label. The rig's mining is the chain's plumbing, not a flow that exposes a transparent address.
- **`unshielding` (D) FAIL** — "shielded value left the pool between these snapshots": 50,000
  zatoshis, which is exactly the five ZIP-317 fees (5 × 10,000) of the five payments. A shielded
  payment pays its fee out of the shielded pool and the fee lands in the miner's coinbase, so *every*
  Zcash flow shows this movement. Fixed in `canary-properties`: a decrease the run's own recorded
  fees explain (`observed.fee_zat`, which the runner records) is no longer a finding; an unexplained
  decrease still is.
- **`report-leak` (N) FAIL** — "any non-empty `discloses` list is excess by construction": my receipt
  declared `discloses = ["amount_zat", "txid"]`. Fixed in the runner: the receipt's `claims` now state
  exactly what its body carries ("invoice 4471 was paid 1234567 zatoshis in transaction …") and
  `discloses` is empty.

After those three fixes the same flow scored **12 pass · 0 fail** — and the two failures that were
real (the payment heights) were caught by the verifier, not by the properties.

### 2. A verified report needs the chain's height, not the wallet's tip

Covered above: `mined_height_of` (indexer `GetTransaction` → `height`) replaced the tip read. `0` from
the indexer means mempool, so a payment that has not been mined fails the run loudly rather than
recording a wrong height.

### 3. Two connections to the same wallet file

`ops::fund` / `ops::shield` open the wallet themselves (they are the CLI paths a human runs). The
runner closes its handle before calling them and re-bootstraps afterwards, so two SQLite connections
never contend for the same file. (The runs recorded here never needed funding — the payer already
held shielded funds — but the path exists and is used whenever the invoices exceed the payer's
spendable Orchard value.)

### 4. Cost per run, so nobody is surprised

A five-invoice private run takes **~10 minutes** wall clock in a debug build: five Orchard proofs
(proving dominates), 3 blocks mined per payment plus 11 at the end, and a wallet sync per step. The
report is deterministic given the chain; the wall clock is not.

## Reproduce

```sh
cd /home/kiter/hackathon/canary
cargo build --bin canary-probe
export CANARY_LWD_URL=http://127.0.0.1:9067

./target/debug/canary-probe flow --definition flows/checkout-private.json
./target/debug/canary-probe flow --definition flows/checkout-control.json   # the insecure contrast
./target/debug/canary-probe flow --definition flows/checkout-private-batch.json  # one tx for all five

./target/debug/canary-report verify data/runs/checkout-private/report.json --node http://127.0.0.1:18232
./target/debug/canary-report verify data/runs/checkout-private-batch/report.json --node http://127.0.0.1:18232
./target/debug/canary-report matrix --runs data/runs --out docs/LEAK-MATRIX.md

# anchoring: a report becomes a claim with a chain timestamp (docs/ANCHOR-NOTES.md)
./target/debug/canary-probe anchor --report data/runs/checkout-private-batch/report.json
UFVK=$(python3 -c "import json;print(json.load(open('data/runs/checkout-private-batch/anchor.json'))['beacon_ufvk'])")
./target/debug/canary-report verify-anchor --report data/runs/checkout-private-batch/report.json \
    --beacon-ufvk "$UFVK" --node http://127.0.0.1:18232
./target/debug/canary-report verify-receipt data/receipts/invoice-4471.json \
    --fvk "$(cat data/receipts/merchant.ufvk)"      # offline: no node, no indexer, no rig
./target/debug/canary-report index --reports data/runs --out data/runs/index.html --now 2026-10-25T00:00:00Z
```

Receipts are extracted from a run (`run.json` → `receipts[i]`) into `data/receipts/`; the merchant
and payer UFVKs used for the binding checks come from the wallets' own `accounts` table.

## Known gaps

- One payer per run (`payer_account`); the payer-linkability property's "two payers" setup is two
  runs with different `payer_account` values, not one run.
- Receipts are signed and check offline (`verify-receipt`, exit 0 / 2 for a bad signature / 1 for a
  wrong signer), but the *claims* they carry are still free text: a machine-readable claim schema,
  and the `report-leak` check applied to third-party receipt formats, is later work.
- Receipt signing publishes the merchant's `ak` (32 bytes). That is the spend-authorizing public key,
  not a viewing key — it reveals no note contents or history — but it is a linkable identity: a
  receipt therefore says "the holder of this spend authority issued it". A merchant that wants to
  keep its wallet identity unlinked needs a dedicated receipt key, which is a later increment.
- The runner settles invoices one transaction at a time by default and can put all of them in one
  transaction (`settlement: "batch"`); what batching costs is stated, measured and shown in the
  matrix — `payer-linkability` fails on a batched run because the batch *is* the link. What batching
  does not do yet: split a batch across more than one transaction, or batch across several payers.
- The merchant side is the rig's own wallet and the run says so (`merchant_is_simulated: true`); the
  verifier skips merchant-side fields for exactly that reason rather than pretending they are chain
  facts.
