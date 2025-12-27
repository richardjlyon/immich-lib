# Roadmap: immich-lib

## Overview

Build a Rust library for the Immich API focused on duplicate management, paired with a binary tool that prioritizes metadata completeness over file size when selecting which duplicate to keep. The journey goes from API client foundation through duplicate discovery, metadata scoring, analysis output, and finally safe execution with backups.

## Domain Expertise

None

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation** - Rust project setup, API client, authentication
- [x] **Phase 2: Duplicate Discovery** - Fetch duplicate groups and asset metadata
- [x] **Phase 3: Metadata Scoring** - Scoring algorithm for metadata completeness
- [x] **Phase 4: Analysis Stage** - Analyze command with JSON output
- [ ] **Phase 5: Execution Stage** - Execute command with download/delete

## Phase Details

### Phase 1: Foundation ✓
**Goal**: Set up Rust project structure with working Immich API client and authentication
**Depends on**: Nothing (first phase)
**Research**: Complete (01-RESEARCH.md)
**Plans**: 2
**Completed**: 2025-12-26

Plans:
- [x] 01-01: Project Structure & Types (3 tasks, autonomous) ✓
- [x] 01-02: HTTP Client & Authentication (2 tasks, has checkpoint) ✓

### Phase 2: Duplicate Discovery ✓
**Goal**: Fetch duplicate groups from Immich and retrieve asset metadata for comparison
**Depends on**: Phase 1
**Research**: Complete (02-RESEARCH.md)
**Plans**: 1
**Completed**: 2025-12-26

Plans:
- [x] 02-01: Data Model Completion (2 tasks, autonomous) ✓

### Phase 3: Metadata Scoring ✓
**Goal**: Implement scoring algorithm that ranks assets by metadata completeness
**Depends on**: Phase 2
**Research**: Not needed
**Plans**: 1
**Completed**: 2025-12-26

Plans:
- [x] 03-01: Metadata Scoring Algorithm (3 tasks, autonomous) ✓

### Phase 4: Analysis Stage ✓
**Goal**: Build the analyze CLI command that outputs scored duplicate groups to JSON
**Depends on**: Phase 3
**Research**: Not needed
**Plans**: 1
**Completed**: 2025-12-26

Plans:
- [x] 04-01: Analyze Command (3 tasks, autonomous) ✓

### Phase 5: Execution Stage ✓
**Goal**: Build the execute command that downloads backups and deletes duplicates
**Depends on**: Phase 4
**Research**: Complete (05-RESEARCH.md)
**Plans**: 4
**Completed**: 2025-12-26

Plans:
- [x] 05-01: API Client Extensions (3 tasks, autonomous) ✓
- [x] 05-02: Executor Module (3 tasks, autonomous) ✓
- [x] 05-03: Execute CLI Command (2 tasks + checkpoint, interactive) ✓
- [x] 05-04: Winner Selection Fix (3 tasks, autonomous) ✓

### Phase 6: Synthetic Integration Tests ✓
**Goal**: Comprehensive integration tests using generated images in isolated Docker environment
**Depends on**: Phase 5
**Research**: Complete
**Plans**: 11
**Completed**: 2025-12-27

Plans:
- [x] 06-01: Test Candidate Finder - scan real Immich, categorize by scenario ✓
- [x] 06-02: Review & Refine Test Matrix - checkpoint to assess findings ✓
- [x] 06-03: Test Image Generator - create synthetic images with controlled EXIF ✓
- [x] 06-03.1: Real Image Fixture Refactor (INSERTED) - use open-license photos for CLIP compatibility ✓
- [x] 06-04: Docker Test Environment - Immich stack + seed/snapshot/reset ✓
- [x] 06-05: Integration Test Suite - Rust tests for all scenarios ✓
  - [x] 06-05-01: Test Harness ✓
  - [x] 06-05-02: Winner/Consolidation Tests ✓
  - [x] 06-05.2: Unique Base Images (INSERTED) - 32 unique photos for scenario isolation ✓
  - [x] 06-05-03: Edge Case Tests - pivoted to recorded fixture unit tests ✓

### Phase 7: Live Instance Validation
**Goal**: Validate against real duplicates in cloned instance before production use
**Depends on**: Phase 6
**Note**: Personal validation phase, uses Docker test instance
**Status**: In progress

Plans:
- [x] 07-01: Validation Runner - full workflow against Docker Immich ✓
- [ ] 07-02: End-State Verification - verify all surviving images

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 2/2 | Complete | 2025-12-26 |
| 2. Duplicate Discovery | 1/1 | Complete | 2025-12-26 |
| 3. Metadata Scoring | 1/1 | Complete | 2025-12-26 |
| 4. Analysis Stage | 1/1 | Complete | 2025-12-26 |
| 5. Execution Stage | 4/4 | Complete | 2025-12-26 |
| 6. Synthetic Integration Tests | 11/11 | Complete | 2025-12-27 |
| 7. Live Instance Validation | 1/2 | In progress | - |
