# CANARY leak matrix

Every cell is one property of the leak library applied to one run of one flow definition.
`pass` means the property held; **FAIL** means it did not; `skip` means the run did not
exercise it. Each column names the `flow_commit` it is about, so a claim is pinned to an
exact revision of a definition, and the re-check line under the table re-derives that run's
report from the chain.

| property | canary-checkout `40d5f3a7` | canary-checkout-batch `f2f15593` | canary-checkout-control `9e42ff0e` | canary-checkout-jittered `e3b7fb25` |
|---|---|---|---|---|
| `payer-linkability` (disqualifying) | pass | **FAIL** | pass | pass |
| `policy-downgrade` (disqualifying) | pass | pass | **FAIL** | pass |
| `receiver-reuse` (disqualifying) | pass | pass | **FAIL** | pass |
| `transparent-fallback` (disqualifying) | pass | pass | pass | pass |
| `unshielding` (disqualifying) | pass | pass | pass | pass |
| `amount-fingerprint` (warn) | pass | pass | **FAIL** | pass |
| `memo-absent` (warn) | pass | pass | **FAIL** | pass |
| `memo-content` (warn) | pass | pass | skip | pass |
| `same-block-roundtrip` (warn) | pass | pass | pass | pass |
| `server-metadata` (warn) | pass | pass | pass | pass |
| `timing-correlation` (warn) | pass | pass | pass | pass |
| `report-leak` (note) | pass | pass | pass | pass |
| **verdict** | **12 pass · 0 fail · 0 skip** | **11 pass · 1 fail · 0 skip** | **7 pass · 4 fail · 1 skip** | **12 pass · 0 fail · 0 skip** |

- `canary-checkout` @ `40d5f3a7` — run in `data/runs/checkout-private`; report `9774534adb484535539fa8d647e1b2df834afa04903df505e8cf284b264898a0`
  ```
  canary-report verify data/runs/checkout-private/report.json --node <node>
  ```
- `canary-checkout-batch` @ `f2f15593` — run in `data/runs/checkout-private-batch`; report `ffa951496701533a024edfb34dd1691256f327e71fb188b018fa4fdad6860fe0`
  ```
  canary-report verify data/runs/checkout-private-batch/report.json --node <node>
  ```
- `canary-checkout-control` @ `9e42ff0e` — run in `data/runs/checkout-control`; report `f0bf4917001b86b49086961dab0f76b3c746e26a350a61702d69d78cea38dd0f`
  ```
  canary-report verify data/runs/checkout-control/report.json --node <node>
  ```
- `canary-checkout-jittered` @ `e3b7fb25` — run in `data/runs/checkout-private-jitter`; report `49d61e58e938ab999810421a1bb18c2a257e09d230ca965a93d1855233b582a8`
  ```
  canary-report verify data/runs/checkout-private-jitter/report.json --node <node>
  ```

Regenerate: `canary-report matrix --runs data/runs --out docs/LEAK-MATRIX.md`.
