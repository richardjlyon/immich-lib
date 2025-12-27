#!/bin/sh
# Wait for Immich to be ready
# Polls health endpoints until services are up or timeout

set -e

BASE_URL="${BASE_URL:-http://localhost:2283}"
TIMEOUT="${TIMEOUT:-120}"
INTERVAL=2

echo "Waiting for Immich at ${BASE_URL} (timeout: ${TIMEOUT}s)..."

elapsed=0
while [ $elapsed -lt $TIMEOUT ]; do
    # Check server ping
    if curl -s "${BASE_URL}/api/server/ping" 2>/dev/null | grep -q "pong"; then
        echo "Server responding..."
        
        # Check ML is available via features endpoint (duplicateDetection requires ML)
        if curl -s "${BASE_URL}/api/server/features" 2>/dev/null | grep -q '"duplicateDetection":true'; then
            echo "Immich ready! (duplicate detection enabled)"
            exit 0
        fi
    fi
    
    sleep $INTERVAL
    elapsed=$((elapsed + INTERVAL))
    printf "."
done

echo ""
echo "ERROR: Timeout waiting for Immich to be ready after ${TIMEOUT}s"
exit 1
