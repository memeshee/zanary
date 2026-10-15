#!/usr/bin/env bash
#
# The agent run: three invoice → pay → verify cycles with no manual step in between.
#
# The merchant issues an invoice (a fresh diversified Orchard receiver + a ZIP-321 URI on disk), the
# payer pays it through `zanary-probe pay --invoice` — the same propose/sign/broadcast path the flow
# runner uses — and the merchant's *own* wallet then fetches the transaction and trial-decrypts it
# with its own UFVK, checking that the memo names the invoice. Nothing is trusted from the payer's
# side of the conversation.
#
# Phase 4's gate is that this works three times in a row, unassisted. Run it from the repo root:
#
#   scripts/agent-pay-loop.sh
#
set -euo pipefail
cd "$(dirname "$0")/.."

export ZANARY_LWD_URL="${ZANARY_LWD_URL:-http://127.0.0.1:9067}"
POLICY="${POLICY:-AllowRevealedAmounts}"
BIN=./target/debug/zanary-probe
mkdir -p data/invoices

for n in 1 2 3; do
    id="$((8100 + n))"
    file="data/invoices/invoice-${id}.json"
    amount="0.1${n}234567"

    echo
    echo "================ cycle $n — invoice $id ================"
    "$BIN" invoice --out "$file" --id "$id" --amount-zec "$amount"

    echo "--- the payer pays it ---"
    "$BIN" pay --invoice "$file" --policy "$POLICY" | tee "data/invoices/pay-${id}.log"
    txid=$(grep -oE '[0-9a-f]{64}' "data/invoices/pay-${id}.log" | head -1)
    if [ -z "$txid" ]; then
        echo "cycle $n: no txid in the pay output — stopping" >&2
        exit 1
    fi

    echo "--- the merchant verifies it ---"
    "$BIN" verify-invoice --invoice "$file" --txid "$txid"

    echo "cycle $n: invoice $id settled — txid $txid"
done

echo
echo "================ all three cycles settled ================"
for n in 1 2 3; do
    id="$((8100 + n))"
    txid=$(grep -oE '[0-9a-f]{64}' "data/invoices/pay-${id}.log" | head -1)
    echo "  invoice $id · $txid"
done
