# Test Fixtures Manifest

Generated: 2025-12-27

## Summary

- **Total scenarios:** 34
- **Generated:** 32 (fully working)
- **Partial:** 2 (X6, X8 - some files skipped due to encoding limitations)

## Regeneration

```bash
./target/debug/immich-dupes generate-fixtures
```

Or regenerate specific categories:
```bash
./target/debug/immich-dupes generate-fixtures --scenario W  # Winner Selection
./target/debug/immich-dupes generate-fixtures --scenario C  # Consolidation
./target/debug/immich-dupes generate-fixtures --scenario F  # Conflicts
./target/debug/immich-dupes generate-fixtures --scenario X  # Edge Cases
```

## Winner Selection (W1-W8)

Tests for dimension-based winner selection algorithm.

| Scenario | Files | Status | Description |
|----------|-------|--------|-------------|
| W1 | w1_large.jpg, w1_small.jpg | ✓ | Clear dimension winner (2000x1500 vs 1000x750) |
| W2 | w2_a.jpg, w2_b.jpg | ✓ | Same dimensions, different file size |
| W3 | w3_a.jpg, w3_b.jpg | ✓ | Same dimensions, same file size |
| W4 | w4_with_dims.jpg, w4_no_dims.jpg | ✓ | Some assets missing dimensions |
| W5 | w5_no_dims.jpg, w5_with_dims.jpg | ✓ | Only one asset has dimensions |
| W6 | w6_a.jpg, w6_b.jpg | ✓ | All assets missing dimensions |
| W7 | w7_small.jpg, w7_large.jpg, w7_medium.jpg | ✓ | Three or more duplicates |
| W8 | w8_wide.jpg, w8_tall.jpg | ✓ | Same pixel count, different aspect ratio |

## Consolidation (C1-C8)

Tests for metadata consolidation from losers to winner.

| Scenario | Files | Status | Description |
|----------|-------|--------|-------------|
| C1 | c1_winner_no_gps.jpg, c1_loser_has_gps.jpg | ✓ | Winner lacks GPS, loser has it |
| C2 | c2_winner_no_dt.jpg, c2_loser_has_dt.jpg | ✓ | Winner lacks datetime, loser has it |
| C3 | c3_winner_no_desc.jpg, c3_loser_has_desc.jpg | ✓ | Winner lacks description, loser has it |
| C4 | c4_winner_bare.jpg, c4_loser_rich.jpg | ✓ | Winner lacks all metadata, loser has everything |
| C5 | c5_a_gps.jpg, c5_b_gps.jpg | ✓ | Both have same GPS (no consolidation needed) |
| C6 | c6_winner.jpg, c6_loser_gps.jpg, c6_loser_dt.jpg | ✓ | Multiple losers contribute different metadata |
| C7 | c7_winner.jpg, c7_loser.jpg | ✓ | No loser has the needed metadata |
| C8 | c8_winner_full.jpg, c8_loser_bare.jpg | ✓ | Winner already has all metadata |

## Conflicts (F1-F7)

Tests for conflict detection when metadata differs between assets.

| Scenario | Files | Status | Description |
|----------|-------|--------|-------------|
| F1 | f1_london.jpg, f1_paris.jpg | ✓ | GPS conflict (London vs Paris) |
| F2 | f2_pos_a.jpg, f2_pos_b.jpg | ✓ | GPS within threshold (~5m, should NOT conflict) |
| F3 | f3_tz_a.jpg, f3_tz_b.jpg | ✓ | Timezone conflict (UTC vs PST) |
| F4 | f4_canon.jpg, f4_nikon.jpg | ✓ | Camera conflict (Canon vs Nikon) |
| F5 | f5_morning.jpg, f5_evening.jpg | ✓ | Capture time conflict (12h difference) |
| F6 | f6_a.jpg, f6_b.jpg | ✓ | Multiple conflicts (GPS + camera + timezone) |
| F7 | f7_a.jpg, f7_b.jpg | ✓ | No conflicts (metadata matches) |

## Edge Cases (X1-X11)

Tests for unusual scenarios and format handling.

| Scenario | Files | Status | Description |
|----------|-------|--------|-------------|
| X1 | x1_single.jpg | ✓ | Single asset group (degenerate case) |
| X2 | x2_dup_00.jpg ... x2_dup_11.jpg | ✓ | Large group (12 duplicates) |
| X3 | x3_large.jpg, x3_small.jpg | ✓ | Large file (48MP - 8000x6000) |
| X4 | x4_photo (1).jpg, x4_photo-copy_2024.jpg | ✓ | Special characters in filename |
| X5 | x5_video_hd.mp4, x5_video_sd.mp4 | ✓ | Video duplicates (MP4) |
| X6 | x6_photo_converted.jpg | ⚠️ | HEIC skipped (encoder not available) |
| X7 | x7_image.png, x7_image.jpg | ✓ | PNG format handling |
| X8 | x8_photo.jpg | ⚠️ | RAW (.cr3) skipped (encoder not available) |
| X9 | x9_unicode.jpg, x9_plain.jpg | ✓ | Unicode in description (Japanese, emoji) |
| X10 | x10_old.jpg, x10_scan.jpg | ✓ | Very old date (1985) |
| X11 | x11_future.jpg, x11_normal.jpg | ✓ | Future date (2030) |

## Notes

### Skipped Files

**X6 (HEIC):** HEIC encoding requires platform-specific encoders (Apple frameworks on macOS, or libheif with specific codecs). The scenario still tests HEIC vs JPEG comparison using the converted JPEG only.

**X8 (RAW):** RAW formats like CR3, NEF, ARW require proprietary encoders. The scenario tests the presence of camera metadata even without the actual RAW file.

### Dependencies

- **exiftool:** Required for embedding EXIF metadata
- **ffmpeg:** Required for video generation (X5)
- **image crate:** Used for JPEG/PNG generation

### Fixture Structure

Each scenario directory contains:
- Image/video files as specified
- `manifest.json` with scenario metadata:
  - `scenario`: Scenario code (e.g., "W1")
  - `description`: What the scenario tests
  - `images`: List of files in the group
  - `expected_winner`: Filename of the expected winner
