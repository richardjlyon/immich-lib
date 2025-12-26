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
- [ ] **Phase 3: Metadata Scoring** - Scoring algorithm for metadata completeness
- [ ] **Phase 4: Analysis Stage** - Analyze command with JSON output
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

### Phase 3: Metadata Scoring
**Goal**: Implement scoring algorithm that ranks assets by metadata completeness
**Depends on**: Phase 2
**Research**: Unlikely (internal algorithm, no external APIs)
**Plans**: TBD

Plans:
- [ ] 03-01: TBD

### Phase 4: Analysis Stage
**Goal**: Build the analyze CLI command that outputs scored duplicate groups to JSON
**Depends on**: Phase 3
**Research**: Unlikely (internal patterns, JSON serialization)
**Plans**: TBD

Plans:
- [ ] 04-01: TBD

### Phase 5: Execution Stage
**Goal**: Build the execute command that downloads backups and deletes duplicates
**Depends on**: Phase 4
**Research**: Likely (download/delete API patterns, rate limiting)
**Research topics**: Asset download endpoint, delete endpoint, rate limiting strategy for 2000+ operations
**Plans**: TBD

Plans:
- [ ] 05-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 2/2 | Complete | 2025-12-26 |
| 2. Duplicate Discovery | 1/1 | Complete | 2025-12-26 |
| 3. Metadata Scoring | 0/? | Not started | - |
| 4. Analysis Stage | 0/? | Not started | - |
| 5. Execution Stage | 0/? | Not started | - |
