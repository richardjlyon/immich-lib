#!/bin/bash
# Test script for restore command
# Tests the restore functionality against Docker Immich instance

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Load API key
API_KEY=$(cat "$SCRIPT_DIR/.api_key")
IMMICH_URL="http://localhost:2283"
BACKUP_DIR="$SCRIPT_DIR/validation-backups"

echo "=== Restore Command Test ==="
echo "Server: $IMMICH_URL"
echo "Backup dir: $BACKUP_DIR"
echo ""

# Count files in backup dir
FILE_COUNT=$(ls -1 "$BACKUP_DIR"/*.jpg "$BACKUP_DIR"/*.mp4 2>/dev/null | wc -l | tr -d ' ')
echo "Files in backup directory: $FILE_COUNT"
echo ""

# Get initial asset count
INITIAL_COUNT=$(curl -s -H "x-api-key: $API_KEY" "$IMMICH_URL/api/assets/statistics" | grep -o '"total":[0-9]*' | cut -d: -f2)
echo "Initial asset count in Immich: $INITIAL_COUNT"
echo ""

# Run dry-run first
echo "=== Running Dry-Run ==="
IMMICH_URL="$IMMICH_URL" IMMICH_API_KEY="$API_KEY" "$PROJECT_ROOT/target/debug/immich-dupes" restore \
    --backup-dir "$BACKUP_DIR" \
    --dry-run
echo ""

# Ask for confirmation before actual restore
read -p "Proceed with actual restore? (y/N) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Run actual restore
echo ""
echo "=== Running Actual Restore ==="
IMMICH_URL="$IMMICH_URL" IMMICH_API_KEY="$API_KEY" "$PROJECT_ROOT/target/debug/immich-dupes" restore \
    --backup-dir "$BACKUP_DIR"
echo ""

# Get final asset count
FINAL_COUNT=$(curl -s -H "x-api-key: $API_KEY" "$IMMICH_URL/api/assets/statistics" | grep -o '"total":[0-9]*' | cut -d: -f2)
echo "Final asset count in Immich: $FINAL_COUNT"
echo "Assets added: $((FINAL_COUNT - INITIAL_COUNT))"
echo ""

echo "=== Test Complete ==="
echo "Restore command tested successfully."
