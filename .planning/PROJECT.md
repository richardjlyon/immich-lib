# immich-lib

## Vision

A Rust library for the Immich API focused on duplicate management, paired with a binary tool that makes intelligent decisions about which duplicate to keep. Unlike Immich's built-in de-duplication which blindly favors larger files, this tool prioritizes metadata completeness—ensuring GPS coordinates, timezone information, camera data, and other valuable EXIF data are never lost.

The tool addresses a real data loss problem: Immich's de-dupe function will trash a 1.2 MiB photo with full GPS and timezone data in favor of a 2.5 MiB version with "Unknown" location. For a library with 2000+ duplicates, manual review isn't feasible, but automated deletion without smart selection means permanent metadata loss.

## Problem

Immich's duplicate detection is trustworthy—it correctly identifies duplicate images. But its resolution logic is naive: pick the largest file, delete the rest. This ignores that:

- Smaller files often have richer EXIF metadata (GPS, timezone, camera info)
- Photos imported from different sources may have varying metadata completeness
- Once deleted, that metadata is gone forever—you can't recover GPS coordinates from a photo that never had them

The user has 2000 duplicates. Manual review of each pair is impractical. But blindly trusting Immich's selection means accepting metadata loss across the entire library.

## Success Criteria

How we know this worked:

- [ ] Library successfully authenticates and queries Immich duplicate API
- [ ] Analysis correctly identifies metadata-richer files that Immich would discard
- [ ] Generated JSON is complete, auditable, and human-reviewable
- [ ] Execution downloads originals before any deletion
- [ ] Zero metadata loss—no file with richer EXIF deleted in favor of metadata-poor alternative
- [ ] Processes 2000 duplicates without manual intervention (beyond spot-checking)

## Scope

### Building

**Library (`immich-lib`)**:
- Immich API client (authentication, connection handling)
- Duplicate groups endpoint (fetch duplicate sets)
- Asset metadata endpoint (fetch EXIF/metadata for comparison)
- Asset download endpoint (backup before deletion)
- Asset delete endpoint (remove duplicates)

**Binary (`immich-dedupe`)**:
- Stage 1 - Analyze: Query duplicates, score by metadata completeness, output JSON
- Stage 2 - Execute: Consume JSON, download backups, delete losers
- Metadata scoring algorithm (GPS, timezone, camera, lens, completeness weighting)
- Visualizer: Side-by-side metadata comparison showing why selections differ from Immich

### Not Building

- Metadata merging (copying GPS from one file to another—just pick the best file)
- Image quality analysis (sharpness, noise, compression artifacts)
- Web UI (CLI and possibly TUI only)
- Independent duplicate detection (trust Immich's detection)
- Full Immich API coverage (only de-dupe-related endpoints)

## Context

**Immich's current behavior**: The duplicate view shows pairs side-by-side with metadata comparison. It pre-selects "Keep" on the larger file and "Trash" on the smaller, regardless of metadata quality. Users must manually review each pair to override.

**Why two stages**: With 2000 duplicates, the process must be:
1. Automated analysis with reviewable output (JSON)
2. Spot-check the JSON / visualize problem cases
3. Execute with backup safety net

**Backup strategy**: Download original files to local disk before deletion. This provides a recovery path if the algorithm makes mistakes.

## Constraints

- **Language**: Pure Rust—library and binary, no Python/JS components
- **API dependency**: Relies on Immich's duplicate detection being correct
- **Complexity budget**: Visualizer must stay lightweight—don't let it bloat the project

## Decisions Made

Key decisions from project exploration:

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Duplicate detection | Trust Immich | Their detection works, only selection logic is broken |
| Selection priority | Metadata completeness | GPS, timezone, camera info matter more than file size |
| Workflow | Two-stage (analyze → execute) | Allows review without 2000 manual confirmations |
| Backup method | Download originals | Full recovery possible, not dependent on Immich trash |
| Library scope | De-dupe focused | Keep scope tight, don't build what we don't need |

## Open Questions

Things to figure out during execution:

- [ ] Immich API authentication method (API key? OAuth?)
- [ ] Rate limiting considerations for 2000+ API calls
- [ ] Metadata scoring weights (how much is GPS worth vs. camera info?)
- [ ] JSON schema design for the analysis output
- [ ] Visualizer approach (TUI with ratatui? Simple CLI table? HTML report?)

---
*Initialized: 2025-12-26*
