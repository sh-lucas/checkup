#!/usr/bin/env bash
# Usage:
#   bash bench/run.sh rust <label>   # build + run current Rust branch
#   bash bench/run.sh bun            # run bun server

set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
BENCH="$REPO/bench"
PORT=3000
DB="$BENCH/bench.db"
LABEL="${2:-rust}"
MODE="${1:-rust}"

mkdir -p "$BENCH/results"

wait_ready() {
  local i=0
  until curl -sf "http://localhost:$PORT/" >/dev/null 2>&1; do
    sleep 0.5; i=$((i+1))
    [[ $i -lt 30 ]] || { echo "server did not start" >&2; exit 1; }
  done
}

# Boot server so it runs sqlx migrations, then kill and seed.
setup_db() {
  local cmd=("$@")
  # kill anything already on this port
  fuser -k "${PORT}/tcp" 2>/dev/null || true
  sleep 0.3
  rm -f "$DB" "$DB-wal" "$DB-shm"
  DATABASE_URL="sqlite:///$DB" PORT=$PORT "${cmd[@]}" &
  local mpid=$!
  wait_ready
  kill "$mpid"; wait "$mpid" 2>/dev/null || true
  sqlite3 "$DB" "PRAGMA wal_checkpoint(FULL);" >/dev/null
  sqlite3 "$DB" < "$BENCH/seed.sql"
  echo "  DB ready."
}

if [[ "$MODE" == "rust" ]]; then
  echo "Building..."
  TURSO_LIB="$REPO/turso/target/release"
  if [[ -f "$TURSO_LIB/libturso_sqlite3.a" ]]; then
    echo "  Linking against turso sqlite3..."
    ln -sf "$TURSO_LIB/libturso_sqlite3.a" "$TURSO_LIB/libsqlite3.a"
    SQLITE3_LIB_DIR="$TURSO_LIB" SQLITE3_INCLUDE_DIR="/usr/include" SQLITE3_STATIC=1 \
      cargo build --release --manifest-path "$REPO/Cargo.toml" -q
  else
    cargo build --release --manifest-path "$REPO/Cargo.toml" -q
  fi
  BIN="$REPO/target/release/checkup"
  setup_db "$BIN"
  echo "Starting server..."
  DATABASE_URL="sqlite:///$DB" PORT=$PORT taskset -c 0 "$BIN" &
  PID=$!

elif [[ "$MODE" == "bun" ]]; then
  LABEL="bun"
  setup_db bun run "$BENCH/bun-server/index.ts"
  echo "Starting bun server..."
  DATABASE_URL="sqlite:///$DB" PORT=$PORT \
    taskset -c 0 bun run "$BENCH/bun-server/index.ts" &
  PID=$!

else
  echo "Usage: $0 rust [label] | bun"; exit 1
fi

wait_ready
echo "Running k6 ($LABEL)..."
BASE_URL="http://localhost:$PORT" k6 run \
  --out "json=$BENCH/results/$LABEL.json" \
  "$BENCH/bench.js"

kill $PID 2>/dev/null; wait $PID 2>/dev/null || true
echo "Done. Results: bench/results/$LABEL.json"
