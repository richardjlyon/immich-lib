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

Phase: 10 of 10 (CLI Command)
Plan: 1 of 2 in current phase
Status: In progress
Last activity: 2025-12-28 - Completed 10-01-PLAN.md

Progress: ████░░░░░░ 40% (4/6 plans in v1.1)

## Performance Metrics

**Velocity:**
- Total plans completed: 26
- Average duration: 13 min
- Total execution time: 328 min

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1 | 2/2 | 8 min | 4 min |
| 2 | 1/1 | 2 min | 2 min |
| 3 | 1/1 | 3 min | 3 min |
| 4 | 1/1 | 5 min | 5 min |
| 5 | 4/4 | 81 min | 20 min |
| 6 | 11/11 | 199 min | 18 min |
| 7 | 3/3 | 45 min | 15 min |
| 8 | 1/1 | 2 min | 2 min |
| 9 | 2/2 | 6 min | 3 min |
| 10 | 1/2 | 2 min | 2 min |

**Recent Trend:**
- Last 5 plans: 08-01 (2 min), 09-01 (TBD), 09-02 (6 min), 10-01 (2 min)
- Trend: Fast execution on CLI plans

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
| 06-05-02 | Separate test functions | Each test does own setup/teardown, avoids shared state |
| 06-05-02 | ScenarioResult for reporting | Clean separation of test execution and result reporting |
| 06-05-02 | Warn not fail for missing groups | Immich may not detect all synthetic pairs as duplicates |
| 06-05.2-01 | Remove X6/X8 over stubbing | Can't generate valid HEIC/RAW without proprietary encoders |
| 06-05.2-01 | Lorem Picsum for base images | Direct image downloads, Unsplash Source was returning HTML |
| 06-05.2-01 | Seed-based URLs for reproducibility | Ensures same unique images if regenerated |
| 06-05.2 | maxDistance=0.06 for CLIP | Default 0.01 too strict for synthetic scale/quality variations |
| 06-05.2 | 100% vs 99% scale + quality diff | Maintains CLIP similarity while giving dimension winner |
| 06-05.2 | W3 needs different EXIF | Identical files deduped at upload; added description metadata |
| 06-05-03 | Pivot to recorded fixtures | Live Docker tests tested Immich's CLIP, not our code |
| 06-05-03 | Record real API responses | Guarantees mock data matches actual Immich format |
| 06-05-03 | 24 unit tests | Fast (0.01s), reliable, tests our scoring logic |
| 07-01 | Separate verification scripts | Winners, consolidation tested independently |
| 07-01 | Camera info not consolidatable | Immich API limitation - make/model read-only |
| 07-02 | Trashed = deleted for verification | Both trashed and 404 assets count as deleted |
| 07-03 | Strip UUID prefix from backup filenames | Restores original filename when re-uploading |
| 07-03 | Device ID = "immich-dupes-restore" | Traceability for restored assets |
| 08-01 | Timestamp+camera pairing | Live Photo Video Index not exposed by Immich API |
| 08-01 | 4:3 always wins selection | More pixels, complete scene, can crop later |
| 08-01 | No pixel analysis needed | Detection purely metadata-based via Immich API |
| 08-01 | Consolidate to 3 phases | Original 4 phases → 3 (eliminated pixel analysis) |
| 09-01 | Orientation-agnostic ratio | Use max/min dims to handle portrait/landscape |
| 09-01 | PairingKey grouping | timestamp_second + make + model + GPS optional |
| 09-01 | Serialize for AssetResponse | Required for LetterboxPair JSON serialization |
| 09-02 | 1000 page size for pagination | Standard Immich default for asset listing |
| 09-02 | Filter trashed in get_all_assets | Matches letterbox detection behavior |
| 09-02 | Track skipped counts in analysis | Transparency for non-iPhone and ambiguous groups |

### Roadmap Evolution

| Date | Change | Reason |
|------|--------|--------|
| 2025-12-27 | Inserted 06-05.2 after 06-05-02 | Base image reuse causes CLIP to group unrelated scenarios together |
| 2025-12-28 | Created milestone v1.1 | iPhone letterbox duplicates, 4 phases (8-11) |
| 2025-12-28 | Consolidated v1.1 to 3 phases (8-10) | Research showed no pixel analysis needed; Phase 11 eliminated |

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

Last session: 2025-12-28
Stopped at: Completed 10-01-PLAN.md
Resume file: None
Note: Phase 10 in progress. Ready for 10-02: letterbox execute command.
