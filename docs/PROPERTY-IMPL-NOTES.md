# CANARY property implementation notes (`canary-properties`)

Implementation of the 12 leak classes from `docs/PROPERTIES.md` as pure functions over
`canary_observation::FlowRun`. No network, no filesystem, no clock, no randomness — every verdict
is derived from recorded observations only, and labels are never trusted.

- Code: `crates/properties/src/properties/` — one module per property plus `common.rs`
  (address classification, memo decoding, pattern scanners) and `fixtures.rs` + `tests.rs` (proof suite).
- Registry: `properties::registry()` fills `all_properties()` in report order
  (D, D, D, D, D, W, W, W, W, W, W, N). `evaluate()` re-sorts by severity → verdict → id.
- Run the proof: `cargo test -p canary-properties` — 33 tests, all green.

## One-line detection rules

| id | severity | detection rule |
|---|---|---|
| `transparent-fallback` | D | Fail if any transparent address (prefix `t1/t2/t3/tm`, len 26–64, alnum) appears as a payment recipient, a payment sender receiver, a funding receiver, a value in `Payment.observed`, or a payment `kind == Transparent`. |
| `unshielding` | D | Fail if (a) any payment `pool_out == Transparent`, (b) between consecutive `pool_totals` snapshots shielded (sapling+orchard) decreases while transparent increases, or (c) a payment spending `pool_in == Transparent` follows an earlier payment that ended transparent/`CrossPool` (a route through transparent). |
| `receiver-reuse` | D | Fail if any recipient receiver string appears on ≥2 payments with distinct labels (via `FlowRun::receiver_reuse_groups`). Exact-string match; diversified-UA family reuse is a documented gap. |
| `payer-linkability` | D | Score every payment pair on five weighted signals; union pairs with confidence ≥ 0.70 (documented constant); fail if any cluster links ≥2 payments. Signals: shared sender receiver 0.40, sender receiver traces to funding 0.35, exact amount 0.25, memo structure 0.20, same height 0.15, adjacent height 0.10. |
| `policy-downgrade` | D | Fail if any recorded `Payment.policy` token normalizes to a known downgrade fragment (`AllowRevealedAmounts`, `AllowRevealedSenders`, `AllowRevealedRecipients`, `AllowLinkingAccountAddresses`, `AllowFullyTransparent`, equivalents), or if a shielded→transparent/`CrossPool` spend reveals amounts implicitly. |
| `memo-absent` | W | For every payment with an `invoice_id`, fail if no memo bytes exist, or if the invoice id is not contained in the memo plaintext (normalized lowercased-alphanumerics containment, either direction). |
| `memo-content` | W | Fail if any memo plaintext contains an email-shaped token, phone/gov-id-shaped run (`###-##-####`, ≥9 digits with separators, ≥10 unseparated), a ≥10-digit run, a key-like label (`email:`, `ssn`, `passport`, …), or ≥4 purely alphabetic words (free text). |
| `amount-fingerprint` | W | Fail if a payment amount is round (divisible by 0.1 ZEC = 10,000,000 zat), exactly repeats another payment amount, exactly echoes a funding amount, or sits within 5,000 zat of a funding amount with exactly one funding candidate. |
| `timing-correlation` | W | For each spend, fail if the nearest strictly-earlier funding height is within 1 block (window constant); delay 0 is deliberately excluded (owned by `same-block-roundtrip`). Evidence includes the within-record candidate count at the spend height. |
| `same-block-roundtrip` | W | Fail if any payment height equals any funding height; evidence states whether the amounts also match (D-adjacent with `amount-fingerprint`). |
| `server-metadata` | W | Fail if a merchant-side request carries an identity-shaped field/header key (token match: email, name, user, customer, payer, wallet, address, kyc, …; `user-agent` exempt), an identity-shaped value under a non-allow-listed key, or linkage across ≥2 distinct payments (same `remote_ip`, or same identity-shaped key/value pair). |
| `report-leak` | N | Fail if an exported artifact has any `discloses` entries, contains key-material tokens (IVK/OVK/UFVK/FVK/viewing key/spending key/seed/mnemonic), an address-shaped token not recorded anywhere in the run, or an email-shaped value. |

## Constants (documented defaults, all `pub`)

| constant | default | property |
|---|---|---|
| `PAYER_LINKABILITY_THRESHOLD` (`payer_linkability::CONFIDENCE_THRESHOLD`) | 0.70 | payer-linkability |
| `TIMING_WINDOW_BLOCKS` | 1 (adjacent block) | timing-correlation |
| `AMOUNT_EPSILON_ZAT` | 5,000 zat | amount-fingerprint |
| `ROUND_GRANULARITY_ZAT` | 10,000,000 zat (0.1 ZEC) | amount-fingerprint |
| `memo_content::FREE_TEXT_WORD_THRESHOLD` | 4 words | memo-content |
| `memo_content::LONG_DIGIT_RUN_LEN` | 10 digits | memo-content |
| weight points (`W_*_POINTS`) | 40/35/25/20/15/10 | payer-linkability |

Every failure `note` restates the constants that produced it. Confidence in `payer-linkability`
is integer points / 100, so threshold comparisons are exact (no float drift).

## Skip conditions (a guessed Pass is a bug)

Every property returns `Verdict::Skip` with a stated reason when the observation set cannot decide:

- **All 12** on an empty run (`empty_run_skips_everything_with_a_reason` test asserts this).
- `transparent-fallback`: zero addresses recorded anywhere.
- `unshielding`: no payments; or unknown/`CrossPool` pool direction on some payment with no
  `pool_totals` snapshots to resolve it.
- `receiver-reuse`, `payer-linkability`: fewer than 2 payments.
- `policy-downgrade`: no `Payment.policy` captured on any payment and no implicit reveal — the
  args were never observed, so "no downgrade" cannot be asserted.
- `memo-absent`: no payment claims an invoice (flow not reconciliation-shaped).
- `memo-content`: no memo bytes on any payment.
- `amount-fingerprint`, `timing-correlation`, `same-block-roundtrip`: no payments, or no
  funding/heights to compare against.
- `timing-correlation` (extra): every qualifying spend is same-block with its only inflow — that
  case is `same-block-roundtrip`'s.
- `server-metadata`: no merchant-side requests recorded.
- `report-leak`: no exported artifacts.

## What the model cannot decide (gaps, and suggestions for the parent)

1. **No timestamps.** `timing-correlation`'s spec says "same block or < 10 minutes"; the model
   records heights only, so the window is expressed in blocks (`TIMING_WINDOW_BLOCKS = 1`) and
   delay 0 is delegated to `same-block-roundtrip`. Additive fix: a timestamp field on `Payment`
   and `Funding` would let the window be minutes.
2. **No chain-wide candidate counts.** The spec asks for "the pool-wide candidate count at that
   height"; the rig records its own payments/fundings only, so the evidence reports the
   within-record count and says so. Additive fix: per-height tx/value counts in `PoolTotals`.
3. **No payer identity on payments.** `payer-linkability` cannot know who paid; it clusters by
   pairwise signals and reports confidence. The spec's "control" (same analysis on a
   deliberately clean flow) is not expressible inside a pure function over one run — the unit
   tests use the clean fixture as the control instead.
4. **Change outputs are not recorded.** "Common change behaviour" is approximated by shared
   sender receivers and by sender receivers tracing to a funding receiver. Additive fix: change
   output receivers on `Payment`.
5. **Diversified unified addresses are opaque strings.** Same-account diversified receivers share
   no decodable prefix without parsing bech32m payloads, so `receiver-reuse` detects exact-string
   repeats only (the spec's "repeats within one diversified-UA family" clause is only partially
   covered). Additive fix: an optional `ua_account` tag on receivers.
6. **Funding-leg ambiguity.** `Funding.receiver` is "the address value entered the flow's wallet
   at". On regtest the harness may fund via a transparent coinbase, and
   `transparent-fallback` will (correctly per spec: "accepted ... anywhere in the flow") fire on
   that leg. If the parent wants the property to isolate the *flow's own* choices, record the
   shielded wallet receiver (post-shield) in `Funding.receiver`, or add an additive
   `Funding.via` tag for harness-side legs; the failure note already explains this.
7. **Memo names.** `memo-content` flags emails/phones/ids/labels/prose but has no name
   dictionary/NER; a bare personal name is not detected. Documented rather than guessed.
8. **Receipt bodies are free text.** `report-leak` pattern-matches key material, addresses and
   PII; it cannot "verify the claim" of an arbitrary artifact body — it flags excess disclosure
   only, and says so in the note.
9. **Policy capture.** If `Payment.policy` is `None` for every payment, `policy-downgrade`
   skips (the args were never observed). Recording the verbatim default (e.g. `"FullPrivacy"`)
   on every payment makes the property decidable and passing.
10. **Transparent-address detection is prefix+shape.** Address strings are untagged in the
    model; classification uses Zcash prefixes (`t1/t2/t3/tm` transparent; `zs/zc/u1/utest/uregtest`
    shielded) plus alphanumeric/length checks. Base64 blobs inside receipt bodies starting with a
    shielded prefix are a theoretical false-positive source for `report-leak`'s address scan.

## Proof suite (crates/properties/src/properties/tests.rs)

- `leaky_fixture_fails_every_property` — all 12 fail (every D and W required; N included),
  every failure carries ≥1 evidence string.
- `clean_fixture_passes_every_property` — all 12 pass (stronger than the required "every D passes").
- 12 `flip_*` tests — one observation changed in the clean fixture, and *exactly* the target
  property fails (`assert_only_fails`).
- Honesty tests: `empty_run_skips_everything_with_a_reason`, `narration_is_not_trusted`
  (clean-sounding labels on leaky bytes still fail; scary labels on clean bytes still pass).
- Boundary tests: policy negation (`NoAllowRevealedAmounts` is not a downgrade), timing window
  boundary (delay 2 passes), amount epsilon boundary (diff == epsilon fails, epsilon+1 passes),
  memo binding must contain the invoice, IP linkage across two payments, key material in a
  receipt body, single-payment skips, unknown pool direction skips.
- Fixture txids are deterministic (`0x11` → `"1111…"`) so every evidence string is eyeball-checkable.
