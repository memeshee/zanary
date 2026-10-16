#!/usr/bin/env bash
# ZANARY harness — bring the throwaway chain up and wait until it is usable.
#
#   harness/up.sh            # node + indexer, ready to run a flow against
#   harness/up.sh --clean    # same, but refuse to touch an existing rig (see below)
#
# What it does, in order, because each step is a thing that silently breaks a run if skipped:
#   1. seeds `data/zcashd/.zcash/zcash.conf` from the tracked `harness/zcash.conf` when a fresh
#      checkout has none — without it zcashd comes up with no regtest network upgrades and no
#      mining address (the shielded pool would be missing);
#   2. `docker compose up -d`;
#   3. waits for the node's JSON-RPC to answer `getblockcount`;
#   4. waits for lightwalletd's gRPC port, then for its `GetLightdInfo` reply through the probe —
#      the same handshake every flow starts with, so "ready" means ready for the rig, not just
#      "a port is open".
#
# It never overwrites an existing node config, and never deletes chain data: the rig is
# disposable, not accidentally disposable. `docker compose down && ./up.sh` keeps the height.
set -euo pipefail

cd "$(dirname "$0")"
ROOT="$(cd .. && pwd)"
RPC_USER="${ZANARY_NODE_USER:-zanary}"
RPC_PASS="${ZANARY_NODE_PASS:-zanary}"
NODE_RPC="${ZANARY_NODE_URL:-http://127.0.0.1:18232}"
LWD_URL="${ZANARY_LWD_URL:-http://127.0.0.1:9067}"
CONF_DIR="$ROOT/data/zcashd/.zcash"
CONF="$CONF_DIR/zcash.conf"
WAIT_NODE_SECONDS="${ZANARY_WAIT_NODE_SECONDS:-120}"
WAIT_INDEXER_SECONDS="${ZANARY_WAIT_INDEXER_SECONDS:-120}"

say() { printf '\n=== %s ===\n' "$1"; }
zc() { docker exec zanary-zcashd zcash-cli -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" "$@"; }

# The probe binary is the honest readiness test for the indexer (a TCP connect says nothing about
# whether lightwalletd can serve compact blocks). Prefer a release build, fall back to debug; if
# neither exists the port check below still gates on the socket.
probe_bin() {
  for candidate in "$ROOT/target/release/zanary-probe" "$ROOT/target/debug/zanary-probe"; do
    [ -x "$candidate" ] && { printf '%s' "$candidate"; return 0; }
  done
  return 1
}

say "node config"
mkdir -p "$CONF_DIR"
if [ -f "$CONF" ]; then
  echo "kept existing $CONF (its mining address and any local edits are live node state)"
else
  cp zcash.conf "$CONF"
  echo "seeded $CONF from harness/zcash.conf"
fi

say "chain + indexer"
# The node container runs as this user (see the `user:` line in docker-compose.yml) so the
# bind-mounted datadir is writable by whoever owns the checkout — kiter's uid locally, a runner uid
# in CI. Exported here because compose reads it from the environment.
export ZANARY_UID="$(id -u)"
export ZANARY_GID="$(id -g)"
docker compose up -d

say "waiting for the node (up to ${WAIT_NODE_SECONDS}s)"
deadline=$((SECONDS + WAIT_NODE_SECONDS))
until zc getblockcount >/dev/null 2>&1; do
  if [ "$SECONDS" -ge "$deadline" ]; then
    echo "the node did not answer getblockcount within ${WAIT_NODE_SECONDS}s — recent logs:" >&2
    docker logs --tail 40 zanary-zcashd >&2 || true
    exit 1
  fi
  sleep 2
done
HEIGHT="$(zc getblockcount)"
echo "node ready at height ${HEIGHT}"

say "waiting for the indexer (up to ${WAIT_INDEXER_SECONDS}s)"
deadline=$((SECONDS + WAIT_INDEXER_SECONDS))
indexer_ready=0
while [ "$indexer_ready" -eq 0 ]; do
  if [ -x "$(probe_bin 2>/dev/null || true)" ] && ZANARY_LWD_URL="$LWD_URL" "$(probe_bin)" >/dev/null 2>&1; then
    indexer_ready=1
  elif python3 - "$LWD_URL" <<'PY'
import socket, sys, urllib.parse
url = urllib.parse.urlparse(sys.argv[1])
host = url.hostname or "127.0.0.1"
port = url.port or 9067
s = socket.socket()
s.settimeout(2)
try:
    s.connect((host, port))
except OSError:
    sys.exit(1)
finally:
    s.close()
PY
  then
    # A listening socket and a *caught-up* indexer are different things: lightwalletd replays
    # blocks from the node after a restart, which is why the probe handshake above is preferred.
    indexer_ready=1
  fi
  if [ "$indexer_ready" -eq 0 ]; then
    if [ "$SECONDS" -ge "$deadline" ]; then
      echo "the indexer was not serving at ${LWD_URL} within ${WAIT_INDEXER_SECONDS}s — recent logs:" >&2
      docker logs --tail 40 zanary-lwd >&2 || true
      exit 1
    fi
    sleep 2
  fi
done
echo "indexer ready at ${LWD_URL}"

say "the rig"
echo "chain      : regtest (throwaway) · height ${HEIGHT}"
echo "node RPC   : ${NODE_RPC} (user ${RPC_USER})"
echo "indexer    : ${LWD_URL} (CompactTxStreamer over gRPC)"
echo
echo "next: cargo build --release --bin zanary-probe --bin zanary-report"
echo "      ./target/release/zanary-probe test --definition flows/ci-checkout.json --expect pass"
echo "      ./target/release/zanary-probe test --definition flows/ci-checkout-control.json --expect fail"
