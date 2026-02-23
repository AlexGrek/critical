#!/usr/bin/env bash
set -euo pipefail

LOG_FILE=$(mktemp /tmp/make_test_run_XXXXXX.log)
RUN=0

cleanup() {
    rm -f "$LOG_FILE"
}

on_error() {
    echo ""
    echo "=== FAILURE on run #$RUN ==="
    echo "=== Full log from failed run: ==="
    echo ""
    cat "$LOG_FILE"
    rm -f "$LOG_FILE"
    exit 1
}

trap on_error ERR

echo "Starting test loop. Log: $LOG_FILE"
echo "Press Ctrl+C to stop."
echo ""

while true; do
    RUN=$((RUN + 1))
    echo -n "Run #$RUN ... "

    > "$LOG_FILE"
    if make test >> "$LOG_FILE" 2>&1; then
        echo "OK"
        > "$LOG_FILE"
    fi
done
