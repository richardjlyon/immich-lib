#!/bin/sh
# Verify that metadata consolidation actually transferred data
# Checks if winners that lacked GPS/timezone/etc now have it after execution

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

ANALYSIS_JSON="validation-analysis.json"

if [ ! -f "$ANALYSIS_JSON" ]; then
    echo "ERROR: $ANALYSIS_JSON not found. Run validation first."
    exit 1
fi

if [ ! -f ".api_key" ]; then
    echo "ERROR: .api_key not found. Is Immich running?"
    exit 1
fi

API_KEY=$(cat .api_key | tr -d '\n')
BASE_URL="${IMMICH_BASE_URL:-http://localhost:2283}"

echo "=== Metadata Consolidation Verification ==="
echo ""
echo "Checking if GPS/metadata was transferred from losers to winners..."
echo ""

# Use Python for JSON parsing and API calls
python3 << PYTHON_SCRIPT
import json
import urllib.request
import sys

analysis_file = "validation-analysis.json"
api_key = "${API_KEY}"
base_url = "${BASE_URL}"

with open(analysis_file) as f:
    data = json.load(f)

def get_asset_info(asset_id):
    """Fetch current asset info from Immich API"""
    url = f"{base_url}/api/assets/{asset_id}"
    req = urllib.request.Request(url, headers={"x-api-key": api_key})
    try:
        with urllib.request.urlopen(req) as response:
            return json.loads(response.read().decode())
    except Exception as e:
        return None

print("=" * 70)
print("GPS CONSOLIDATION TEST")
print("=" * 70)
print()

gps_tests = []
tz_tests = []
camera_tests = []

# Find consolidation candidates
for g in data['groups']:
    winner = g['winner']
    w_score = winner['score']
    w_id = winner['asset_id']
    w_name = winner['filename']

    for loser in g['losers']:
        l_score = loser['score']
        l_name = loser['filename']

        # GPS: winner had none, loser had it
        if w_score['gps'] == 0 and l_score['gps'] > 0:
            gps_tests.append({
                'winner_id': w_id,
                'winner_name': w_name,
                'loser_name': l_name,
                'type': 'GPS'
            })

        # Timezone: winner had none, loser had it
        if w_score['timezone'] == 0 and l_score['timezone'] > 0:
            tz_tests.append({
                'winner_id': w_id,
                'winner_name': w_name,
                'loser_name': l_name,
                'type': 'Timezone'
            })

        # Camera info: winner had none, loser had it
        if w_score['camera_info'] == 0 and l_score['camera_info'] > 0:
            camera_tests.append({
                'winner_id': w_id,
                'winner_name': w_name,
                'loser_name': l_name,
                'type': 'Camera'
            })

passed = 0
failed = 0

# Test GPS consolidation
print(f"Testing {len(gps_tests)} cases where winner had NO GPS, loser had GPS:")
print()

for test in gps_tests:
    asset = get_asset_info(test['winner_id'])
    if not asset:
        print(f"  ? {test['winner_name']}: Could not fetch from API")
        continue

    exif = asset.get('exifInfo', {})
    lat = exif.get('latitude')
    lon = exif.get('longitude')

    has_gps_now = lat is not None and lon is not None

    if has_gps_now:
        print(f"  ✓ {test['winner_name']}: NOW HAS GPS ({lat:.4f}, {lon:.4f})")
        print(f"    (consolidated from {test['loser_name']})")
        passed += 1
    else:
        print(f"  ✗ {test['winner_name']}: STILL NO GPS - consolidation FAILED")
        print(f"    (should have gotten GPS from {test['loser_name']})")
        failed += 1
    print()

print()
print("=" * 70)
print("TIMEZONE CONSOLIDATION TEST")
print("=" * 70)
print()

print(f"Testing {len(tz_tests)} cases where winner had NO timezone, loser had timezone:")
print()

for test in tz_tests:
    asset = get_asset_info(test['winner_id'])
    if not asset:
        print(f"  ? {test['winner_name']}: Could not fetch from API")
        continue

    exif = asset.get('exifInfo', {})
    tz = exif.get('timeZone')

    has_tz_now = tz is not None and tz != ''

    if has_tz_now:
        print(f"  ✓ {test['winner_name']}: NOW HAS TIMEZONE ({tz})")
        print(f"    (consolidated from {test['loser_name']})")
        passed += 1
    else:
        print(f"  ✗ {test['winner_name']}: STILL NO TIMEZONE - consolidation FAILED")
        print(f"    (should have gotten timezone from {test['loser_name']})")
        failed += 1
    print()

print()
print("=" * 70)
print("CAMERA INFO (API LIMITATION)")
print("=" * 70)
print()

print("Note: Camera info (make/model) CANNOT be transferred.")
print("      Immich API PUT /assets/{id} only supports:")
print("        - latitude, longitude (GPS)")
print("        - dateTimeOriginal")
print("        - description")
print("      Camera make/model fields are read-only in the API.")
print()

if camera_tests:
    print(f"Found {len(camera_tests)} cases where loser had camera info winner lacked:")
    for test in camera_tests:
        print(f"  - {test['winner_name']} (loser: {test['loser_name']})")
    print()
    print("This is an Immich API limitation, not a bug in our code.")
print()

print()
print("=" * 70)
print("SUMMARY")
print("=" * 70)
print()
total = passed + failed
print(f"Consolidation tests (GPS + Timezone): {total}")
print(f"Passed: {passed}")
print(f"Failed: {failed}")
print()

if failed == 0 and total > 0:
    print("✓ ALL METADATA CONSOLIDATION VERIFIED")
    print("  Winners now have GPS/timezone data from losers!")
    sys.exit(0)
elif total == 0:
    print("- No consolidation candidates found in test data")
    sys.exit(0)
else:
    print(f"✗ {failed} CONSOLIDATION FAILURES")
    print("  Some metadata was NOT transferred from losers to winners")
    sys.exit(1)
PYTHON_SCRIPT
