# Test Scenario Coverage

## Summary

- **Total scenarios**: 34
- **Covered by real data**: 23 (68%)
- **Need synthetic images**: 11 (32%)
- **Total groups analyzed**: 2370

## Coverage by Category

### Winner Selection (W1-W8)

| Scenario | Description | Count | Status | Notes |
|----------|-------------|-------|--------|-------|
| W1 | Clear dimension winner | 69 | ✓ | Good coverage |
| W2 | Same dimensions, different size | 1963 | ✓ | Most common scenario |
| W3 | Same dimensions, same size | 302 | ✓ | Good coverage |
| W4 | Some missing dimensions | 0 | ✗ | Need synthetic |
| W5 | Only one has dimensions | 0 | ✗ | Need synthetic |
| W6 | All missing dimensions | 0 | ✗ | Need synthetic |
| W7 | 3+ duplicates | 98 | ✓ | Good coverage |
| W8 | Same pixels, different aspect | 36 | ✓ | Portrait/landscape rotations |

**Winner Selection Summary**: 5/8 covered (62%)

### Consolidation (C1-C8)

| Scenario | Description | Count | Status | Notes |
|----------|-------------|-------|--------|-------|
| C1 | Winner lacks GPS, loser has | 619 | ✓ | High priority consolidation case |
| C2 | Winner lacks datetime, loser has | 0 | ✗ | Need synthetic |
| C3 | Winner lacks description, loser has | 8 | ⚠ | Low count, may want synthetic too |
| C4 | Winner lacks all, loser has all | 0 | ✗ | Need synthetic |
| C5 | Both have GPS | 838 | ✓ | Common case |
| C6 | Multiple losers contribute | 3 | ⚠ | Low count, may want synthetic too |
| C7 | No loser has needed metadata | 0 | ✗ | Need synthetic |
| C8 | Winner has everything | 54 | ✓ | No consolidation needed |

**Consolidation Summary**: 5/8 covered (62%), 2 partial

### Conflicts (F1-F7)

| Scenario | Description | Count | Status | Notes |
|----------|-------------|-------|--------|-------|
| F1 | GPS conflict | 15 | ✓ | Different locations |
| F2 | GPS within threshold | 275 | ✓ | ~11m tolerance working |
| F3 | Timezone conflict | 222 | ✓ | Common with imported photos |
| F4 | Camera conflict | 1 | ⚠ | Single example, may want synthetic |
| F5 | Capture time conflict | 925 | ✓ | Very common |
| F6 | Multiple conflicts | 84 | ✓ | Good coverage |
| F7 | No conflicts | 1293 | ✓ | Most groups conflict-free |

**Conflicts Summary**: 7/7 covered (100%), 1 partial

### Edge Cases (X1-X11)

| Scenario | Description | Count | Status | Notes |
|----------|-------------|-------|--------|-------|
| X1 | Single asset group | 0 | ✗ | Need synthetic (degenerate case) |
| X2 | Large group (10+) | 0 | ✗ | Need synthetic |
| X3 | Large file (>50MB) | 93 | ✓ | TIFF files |
| X4 | Special chars in filename | 1 | ⚠ | Single example |
| X5 | Video | 0 | ✗ | Need synthetic |
| X6 | HEIC | 356 | ✓ | iPhone photos |
| X7 | PNG | 2 | ⚠ | Low count |
| X8 | RAW | 4 | ⚠ | Low count (DNG files) |
| X9 | Unicode description | 0 | ✗ | Need synthetic |
| X10 | Very old date (<1990) | 19 | ✓ | Scanned photos |
| X11 | Future date | 0 | ✗ | Need synthetic (invalid data) |

**Edge Cases Summary**: 6/11 covered (55%), 3 partial

## Unexpected Patterns

Based on the scan, no unexpected patterns were surfaced beyond the defined scenarios. The scenario definitions appear comprehensive.

Notable observations:
- W2 (same dimensions, different size) dominates with 1963 groups - most duplicates differ only in compression
- F5 (capture time conflict) at 925 groups is surprisingly common - likely timezone handling differences
- F3 (timezone conflict) at 222 groups confirms timezone normalization is important
- C1 (winner lacks GPS) at 619 groups validates the consolidation feature's importance

## Synthetic Image Requirements

### Must Create (11 scenarios with 0 coverage)

1. **W4**: Some missing dimensions - create images where EXIF width/height is missing for some
2. **W5**: Only one has dimensions - pair with/without dimension metadata
3. **W6**: All missing dimensions - stripped EXIF images
4. **C2**: Winner lacks datetime, loser has - dimension-rich but datetime-poor
5. **C4**: Winner lacks all, loser has all - maximum consolidation case
6. **C7**: No loser has needed metadata - winner can't be enriched
7. **X1**: Single asset group - edge case for grouping logic
8. **X2**: Large group (10+) - test batch processing
9. **X5**: Video - MP4/MOV duplicates
10. **X9**: Unicode description - emoji, CJK characters
11. **X11**: Future date - invalid timestamp handling

### Consider Creating (5 scenarios with low coverage)

1. **C3**: Description consolidation (8 examples) - more controlled test cases
2. **C6**: Multiple losers contribute (3 examples) - complex consolidation
3. **F4**: Camera conflict (1 example) - rare but important
4. **X4**: Special chars (1 example) - filename edge cases
5. **X7/X8**: PNG/RAW (2-4 examples) - format handling

## Synthetic Image Priorities

### P1 - Must Have (core functionality)

Critical paths with no real examples:

| Scenario | Rationale |
|----------|-----------|
| W4, W5, W6 | Dimension fallback logic untested |
| C2 | Datetime consolidation untested |
| C4 | Maximum consolidation case |
| X5 | Video handling completely untested |

### P2 - Should Have (important edge cases)

Important coverage gaps:

| Scenario | Rationale |
|----------|-----------|
| C7 | Edge case: no enrichment possible |
| X2 | Batch processing with 10+ assets |
| X9 | Unicode handling in descriptions |
| X11 | Invalid date handling |

### P3 - Nice to Have (rare edge cases)

Low priority, already have examples:

| Scenario | Rationale |
|----------|-----------|
| X1 | Single asset is degenerate case |
| C3, C6, F4 | Have real examples, more would be nice |
| X4, X7, X8 | Format edge cases with some coverage |

### Skip (covered by real data)

Sufficient real examples - no synthetic needed:

| Scenario | Count | Reason |
|----------|-------|--------|
| W1, W2, W3 | 69-1963 | Dimension winner well-tested |
| W7, W8 | 36-98 | Multi-dupe and aspect covered |
| C1, C5, C8 | 54-838 | GPS consolidation covered |
| F1-F3, F5-F7 | 15-1293 | Conflict detection covered |
| X3, X6, X10 | 19-356 | Large/HEIC/old dates covered |

## Test Matrix Refinements

Based on findings:

1. **No changes needed** - All 34 scenarios are valid and testable
2. **Scenario counts** - W2 dominance suggests tests should weight common cases
3. **Conflict frequency** - F5/F3 being common validates timezone handling priority
4. **Format coverage** - HEIC well-covered (356), RAW sparse (4) - reflects typical photo library

---
*Generated: 2025-12-27*
*Source: find-test-candidates scan of 2370 duplicate groups*
