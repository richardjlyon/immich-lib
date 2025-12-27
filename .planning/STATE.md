# Project State

## Project Summary

**Building:** A Rust library for the Immich API focused on duplicate management, paired with a binary tool that prioritizes metadata completeness over file size when selecting which duplicate to keep.

**Core requirements:**
- Library successfully authenticates and queries Immich duplicate API
- Analysis correctly identifies metadata-richer files that Immich would discard
- Generated JSON is complete, auditable, and human-reviewable
- Execution downloads originals before any deletion
- Zero metadata loss—no file with richer EXIF deleted in favor of metadata-poor alternative

**Constraints:**
- Pure Rust—library and binary, no Python/JS components
- Relies on Immich's duplicate detection being correct
- Visualizer must stay lightweight

## Current Position

Phase: 6 of 7 (Synthetic Integration Tests)
Plan: 1 of 3 in 06-05 section (7 of 8 total phase plans)
Status: In progress
Last activity: 2025-12-27 - Completed 06-05-01-PLAN.md (Test Harness)

Progress: █████████████████░ 94% (17/18 plans)

## Performance Metrics

**Velocity:**
- Total plans completed: 17
- Average duration: 12 min
- Total execution time: 198 min

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1 | 2/2 | 8 min | 4 min |
| 2 | 1/1 | 2 min | 2 min |
| 3 | 1/1 | 3 min | 3 min |
| 4 | 1/1 | 5 min | 5 min |
| 5 | 4/4 | 81 min | 20 min |
| 6 | 8/8 | 124 min | 16 min |

**Recent Trend:**
- Last 5 plans: 06-03-02 (8 min), 06-03-03 (22 min), 06-03.1-01 (25 min), 06-04-01 (45 min), 06-05-01 (8 min)
- Trend: Test infrastructure setup went smoothly

*Updated after each plan completion*

## Accumulated Context

### Decisions Made

| Phase | Decision | Rationale |
|-------|----------|-----------|
| - | Trust Immich duplicate detection | Their detection works, only selection logic is broken |
| - | Metadata completeness priority | GPS, timezone, camera info matter more than file size |
| - | Two-stage workflow | Allows review without 2000 manual confirmations |
| - | Download originals for backup | Full recovery possible, not dependent on Immich trash |
| 01-01 | Rust 2024 edition | User has Rust 1.92.0, latest edition available |
| 01-01 | clap env feature | Environment variable support for CLI args |
| 01-02 | 30-second timeout | Reasonable default for API requests |
| 01-02 | url::Url type | Proper URL manipulation, not string concatenation |
| 02-01 | serde(default) for optional fields | Graceful handling of missing API data |
| 02-01 | has_gps() requires both coords | GPS only valid with latitude AND longitude |
| 02-01 | has_camera_info() either/or | Make OR model indicates camera info present |
| 03-01 | GPS conflict threshold 0.0001 deg | ~11m tolerance for rounding differences |
| 03-01 | String normalization lowercase+trim | Case-insensitive conflict detection |
| 03-01 | Serde tag format for conflicts | Clean JSON with snake_case type tags |
| 04-01 | AnalysisReport includes needs_review_count | Quick filtering of conflict groups |
| 04-01 | total_assets = winner + losers | Full asset count per group |
| 04-01 | Conditional conflict message | Console mentions conflicts only when count > 0 |
| 05-01 | Streaming downloads with bytes_stream() | Handles large files without memory buffering |
| 05-01 | Internal request structs for API calls | Clean public API, serialization details hidden |
| 05-01 | skip_serializing_if for optional fields | Minimal JSON payloads sent to API |
| 05-02 | governor GCRA for rate limiting | Proven algorithm, clean async API |
| 05-02 | Two-phase download-then-delete | Only delete assets successfully backed up |
| 05-02 | Asset ID prefix in filenames | Prevents collision when multiple files have same name |
| 05-02 | Defer metadata consolidation | Requires re-fetching EXIF data not in ScoredAsset |
| 05-04 | Winner = largest dimensions | User clarified: keep best quality, not most metadata |
| 05-04 | Fetch during execution | Consolidation fetches assets per-group vs storing in JSON |
| 05-04 | Owned values in consolidation | Avoids lifetime issues with async fetch results |
| 06-03-02 | Optional url/api_key CLI args | generate-fixtures doesn't need Immich connection |
| 06-03-02 | scenario.code() for directories | Clean directory names (w1, c1) without spaces |
| 06-03-03 | PNG via image crate directly | Limited EXIF support acceptable for format tests |
| 06-03-03 | Video via ffmpeg libx264 | Portable encoding for video duplicate tests |
| 06-03-03 | HEIC/RAW return explicit errors | Better than creating invalid files with wrong extension |
| 06-03.1-01 | Transform-only approach | User preference: simpler than dual-mode |
| 06-03.1-01 | Same base image per group | Ensures CLIP semantic similarity for duplicate detection |
| 06-03.1-01 | Lanczos3 filter for resizing | High-quality resize for realistic test images |
| 06-05-01 | reqwest::blocking for tests | Simpler than async, avoids tokio runtime in tests |
| 06-05-01 | 120s duplicate timeout | ML processing varies, needs generous timeout |
| 06-05-01 | 5s poll interval | Balance responsiveness vs API load |

### Deferred Issues

None yet.

### Blockers/Concerns Carried Forward

None - all concerns resolved.

## Project Alignment

Last checked: Project start
Status: ✓ Aligned
Assessment: No work done yet - baseline alignment.
Drift notes: None

## Session Continuity

Last session: 2025-12-27
Stopped at: Completed 06-05-01-PLAN.md (Integration Test Harness)
Resume file: .planning/phases/06-synthetic-integration-tests/06-05-02-PLAN.md (next)
Note: Test harness ready with setup/teardown/wait_for_duplicates. Manifest parser and assertions ready. Next: Winner selection tests (06-05-02).
