# ZANARY — wallet send path (address / fund / shield / pay)

What this documents: the `zanary-probe` subcommands that let the ZANARY wallet move
value with its **own** librustzcash code on the regtest harness, plus the funding path
that gets coinbase from the node's wallet to the probe's transparent receiver.

Everything shielded here — the shielding transaction, the Orchard payment, the ZIP-302
memo, the decryption that reads the memo back — is built, proved, signed and read by
`crates/probe` itself (`zcash_client_backend` 0.24 / `zcash_client_sqlite` 0.22 /
`orchard` 0.15.5 / `zcash_primitives` 0.30). The zcashd node is used as the chain and
as a transparent-only wallet: `getblockcount`, `getblockhash`, `getblock`,
`getwalletinfo`, `getaddressutxos`, `sendtoaddress`. **No `z_*` RPC is ever called** —
the node's shielded wallet is never touched.

Wallet: `data/wallet/zanary-wallet.db` (ZIP-32 account 0 `zanary-dev`, account 1
`zanary-dev-1`, both derived from the dev test-vector seed). Harness: docker
`zanary-zcashd` (regtest, RPC `127.0.0.1:18232`) + `zanary-lwd` (lightwalletd v0.5.5,
`127.0.0.1:9067`).

## 1. Subcommands

| command | what it does |
| --- | --- |
| `address --transparent` / `address --orchard [--account N]` | derives and prints the account's transparent receiver and its Orchard-only receiver from the imported dev UFVK |
| `fund --zec <amount>` | mines first if the node has no mature coinbase, moves transparent coinbase to the wallet's transparent receiver with the node's `sendtoaddress` (transparent only), mines to confirm, prints the txid |
| `shield [--account N] [--policy <p>]` | spends the wallet's transparent UTXOs into its own Orchard pool — proposal, Orchard proving, signing and broadcast all in our code |
| `pay --to <address> --zec <amount> --memo <text> [--policy <p>]` | sends a shielded Orchard payment with a ZIP-302 memo, prints the txid, the balance delta, and a recipient-side decryption of the mined transaction |

Unchanged by this work: the no-subcommand path (lightd info) and the `wallet`
subcommand (D1 gate) — same content as before, and the send paths' additions (the
spend-detection sweep, the UTXO-refresh skip) print nothing by default.

Source map: `crates/probe/src/core.rs` (network params, wallet open/import, inlined
sync loop, transparent-UTXO refresh, spend detection, tx-index annotation),
`crates/probe/src/ops.rs` (the four operations, `PrivacyPolicy`, `build_sign_broadcast`),
`crates/probe/src/node.rs` (hand-rolled zcashd JSON-RPC over TCP, transparent methods only).

The privacy policy argument is zcashd's ladder, enforced by our own code before anything
is built: `FullPrivacy`, `AllowRevealedAmounts`, `AllowRevealedRecipients`,
`AllowRevealedSenders`, `AllowFullyTransparent`, `NoPrivacy`. `shield` requires
`AllowRevealedSenders`; `pay` requires `AllowRevealedAmounts` for an Orchard-only
recipient and `AllowRevealedRecipients` when the recipient's UA also carries a
transparent receiver. An unsatisfied policy refuses before broadcast, with a non-zero
exit (observed):

```
$ ./target/debug/zanary-probe shield --policy FullPrivacy
Error: privacy policy FullPrivacy is too strict for this shielding transaction: it would reveal the transparent senders and their amounts.
  Re-run with `--policy AllowRevealedSenders` (or a more permissive policy). Nothing was broadcast.
$ echo $?
1

$ ./target/debug/zanary-probe shield --policy Bogus
Error: unknown privacy policy `Bogus` — expected one of FullPrivacy, AllowRevealedAmounts, AllowRevealedRecipients, AllowRevealedSenders, AllowFullyTransparent, NoPrivacy
$ echo $?
1
```

The same shape applies to the send paths' other failures: a rejected broadcast exits 1
with the node's error code (`the indexer rejected the transaction: Missing inputs
(error_code -25)`), and a build failure exits 1 with the builder's error (`could not
propose a shielding transaction: …`).

## 2. Verification run (fresh, contiguous, tip 337 → 370)

Harness state at the start: chain height 337, node wallet 862.99990000 ZEC spendable,
0 immature; probe wallet transparent 596.87935000 ZEC · orchard 483.49575000 ZEC.

### 2.1 `mine` — happens inside `fund`/`shield`/`pay`

Every operation mines its own confirmation blocks (11 blocks, the ZIP-315 default
window), so the txids below are all mined. Raw `getblockcount` before/after each step is
recorded in the outputs.

### 2.2 `fund --zec 20` — txid `878a2c014368fa997eb30245d3a66abe84049d5a238a4961a70cf1486ec455b5`

```
$ ZANARY_LWD_URL=http://127.0.0.1:9067 ./target/debug/zanary-probe fund --zec 20
zanary-probe 0.0.1 fund — ZANARY (ZECATHON, Core & Tooling)
  network           : regtest (throwaway harness chain)
  indexer endpoint  : http://127.0.0.1:9067
  wallet db         : ./data/wallet/zanary-wallet.db
  account           : zanary-dev (AccountUuid(c9ec347b-c703-4e40-a1be-925cd34604ec))
  recipient (t)     : tmF1xjfhsSzhy55dmhorzTnKjtHhZmPKzts
  amount            : 20.00000000 ZEC
  node wallet       : 862.99990000 ZEC spendable · 0.00000000 ZEC immature · chain height 337
  sendtoaddress     : 878a2c014368fa997eb30245d3a66abe84049d5a238a4961a70cf1486ec455b5
  mined             : 11 block(s) to confirm (chain height 348)
  scanned           : 338 ..= 348 (11 blocks, priority Historic)
  wallet view       : transparent 651.25455000 ZEC · orchard 483.49575000 ZEC · total 1134.75030000 ZEC
```

`651.25455000 = 596.87935000 + 20.00000000 + 34.37500000` (11 mined blocks × 3.125 ZEC
subsidy) — the wallet's own view of the new UTXO and of the mining rewards it also
receives, because the node mines to this wallet's address.

### 2.3 `shield` — txid `826ed1383cbbdc7604441a311449838ba85001f98d6fca63fab841acd474e94e`

```
$ ZANARY_LWD_URL=http://127.0.0.1:9067 ./target/debug/zanary-probe shield
zanary-probe 0.0.1 shield — ZANARY (ZECATHON, Core & Tooling)
  network           : regtest (throwaway harness chain)
  indexer endpoint  : http://127.0.0.1:9067
  wallet db         : ./data/wallet/zanary-wallet.db
  account           : zanary-dev (AccountUuid(c9ec347b-c703-4e40-a1be-925cd34604ec))
  policy            : AllowRevealedSenders (shielding reveals the transparent senders and amounts)
  before            : transparent 651.25455000 ZEC · orchard 483.49575000 ZEC · total 1134.75030000 ZEC
  proposal          : 1 step(s) · fee 0.00180000 ZEC · shielding 226.25000000 ZEC
  broadcast         : accepted by the indexer (14178 bytes)
  txid              : 826ed1383cbbdc7604441a311449838ba85001f98d6fca63fab841acd474e94e
  mined             : 11 block(s) (chain height 348 -> 359)
  scanned           : 349 ..= 359 (11 blocks, priority Historic)
  after             : transparent 459.38135000 ZEC · orchard 709.74395000 ZEC · total 1169.12530000 ZEC
  balance delta    : 34.37500000 ZEC
    orchard        : 226.24820000 ZEC
    sapling        : 0.00000000 ZEC
    transparent    : -191.87320000 ZEC
```

Cross-check against the node: `getrawtransaction 826ed138… 1` → **34 transparent inputs,
no transparent outputs** (`vout` empty), `orchard.valueBalance = -226.2482`. The wallet's
own spend records for this transaction break the 34 inputs down as 33 coinbase outputs
(33 × 6.25000000 = 206.25000000 ZEC) plus exactly the fresh `fund`'s output
(`878a2c014368fa997eb30245d3a66abe84049d5a238a4961a70cf1486ec455b5`, 20.00000000 ZEC) —
the funding path's coin arriving in the Orchard pool. So the shielding transaction's
inputs are fully consumed into the Orchard pool; the only value that does not arrive
there is the 0.00180000 ZEC fee, which the block's coinbase pays back to this same wallet
(`226.25000000 - 226.24820000 = 0.00180000`).
`transparent -191.87320000 = -226.25000000 + 34.37500000 + 0.00180000`.

### 2.4 `pay` — txid `ee6d9b0f8e3e46337a2ddfb1efadb283f1699041c4e2920a5cc27965852e2ec2`

Recipient: account 1's Orchard receiver
`uregtest1lqhms8gkj6dcgftfgtugnrj2f8dglkdtr60tsra3x5s9sg4seaafllpwhwmr2mfu78sn7es6uvwnydmcp4h2y5653m27klxfhghjwgcd`
(derived by `address --orchard --account 1`), i.e. a second receiver of the same wallet
identity but a different ZIP-32 account and diversifier.

```
$ ZANARY_LWD_URL=http://127.0.0.1:9067 ./target/debug/zanary-probe pay \
    --to uregtest1lqhms8gkj6dcgftfgtugnrj2f8dglkdtr60tsra3x5s9sg4seaafllpwhwmr2mfu78sn7es6uvwnydmcp4h2y5653m27klxfhghjwgcd \
    --zec 7.25 --memo "ZANARY fresh-run memo: shielded Orchard payment, readable at the recipient" \
    --policy AllowRevealedAmounts
zanary-probe 0.0.1 pay — ZANARY (ZECATHON, Core & Tooling)
  network           : regtest (throwaway harness chain)
  indexer endpoint  : http://127.0.0.1:9067
  wallet db         : ./data/wallet/zanary-wallet.db
  from account      : zanary-dev (AccountUuid(c9ec347b-c703-4e40-a1be-925cd34604ec))
  to                : uregtest1lqhms8gkj6dcgftfgtugnrj2f8dglkdtr60tsra3x5s9sg4seaafllpwhwmr2mfu78sn7es6uvwnydmcp4h2y5653m27klxfhghjwgcd
  recipient         : unified (orchard receivers)
  amount            : 7.25000000 ZEC
  memo              : "ZANARY fresh-run memo: shielded Orchard payment, readable at the recipient" (74 bytes, ZIP-302)
  policy            : AllowRevealedAmounts
  before            : transparent 459.38135000 ZEC · orchard 709.74395000 ZEC · total 1169.12530000 ZEC
  proposal          : 1 step(s) · fee 0.00010000 ZEC
  broadcast         : accepted by the indexer (9165 bytes)
  txid              : ee6d9b0f8e3e46337a2ddfb1efadb283f1699041c4e2920a5cc27965852e2ec2
  mined             : 11 block(s) (chain height 359 -> 370)
  scanned           : 360 ..= 370 (11 blocks, priority Historic)
  after             : transparent 493.75645000 ZEC · orchard 702.49385000 ZEC · total 1196.25030000 ZEC
  balance delta    : 27.12500000 ZEC
    orchard        : -7.25010000 ZEC
    sapling        : 0.00000000 ZEC
    transparent    : 34.37510000 ZEC

  --- recipient-side check (our code decrypts the mined transaction) ---
  orchard output #1 → account AccountUuid(48b70fe7-8e0f-4f7c-a216-99b44807a3bf) · Incoming · 7.25000000 ZEC · memo: "ZANARY fresh-run memo: shielded Orchard payment, readable at the recipient"
  orchard output #0 → account AccountUuid(c9ec347b-c703-4e40-a1be-925cd34604ec) · AccountInternal · 17.74960000 ZEC · memo: "\ufffd"
  orchard output #1 → account AccountUuid(c9ec347b-c703-4e40-a1be-925cd34604ec) · Outgoing · 7.25000000 ZEC · memo: "ZANARY fresh-run memo: shielded Orchard payment, readable at the recipient"
```

**The memo is readable at the recipient.** The last block is our own code
(`decrypt_transaction` + `DecryptedTransaction::orchard_outputs()`) decrypting the mined
transaction: output #1 decrypts for account 1 (`48b70fe7…`, the recipient) as
**Incoming** 7.25000000 ZEC with the exact 74-byte memo, and for the sending account
(`c9ec347b…`) as Outgoing. Output #0 is the 17.74960000 ZEC Orchard change back to the
sender (`AccountInternal`).

`orchard -7.25010000 = -(7.25000000 + 0.00010000 fee)`; `transparent +34.37510000 = 11 ×
3.12500000 + 0.00010000` (the fee returns in the coinbase). The payment spends **no
transparent inputs at all** — `getrawtransaction ee6d9b0f… 1` reports no `vin` and no
`vout`, with `orchard.valueBalance = 0.0001`, i.e. the 0.00010000 ZEC fee is the only
value that leaves the Orchard pool.

The balance chain across the three steps is exact:

```
fund      wallet view : transparent 651.25455000 · orchard 483.49575000 · total 1134.75030000
shield    before      : transparent 651.25455000 · orchard 483.49575000 · total 1134.75030000
shield    after       : transparent 459.38135000 · orchard 709.74395000 · total 1169.12530000
pay       before      : transparent 459.38135000 · orchard 709.74395000 · total 1169.12530000
pay       after       : transparent 493.75645000 · orchard 702.49385000 · total 1196.25030000
```

### 2.5 `address` — raw output

```
$ ./target/debug/zanary-probe address --transparent
  --- receivers ---
  transparent       : tmF1xjfhsSzhy55dmhorzTnKjtHhZmPKzts

$ ./target/debug/zanary-probe address --orchard
  --- receivers ---
  orchard           : uregtest106rqhh28qq6fhdtay885kgugvq9f7sncurjhnkq5z75gu0km9jvgedvvfj093gwkx4egted9q8j82ezwhjjcqv94l2ky8694kyck5c80
  diversifier index : DiversifierIndex([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])

$ ./target/debug/zanary-probe address --orchard --account 1
  orchard           : uregtest1lqhms8gkj6dcgftfgtugnrj2f8dglkdtr60tsra3x5s9sg4seaafllpwhwmr2mfu78sn7es6uvwnydmcp4h2y5653m27klxfhghjwgcd
  diversifier index : DiversifierIndex([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
```

The no-subcommand path still prints the lightd info block and the `wallet` subcommand
still prints the D1 summary (orchard receiver, diversifier index, chain tip, fully
scanned height, balance breakdown, spendable-now) — unchanged by this work.

### 2.6 Earlier evidence, same code path (kept for the record)

Before the fresh run above, the same three commands produced:
`fund --zec 12` → `f77e5f785f4b7c93db5c9d6afed052662d1e90041f15c780e9ba13c4425d769f`,
then `shield` → `dffad397c2122c68ce984c7dde45746ba36553722448e6a05192043cbd1860a7`
(69 transparent inputs, 437.00000000 ZEC total, including that fund's 12.00000000 ZEC
output — verified by reading the wallet DB's spend records), then `pay --zec 3.5` →
`6c0cd2d289134041fbab2c2f3c8f2d928537a6021e760275ba745de172272478` with the memo
"zanary-probe pay: ZIP-302 memo on the regtest harness" read back at the recipient
(`Incoming · 3.50000000 ZEC`). The shield's `after` line is byte-identical to the pay's
`before` line (`transparent 562.50425000 ZEC · orchard 486.99585000 ZEC · total
1049.50010000 ZEC`), which is the balance chain that ties the two runs together.

## 3. Traps hit (symptom → evidence → cause → fix)

### 3.1 `Missing inputs (error_code -25)`: the wallet never learned about a transparent spend it did not initiate

Symptom — the first `shield` built a perfectly good proposal and the node refused it:

```
  before            : transparent 843.25070000 ZEC · orchard 49.99940000 ZEC · total 893.25010000 ZEC
  proposal          : 1 step(s) · fee 0.00185000 ZEC · shielding 224.50000000 ZEC
Error: the indexer rejected the transaction: Missing inputs (error_code -25)
  nothing was mined; the wallet has recorded the transaction as sent
```

Evidence — the proposal's 35 inputs (34 mature coinbase outputs + the 12.0 ZEC funded
output) included 8 outputs that another agent's shielding transaction, built from the
same dev UFVK in the same wallet DB, had already spent on chain (`bf12c7995037359e`,
mined at 261, and `02711b952c58c1b5`, mined at 273). Diffing the wallet DB against the
node settled it: `getaddressutxos` for `tmF1xf…` returned 126 unspent outputs, the
wallet's own unspent set held 134, and the 8 extra ones were exactly the spent ones
(0 outputs in the DB missing from the node's list, 8 extra).

Cause — a wallet only learns that one of its transparent outputs was spent when it sees
the spending transaction. `zcash_client_backend::sync::run` delegates that to its caller
through `TransactionDataRequest::TransactionsInvolvingAddress` (one request per stored
UTXO, served by an address-scoped transaction lookup); the probe's inlined sync loop
never processed the wallet's request queue, so spends made by anything other than this
wallet stayed invisible. The selector then picked already-spent coins and the node's
`sendrawtransaction` answered `-25`.

Fix — `detect_transparent_spends()` in `crates/probe/src/core.rs`, run at the end of
every sync: for each account's transparent receivers, `GetTaddressTransactions(address,
birthday..tip)`; every returned transaction the wallet does not already know is parsed
and passed to `decrypt_and_store_transaction`, which records the spend. After it ran,
the DB showed 8 spend records with `mined_height` set (50.0 ZEC = 8 × 6.25) — the real
spends, permanently marked.

Cost control — the sweep skips any transaction that does not spend an output of a
transaction the wallet already knows (`get_transaction(prevout_txid)`): that skips the
coinbase transactions that pay the wallet and everything that only touches other
people's coins, so a steady-state sweep stores nothing and costs ~1–2 s.

### 3.2 A rejected send marks its own inputs spent — and the wallet keeps them spent until that transaction expires

Symptom — the very next `shield` could not build anything:

```
Error: could not propose a shielding transaction: Change output generation failed:
  Insufficient funds: required 10000 zatoshis, but only 0 zatoshis were available.
```

Evidence — after the rejected broadcast, `create_proposed_transactions` had already
stored the transaction (`b3be795ddafcaf11`, `mined_height = NULL`,
`expiry_height = 314`) and 35 spend records pointing at it: 27 of those outputs were
still unspent on chain (the node listed them) and 8 were the genuinely spent ones from
3.1. Every mature UTXO was therefore excluded from selection, leaving 0 zatoshis.

Cause — the wallet records a send when it is built, not when it is mined, and it counts
those spends against the outputs' spendability while the sending transaction is
unexpired. There is no API to unmark a spend.

Fix / self-heal — the wallet's own expiry semantics are the repair path: both the
balance query and the UTXO selection count a spend only while the spending transaction
is mined or unexpired. Mining past the failed transaction's expiry (314) restored the
27 outputs: at tip 315 the wallet reported `transparent 965.12570000 ZEC · orchard
49.99940000 ZEC · spendable now 49.99940000 ZEC`, and the next `shield` selected
437.00000000 ZEC and broadcast (`dffad397…`). The 8 genuinely spent outputs stayed
excluded because 3.1's sweep had marked them with a **mined** spend — without that, they
would have come back at expiry and broken the next send the same way.

### 3.3 `put_received_transparent_utxo` costs ~350 ms per output in a debug build, and every sync re-reports every UTXO

Symptom — every command paid ~45 s of sync: with `ZANARY_TIMINGS=1` the refresh loop
showed 126 UTXO replies taking 44 s, and the whole bootstrap ~50 s.

Evidence — before the change, `TIMING utxo reply #125: 44.6s` and the whole bootstrap
~50 s. After it, on the same wallet: `TIMING utxo acct …: 121 reply(s), 0 stored,
388.18947ms`, `TIMING utxo refresh: 459.751649ms`, `TIMING spend detection:
1.096665563s` — the whole sync is ~1.1 s when nothing is new (a sync that does discover
new outputs still pays ~350 ms per new output: `168 reply(s), 42 stored, 6.955488808s`
after a 42-block mining burst).

Cause — `put_transparent_output` (zcash_client_sqlite 0.22) ends with
`update_gap_limits`, which regenerates the account's gap window — an address-derivation
loop that costs hundreds of milliseconds per call in an unoptimized build — and
lightwalletd's `GetAddressUtxos` returns the whole window on every sync, so every output
was re-stored every time.

Fix — the refresh asks the wallet first (`get_received_outputs(txid, …)`: does this
output already exist, regardless of spendability?) and only writes genuinely new
outputs. New outputs still pay the ~350 ms, which is fine.

### 3.4 lightwalletd's `GetAddressUtxos` has no `tx_index`, so the wallet's coinbase-maturity exclusion is a no-op

Context — transparent outputs discovered through `GetAddressUtxos` are stored with
`tx_index = NULL`, because the reply has no such field (lightwallet-protocol issue #17;
`zcash_client_sqlite` carries a FIXME on `excluding_immature_coinbase_outputs`). The
wallet's queries read NULL as "not coinbase", so immature coinbase outputs would be
selectable, and the node enforces the 100-block coinbase maturity rule on regtest too —
the probe never observed that rejection because the fix below keeps immature coinbase
out of the selection (verified from the wallet's spend records: the 226.25 ZEC shield
took 33 coinbase outputs worth 206.25 ZEC plus the 20.0 ZEC non-coinbase output, and
left the ~100 immature ones alone).

Fix — `annotate_transparent_tx_indices()` (core.rs) reads each block's transaction order
from the node (`getblockhash` + `getblock`) for transactions the wallet knows but has no
index for, and writes `transactions.tx_index` — the same column the wallet's own scan
path records through `put_tx_meta`. Rows that already have an index are never touched;
the write uses a second connection to the same SQLite file with a busy timeout, because
the wallet API exposes no way to set that column. The indexer's compact blocks cannot be
used for this: they omit transparent-only transactions entirely (observed — only the 2
shielded transactions of the whole chain came through them).

### 3.5 `GetTaddressTransactions` details that cost time to discover

- The request must carry a block range with a start height, or lightwalletd answers
  `InvalidArgument: GetTaddressTransactions: must specify a start block height`; an
  unset end is defaulted to the chain tip and the span is capped at 10,000,000 blocks.
  The probe sends `birthday..tip` (inclusive on both ends).
- In this version of `zcash_client_backend`, `BlockRange` requires `pool_types` to be
  set (`error[E0063]: missing field pool_types in initializer of BlockRange`), even
  though this RPC ignores it — pass `vec![]`.
- Both spellings exist in lightwalletd v0.5.5 (`GetTaddressTxids`, deprecated, and
  `GetTaddressTransactions`); both stream `RawTransaction` (raw bytes + mined height,
  0 = mempool), not txids. The probe uses `get_taddress_transactions`.

### 3.6 `sync::run` (0.24) cannot run against this node

`zcash_client_backend::sync::run` unconditionally fetches Ironwood subtree roots, and
the harness node refuses: `GetSubtreeRoots: z_getsubtreesbyindex, error: -8: Requested
pool must be "sapling" or "orchard"`. The probe inlines the sync loop instead (latest
tip → the wallet's scan queue → `GetBlockRange` into a RAM block cache → `GetTreeState`
below the range → `scan_cached_blocks`), skipping the subtree-root fast-forward, which
is a no-op here because every pool reports zero complete subtrees. (Carried over from
the D1 work; unchanged by this task.)

### 3.7 lightwalletd's streaming `GetAddressUtxos` is pathologically slow here

The streaming variant flushes ~300 ms per message on this harness; the unary
`get_address_utxos` returns the same 126-output set in ~10 ms. The probe uses the unary
call. (Carried over from the D1 work.)

### 3.8 Reading balance deltas: the node mines to this wallet, and the subsidy halves mid-run

Every operation mines its own 11 confirmation blocks (ZIP-315 default window) **to this
wallet's address**, so a delta is not just the transferred amount. The harness chain's
subsidy also changes partway through the work: 6.25 ZEC per block up to about height
282, 3.125 ZEC after that. Example from the fresh `shield`: `balance delta 34.37500000
ZEC` with `transparent -191.87320000` and `orchard +226.24820000` — 226.25 shielded
minus the 0.0018 fee, plus 11 × 3.125 mined, plus the fee returning in the coinbase.
The node's own wallet balance (862.99990000 ZEC spendable before the fresh `fund`) is
separate from the probe wallet's balance: it is the source of `fund` sends, and it is
reported by the `fund` command as `node wallet`.

## 4. Reproduce

```sh
cd /home/kiter/hackathon/zanary
cargo build --bin zanary-probe

export ZANARY_LWD_URL=http://127.0.0.1:9067

./target/debug/zanary-probe                        # lightd info (unchanged)
./target/debug/zanary-probe wallet                 # D1 gate (unchanged)
./target/debug/zanary-probe address --transparent
./target/debug/zanary-probe address --orchard
./target/debug/zanary-probe address --orchard --account 1

./target/debug/zanary-probe fund --zec 20          # node -> probe transparent
./target/debug/zanary-probe shield                 # transparent -> own Orchard
./target/debug/zanary-probe pay --to <address> --zec 7.25 --memo "text" --policy AllowRevealedAmounts

ZANARY_TIMINGS=1 ./target/debug/zanary-probe wallet   # per-phase timings on stderr/stdout
```

`ZANARY_TIMINGS=1` also re-enables the informational lines that are silent by default
(the UTXO-refresh counters and the tx-index annotation count), so the content of the
`wallet` and no-subcommand output is what it was before this work. Other knobs:
`ZANARY_DATA_DIR` (default `./data/wallet`), `ZANARY_DEBUG_TX_INDEX=1`, `RUST_LOG`
(default `info`; the library's own `INFO … Storing decrypted transaction` lines come
from the logger that was already in `main.rs`, and the spend-detection sweep can emit
them when it finds a spend — `RUST_LOG=warn` silences that stream).

## 5. Known gaps

- The wallet's transaction-data-request queue is still not processed: 3.1's sweep covers
  the address-scoped spend detection (`TransactionsInvolvingAddress`), and `GetStatus` /
  `Enhancement` requests for the probe's own sent transactions are not answered. In
  practice the scan plus the expiry semantics cover the cases that arise here — a sent
  transaction that never gets mined stays unmined until its expiry and then stops
  counting — but a long-lived wallet would want the real request loop.
- The sweep is O(receivers) per sync: ~20 addresses, one `GetTaddressTransactions` call
  each, ~1–2 s even when nothing is new.
- 3.3's skip-if-known check asks the wallet per output (one SELECT per UTXO reply), so a
  large wallet pays ~12 ms per reply in the refresh.
- Shielded value is Orchard-only; the wallet tracks Sapling but nothing uses it. The
  `shield`/`pay` builders still take Sapling provers by type (`zcash_proofs`
  `bundled-prover`, so no `~/.zcash-params` is needed).
- `pay`'s recipient-side check can read the memo because the recipient (account 1) is
  inside this wallet's UFVK set; a recipient in a different wallet would need its own
  wallet to read the memo.
- `fund` needs the node's wallet to have spendable transparent funds; the command
  reports both numbers (`node wallet : N spendable · M immature · chain height H`) and,
  when no mature coinbase covers the transfer, mines 110 blocks to mature some first
  (that branch is implemented and prints
  `no mature coinbase covers the transfer — mining 110 blocks to mature coinbase...`,
  but the runs recorded here did not exercise it: the node always had mature coinbase).

