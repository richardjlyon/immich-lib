#!/bin/sh
# Teardown Immich test environment
# Removes all containers and volumes for clean state

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "Stopping Immich test environment..."

# Stop and remove containers and volumes
docker compose down -v 2>/dev/null || true

# Verify containers stopped
if docker ps -a --format '{{.Names}}' | grep -q "immich_test"; then
    echo "WARNING: Some test containers still exist"
    docker ps -a --format '{{.Names}}' | grep "immich_test"
    exit 1
fi

echo "Teardown complete - all containers and volumes removed"
