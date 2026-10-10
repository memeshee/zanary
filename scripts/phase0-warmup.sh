#!/usr/bin/env bash
# CANARY — Phase 0 warm-up.
# Builds the Zcash dependency graph (the long pole) and pulls chain images,
# while snapshotting the API docs we will need for wallet wiring.
# Safe to re-run. Logs to ../phase0-warmup.log when invoked by the agent.
set -uo pipefail
cd "$(dirname "$0")/.."
echo "=== CANARY phase 0 warm-up ==="
date -u +"started %Y-%m-%dT%H:%M:%SZ"

echo; echo "--- toolchain ---"
rustc --version; cargo --version; rustup show active-toolchain 2>/dev/null

echo; echo "--- cargo fetch ---"
cargo fetch 2>&1 | tail -5

echo; echo "--- debug build (dependency warm-up) ---"
SECONDS=0
if cargo build --workspace 2>&1 | tail -40; then
  echo "cargo build OK in ${SECONDS}s"
else
  echo "cargo build FAILED after ${SECONDS}s (see above) — fix before D1 gate"
fi

echo; echo "--- running the probe ---"
cargo run -q --bin canary-probe 2>&1 | tail -10

echo; echo "--- docker images (best effort; failures are non-fatal) ---"
for img in zfnd/zebra:latest zfnd/zebra:v6.4.2 zfnd/lightwalletd:latest zfnd/zallet:latest electriccoinco/zcashd:latest; do
  echo "-- $img"
  timeout 900 docker pull "$img" 2>&1 | tail -2
done

echo; echo "--- api docs snapshot (offline reference for the wallet wiring) ---"
mkdir -p docs/refs
for u in \
  "https://docs.rs/zcash_client_backend/0.24.0/zcash_client_backend/" \
  "https://docs.rs/crate/zcash_client_backend/0.24.0/features" \
  "https://docs.rs/zip321/0.9.0/zip321/" \
  "https://docs.rs/pczt/0.9.3/pczt/" \
  "https://docs.rs/zcash_proofs/0.30.0/zcash_proofs/" ; do
  n=$(echo "$u" | sed -e 's|https://docs.rs/||' -e 's|/|_|g')
  if curl -sL --max-time 45 "$u" -o "docs/refs/$n.html"; then
    echo "saved docs/refs/$n.html ($(wc -c < "docs/refs/$n.html") bytes)"
  else
    echo "FAILED $u"
  fi
done

echo; echo "--- rust version requirement check ---"
grep -rhoE 'rust-version = "[^"]+"' ~/.cargo/registry/src/*/zcash_client_backend-0.24.0/Cargo.toml 2>/dev/null | head -3

echo; date -u +"finished %Y-%m-%dT%H:%M:%SZ"
echo "=== WARM-UP DONE ==="
