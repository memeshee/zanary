# ZANARY leak matrix

Every cell is one property of the leak library applied to one run of one flow definition.
`pass` means the property held; **FAIL** means it did not; `skip` means the run did not
exercise it. Each column names the `flow_commit` it is about, so a claim is pinned to an
exact revision of a definition, and the re-check line under the table re-derives that run's
report from the chain.

| property | canary-checkout `40d5f3a7` | canary-checkout-batch `f2f15593` | canary-checkout-control `9e42ff0e` | canary-checkout-jittered `e3b7fb25` | zanary-ci-checkout `4d8279bb` | zanary-ci-checkout-control `05cc6aeb` | zanary-payer/3178c172 `bf1029fb` |
|---|---|---|---|---|---|---|---|
| `payer-linkability` (disqualifying) | pass | **FAIL** | pass | pass | pass | pass | skip |
| `policy-downgrade` (disqualifying) | pass | pass | **FAIL** | pass | pass | **FAIL** | pass |
| `receiver-reuse` (disqualifying) | pass | pass | **FAIL** | pass | pass | **FAIL** | skip |
| `transparent-fallback` (disqualifying) | pass | pass | pass | pass | pass | pass | pass |
| `unshielding` (disqualifying) | pass | pass | pass | pass | pass | pass | pass |
| `amount-fingerprint` (warn) | pass | pass | **FAIL** | pass | pass | **FAIL** | pass |
| `memo-absent` (warn) | pass | pass | **FAIL** | pass | pass | **FAIL** | pass |
| `memo-content` (warn) | pass | pass | skip | pass | pass | skip | pass |
| `same-block-roundtrip` (warn) | pass | pass | pass | pass | pass | pass | pass |
| `server-metadata` (warn) | pass | pass | pass | pass | pass | pass | skip |
| `timing-correlation` (warn) | pass | pass | pass | pass | pass | pass | pass |
| `report-leak` (note) | pass | pass | pass | pass | pass | pass | skip |
| `payer-policy` (disqualifying) | — | — | — | — | — | — | pass |
| **verdict** | **12 pass · 0 fail · 0 skip** | **11 pass · 1 fail · 0 skip** | **7 pass · 4 fail · 1 skip** | **12 pass · 0 fail · 0 skip** | **12 pass · 0 fail · 0 skip** | **7 pass · 4 fail · 1 skip** | **9 pass · 0 fail · 4 skip** |

- `canary-checkout` @ `40d5f3a7` — run in `data/matrix-runs/checkout-private`; report `f23c0cd052b070cd7d7ad646263d0a4bb402c085afe7f846255b92f91bbf7660`
  ```
  zanary-report verify data/matrix-runs/checkout-private/report.json --node <node>
  ```
- `canary-checkout-batch` @ `f2f15593` — run in `data/matrix-runs/checkout-private-batch`; report `47a8756c33b2256f7ac6d2e2aece6a7ec8d74b0c17418780579661adff236876`
  ```
  zanary-report verify data/matrix-runs/checkout-private-batch/report.json --node <node>
  ```
- `canary-checkout-control` @ `9e42ff0e` — run in `data/matrix-runs/checkout-control`; report `8ec2d3fcea7acbffe13e6617b04a5d32f5f76ffd6b404fce7440c19538c591bc`
  ```
  zanary-report verify data/matrix-runs/checkout-control/report.json --node <node>
  ```
- `canary-checkout-jittered` @ `e3b7fb25` — run in `data/matrix-runs/checkout-private-jitter`; report `8155952cac19e32a74e5968b1deaf31ee5c0b3fb87264aab57ddd973ec4a5f74`
  ```
  zanary-report verify data/matrix-runs/checkout-private-jitter/report.json --node <node>
  ```
- `zanary-ci-checkout` @ `4d8279bb` — run in `data/matrix-runs/ci-checkout`; report `66aa1c8d1b4b4df278d26c8b50f1d83a4fccd98e7fffd4bd114fdf829dfc541d`
  ```
  zanary-report verify data/matrix-runs/ci-checkout/report.json --node <node>
  ```
- `zanary-ci-checkout-control` @ `05cc6aeb` — run in `data/matrix-runs/ci-checkout-control`; report `7c1702504dfc4f2400eae5761de61e1e8ceac472a46e6fab870ce4b92d35d938`
  ```
  zanary-report verify data/matrix-runs/ci-checkout-control/report.json --node <node>
  ```
- `zanary-payer/3178c172` @ `bf1029fb` — run in `data/matrix-runs/payer-pass`; report `aaaedbba00544d83e7ddb33dc54eac1a053bccfe869150d3014d63ed68a295fb`
  ```
  zanary-report verify data/matrix-runs/payer-pass/report.json --node <node>
  ```

Regenerate: `zanary-report matrix --runs data/runs --out docs/LEAK-MATRIX.md`.

---

Generated with `zanary-report matrix --runs data/matrix-runs --out docs/LEAK-MATRIX.md`, where
`data/matrix-runs/` is a curated set of symlinks into `data/runs/` — the four canonical flow runs, the
two CI flows, and the bounded payer's run — so the table stays about the library rather than about every
smoke run this working copy has ever produced. `—` in a cell means the run predates that property: the
recorded report is the artifact, and a property added later is not back-dated onto it.
