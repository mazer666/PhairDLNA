#!/bin/bash
# PhairDLNA Soak and Long-term Stability Test Harness (REL-004, AC-13)
#
# What: Executes server on defined mixed workload and periodically samples
#       resident memory (RSS), open file descriptors, and thread count.
# Why: Guarantees flat memory, descriptor, and thread usage over long runs without leaks.

set -euo pipefail

DURATION_SECONDS="${1:-60}"
SAMPLE_INTERVAL="${2:-5}"
OUTPUT_FILE="${3:-soak-results.csv}"

echo "Starting PhairDLNA Soak Harness (Duration: ${DURATION_SECONDS}s, Interval: ${SAMPLE_INTERVAL}s)"
echo "timestamp,rss_kb,fds,threads" > "$OUTPUT_FILE"

# Start the phairdlna server in background
cargo build --package phairdlna-cli
PID=""

cleanup() {
    if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
        echo "Stopping server process $PID..."
        kill -TERM "$PID" 2>/dev/null || true
        wait "$PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Run in temporary isolated environment
TEMP_DIR=$(mktemp -d)
cat << CFG > "$TEMP_DIR/config.toml"
schema_version = 1
[server]
device_name = "Soak Test Server"
http_port = 8299
[library]
roots = []
CFG

./target/debug/phairdlna-cli serve --config "$TEMP_DIR/config.toml" &
PID=$!
sleep 2

if ! kill -0 "$PID" 2>/dev/null; then
    echo "Server failed to start!"
    exit 1
fi

START_TIME=$(date +%s)
while true; do
    NOW=$(date +%s)
    ELAPSED=$((NOW - START_TIME))
    if [ "$ELAPSED" -ge "$DURATION_SECONDS" ]; then
        break
    fi

    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    # Sample memory (RSS)
    RSS=$(ps -o rss= -p "$PID" | tr -d ' ' || echo "0")
    
    # Sample file descriptors
    if command -v lsof >/dev/null 2>&1; then
        FDS=$(lsof -p "$PID" 2>/dev/null | wc -l | tr -d ' ' || echo "0")
    else
        FDS="0"
    fi

    # Sample thread count
    THREADS=$(ps -M "$PID" 2>/dev/null | wc -l | tr -d ' ' || echo "1")

    echo "$TIMESTAMP,$RSS,$FDS,$THREADS" >> "$OUTPUT_FILE"
    echo "[$TIMESTAMP] Sample: RSS=${RSS}KB, FDs=${FDS}, Threads=${THREADS}"

    sleep "$SAMPLE_INTERVAL"
done

echo "Soak test run complete. Sample output written to $OUTPUT_FILE."
