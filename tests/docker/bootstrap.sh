#!/bin/sh
# Bootstrap fresh Immich test environment
# Creates clean instance, waits for ready, creates admin user, outputs API key

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

BASE_URL="http://localhost:2283"
ADMIN_EMAIL="admin@test.local"
ADMIN_PASSWORD="testpassword123"
ADMIN_NAME="Test Admin"

echo "=== Immich Test Environment Bootstrap ==="

# Step 1: Clean any existing state
echo "Cleaning existing containers..."
docker compose down -v 2>/dev/null || true

# Step 2: Start services
echo "Starting Immich services..."
docker compose up -d

# Step 3: Wait for ready
echo "Waiting for services to be ready..."
./wait-for-ready.sh

# Step 4: Create admin user
echo "Creating admin user..."
SIGNUP_RESPONSE=$(curl -s -X POST "${BASE_URL}/api/auth/admin-sign-up" \
    -H "Content-Type: application/json" \
    -d "{
        \"email\": \"${ADMIN_EMAIL}\",
        \"password\": \"${ADMIN_PASSWORD}\",
        \"name\": \"${ADMIN_NAME}\"
    }")

# Check if admin already exists (response will differ)
if echo "$SIGNUP_RESPONSE" | grep -q "error"; then
    echo "Admin may already exist, attempting login..."
fi

# Step 5: Login to get access token
echo "Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "${BASE_URL}/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{
        \"email\": \"${ADMIN_EMAIL}\",
        \"password\": \"${ADMIN_PASSWORD}\"
    }")

ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"accessToken":"[^"]*"' | cut -d'"' -f4)

if [ -z "$ACCESS_TOKEN" ]; then
    echo "ERROR: Failed to get access token"
    echo "Response: $LOGIN_RESPONSE"
    exit 1
fi

# Step 6: Create API key with full permissions
echo "Creating API key..."
API_KEY_RESPONSE=$(curl -s -X POST "${BASE_URL}/api/api-keys" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -d '{"name": "test-harness", "permissions": ["all"]}')

API_KEY=$(echo "$API_KEY_RESPONSE" | grep -o '"secret":"[^"]*"' | cut -d'"' -f4)

if [ -z "$API_KEY" ]; then
    echo "ERROR: Failed to create API key"
    echo "Response: $API_KEY_RESPONSE"
    exit 1
fi

# Save API key to file for reliable passing to seed script
echo "${API_KEY}" > .api_key
chmod 600 .api_key

# Step 7: Configure duplicate detection threshold for synthetic fixtures
# Default 0.01 is too strict for scale/quality-based synthetic duplicates
echo "Configuring duplicate detection threshold..."
CONFIG=$(curl -s -H "x-api-key: ${API_KEY}" "${BASE_URL}/api/system-config")
# Update maxDistance from 0.01 to 0.06 for synthetic fixture detection
UPDATED_CONFIG=$(echo "$CONFIG" | sed 's/"maxDistance":0.01/"maxDistance":0.06/')
curl -s -X PUT -H "x-api-key: ${API_KEY}" -H "Content-Type: application/json" \
    -d "$UPDATED_CONFIG" "${BASE_URL}/api/system-config" > /dev/null

echo ""
echo "=== Immich Ready ==="
echo "URL: ${BASE_URL}"
echo "Admin: ${ADMIN_EMAIL} / ${ADMIN_PASSWORD}"
echo "API Key: ${API_KEY}"
echo ""
echo "Next step - seed fixtures:"
echo "  ./seed-fixtures.sh"
echo ""
echo "(API key saved to .api_key file)"
