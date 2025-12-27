#!/bin/sh
# Verify that winners in database are better than backup (loser) files
# Compares dimensions and metadata scores to confirm correct selection

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

ANALYSIS_JSON="validation-analysis.json"
BACKUP_DIR="validation-backups"

if [ ! -f "$ANALYSIS_JSON" ]; then
    echo "ERROR: $ANALYSIS_JSON not found. Run validation first."
    exit 1
fi

if [ ! -d "$BACKUP_DIR" ]; then
    echo "ERROR: $BACKUP_DIR not found. Run validation first."
    exit 1
fi

echo "=== Winner vs Loser Verification ==="
echo ""
echo "Criterion: Winner should have larger or equal dimensions (width × height)"
echo "Tie-breaker: Larger file size"
echo ""

# Use Python for JSON parsing (more reliable than shell)
python3 << 'PYTHON_SCRIPT'
import json
import os
from pathlib import Path

analysis_file = "validation-analysis.json"
backup_dir = Path("validation-backups")

with open(analysis_file) as f:
    data = json.load(f)

total_groups = len(data["groups"])
correct = 0
incorrect = 0
ties = 0

print(f"Analyzing {total_groups} duplicate groups...\n")
print("-" * 80)

for group in data["groups"]:
    dup_id = group["duplicate_id"][:8]
    winner = group["winner"]
    losers = group["losers"]

    w_dims = winner.get("dimensions", [0, 0])
    w_pixels = w_dims[0] * w_dims[1] if w_dims else 0
    w_size = winner.get("file_size", 0)
    w_name = winner["filename"]

    group_correct = True
    group_details = []

    for loser in losers:
        l_dims = loser.get("dimensions", [0, 0])
        l_pixels = l_dims[0] * l_dims[1] if l_dims else 0
        l_size = loser.get("file_size", 0)
        l_name = loser["filename"]

        # Check backup file exists
        backup_files = list(backup_dir.glob(f"*_{l_name}"))
        backup_exists = len(backup_files) > 0

        # Compare: winner should have >= pixels, or if equal, >= file size
        if w_pixels > l_pixels:
            status = "✓"
            reason = f"winner {w_dims[0]}×{w_dims[1]} > loser {l_dims[0]}×{l_dims[1]}"
        elif w_pixels == l_pixels:
            if w_size >= l_size:
                status = "≈"
                reason = f"same dims, winner size {w_size:,} >= loser {l_size:,}"
                ties += 1
            else:
                status = "✗"
                reason = f"same dims but loser size {l_size:,} > winner {w_size:,}"
                group_correct = False
        else:
            status = "✗"
            reason = f"LOSER {l_dims[0]}×{l_dims[1]} > winner {w_dims[0]}×{w_dims[1]}"
            group_correct = False

        backup_status = "backup ✓" if backup_exists else "backup MISSING"
        group_details.append(f"  {status} {l_name}: {reason} [{backup_status}]")

    if group_correct:
        correct += 1
        print(f"✓ {dup_id} | Winner: {w_name} ({w_dims[0]}×{w_dims[1]}, {w_size:,}b)")
    else:
        incorrect += 1
        print(f"✗ {dup_id} | Winner: {w_name} ({w_dims[0]}×{w_dims[1]}, {w_size:,}b)")

    for detail in group_details:
        print(detail)
    print()

print("-" * 80)
print(f"\n=== Summary ===")
print(f"Total groups:     {total_groups}")
print(f"Correct winners:  {correct}")
print(f"Dimension ties:   {ties} (resolved by file size)")
print(f"Incorrect:        {incorrect}")
print()

if incorrect == 0:
    print("✓ ALL WINNERS VERIFIED - larger dimensions kept in every case")
    exit(0)
else:
    print(f"✗ {incorrect} GROUPS HAD INCORRECT WINNER SELECTION")
    exit(1)
PYTHON_SCRIPT
