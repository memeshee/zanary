#!/usr/bin/env bash
# The bounded-payer acceptance test, as a script a judge can run:
#
#   harness/up.sh && scripts/bounded-payer-demo.sh
#
# Four invoices, a grant that allows exactly one of them, and then the same four under a tripped kill
# switch. What is asserted is the thing the grant promises: *over-cap and off-allowlist attempts are
# refused, and the refusals are in the ledger* — one refusal per rule, each naming the rule that
# refused it. Exits non-zero if any of that is not in the evidence.
#
# Amounts are deliberately non-round: round amounts fail `amount-fingerprint`, which is how the first
# draft of this demo leaked (see docs/BOUNDED-PAYER.md).
set -euo pipefail

# Same reason as scripts/demo-run.sh: the ledger is the output, the wallet's migration log is not.
export RUST_LOG="${RUST_LOG:-warn}"

BIN="${ZANARY_PROBE:-./target/release/zanary-probe}"
DIR="${ZANARY_PAYER_DEMO_DIR:-data/payer-demo}"
BUDGET_ZAT="${ZANARY_PAYER_BUDGET_ZAT:-12000000}"   # 0.12 ZEC
CAP_ZAT="${ZANARY_PAYER_CAP_ZAT:-10000000}"         # 0.1 ZEC

say() { printf '\n=== %s ===\n' "$1"; }
die() { printf '\nerror: %s\n' "$1" >&2; exit 1; }

[ -x "$BIN" ] || die "no probe binary at $BIN — build it first: cargo build --release --bin zanary-probe"
mkdir -p "$DIR/invoices" "$DIR/grants" "$DIR/runs"

say "four invoices"
# 8301 fits the grant · 8302 does not fit the budget after 8301 · 8303 is over the cap ·
# 8304 is a receiver the grant never named.
for spec in "8301 0.0987654" "8302 0.0432109" "8303 0.5" "8304 0.02"; do
  set -- $spec
  "$BIN" invoice --out "$DIR/invoices/$1.json" --id "$1" --amount-zec "$2" >/dev/null
  echo "  issued $1 ($2 ZEC)"
done

say "the grant: one of them payable, three rules to refuse the others"
python3 - "$DIR" "$BUDGET_ZAT" "$CAP_ZAT" <<'PY'
import json, sys
dir, budget, cap = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
inv = {i: json.load(open(f"{dir}/invoices/{i}.json")) for i in ("8301", "8302", "8303", "8304")}
grant = {
    "budget_zat": budget,
    "per_payment_cap_zat": cap,
    # 8303 is allowlisted on purpose: it is refused for the *cap*, so the ledger names the cap
    # rather than hiding that refusal behind the allowlist. 8304 is the stranger.
    "allowlist": [inv[i]["receiver"] for i in ("8301", "8302", "8303")],
    "expires_at": "2030-01-01T00:00:00Z",
    "revoked": False,
}
json.dump(grant, open(f"{dir}/grants/allowed.json", "w"), indent=2)
revoked = dict(grant, revoked=True)
json.dump(revoked, open(f"{dir}/grants/killed.json", "w"), indent=2)
print(f"  allowlist {len(grant['allowlist'])} receiver(s) · budget {budget} zat · cap {cap} zat")
PY

say "the payer, under the grant"
"$BIN" payer --grant "$DIR/grants/allowed.json" --invoices "$DIR/invoices" \
  --out "$DIR/runs/allowed" --expect-refused 3 --ttl-days 30

say "the kill switch: the same invoices, authority withdrawn"
"$BIN" payer --grant "$DIR/grants/killed.json" --invoices "$DIR/invoices" \
  --out "$DIR/runs/killed" --expect-refused 4 --ttl-days 30

say "the ledger, as evidence rather than a summary"
python3 - "$DIR" <<'PY'
import json, sys
dir = sys.argv[1]

def ledger(name):
    with open(f"{dir}/runs/{name}/ledger.jsonl") as fh:
        return [json.loads(line) for line in fh if line.strip()]

def findings(name):
    report = json.load(open(f"{dir}/runs/{name}/report.json"))
    return {f["property"]: f["verdict"] for f in report["findings"]}

problems = []

allowed = ledger("allowed")
if len(allowed) != 4:
    problems.append(f"the allowed run has {len(allowed)} ledger lines, expected 4")
paid = [d for d in allowed if d["decision"] == "paid"]
refused = [d for d in allowed if d["decision"] == "refused"]
if [d["invoice_id"] for d in paid] != ["8301"]:
    problems.append(f"paid set is {[d['invoice_id'] for d in paid]}, expected exactly ['8301']")
if len(refused) != 3:
    problems.append(f"{len(refused)} refusals, expected 3")
for invoice, needle in (("8302", "budget"), ("8303", "per-payment cap"), ("8304", "allowlist")):
    reason = next((d["reason"] for d in refused if d["invoice_id"] == invoice), "")
    if needle not in reason:
        problems.append(f"invoice {invoice} was refused for {reason!r}, which does not name the {needle}")
if paid and not paid[0].get("txid"):
    problems.append("the paid invoice has no txid in the ledger")

killed = ledger("killed")
if len(killed) != 4 or any(d["decision"] != "refused" for d in killed):
    problems.append("the kill-switch run did not refuse all four invoices")
elif not all("kill switch" in d["reason"] for d in killed):
    problems.append("a kill-switch refusal does not name the kill switch")

killed_findings = findings("killed")
if killed_findings.get("payer-policy") != "pass":
    problems.append(f"payer-policy on the kill-switch run is {killed_findings.get('payer-policy')!r}")

allowed_findings = findings("allowed")
if allowed_findings.get("payer-policy") != "pass":
    problems.append(f"payer-policy is {allowed_findings.get('payer-policy')!r}, expected pass")

for d in allowed:
    print(f"  {d['decision']:<8} {d['invoice_id']} · {d['amount_zat']} zat · {d['reason']}")
print()
print(f"  allowed run: {len(paid)} paid · {len(refused)} refused · payer-policy {allowed_findings.get('payer-policy')}")
print(f"  killed run : {len(killed)} refused · payer-policy {killed_findings.get('payer-policy')}")

if problems:
    print()
    for problem in problems:
        print(f"  FAIL {problem}")
    sys.exit(1)
print()
print("  the grant held: every payment it allowed was made, every refusal names a rule, and the")
print("  kill switch stops everything — all of it in the ledger, not in this script's opinion")
PY
