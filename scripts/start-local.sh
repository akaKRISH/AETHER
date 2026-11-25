#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/.."
: "${AETHER_TOKEN:?Set a local AETHER_TOKEN before starting}"
(cd web && npm ci --no-audit --no-fund && npm run build)
exec cargo run --locked --release --bin aether-lab -- serve "${1:-8080}"
