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

- `canary-checkout` @ `40d5f3a7` — run in `data/runs/checkout-private`; report `f23c0cd052b070cd7d7ad646263d0a4bb402c085afe7f846255b92f91bbf7660`
  ```
  canary-report verify data/runs/checkout-private/report.json --node <node>
  ```
- `canary-checkout-batch` @ `f2f15593` — run in `data/runs/checkout-private-batch`; report `47a8756c33b2256f7ac6d2e2aece6a7ec8d74b0c17418780579661adff236876`
  ```
  canary-report verify data/runs/checkout-private-batch/report.json --node <node>
  ```
- `canary-checkout-control` @ `9e42ff0e` — run in `data/runs/checkout-control`; report `8ec2d3fcea7acbffe13e6617b04a5d32f5f76ffd6b404fce7440c19538c591bc`
  ```
  canary-report verify data/runs/checkout-control/report.json --node <node>
  ```
- `canary-checkout-jittered` @ `e3b7fb25` — run in `data/runs/checkout-private-jitter`; report `8155952cac19e32a74e5968b1deaf31ee5c0b3fb87264aab57ddd973ec4a5f74`
  ```
  canary-report verify data/runs/checkout-private-jitter/report.json --node <node>
  ```

Regenerate: `canary-report matrix --runs data/runs --out docs/LEAK-MATRIX.md`.
