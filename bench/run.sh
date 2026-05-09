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

# seed: just insert data (migrations already ran)
seed() {
  sqlite3 "$DB" < "$BENCH/seed.sql"
}

wait_ready() {
  for _ in $(seq 40); do
    curl -sf "http://localhost:$PORT/" >/dev/null 2>&1 && return
    sleep 0.3
  done
  echo "server did not start" >&2; exit 1
}

if [[ "$MODE" == "rust" ]]; then
  echo "Building..."
  cargo build --release --manifest-path "$REPO/Cargo.toml" -q

  echo "Migrating + seeding..."
  rm -f "$DB" "$DB-wal" "$DB-shm"
  DATABASE_URL="sqlite:///$DB" PORT=$PORT "$REPO/target/release/checkup" &
  PID=$!
  wait_ready
  kill $PID; wait $PID 2>/dev/null; sleep 0.3
  seed

  echo "Starting server..."
  DATABASE_URL="sqlite:///$DB" PORT=$PORT \
    taskset -c 0 "$REPO/target/release/checkup" &
  PID=$!

elif [[ "$MODE" == "bun" ]]; then
  LABEL="bun"
  echo "Migrating + seeding..."
  rm -f "$DB" "$DB-wal" "$DB-shm"
  DATABASE_URL="sqlite:///$DB" PORT=$PORT \
    bun run "$BENCH/bun-server/index.ts" &
  PID=$!
  wait_ready
  kill $PID; wait $PID 2>/dev/null; sleep 0.3
  seed

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

kill $PID 2>/dev/null; wait $PID 2>/dev/null
echo "Done. Results: bench/results/$LABEL.json"
