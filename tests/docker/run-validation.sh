#!/bin/sh
# Run full validation workflow: analyze -> execute against Docker Immich instance
# Preserves end state for verification in 07-02

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Output files
ANALYSIS_JSON="validation-analysis.json"
BACKUP_DIR="validation-backups"
REPORT_FILE="validation-report.txt"

echo "=== Immich Duplicates Validation Run ==="
echo "Started: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo ""

# Step 1: Source environment (or use .api_key from bootstrap)
if [ -f ".api_key" ]; then
    API_KEY=$(cat .api_key | tr -d '\n')
else
    echo "ERROR: No .api_key file found. Run bootstrap.sh first."
    exit 1
fi

BASE_URL="${IMMICH_BASE_URL:-http://localhost:2283}"

echo "API URL: ${BASE_URL}"
echo ""

# Step 2: Bootstrap (start Docker, create admin, get API key)
echo "=== Step 1: Bootstrap Environment ==="
./bootstrap.sh

# Re-read API key after bootstrap (it gets regenerated)
API_KEY=$(cat .api_key | tr -d '\n')

# Step 3: Seed fixtures (upload test images, wait for ML)
echo ""
echo "=== Step 2: Seed Fixtures ==="
./seed-fixtures.sh

# Step 4: Run analyze command
echo ""
echo "=== Step 3: Analyze Duplicates ==="

# Build the binary first if needed
echo "Building immich-dupes..."
(cd ../.. && cargo build --release)

# Run analyze
echo "Running analysis..."
../../target/release/immich-dupes \
    --url "${BASE_URL}" \
    --api-key "${API_KEY}" \
    analyze \
    --output "${ANALYSIS_JSON}"

# Check analysis output
if [ -f "${ANALYSIS_JSON}" ]; then
    group_count=$(grep -o '"duplicate_id"' "${ANALYSIS_JSON}" | wc -l | tr -d ' ')
    echo "Analysis complete: ${group_count} duplicate groups found"
else
    echo "ERROR: Analysis JSON not created"
    exit 1
fi

# Step 5: Create backup directory
echo ""
echo "=== Step 4: Execute Workflow ==="
rm -rf "${BACKUP_DIR}"
mkdir -p "${BACKUP_DIR}"

# Step 6: Run execute command (capture output)
echo "Running execute (download backups, delete losers)..."
../../target/release/immich-dupes \
    --url "${BASE_URL}" \
    --api-key "${API_KEY}" \
    execute \
    --input "${ANALYSIS_JSON}" \
    --backup-dir "${BACKUP_DIR}" \
    --yes \
    2>&1 | tee "${REPORT_FILE}"

# Step 7: Summary
echo ""
echo "=== Validation Complete ==="
echo ""
echo "Analysis JSON: ${SCRIPT_DIR}/${ANALYSIS_JSON}"
echo "Backup files:  ${SCRIPT_DIR}/${BACKUP_DIR}/"
echo "Report:        ${SCRIPT_DIR}/${REPORT_FILE}"
echo ""

# Count backup files
backup_count=$(find "${BACKUP_DIR}" -type f | wc -l | tr -d ' ')
echo "Backup files downloaded: ${backup_count}"

# Check remaining duplicates
remaining=$(curl -s "${BASE_URL}/api/duplicates" -H "x-api-key: ${API_KEY}" | grep -o '"duplicateId"' | wc -l | tr -d ' ')
echo "Remaining duplicate groups: ${remaining}"
echo ""
echo "Finished: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo ""
echo "NOTE: Docker containers left running for verification."
echo "Run './teardown.sh' when done inspecting."
