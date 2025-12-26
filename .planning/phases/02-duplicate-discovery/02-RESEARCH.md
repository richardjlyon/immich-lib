# Phase 2: Duplicate Discovery - Research

**Researched:** 2025-12-26
**Domain:** Immich REST API - Duplicate and Asset Endpoints
**Confidence:** HIGH

<research_summary>
## Summary

Researched the Immich API for fetching duplicate groups and asset metadata. The API provides a straightforward REST endpoint (`GET /duplicates`) that returns duplicate groups with full asset details embedded, including EXIF metadata. This means a single API call retrieves both duplicate groupings AND all metadata needed for scoring.

Key finding: The `/duplicates` endpoint returns `DuplicateResponseDto[]` where each group contains `AssetResponseDto[]` with embedded `ExifResponseDto`. No separate calls are needed to fetch asset metadata - it comes with the duplicates response.

**Primary recommendation:** Use the existing `get_duplicates()` method from Phase 1, which already returns the correct structure. Phase 2 focuses on ensuring all needed fields are captured in the response types and handling any edge cases (empty EXIF, pagination for large libraries).
</research_summary>

<standard_stack>
## Standard Stack

### Core (Already Implemented in Phase 1)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.12+ | HTTP client | Async, ergonomic, well-maintained |
| serde | 1.0 | JSON serialization | De-facto standard for Rust JSON |
| serde_json | 1.0 | JSON parsing | Companion to serde |
| url | 2.5+ | URL manipulation | Proper URL joining, not string concat |
| thiserror | 2.0+ | Error types | Clean library error definitions |

### Supporting (May Need for Phase 2)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| chrono | 0.4 | DateTime parsing | If we need to parse dateTimeOriginal strings |
| tracing | 0.1 | Logging | Debug API calls (already in project) |

### Already Have - No New Dependencies Needed
Phase 1 already established the HTTP client pattern. Phase 2 extends the existing types - no new libraries required.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Project Structure (Matches Phase 1)
```
src/
├── client.rs          # ImmichClient - add any new API methods here
├── models/
│   ├── mod.rs
│   ├── asset.rs       # AssetResponse - may need additional fields
│   ├── duplicate.rs   # DuplicateGroup - already complete
│   └── exif.rs        # ExifInfo - may need additional fields
└── error.rs           # ImmichError - already handles API errors
```

### Pattern 1: Response Types Match API Schema
**What:** Serde structs mirror the API's JSON structure exactly
**When to use:** Always - the API is the source of truth
**Example:**
```rust
// Source: Immich OpenAPI spec - ExifResponseDto
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExifInfo {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub time_zone: Option<String>,
    pub date_time_original: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    // ... all fields nullable per API spec
}
```

### Pattern 2: Optional Fields with Default Handling
**What:** Use `Option<T>` for all nullable API fields, `#[serde(default)]` for missing fields
**When to use:** Any field marked as nullable in OpenAPI spec
**Example:**
```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetResponse {
    // Required fields (always present)
    pub id: String,
    pub checksum: String,

    // Optional fields (may be null or missing)
    #[serde(default)]
    pub exif_info: Option<ExifInfo>,

    pub duplicate_id: Option<String>,  // null if not a duplicate
}
```

### Pattern 3: Graceful Degradation for Missing EXIF
**What:** Don't fail if EXIF is missing - it's expected for many assets
**When to use:** When processing assets without metadata
**Example:**
```rust
impl AssetResponse {
    pub fn has_gps(&self) -> bool {
        self.exif_info
            .as_ref()
            .map(|e| e.latitude.is_some() && e.longitude.is_some())
            .unwrap_or(false)
    }
}
```

### Anti-Patterns to Avoid
- **Separate calls for each asset's EXIF:** The duplicates endpoint already includes EXIF
- **Unwrapping Options:** Always use `?` or pattern matching for nullable fields
- **Ignoring API error responses:** Check status codes and parse error messages
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| URL path joining | String concatenation | `url::Url::join()` | Handles trailing slashes, encoding |
| JSON parsing | Manual parsing | `serde` derive macros | Type-safe, compile-time checked |
| HTTP requests | Raw sockets | `reqwest` | Handles TLS, connection pooling, timeouts |
| DateTime parsing | Manual string parsing | `chrono::DateTime::parse_from_rfc3339` | Handles timezones, edge cases |
| Error types | String errors | `thiserror` derive | Structured errors with context |

**Key insight:** The Immich API returns well-structured JSON. Serde handles all parsing automatically. The only custom logic needed is for scoring metadata completeness (Phase 3), not for fetching or parsing.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Assuming EXIF Always Present
**What goes wrong:** Code panics or returns errors for assets without EXIF
**Why it happens:** Many photos (screenshots, downloads, old scans) have no EXIF
**How to avoid:** Always use `Option<ExifInfo>` and handle `None` gracefully
**Warning signs:** `.unwrap()` on exif_info, test failures with synthetic data

### Pitfall 2: Ignoring API Field Casing
**What goes wrong:** Deserialization fails silently, fields are `None`
**Why it happens:** Immich uses camelCase, Rust convention is snake_case
**How to avoid:** Always add `#[serde(rename_all = "camelCase")]`
**Warning signs:** Fields always `None` despite API returning data

### Pitfall 3: Missing Fields in Response Types
**What goes wrong:** New API fields are ignored, or required fields fail to parse
**Why it happens:** OpenAPI spec has more fields than we modeled
**How to avoid:** Add `#[serde(default)]` for optional fields we don't use yet
**Warning signs:** Deserialization errors after Immich updates

### Pitfall 4: Large Library Pagination
**What goes wrong:** Memory exhaustion or timeouts with 10k+ duplicates
**Why it happens:** GET /duplicates returns all groups at once
**How to avoid:** The API supports `page` and `limit` query params - use them if needed
**Warning signs:** Slow responses, OOM in testing with large datasets

### Pitfall 5: Timezone Handling in Dates
**What goes wrong:** Incorrect date comparisons, off-by-one-day errors
**Why it happens:** `dateTimeOriginal` is local time, `fileCreatedAt` is UTC
**How to avoid:** Use `timeZone` field to interpret `dateTimeOriginal`
**Warning signs:** Photos grouped incorrectly by date
</common_pitfalls>

<code_examples>
## Code Examples

Verified patterns from official sources and Phase 1 implementation:

### GET /duplicates Response Structure
```json
// Source: Immich OpenAPI spec - GET /duplicates response
[
  {
    "duplicateId": "uuid-string",
    "assets": [
      {
        "id": "asset-uuid",
        "checksum": "base64-sha1",
        "originalFileName": "IMG_1234.jpg",
        "fileCreatedAt": "2023-10-27T10:00:00.000Z",
        "localDateTime": "2023-10-27T10:00:00.000Z",
        "type": "IMAGE",
        "isTrashed": false,
        "isFavorite": false,
        "isArchived": false,
        "exifInfo": {
          "latitude": 40.7128,
          "longitude": -74.0060,
          "city": "New York",
          "country": "United States",
          "timeZone": "America/New_York",
          "dateTimeOriginal": "2023-10-27T06:00:00.000-04:00",
          "make": "Apple",
          "model": "iPhone 15 Pro",
          "lensModel": "iPhone 15 Pro back camera",
          "exposureTime": "1/125",
          "fNumber": 1.78,
          "focalLength": 6.765,
          "iso": 100,
          "exifImageWidth": 4032,
          "exifImageHeight": 3024,
          "fileSizeInByte": 4521984,
          "description": null,
          "rating": null
        }
      }
    ]
  }
]
```

### Extending AssetResponse (if needed)
```rust
// Source: Phase 1 implementation + OpenAPI spec
// Additional fields that may be useful for Phase 3 scoring:

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetResponse {
    // ... existing fields ...

    // Additional fields from API (add if needed for scoring)
    #[serde(default)]
    pub has_metadata: bool,

    #[serde(default)]
    pub duplicate_id: Option<String>,

    #[serde(default)]
    pub owner_id: String,
}
```

### Checking for EXIF Completeness
```rust
// Pattern for Phase 3 scoring - preview
impl ExifInfo {
    /// Returns true if GPS coordinates are present
    pub fn has_gps(&self) -> bool {
        self.latitude.is_some() && self.longitude.is_some()
    }

    /// Returns true if camera info is present
    pub fn has_camera_info(&self) -> bool {
        self.make.is_some() || self.model.is_some()
    }

    /// Returns true if original capture time is known
    pub fn has_capture_time(&self) -> bool {
        self.date_time_original.is_some()
    }
}
```
</code_examples>

<sota_updates>
## State of the Art (2024-2025)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Separate asset fetch | EXIF embedded in duplicates response | Current | No extra API calls needed |
| No pagination | `page` and `limit` params available | Recent | Handles large libraries |

**New tools/patterns to consider:**
- **Immich API v2 stable:** DELETE /duplicates/{id} now stable (was beta)
- **CLIP-based detection:** Duplicates use ML embeddings, not just hash matching

**Deprecated/outdated:**
- None identified for duplicate detection APIs
</sota_updates>

<open_questions>
## Open Questions

Things that couldn't be fully resolved:

1. **Pagination limits for /duplicates**
   - What we know: API supports `page` and `limit` query params
   - What's unclear: Default limit, max limit, whether needed for typical libraries
   - Recommendation: Start without pagination, add if testing shows issues with large libraries

2. **Rate limiting behavior**
   - What we know: No documented rate limits for self-hosted Immich
   - What's unclear: Whether aggressive querying could cause issues
   - Recommendation: Use reasonable timeouts (already 30s), add retry logic if needed
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- /websites/api_immich_app - GET /duplicates, DuplicateResponseDto, AssetResponseDto, ExifResponseDto
- https://docs.immich.app/openapi.json - Full OpenAPI spec with all field definitions
- Phase 1 implementation - Verified working client code

### Secondary (MEDIUM confidence)
- /immich-app/immich GitHub docs - Duplicate detection uses CLIP embeddings
- /websites/serde_rs - Option handling, camelCase rename patterns
- /websites/rs_reqwest - Bearer auth, error handling patterns

### Tertiary (LOW confidence - needs validation)
- None - all findings verified against official sources
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Immich REST API v2
- Ecosystem: Already using reqwest + serde (Phase 1)
- Patterns: Response type design, Option handling, error handling
- Pitfalls: EXIF nullability, field casing, large library handling

**Confidence breakdown:**
- Standard stack: HIGH - using Phase 1 stack, no changes needed
- Architecture: HIGH - extends existing patterns
- Pitfalls: HIGH - documented in API spec and verified
- Code examples: HIGH - from OpenAPI spec and working Phase 1 code

**Research date:** 2025-12-26
**Valid until:** 2026-01-26 (30 days - Immich API stable)
</metadata>

---

*Phase: 02-duplicate-discovery*
*Research completed: 2025-12-26*
*Ready for planning: yes*
