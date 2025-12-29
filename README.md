# immich-dupes

[Changelog](CHANGELOG.md)

> **Warning: External Libraries Not Supported**
>
> This tool's metadata consolidation (GPS, timezone transfer) **does not work** with Immich External Libraries (library imports). Immich reads metadata from the source files for external libraries, so API updates don't persist.
>
> This tool only works correctly with **uploaded assets** (files uploaded via the Immich app/web/CLI that Immich manages directly).

A Rust CLI tool for intelligent Immich duplicate management. Unlike Immich's built-in de-duplication which favors larger files, this tool selects the highest-quality image by dimensions while preserving metadata through consolidation.

**Features:**
- Smart duplicate removal with metadata preservation
- iPhone letterbox detection (4:3 vs 16:9 crop pairs)

## The Problem

Immich's duplicate detection works well, but its resolution logic is naive: keep the largest file, trash the rest. This ignores that:

- Smaller files often have richer EXIF metadata (GPS, timezone, camera info)
- Photos from different sources have varying metadata completeness
- Once deleted, metadata is gone forever

With 2000+ duplicates, manual review isn't feasible. This tool automates smart selection with full backup safety.

## How It Works

1. **Analyze** - Scans Immich duplicates, scores by metadata completeness, outputs reviewable JSON
2. **Execute** - Downloads backups, consolidates metadata to winners, deletes losers
3. **Verify** - Validates winners exist and losers are deleted
4. **Restore** - Re-uploads backed-up files if needed

### Winner Selection

The tool selects winners by **largest dimensions** (width × height), ensuring you keep the highest quality image. Metadata from losers (GPS, timezone) is consolidated to the winner before deletion.

## Installation

### Homebrew (macOS)

```bash
brew install richardjlyon/tap/immich-dupes
```

### Pre-built binaries

Download from [Releases](https://github.com/richardjlyon/immich-lib/releases/latest) for Linux, macOS, and Windows.

### From source

```bash
cargo install --git https://github.com/richardjlyon/immich-lib
```

## Usage

### Setup

Set your Immich credentials:

```bash
export IMMICH_URL="https://your-immich-server.com"
export IMMICH_API_KEY="your-api-key"
```

Or use command-line flags: `-u <URL> -a <API_KEY>`

### Analyze Duplicates

```bash
immich-dupes analyze -o duplicates.json
```

This outputs a JSON file with all duplicate groups, scored assets, and conflict detection. Review the file to spot-check decisions.

### Execute Removal

```bash
immich-dupes execute -i duplicates.json -b ./backups
```

This will:
1. Download all loser files to `./backups/`
2. Consolidate GPS/timezone metadata from losers to winners
3. Move losers to Immich trash (or permanently delete with `--force`)

**Options:**
- `--skip-review` - Skip groups with metadata conflicts that need manual review
- `--yes` - Skip confirmation prompt
- `--rate-limit <N>` - Max API requests per second (default: 10)
- `--concurrent <N>` - Max concurrent operations (default: 5)

### Verify Results

```bash
immich-dupes verify duplicates.json
```

Checks that all winners still exist and all losers have been deleted.

### Restore Backups

If something went wrong:

```bash
immich-dupes restore -b ./backups
```

Re-uploads all backed-up files to Immich.

## Example Workflow

```bash
# 1. Analyze your duplicates
immich-dupes analyze -o analysis.json

# 2. Review the JSON (optional)
cat analysis.json | jq '.groups | length'  # See group count
cat analysis.json | jq '.needs_review_count'  # See conflicts

# 3. Execute with backups
immich-dupes execute -i analysis.json -b ./backups --skip-review

# 4. Verify the results
immich-dupes verify analysis.json

# 5. If needed, restore
immich-dupes restore -b ./backups
```

## iPhone Letterbox Detection

iPhones can save photos in both 4:3 (full sensor) and 16:9 (cropped) formats simultaneously. This creates near-duplicate pairs that Immich's CLIP-based detection doesn't catch because they're semantically identical but have different aspect ratios.

The `letterbox` command finds and removes these pairs:

```bash
# 1. Analyze for letterbox pairs
immich-dupes letterbox analyze -o letterbox.json

# 2. Review the analysis
cat letterbox.json | jq '.pairs | length'  # Count of pairs found

# 3. Execute removal (backs up 16:9 crops, keeps 4:3 originals)
immich-dupes letterbox execute -i letterbox.json -b ./letterbox-backups

# 4. Verify results
immich-dupes letterbox verify letterbox.json
```

### How It Works

1. **Detection** - Matches photos by timestamp + camera make/model + GPS
2. **Selection** - Always keeps the 4:3 version (more pixels, full scene)
3. **Backup** - Downloads 16:9 crops before deletion
4. **Cleanup** - Moves 16:9 crops to trash (or deletes with `--force`)

### Output

The analysis shows:
- Pairs found (4:3 + 16:9 matches)
- Space recoverable (size of 16:9 files to delete)
- Skipped non-iPhone assets
- Skipped ambiguous groups (multiple candidates)

## What Gets Consolidated

When a loser has metadata the winner lacks, it's transferred:

| Field | Consolidated |
|-------|--------------|
| GPS coordinates | Yes |
| Timezone | Yes |
| Description | Yes |
| Camera make/model | No (Immich API limitation) |

## Safety Features

- **Two-stage workflow** - Review JSON before any deletions
- **Full backups** - Original files downloaded before deletion
- **Trash by default** - Uses Immich trash, not permanent delete
- **Conflict detection** - Flags groups with conflicting metadata for review
- **Verification** - Confirm end state matches expectations
- **Restore capability** - Re-upload backups if needed

## License

MIT
