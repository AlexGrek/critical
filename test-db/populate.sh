#!/usr/bin/env bash
#
# Populate a dev database with test data using cr1t apply.
#
# Prerequisites:
#   - Backend running on BACKEND_URL (default: http://localhost:3742)
#   - cr1t binary built (cargo build --bin cr1t)
#
# Usage:
#   bash test-db/populate.sh
#   BACKEND_URL=http://myhost:3742 bash test-db/populate.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_URL="${BACKEND_URL:-http://localhost:3742}"
ADMIN_USER="admin"
ADMIN_PASS="admin123"

# Resolve cr1t binary
CR1T="${CR1T:-$(dirname "$SCRIPT_DIR")/target/debug/cr1t}"
if [[ ! -x "$CR1T" ]]; then
    echo ">>> cr1t binary not found at $CR1T — building..."
    (cd "$SCRIPT_DIR/.." && cargo build --bin cr1t)
fi

echo ">>> Backend: $BACKEND_URL"
echo ">>> cr1t:    $CR1T"
echo ""

# --- Step 1: Register bootstrap admin user ---
echo ">>> Registering bootstrap admin user '$ADMIN_USER'..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    -X POST "$BACKEND_URL/api/v1/register" \
    -H "Content-Type: application/json" \
    -d "{\"user\": \"$ADMIN_USER\", \"password\": \"$ADMIN_PASS\"}")

if [[ "$HTTP_CODE" == "201" ]]; then
    echo "    Created."
elif [[ "$HTTP_CODE" == "409" ]]; then
    echo "    Already exists (OK)."
else
    echo "    Failed (HTTP $HTTP_CODE). Is the backend running?"
    exit 1
fi

# --- Step 2: Login as admin via cr1t ---
echo ">>> Logging in as '$ADMIN_USER'..."
echo "$ADMIN_PASS" | "$CR1T" login --url "$BACKEND_URL" --user "$ADMIN_USER" 2>&1 | sed 's/^/    /'

# --- Step 3: Apply YAML files in order ---
for yaml in "$SCRIPT_DIR"/0*.yaml; do
    basename=$(basename "$yaml")
    echo ">>> Applying $basename..."
    "$CR1T" apply -f "$yaml" 2>&1 | sed 's/^/    /'
done

echo ""
echo ">>> Database populated successfully."
echo ""
echo "    Users:"
echo "      admin / $ADMIN_PASS  (godmode)"
echo "      alice / alice123     (engineering lead, group creator)"
echo "      bob   / bob123       (senior dev, group creator)"
echo "      carol / carol123     (devops engineer, group creator)"
echo "      dave  / dave123      (junior dev)"
echo "      eve   / eve123       (QA engineer)"
echo ""
echo "    Groups: platform_admins, engineering, devops, viewers"
echo "    Projects: critical, infra, docs"
