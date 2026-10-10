#!/usr/bin/env bash
# CANARY harness — seed the throwaway chain and produce evidence.
#
#   cd harness && docker compose up -d && ./seed.sh
#
# Idempotent enough to re-run: it mines to a target height, creates a fresh
# account, shields coinbase into Orchard, then sends a shielded payment whose
# memo carries a structured record, and prints every id a judge can check.
#
# The chain is DISPOSABLE BY DESIGN. If `getblockcount` comes back 0 the node
# has been reindexed (see HARNESS.md); just run this script again — mining 140
# blocks takes about two seconds on regtest.
set -euo pipefail

ZC="docker exec canary-zcashd zcash-cli -regtest -rpcuser=canary -rpcpassword=canary"
TARGET_HEIGHT="${TARGET_HEIGHT:-140}"
MERCHANT_MEMO_TEXT="${MERCHANT_MEMO_TEXT:-canary regtest memo: invoice 4471 / receipt seed 0xdeadbeef}"
OUT_DIR="$(cd "$(dirname "$0")/.." && pwd)/data"

say() { printf '\n=== %s ===\n' "$1"; }

say "node"
$ZC getblockchaininfo | python3 -c "import sys,json;d=json.load(sys.stdin);print('chain',d['chain'],'blocks',d['blocks'])"

say "mine to height ${TARGET_HEIGHT}"
CUR=$($ZC getblockcount)
if [ "$CUR" -lt "$TARGET_HEIGHT" ]; then
  $ZC generate "$((TARGET_HEIGHT - CUR))" >/dev/null
fi
$ZC getblockcount

say "account + orchard-only diversified receiver"
$ZC z_getnewaccount >/dev/null
RECV=$($ZC z_getaddressforaccount 0 '["orchard"]' | python3 -c "import sys,json;print(json.load(sys.stdin)['address'])")
echo "$RECV" | tee "$OUT_DIR/harness-orchard-only.txt"
echo "receiver: $RECV"

say "shield coinbase into orchard"
$ZC generate 4 >/dev/null
$ZC z_shieldcoinbase "*" "$RECV" >/dev/null
sleep 8
SHIELD_TXID=$($ZC z_getoperationresult | python3 -c "
import sys,json
for o in json.load(sys.stdin):
    if o['status'] == 'success': print(o['result']['txid'])
" | head -1)
echo "shield txid: $SHIELD_TXID"
$ZC generate 4 >/dev/null

say "shielded payment carrying a memo (orchard -> orchard, fresh diversified receiver)"
MEMO_HEX=$(python3 -c "import sys;print('$MERCHANT_MEMO_TEXT'.encode().hex())")
$ZC z_sendmany "$RECV" "[{\"address\":\"$RECV\",\"amount\":0.01,\"memo\":\"$MEMO_HEX\"}]" 1 0.0001 "AllowRevealedAmounts" >/dev/null
sleep 15
SEND_TXID=$($ZC z_getoperationresult | python3 -c "
import sys,json
for o in json.load(sys.stdin):
    if o['status'] == 'success': print(o['result']['txid'])
" | head -1)
echo "memo payment txid: $SEND_TXID"
$ZC generate 2 >/dev/null

say "evidence"
$ZC getblockchaininfo | python3 -c "import sys,json;d=json.load(sys.stdin);print('blocks',d['blocks']);print('pools',[(p['id'],p['chainValue']) for p in d.get('valuePools',[])])"
$ZC z_listunspent 0 1000 | python3 -c "
import sys,json
d=json.load(sys.stdin)
print('spendable notes:', len(d))
for n in d:
    memo = n.get('memo') or ''
    try:
        text = bytes.fromhex(memo).decode('utf-8','replace') if memo and not memo.startswith('f6') else '(no memo)'
    except Exception:
        text = '(unreadable)'
    print(' -', n.get('pool'), n.get('amount'), '| memo:', text[:70])
"

cat > "$OUT_DIR/harness-evidence.json" <<JSON
{
  "chain": "regtest",
  "height": $($ZC getblockcount),
  "receiver": "$RECV",
  "shield_txid": "$SHIELD_TXID",
  "memo_payment_txid": "$SEND_TXID",
  "memo_text": "$MERCHANT_MEMO_TEXT"
}
JSON
say "wrote $OUT_DIR/harness-evidence.json"
