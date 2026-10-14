# report fixtures — a real run, checkable offline and against the harness

These files are not mock-ups: the txids, heights, memos and pool totals below were recorded
from the local regtest harness chain on 2026-10-10 by `canary-seed` (see
`crates/probe/src/bin/canary-seed.rs` and `docs/REPORT-NOTES.md`). Two shielded payments were
made, each a coinbase-funded shield into an Orchard output carrying a memo.

## Files

| file | what it is |
|---|---|
| `run.json` | run 1 (invoice 4471): a real `FlowRun` — payment tx `295e61d6…` at height 261 |
| `findings.json` | findings for run 1, produced by the property library where it exists and hand-authored otherwise (the registry is being filled in; see docs/REPORT-NOTES.md) |
| `report.json` | run 1 built into a report: `report_hash 8b4908e1…`, evidence root `4790bfdb…` |
| `run2.json` | run 2 (invoice 4470): payment tx `a3887b72…` at height 273, deliberately paying the **same receiver** — a real receiver-reuse regression |
| `findings2.json` | findings for run 2 (`receiver-reuse` FAILs) |
| `report2.json` | run 2 built into a report: `report_hash 904274f8…`, evidence root `a14443cb…` |
| `index-demo/` | an older flow revision (same real evidence, different `flow_commit`) used to show stale/dead claims in the index |
| `reports/` | the four claims the golden index test renders |
| `index.golden.html` | golden output of `canary-report index` over `reports/` at `--now 2026-10-25T00:00:00Z` |

## Check them yourself

```bash
# against the live harness (needs the docker chain up):
cargo run -p canary-report -- verify crates/report/fixtures/report.json
cargo run -p canary-report -- verify crates/report/fixtures/report2.json

# rebuild report.json from run.json (deterministic; no chain needed):
cargo run -p canary-report -- build --run crates/report/fixtures/run.json \
  --findings crates/report/fixtures/findings.json \
  --verified-at 2026-10-10T20:30:00Z --out /tmp/report.json

# render the claim index:
cargo run -p canary-report -- index --reports crates/report/fixtures/reports \
  --out /tmp/index.html --now 2026-10-25T00:00:00Z

# or emit the bundled run as a starting point:
cargo run -p canary-report -- sample-run --out run.json
```

`verify` exits 0 when every check passes and 1 on any mismatch (try editing one hex digit of
a txid in a copy and re-running).
