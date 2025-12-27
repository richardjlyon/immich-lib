#!/bin/bash
#
# Record Immich API responses for use in unit tests.
#
# Prerequisites:
#   - Docker Immich running (./bootstrap.sh)
#   - Fixtures seeded (./seed-fixtures.sh)
#   - Duplicate detection complete
#
# Output:
#   - ../fixtures/recorded/duplicates.json
#
# Usage:
#   ./record-fixtures.sh
#
# Re-run this script whenever:
#   - Fixture images change
#   - Immich API format changes
#   - New test scenarios are added

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FIXTURES_DIR="$SCRIPT_DIR/../fixtures"
OUTPUT_DIR="$FIXTURES_DIR/recorded"

# Check for API key
if [ ! -f "$SCRIPT_DIR/.api_key" ]; then
    echo "Error: .api_key file not found"
    echo "Run ./bootstrap.sh first"
    exit 1
fi

API_KEY=$(cat "$SCRIPT_DIR/.api_key")
BASE_URL="http://localhost:2283"

# Verify Immich is running
echo "Checking Immich connectivity..."
if ! curl -s -f "$BASE_URL/api/server/ping" > /dev/null 2>&1; then
    echo "Error: Immich not responding at $BASE_URL"
    echo "Run ./bootstrap.sh && ./seed-fixtures.sh first"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Record duplicates endpoint
echo "Recording /api/duplicates..."
DUPLICATES=$(curl -s -H "x-api-key: $API_KEY" "$BASE_URL/api/duplicates")

# Check we got data
GROUP_COUNT=$(echo "$DUPLICATES" | python3 -c "import json,sys; print(len(json.load(sys.stdin)))")
if [ "$GROUP_COUNT" -eq 0 ]; then
    echo "Error: No duplicate groups found"
    echo "Wait for duplicate detection to complete"
    exit 1
fi

# Save with pretty formatting for readability
echo "$DUPLICATES" | python3 -m json.tool > "$OUTPUT_DIR/duplicates.json"

echo ""
echo "=== Recording Complete ==="
echo "Duplicate groups: $GROUP_COUNT"
echo "Output: $OUTPUT_DIR/duplicates.json"
echo ""
echo "Commit this file to use in unit tests."
