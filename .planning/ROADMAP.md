# Roadmap: immich-lib

## Overview

Build a Rust library for the Immich API focused on duplicate management, paired with a binary tool that prioritizes metadata completeness over file size when selecting which duplicate to keep. The journey goes from API client foundation through duplicate discovery, metadata scoring, analysis output, and finally safe execution with backups.

## Milestones

- ✅ [v1.0 MVP](milestones/v1.0-ROADMAP.md) (Phases 1-7) - SHIPPED 2025-12-27
- 🚧 **v1.1 iPhone Letterbox Duplicates** - Phases 8-11 (in progress)

## Completed Milestones

<details>
<summary>v1.0 MVP (Phases 1-7) - SHIPPED 2025-12-27</summary>

### Phase 1: Foundation
**Goal**: Set up Rust project structure with working Immich API client and authentication
**Plans**: 2 | **Completed**: 2025-12-26

- [x] 01-01: Project Structure & Types
- [x] 01-02: HTTP Client & Authentication

### Phase 2: Duplicate Discovery
**Goal**: Fetch duplicate groups from Immich and retrieve asset metadata for comparison
**Plans**: 1 | **Completed**: 2025-12-26

- [x] 02-01: Data Model Completion

### Phase 3: Metadata Scoring
**Goal**: Implement scoring algorithm that ranks assets by metadata completeness
**Plans**: 1 | **Completed**: 2025-12-26

- [x] 03-01: Metadata Scoring Algorithm

### Phase 4: Analysis Stage
**Goal**: Build the analyze CLI command that outputs scored duplicate groups to JSON
**Plans**: 1 | **Completed**: 2025-12-26

- [x] 04-01: Analyze Command

### Phase 5: Execution Stage
**Goal**: Build the execute command that downloads backups and deletes duplicates
**Plans**: 4 | **Completed**: 2025-12-26

- [x] 05-01: API Client Extensions
- [x] 05-02: Executor Module
- [x] 05-03: Execute CLI Command
- [x] 05-04: Winner Selection Fix

### Phase 6: Synthetic Integration Tests
**Goal**: Comprehensive integration tests using generated images in isolated Docker environment
**Plans**: 11 | **Completed**: 2025-12-27

- [x] 06-01: Test Candidate Finder
- [x] 06-02: Review & Refine Test Matrix
- [x] 06-03: Test Image Generator
- [x] 06-03.1: Real Image Fixture Refactor (INSERTED)
- [x] 06-04: Docker Test Environment
- [x] 06-05-01: Test Harness
- [x] 06-05-02: Winner/Consolidation Tests
- [x] 06-05.2: Unique Base Images (INSERTED)
- [x] 06-05-03: Edge Case Tests

### Phase 7: Live Instance Validation
**Goal**: Validate against real duplicates in cloned instance before production use
**Plans**: 3 | **Completed**: 2025-12-27

- [x] 07-01: Validation Runner
- [x] 07-02: End-State Verification
- [x] 07-03: Restore Command

</details>

### 🚧 v1.1 iPhone Letterbox Duplicates (In Progress)

**Milestone Goal:** Detect and remove iPhone 4:3/16:9 crop duplicates - pairs where one is the full 4:3 sensor capture and one is a 16:9 crop of the same moment.

#### Phase 8: Research ✓
**Goal**: Investigate iPhone EXIF patterns and metadata signals for crop pairs
**Depends on**: v1.0 complete
**Completed**: 2025-12-28

Plans:
- [x] 08-01: Document iPhone 16:9 crop patterns

**Key Findings** (see DISCOVERY.md):
- `Live Photo Video Index` links pairs but NOT exposed by Immich API
- Fallback: timestamp + make + model + GPS matching
- No pixel analysis needed - pure metadata detection
- Selection: 4:3 always wins (more pixels, full scene)

#### Phase 9: Detection + Selection
**Goal**: Find candidate pairs and select keeper using timestamp + camera matching
**Depends on**: Phase 8 (research findings)
**Research**: Unlikely (approach documented)
**Status**: In progress (1/2 plans complete)

Plans:
- [x] 09-01: Letterbox Module Core (AspectRatio, LetterboxPair, pairing algorithm)
- [ ] 09-02: Analysis Report (LetterboxAnalysis, ImmichClient integration)

#### Phase 10: CLI Command
**Goal**: Implement `letterbox` subcommand with analyze/execute workflow
**Depends on**: Phase 9
**Research**: Unlikely (follows existing CLI patterns)
**Plans**: TBD

Plans:
- [ ] 10-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v1.0 | 2/2 | Complete | 2025-12-26 |
| 2. Duplicate Discovery | v1.0 | 1/1 | Complete | 2025-12-26 |
| 3. Metadata Scoring | v1.0 | 1/1 | Complete | 2025-12-26 |
| 4. Analysis Stage | v1.0 | 1/1 | Complete | 2025-12-26 |
| 5. Execution Stage | v1.0 | 4/4 | Complete | 2025-12-26 |
| 6. Synthetic Integration Tests | v1.0 | 11/11 | Complete | 2025-12-27 |
| 7. Live Instance Validation | v1.0 | 3/3 | Complete | 2025-12-27 |
| 8. Research | v1.1 | 1/1 | Complete | 2025-12-28 |
| 9. Detection + Selection | v1.1 | 1/2 | In progress | - |
| 10. CLI Command | v1.1 | 0/? | Not started | - |
