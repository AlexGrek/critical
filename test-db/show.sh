#!/usr/bin/env bash
#
# Fetch and display all data from the dev database using cr1t.
# Logs in as admin (godmode) to see everything.
#
# Prerequisites:
#   - Backend running on BACKEND_URL (default: http://localhost:3742)
#   - Database populated (make populate-db)
#
# Usage:
#   bash test-db/show.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_URL="${BACKEND_URL:-http://localhost:3742}"
ADMIN_USER="admin"
ADMIN_PASS="admin123"

CR1T="${CR1T:-$(dirname "$SCRIPT_DIR")/target/debug/cr1t}"
if [[ ! -x "$CR1T" ]]; then
    echo ">>> cr1t binary not found at $CR1T — building..."
    (cd "$SCRIPT_DIR/.." && cargo build --bin cr1t)
fi

# Ensure we're logged in as admin
echo "$ADMIN_PASS" | "$CR1T" login --url "$BACKEND_URL" --user "$ADMIN_USER" >/dev/null 2>&1

KINDS="users groups memberships projects permissions"

for kind in $KINDS; do
    echo "========================================"
    echo "  $kind"
    echo "========================================"
    "$CR1T" get "$kind" 2>&1 || echo "  (none or error)"
    echo ""
done
