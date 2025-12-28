# Roadmap: immich-lib

## Overview

Build a Rust library for the Immich API focused on duplicate management, paired with a binary tool that prioritizes metadata completeness over file size when selecting which duplicate to keep. The journey goes from API client foundation through duplicate discovery, metadata scoring, analysis output, and finally safe execution with backups.

## Milestones

- ✅ [v1.0 MVP](milestones/v1.0-ROADMAP.md) (Phases 1-7) - SHIPPED 2025-12-27
- ✅ [v1.1 iPhone Letterbox Duplicates](milestones/v1.1-ROADMAP.md) (Phases 8-10) - SHIPPED 2025-12-28
- ✅ **v1.2 Configuration UX** (Phase 11) - SHIPPED 2025-12-28

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

<details>
<summary>v1.1 iPhone Letterbox Duplicates (Phases 8-10) - SHIPPED 2025-12-28</summary>

### Phase 8: Research
**Goal**: Investigate iPhone EXIF patterns and metadata signals for crop pairs
**Plans**: 1 | **Completed**: 2025-12-28

- [x] 08-01: Document iPhone 16:9 crop patterns

### Phase 9: Detection + Selection
**Goal**: Find candidate pairs and select keeper using timestamp + camera matching
**Plans**: 2 | **Completed**: 2025-12-28

- [x] 09-01: Letterbox Module Core (AspectRatio, LetterboxPair, pairing algorithm)
- [x] 09-02: Analysis Report (LetterboxAnalysis, ImmichClient integration)

### Phase 10: CLI Command
**Goal**: Implement `letterbox` subcommand with analyze/execute/verify workflow
**Plans**: 2 | **Completed**: 2025-12-28

- [x] 10-01: Analyze + Verify (letterbox analyze, letterbox verify commands)
- [x] 10-02: Execute (letterbox execute with backup-before-delete)

</details>

### ✅ v1.2 Configuration UX (Complete)

**Milestone Goal:** Add config file support with OS-native location, interactive setup prompts, and credential persistence

#### Phase 11: Configuration UX
**Goal**: Config file support with OS-native location, interactive setup, and credential persistence
**Depends on**: Previous milestone complete
**Research**: Complete (Level 1 - directories + dialoguer crates verified)
**Plans**: 2 | **Completed**: 2025-12-28

Plans:
- [x] 11-01: Config Module (directories + toml, load/save, CLI integration)
- [x] 11-02: Interactive Setup (dialoguer prompts, --save flag, credential persistence)

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11

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
| 9. Detection + Selection | v1.1 | 2/2 | Complete | 2025-12-28 |
| 10. CLI Command | v1.1 | 2/2 | Complete | 2025-12-28 |
| 11. Configuration UX | v1.2 | 2/2 | Complete | 2025-12-28 |
