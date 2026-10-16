#!/usr/bin/env bash
#
# The whole story, in one command, from a repo checkout:
#
#   harness/up.sh && scripts/demo-run.sh
#
# Four stages, each one an artifact rather than a claim:
#
#   1. the private flow  — a leak-free checkout, anchored; must PASS
#   2. the control flow  — the same rig pointed at a deliberately leaky flow; must be CAUGHT
#   3. the bounded payer — a budget instead of a key: what the grant allows is paid, what it forbids is
#                          refused, and the refusals are in the ledger
#   4. the index         — the claims those runs just made, rendered as the page a stranger reads
#
# Every stage's exit code is checked and reported at the end; the script does not stop at the first
# failure, because "the control flow stopped being caught" and "the payer leaked" are different news
# and a recording should not hide either. It exits non-zero if any stage failed.
#
# For a recording: this is the single take. Nothing here prints a path a viewer has to be told about.
set -uo pipefail
cd "$(dirname "$0")/.."

# The recording wants the artifacts, not the wallet's own log: `schemerz` prints a migration line per
# wallet open at INFO, which is noise on screen and no information for a viewer. Override with
# ZANARY_LOG=info when debugging the rig itself.
export RUST_LOG="${ZANARY_LOG:-warn}"

BIN="${ZANARY_PROBE:-./target/release/zanary-probe}"
REPORT="${ZANARY_REPORT:-./target/release/zanary-report}"
RUNS="${ZANARY_DEMO_RUNS:-data/runs}"
# Where the payer's ledger lands, in one place: the summary has to read the same directory the payer
# wrote to, including when a caller overrides it.
PAYER_DEMO_DIR="${ZANARY_PAYER_DEMO_DIR:-data/payer-demo}"
export ZANARY_PAYER_DEMO_DIR="$PAYER_DEMO_DIR"
STARTED_AT=$(date -u +%Y-%m-%dT%H:%M:%SZ)

say() { printf '\n\033[1m== %s ==\033[0m\n' "$1"; }
step() { printf '   %s\n' "$1"; }
FAILED=()

stage() {
  local name="$1"; shift
  local started=$SECONDS
  if "$@"; then
    printf '   ✓ %s (%ss)\n' "$name" "$((SECONDS - started))"
  else
    printf '   ✗ %s FAILED (%ss)\n' "$name" "$((SECONDS - started))"
    FAILED+=("$name")
  fi
}

# One line per artifact, read from the artifact itself — the summary below must not be the only
# place a number comes from, and it must be possible to see it disagree.
verdict_of() {
  python3 - "$1" <<'PY'
import json, sys
try:
    report = json.load(open(sys.argv[1]))
except Exception as e:
    print(f"(no report: {e})")
    raise SystemExit
counts = {"pass": 0, "fail": 0, "skip": 0}
for finding in report.get("findings", []):
    counts[finding["verdict"]] = counts.get(finding["verdict"], 0) + 1
verdict = "PASS" if counts["fail"] == 0 else f"CAUGHT {counts['fail']} leak(s)"
print(f"{verdict} · {counts['pass']} pass · {counts['fail']} fail · {counts['skip']} skip · report {report.get('report_hash', '')[:12]}")
PY
}

payer_line() {
  python3 - "$1" <<'PY'
import json, sys
try:
    entries = [json.loads(line) for line in open(sys.argv[1]) if line.strip()]
except Exception as e:
    print(f"(no ledger: {e})")
    raise SystemExit
paid = [d for d in entries if d["decision"] == "paid"]
refused = [d for d in entries if d["decision"] == "refused"]
def rule(reason):
    for needle, name in (
        ("kill switch", "kill switch"),
        ("budget", "budget"),
        ("per-payment cap", "cap"),
        ("allowlist", "allowlist"),
        ("expired", "expiry"),
    ):
        if needle in reason:
            return name
    return "other"
rules = sorted({rule(d["reason"]) for d in refused})
spent = sum(d["amount_zat"] for d in paid)
print(f"{len(paid)} paid ({spent} zat spent) · {len(refused)} refused [{', '.join(rules) or 'none'}]")
PY
}

[ -x "$BIN" ] || { echo "no probe binary at $BIN — build it: cargo build --release --bin zanary-probe --bin zanary-report" >&2; exit 1; }
[ -x "$REPORT" ] || { echo "no report binary at $REPORT — build it: cargo build --release --bin zanary-probe --bin zanary-report" >&2; exit 1; }
mkdir -p "$RUNS"

printf 'ZANARY — privacy claims that expire\n'
printf 'demo started %s\n' "$STARTED_AT"

# --------------------------------------------------------------------------- 1
say "1. a payment flow that does it right (must PASS)"
step "2 invoices · fresh diversified receiver per invoice · one ZIP-302 memo each · FullPrivacy"
stage "private flow" "$BIN" test --definition flows/ci-checkout.json \
  --out "$RUNS/demo-private" --expect pass --anchor --ttl-days 30 \
  --summary "$RUNS/demo-private/test.json"

# --------------------------------------------------------------------------- 2
say "2. the same rig, pointed at a deliberately leaky flow (must be CAUGHT)"
step "one reused receiver · no memos · AllowRevealedAmounts · round amounts — four planted leaks"
stage "control flow" "$BIN" test --definition flows/ci-checkout-control.json \
  --out "$RUNS/demo-control" --expect fail --anchor --ttl-days 30 \
  --summary "$RUNS/demo-control/test.json"

# --------------------------------------------------------------------------- 3
say "3. a bounded payer: a budget instead of a key"
step "one grant · four invoices · then the same four with the kill switch thrown"
stage "bounded payer" scripts/bounded-payer-demo.sh

# --------------------------------------------------------------------------- 4
say "4. the claims those runs made, as the page a stranger reads"
stage "claim index" "$REPORT" index --reports "$RUNS" --out "$RUNS/index.html"

say "summary"
printf '   private flow   : %s\n' "$(verdict_of "$RUNS/demo-private/report.json")"
printf '   control flow   : %s\n' "$(verdict_of "$RUNS/demo-control/report.json")"
printf '   payer ledger   : %s\n' "$(payer_line "$PAYER_DEMO_DIR/runs/allowed/ledger.jsonl")"
printf '   payer (killed) : %s\n' "$(payer_line "$PAYER_DEMO_DIR/runs/killed/ledger.jsonl")"
printf '   index          : %s\n' "$RUNS/index.html"

if [ "${#FAILED[@]}" -ne 0 ]; then
  printf '\n    stages that failed: %s\n' "${FAILED[*]}"
  exit 1
fi
printf '\n    every stage did what it said — the reports are in %s\n' "$RUNS"
