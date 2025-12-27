#!/bin/sh
# Seed test fixtures into Immich
# Uploads all fixture images and waits for ML processing

# Don't use set -e - we handle errors explicitly

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FIXTURES_DIR="${SCRIPT_DIR}/../fixtures"

# Configuration - read API key from file (most reliable), env, or arg
if [ -f "${SCRIPT_DIR}/.api_key" ]; then
    API_KEY=$(cat "${SCRIPT_DIR}/.api_key" | tr -d '\n')
elif [ -n "$IMMICH_API_KEY" ]; then
    API_KEY="$IMMICH_API_KEY"
elif [ -n "$1" ]; then
    API_KEY="$1"
else
    API_KEY=""
fi
BASE_URL="${IMMICH_BASE_URL:-${2:-http://localhost:2283}}"

if [ -z "$API_KEY" ]; then
    echo "ERROR: No API key found."
    echo ""
    echo "Run ./bootstrap.sh first, which saves the key to .api_key"
    echo "Or set IMMICH_API_KEY environment variable"
    exit 1
fi

# Verify API key works before proceeding
echo "Verifying API key..."
verify_response=$(curl -s "${BASE_URL}/api/server/ping" -H "x-api-key: ${API_KEY}" 2>/dev/null)
if ! echo "$verify_response" | grep -q "pong"; then
    echo "ERROR: API key verification failed"
    echo "Response: $verify_response"
    echo ""
    echo "Make sure the API key is correct and Immich is running."
    exit 1
fi

echo "=== Seeding Fixtures to Immich ==="
echo "URL: ${BASE_URL}"
echo "Fixtures: ${FIXTURES_DIR}"

# Count scenarios and assets
scenario_count=0
asset_count=0
failed_count=0

# Process each scenario directory
for scenario_dir in "$FIXTURES_DIR"/*/; do
    [ -d "$scenario_dir" ] || continue
    
    scenario_name=$(basename "$scenario_dir")
    
    # Skip if not a test scenario (check for manifest.json)
    if [ ! -f "${scenario_dir}/manifest.json" ]; then
        continue
    fi
    
    scenario_count=$((scenario_count + 1))
    echo ""
    echo "Processing scenario: ${scenario_name}"
    
    # Upload each image/video in the scenario
    for file in "$scenario_dir"/*; do
        [ -f "$file" ] || continue
        
        filename=$(basename "$file")
        
        # Skip non-media files
        case "$filename" in
            *.jpg|*.jpeg|*.png|*.heic|*.mp4|*.mov)
                ;;
            *)
                continue
                ;;
        esac
        
        # Create unique device asset ID
        device_asset_id="${scenario_name}_${filename}"
        
        # Get file timestamps
        file_date=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
        
        # Upload via multipart form
        response=$(curl -s -X POST "${BASE_URL}/api/assets" \
            -H "x-api-key: ${API_KEY}" \
            -F "assetData=@${file}" \
            -F "deviceAssetId=${device_asset_id}" \
            -F "deviceId=test-harness" \
            -F "fileCreatedAt=${file_date}" \
            -F "fileModifiedAt=${file_date}" \
            -F "isFavorite=false")
        
        # Check for success (response should have "id" field)
        if echo "$response" | grep -q '"id"'; then
            asset_count=$((asset_count + 1))
            printf "  ✓ %s\n" "$filename"
        else
            failed_count=$((failed_count + 1))
            printf "  ✗ %s: %s\n" "$filename" "$response"
        fi
    done
done

echo ""
echo "=== Upload Complete ==="
echo "Scenarios: ${scenario_count}"
echo "Assets uploaded: ${asset_count}"
if [ $failed_count -gt 0 ]; then
    echo "Failed: ${failed_count}"
fi

# Wait for ML jobs to complete
echo ""
echo "Waiting for ML processing..."

max_wait=300
elapsed=0
while [ $elapsed -lt $max_wait ]; do
    # Check job queue status
    jobs_response=$(curl -s "${BASE_URL}/api/jobs" \
        -H "x-api-key: ${API_KEY}")
    
    # Check if any jobs are active or waiting
    # Look for jobCounts with active > 0 or waiting > 0
    active=$(echo "$jobs_response" | grep -o '"active":[0-9]*' | grep -o '[0-9]*' | awk '{sum+=$1}END{print sum+0}')
    waiting=$(echo "$jobs_response" | grep -o '"waiting":[0-9]*' | grep -o '[0-9]*' | awk '{sum+=$1}END{print sum+0}')
    
    total_pending=$((active + waiting))
    
    if [ "$total_pending" -eq 0 ]; then
        echo "ML processing complete!"
        break
    fi
    
    printf "Jobs pending: %d (active: %d, waiting: %d)\r" "$total_pending" "$active" "$waiting"
    sleep 5
    elapsed=$((elapsed + 5))
done

if [ $elapsed -ge $max_wait ]; then
    echo ""
    echo "WARNING: Timeout waiting for ML jobs"
fi

# Wait for duplicate detection to complete (runs after CLIP embeddings)
echo ""
echo "Waiting for duplicate detection..."
sleep 10

dup_count=0
dup_wait=0
while [ $dup_wait -lt 60 ]; do
    duplicates_response=$(curl -s "${BASE_URL}/api/duplicates" \
        -H "x-api-key: ${API_KEY}")

    dup_count=$(echo "$duplicates_response" | grep -o '"duplicateId":"[^"]*"' | sort -u | wc -l | tr -d ' ')

    if [ "$dup_count" -gt 0 ]; then
        break
    fi

    printf "Waiting for duplicates...\r"
    sleep 5
    dup_wait=$((dup_wait + 5))
done

echo "Duplicate groups found: ${dup_count}"
echo ""
echo "=== Seeding Complete ==="
echo ""
echo "Verify in browser:"
echo "  URL:  ${BASE_URL}"
echo "  User: admin@test.local"
echo "  Pass: testpassword123"
