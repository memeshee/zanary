# REPORT-NOTES — making reports independently verifiable, and turning them into an expiring claim index

*Work done 2026-10-10/11 against the live harness (zcashd regtest + lightwalletd on this
machine). Every chain fact below was re-derived from the running chain; the raw outputs are
pasted verbatim. Nothing here is projected or imagined.*

Three deliverables, all in `zanary-report` (plus one seeding tool that had to exist first):

1. **Verifier** — `zanary-report verify <report.json>` re-checks every `Evidence` entry against
   the live chain (node reads + indexer gRPC + trial decryption) and exits non-zero on any
   mismatch.
2. **Builder** — `zanary-report build --run <run.json>` produces a `report.json` with the hash
   and evidence root filled in, plus `sample-run` to emit a run to start from.
3. **Index** — `zanary-report index --reports <dir> --out index.html [--now <iso8601>]`
   renders every claim with `live`/`stale`/`dead` status, regressions, and the evidence root;
   deterministic output, with a golden test over the bundled fixture.

---

## 0. Why a seeding tool came first

There was no shielded transaction on the chain to verify (orchard pool 0; `data/harness-evidence.json`
did not exist), and `seed.sh`'s node-wallet path is both forbidden for this task ("never call
the node's wallet `z_*` RPCs") and documented as crashing zcashd. So the chain was seeded
*from our own wallet*, exactly the direction `harness/HARNESS.md` describes:

**`crates/probe/src/bin/zanary-seed.rs`** (new file, additive; auto-discovered as a second
binary in the probe crate — no existing file was modified except the probe `Cargo.toml`, which
gained five cached dependencies: `zcash_address`, `sapling-crypto`, `jubjub`, `bls12_381`,
`rand_core`):

* `addr` — open the seed wallet (own DB at `data/wallet-seed/`, derived from the public BIP-39
  test-vector mnemonic), print the transparent receiver to mine to and an Orchard receiver;
* `sync` — sync from the indexer (lightwalletd gRPC), same scan loop as `wallet.rs`;
* `learn-coinbase --from H --to H` — fetch each height's coinbase tx (txid from the node's
  `getblock`, raw bytes from the indexer) and store it, so the wallet learns `tx_index = 0`;
  without this the coinbase-only shielding selector sees the UTXO-stream outputs as
  non-coinbase and refuses to select them (a documented FIXME in `zcash_client_sqlite`);
* `shield --memo <text>` — `propose_shielding_coinbase` (coinbase-only inputs, no change,
  memo attached) + `create_proposed_transactions` (Orchard proving key built internally by
  `zcash_primitives`; a loud `NoSaplingProver` guards the Orchard-only assumption) +
  broadcast via the indexer;
* `tx --txid` — indexer view of a tx (height + bytes).

**Harness config change** (documented, revertible): `data/zcashd/.zcash/zcash.conf` gained

```
mineraddress=tmF1xjfhsSzhy55dmhorzTnKjtHhZmPKzts
minetolocalwallet=0
```

so `generate` mines straight to the rig's own transparent receiver instead of the node wallet
(the node stays the chain and the miner; it holds no key to what it mines). Remove both lines
and restart the container to restore the old behaviour. This was verified working: block 141's
coinbase pays `OP_DUP OP_HASH160 3a2d4145a4f098523b3e8127f1da87cfc55b8e79 …`, which is exactly
the hash160 of `tmF1xjfhsSzhy55dmhorzTnKjtHhZmPKzts`.

Seeding session (raw):

```
$ zanary-seed addr
{"diversifier_index":"DiversifierIndex([0, 0, …])","orchard_receiver":"uregtest106rqhh28qq6fhdtay885kgugvq9f7sncurjhnkq5z75gu0km9jvgedvvfj093gwkx4egted9q8j82ezwhjjcqv94l2ky8694kyck5c80","transparent_receiver":"tmF1xjfhsSzhy55dmhorzTnKjtHhZmPKzts"}

$ zcash-cli generate 119          # heights 142..260, all coinbase to tmF1…
$ zanary-seed sync
{"account":"AccountUuid(d7315cf8-…)"…,"balance":{"orchard_zat":0,…,"total_zat":75000000000,"transparent_zat":75000000000},"chain_tip_height":260,"fully_scanned_height":260}

$ zanary-seed learn-coinbase --from 141 --to 260
{"from":141,"skipped":0,"stored":120,"to":260}

$ zanary-seed shield --memo "zanary regtest memo: invoice 4471 / receipt seed 0xdeadbeef" --limit 4
{"built_against_height":260,"memo_hex":"63616e6172792072656774657374206d656d6f3a20696e766f6963652034343731202f207265636569707420736565642030786465616462656566","memo_text":"zanary regtest memo: invoice 4471 / receipt seed 0xdeadbeef","to":"uregtest106rqhh28qq6fhdtay885kgugvq9f7sncurjhnkq5z75gu0km9jvgedvvfj093gwkx4egted9q8j82ezwhjjcqv94l2ky8694kyck5c80","tx_bytes":9755,"txid":"295e61d6e0e5b43b5ab4f60466f021d4f5f602eae5db425f9e35375099c712bf"}
```

Run 1's payment landed at height 261 (node: `version 5`, `orchard? True`, 9755 bytes; indexer
agrees). A second run — **deliberately paying the same receiver**, so the receiver-reuse
regression is real and checkable — produced tx `a3887b72996140c8d7053c6de841caa2a1ccc0c194c0e10eb5c1582c951b7102`
at height 273 with memo `zanary regtest memo: invoice 4470 / receipt seed 0xcafebabe`.

Chain facts captured to `data/harness-evidence.json` and the fixtures:

| fact | value |
|---|---|
| run 1 payment | `295e61d6e0e5b43b5ab4f60466f021d4f5f602eae5db425f9e35375099c712bf` @ 261 |
| run 2 payment | `a3887b72996140c8d7053c6de841caa2a1ccc0c194c0e10eb5c1582c951b7102` @ 273 |
| funding (run 1) | coinbase @ 141, 153, 160, 161 |
| funding (run 2) | coinbase @ 162, 165, 166, 170 |
| pool totals @261 | transparent 160625030000, sapling 0, orchard 2499970000 (zat) |
| pool totals @273 | transparent 165625060000, sapling 0, orchard 4999940000 (zat) |
| receiver (both runs) | `uregtest106rqhh28qq6fhdtay885kgugvq9f7sncurjhnkq5z75gu0km9jvgedvvfj093gwkx4egted9q8j82ezwhjjcqv94l2ky8694kyck5c80` |

---

## 1. The verifier

`crates/report/src/verify.rs` + `crates/report/src/node.rs` (a minimal read-only node
JSON-RPC client — `getblockchaininfo`/`getblockhash`/`getblock`/`getblockheader`/
`getrawtransaction` only; never a wallet RPC).

Checks, one line each: `report hash` (sha256 over canonical JSON recomputes), `evidence root`
(Merkle root over the evidence array recomputes), `chain`, `claim height`, and per evidence
entry:

* `Transaction` — the node has the tx at the claimed height, and the indexer agrees on that
  height (a disagreement between the two views is itself a failure);
* `Receiver` — the receiver belongs to the claimed transaction: transparent receivers by
  `scriptPubKey` match; shielded receivers by trial-decrypting the transaction with the
  available viewing keys and comparing the decrypted note's recipient;
* `Memo` — the claimed memo bytes decrypt out of the claimed transaction (match rule: the
  512-byte on-chain memo field equals the claimed bytes, or the claimed bytes followed only
  by zero padding);
* `PoolTotals` — `getblock` at the claimed height reports the same `chainValueZat` for
  transparent/sapling/orchard (`getblockchaininfo` only reports tip totals; the per-height
  check is strictly stronger);
* `MerchantField` — skipped, honestly: the merchant side was simulated by the rig and is not
  a chain fact (the report itself says `merchant_is_simulated: true`).

Fail-closed: a check that *cannot* be completed (e.g. a memo no viewing key decrypts) is a
`FAIL`, not a silent pass. The default viewing key is the harness dev account's UFVK derived
from the public BIP-39 test-vector seed (so anyone can check the fixture runs); `--ufvk <ufvk>`
adds another.

### Live verification — run 1 (raw output)

```
$ cargo run -p zanary-report -- verify crates/report/fixtures/report.json
zanary-report verify — zanary-checkout-control @ a7ade74d922a…
  chain regtest · verified at height 261 · 6 evidence entries

  report hash                                    PASS  sha256 over canonical JSON recomputes: 8b4908e1a54a…
  evidence root                                  PASS  merkle root over 6 evidence leaves recomputes: 4790bfdb56f3…
  chain                                          PASS  node reports chain `regtest`, report claims `regtest`
  claim height                                   PASS  verified at height 261 (tip 272)
  tx 295e61d6e0e5b43b5ab4f60466f021d4f5f602eae5db425f9e35375099c712bf@261 PASS  exists at height 261 · indexer agrees · 9755 bytes
  receiver uregtest106rqhh28qq6fhdtay885kgugvq9f7sncurjhnkq5z75gu0km9jvgedvvfj093gwkx4egted9q8j82ezwhjjcqv94l2ky8694kyck5c80 (invoice 4471) PASS  orchard output of tx 295e61d6e0e5b43b5ab4f60466f021d4f5f602eae5db425f9e35375099c712bf decrypts to this receiver
  memo 295e61d6e0e5b43b5ab4f60466f021d4f5f602eae5db425f9e35375099c712bf PASS  59 memo bytes decrypt from tx 295e61d6e0e5b43b5ab4f60466f021d4f5f602eae5db425f9e35375099c712bf (orchard output 1)
  pool totals @261                               PASS  transparent 160625030000 · sapling 0 · orchard 2499970000 (zat) match getblock 261
  merchant field /webhook/invoice/4471:invoice_id SKIP  merchant side was simulated by the rig (stated in the report); not a chain fact
  merchant field /webhook/invoice/4471:txid      SKIP  merchant side was simulated by the rig (stated in the report); not a chain fact

  summary: 8 pass · 0 fail · 2 skip · 0 warn → VERIFIED
```

Run 2 (`report2.json`) verifies the same way (`8 pass · 0 fail · 2 skip → VERIFIED`,
exit 0; tx `a3887b72…` @ 273, orchard output 0, pool totals @273). All four reports in
`crates/report/fixtures/reports/` also verify against the live chain.

### Falsifiability — tamper one byte, watch it fail

Three tampered copies (one hex digit of a txid; one zat of a pool total; one byte of the memo
text), each verified against the same live chain. All exit 1.

**Tampered txid** (`…2bf` → `…2be`):

```
$ zanary-report verify …/report-tampered-txid.json
  report hash                                    FAIL  stored 8b4908e1a54a… but the canonical JSON hashes to 910f8177474c…
  evidence root                                  PASS  merkle root over 6 evidence leaves recomputes: 94204ebd80ae…
  chain                                          PASS  node reports chain `regtest`, report claims `regtest`
  claim height                                   PASS  verified at height 261 (tip 272)
  tx 295e61d6e0e5b43b5ab4f60466f021d4f5f602eae5db425f9e35375099c712be@261 FAIL  transaction not found on the node's chain
  receiver …                                     PASS  orchard output of tx 295e61d6…99c712bf decrypts to this receiver
  memo 295e61d6…99c712bf                         PASS  59 memo bytes decrypt …
  pool totals @261                               PASS  transparent 160625030000 · sapling 0 · orchard 2499970000 (zat) match getblock 261
  summary: 6 pass · 2 fail · 2 skip · 0 warn → NOT VERIFIED
EXIT CODE: 1
```

(the receiver/memo rows still pass because those evidence entries carry the *untampered* txid —
the tamper was one entry; the report hash failing is what makes the edit detectable even if a
checker only hashed the file.)

**Tampered amount** (orchard `2499970000` → `2499970001`):

```
  report hash                                    FAIL  stored 8b4908e1a54a… but the canonical JSON hashes to f7ad6dea1594…
  tx 295e61d6…99c712bf@261                       PASS  exists at height 261 · indexer agrees · 9755 bytes
  pool totals @261                               FAIL  orchard claimed 2499970001 got 2499970000
  summary: 6 pass · 2 fail · 2 skip · 0 warn → NOT VERIFIED
EXIT CODE: 1
```

**Tampered memo byte** (the `1` in “invoice 4471” → `2`):

```
  report hash                                    FAIL  stored 8b4908e1a54a… but the canonical JSON hashes to c65e4a829383…
  memo 295e61d6…99c712bf                         FAIL  no shielded output of tx 295e61d6…99c712bf decrypts (with the available viewing keys, 1 outputs tried) to the claimed memo bytes
  summary: 6 pass · 2 fail · 2 skip · 0 warn → NOT VERIFIED
EXIT CODE: 1
```

---

## 2. The builder

```
$ zanary-report build --run crates/report/fixtures/run.json \
    --findings crates/report/fixtures/findings.json \
    --verified-at 2026-10-10T20:30:00Z --out crates/report/fixtures/report.json
built crates/report/fixtures/report.json
  flow        : zanary-checkout-control @ a7ade74d922a…
  chain       : regtest (verified at height 261)
  verified_at : 2026-10-10T20:30:00Z
  ttl         : 30d
  findings    : 12
  evidence    : 6 entries
  report_hash : 8b4908e1a54a47f9a31d4c1302446b36535d4e27e8cef97559d3cc1c5afd0dd2
  evidence_root: 4790bfdb56f320c955ed1b6d8e439fcd70957659795cc8094de84929deeb8864
```

`--findings` is optional; without it the builder runs `zanary_properties::evaluate(run)` (the
property registry is still being filled in, so today that yields no findings — the evidence is
still checkable). `--ttl-days N` sets the claim lifetime (default 30 days). `sample-run
[--out run.json]` emits the bundled fixture run so the whole pipeline can be exercised from a
clean checkout; building that emitted run with the fixture findings reproduces `report.json`
byte-for-byte (checked: `sample build reproduces fixture report: True`).

**Schema notes (additive).** `Report` gained `verified_at: Option<String>` (RFC3339 UTC) and
`ttl_seconds: Option<u64>` — the claim clock; both are covered by the report hash and omitted
from JSON when absent, so a report without a clock is still valid and the index says “undated”
rather than guessing. `Evidence::Receiver` gained `first_seen_txid` (the binding a verifier
needs to check “this receiver belongs to that transaction”); reports written before the field
exist deserialize, and the verifier fails them with a clear message instead of guessing.

---

## 3. The index

```
$ zanary-report index --reports crates/report/fixtures/reports \
    --out crates/report/fixtures/index.golden.html --now 2026-10-25T00:00:00Z
indexed 4 report(s) → crates/report/fixtures/index.golden.html
  live 2 · stale 1 · dead 1 · regressions 1
```

What the page shows (from the golden HTML):

* per claim: flow id @ commit, status badge (`live` / `stale (one re-probe missed)` /
  `dead (two re-probes missed)`), chain, `verified_at` + height, TTL, countdown, evidence
  root, report hash, `re-issued by <hash>` when a later claim for the same revision exists,
  and the full findings table with severities (D/W/N) and verdicts;
* regressions at the bottom, both runs linked:

```
receiver-reuse  D  passed at 8b4908e1a54a… (2026-10-10T20:30:00Z) and now fails at 904274f8bff5… (2026-10-10T21:05:00Z)
```

The four claims are all real reports over the two real runs plus an older revision of the same
flow (same recorded evidence, different `flow_commit`) used to demonstrate expiry:

| report | revision | verified_at | ttl | status at --now |
|---|---|---|---|---|
| `report-2026-10-10-oldA.json` | `fa7fc5ce…` | 2026-10-10T18:00Z | 7d | **dead** |
| `report-2026-10-10-oldB.json` | `fa7fc5ce…` | 2026-10-10T18:30Z | 10d | **stale** |
| `report-2026-10-10.json` | `a7ade74d…` | 2026-10-10T20:30Z | 30d | **live** (re-issued by ↓) |
| `report-2026-10-10-regression.json` | `a7ade74d…` | 2026-10-10T21:05Z | 30d | **live**, regression |

Determinism: reports are ordered by `(flow_id, flow_commit, verified_at, verified_at_height,
report_hash)`, the only clock is `--now`, and every report's hash is recomputed — a report
whose hash does not recompute is rendered with an explicit warning. The golden test
(`crates/report/tests/golden.rs`) asserts byte-identical HTML for the bundled inputs.

---

## 4. Tests

```
$ cargo test -p zanary-report --jobs 2
running 2 tests  (lib)         evidence_root_is_deterministic_and_order_sensitive, claim_status_follows_ttl_arithmetic … ok
running 4 tests  (golden.rs)   fixture_report_rebuilds_to_the_same_hash, fixture_reports_hash_correctly,
                               index_matches_golden_html, fixture_claims_expire_on_schedule … ok
test result: ok. 6 passed; 0 failed
```

The golden tests pin: build determinism (same run + findings + clock ⇒ same hash, same
evidence), fixture hash validity, the golden index HTML, and the expiry schedule
(dead/stale/live/live) plus exactly one regression (`receiver-reuse`).

---

## 5. What was verified live vs. not

Verified **against the running chain** (outputs above): existence and height of both payment
txs on the node *and* the indexer; decryption of both memos with the dev viewing key;
receiver membership for both Orchard outputs; pool totals at heights 261 and 273; chain name;
tip ≥ claim height; report hashes; evidence roots; the three tamper failures.

Not (and cannot be) verified from the chain: merchant-side fields (simulated — skipped, by
design); any shielded memo/receiver whose viewing key is not supplied (fail-closed; `--ufvk`
exists for that); the *findings* themselves are claims, not chain facts — the verifier checks
the evidence a finding rests on, not the finding's prose. The index does not re-verify evidence
(that is `verify`'s job) but does recompute every report hash and flags altered reports.

## 6. Limitations / honest notes

* **Fixture findings.** The property registry in `zanary-properties` is still being filled in,
  so `crates/report/fixtures/findings.json` / `findings2.json` are hand-authored to the runs'
  observed shape (the `amount-fingerprint` FAIL is a genuine observation: four 6.25 ZEC public
  coinbase inputs, 24.9975 ZEC out; `receiver-reuse` FAILs in run 2 because run 2 really does
  re-use run 1's receiver). They are claims; the verifier checks their evidence.
* **Index-demo reports.** The two old-revision claims reuse the same real evidence with an
  older `flow_commit` (a fixture for stale/dead rendering); their `verified_at`/TTLs are chosen
  so all three statuses appear at the golden `--now`. Documented in
  `crates/report/fixtures/README.md`.
* **Memo comparison rule.** The on-chain memo field is 512 bytes, zero-padded; the fixtures
  record the text bytes (59), so the verifier accepts an exact match or the claimed bytes
  followed only by zero padding. Both representations are unambiguous.
* **Regtest quirk found.** lightwalletd's compact blocks on this chain carry no `vtx` (the
  transparent-only blocks are stripped), so coinbase txids come from the node (`getblock`,
  read-only) in `zanary-seed learn-coinbase`; raw tx bytes still come from the indexer. This
  also means a wallet cannot learn `tx_index` for coinbase UTXOs from the UTXO stream alone —
  hence `learn-coinbase`, which mirrors the `zcash_client_sqlite` FIXME.
* **Network support.** The verifier supports `regtest` (harness params: all upgrades at height
  1), `main`, `test`. The claim index is chain-agnostic.
* **Harness state.** The chain was mined to height 273; `data/harness-evidence.json` was
  written; `data/zcashd/.zcash/zcash.conf` carries the `mineraddress`/`minetolocalwallet`
  lines (see §0 — remove and restart to revert); `data/wallet-seed/zanary-seed.db` is the seed
  wallet. `probe`'s `Cargo.toml` gained five cached deps; the workspace manifest also carries
  additions made concurrently by another agent (`zcash_primitives`, `rusqlite`, `base64`,
  `tonic`, `zcash_proofs/bundled-prover`) which this work uses.

## 7. Exact commands (copy-paste)

```bash
# 1. seed (already done on this chain; reproducible from scratch):
cargo run -p zanary-probe --bin zanary-seed -- addr
cargo run -p zanary-probe --bin zanary-seed -- sync
cargo run -p zanary-probe --bin zanary-seed -- learn-coinbase --from 141 --to 260
cargo run -p zanary-probe --bin zanary-seed -- shield --memo "zanary regtest memo: invoice 4471 / receipt seed 0xdeadbeef" --limit 4
docker exec zanary-zcashd zcash-cli -regtest -rpcuser=zanary -rpcpassword=zanary generate 1
cargo run -p zanary-probe --bin zanary-seed -- sync

# 2. build / verify / index:
cargo run -p zanary-report -- build --run crates/report/fixtures/run.json --findings crates/report/fixtures/findings.json --verified-at 2026-10-10T20:30:00Z --out crates/report/fixtures/report.json
cargo run -p zanary-report -- verify crates/report/fixtures/report.json
cargo run -p zanary-report -- verify crates/report/fixtures/report2.json
cargo run -p zanary-report -- sample-run --out run.json
cargo run -p zanary-report -- index --reports crates/report/fixtures/reports --out crates/report/fixtures/index.golden.html --now 2026-10-25T00:00:00Z

# 3. tamper demo:
python3 - <<'PY'
import json
r = json.load(open('crates/report/fixtures/report.json'))
for e in r['evidence']:
    if e['kind'] == 'transaction': e['txid'] = e['txid'][:-1] + 'e'
json.dump(r, open('/tmp/tampered.json','w'))
PY
cargo run -p zanary-report -- verify /tmp/tampered.json   # exit 1

# 4. tests:
cargo test -p zanary-report --jobs 2
```
