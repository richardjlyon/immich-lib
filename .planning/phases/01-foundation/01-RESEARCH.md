# Phase 1: Foundation - Research

**Researched:** 2025-12-26
**Domain:** Rust API client for Immich REST API
**Confidence:** HIGH

<research_summary>
## Summary

Researched the Immich REST API and Rust HTTP client ecosystem for building a duplicate management library. The Immich API is well-documented with OpenAPI specs, uses simple API key authentication via `x-api-key` header, and provides dedicated endpoints for duplicates, assets, and downloads.

The standard Rust approach uses reqwest for async HTTP, serde for JSON serialization, tokio as the async runtime, and thiserror for custom error types. This stack is mature, well-documented, and widely used.

Key finding: The Immich duplicate API returns complete asset data including full EXIF metadata in a single call - no need for separate metadata fetches. This simplifies the architecture significantly.

**Primary recommendation:** Use reqwest + serde + tokio + thiserror stack. Build a thin, hand-written API client rather than using OpenAPI code generation, as the required endpoints are limited (duplicates, assets, download) and hand-written code is easier to maintain.
</research_summary>

<standard_stack>
## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.12.x | Async HTTP client | De facto standard, ergonomic API, built on hyper |
| tokio | 1.48.x | Async runtime | Industry standard, required by reqwest |
| serde | 1.0.x | Serialization framework | The standard for Rust serialization |
| serde_json | 1.0.x | JSON support | Pairs with serde for JSON parsing |
| thiserror | 2.0.x | Error derive macro | Clean custom error types for libraries |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| clap | 4.x | CLI argument parsing | Binary tool interface |
| anyhow | 1.x | Application error handling | Binary (not library) error handling |
| tracing | 0.1.x | Logging/diagnostics | Debug output, optional |
| url | 2.x | URL parsing/building | API endpoint construction |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| reqwest | ureq | ureq is blocking-only, simpler but less flexible |
| thiserror | anyhow | anyhow for apps, thiserror for libraries |
| tokio | async-std | tokio has larger ecosystem, reqwest requires it |
| progenitor | hand-written | OpenAPI codegen adds complexity for few endpoints |

**Installation:**
```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
url = "2"

# Binary only
clap = { version = "4", features = ["derive"] }
anyhow = "1"
```
</standard_stack>

<immich_api>
## Immich API Details

### Authentication
Three supported methods (use API key for tools):

| Type | Header/Param | Description |
|------|--------------|-------------|
| API Key | `x-api-key: <key>` | User-scoped API key (recommended for tools) |
| API Key | `?apiKey=<key>` | Query parameter alternative |
| Session | `x-immich-user-token` | Browser session token |

API keys are created in Immich web UI under user settings. Keys can have scoped permissions.

**Required permissions for this tool:**
- `duplicate.read` - Retrieve duplicate groups
- `asset.read` - Get asset details
- `asset.download` - Download original files
- `asset.delete` - Delete assets (trash)

### Key Endpoints

**Duplicates:**
```
GET /api/duplicates
Response: DuplicateResponseDto[]
  - duplicateId: String
  - assets: AssetResponseDto[]
```

**Assets (included in duplicate response):**
```
AssetResponseDto:
  - id: String
  - originalFileName: String
  - fileCreatedAt: DateTime
  - localDateTime: DateTime
  - type: IMAGE | VIDEO
  - exifInfo: ExifResponseDto (optional)
  - checksum: String (base64 sha1)
  - isTrashed: Boolean
  - isFavorite: Boolean
  - isArchived: Boolean
```

**EXIF Data (key for metadata scoring):**
```
ExifResponseDto:
  - latitude, longitude: Number | null (GPS)
  - city, state, country: String | null (location text)
  - timeZone: String | null
  - dateTimeOriginal: DateTime | null
  - make, model: String | null (camera)
  - lensModel: String | null
  - exposureTime, fNumber, focalLength, iso: Number | null
  - exifImageWidth, exifImageHeight: Number | null
  - fileSizeInByte: Number | null
  - description: String | null
  - rating: Number | null
```

**Download:**
```
GET /api/assets/{id}/original
Response: Binary file stream
Permission: asset.download
```

**Delete:**
```
DELETE /api/assets
Body: { "ids": ["asset-id-1", "asset-id-2"] }
Permission: asset.delete
```

### Base URL Pattern
```
https://{immich-host}/api/{endpoint}
```

### Error Responses
Standard HTTP status codes. Body contains error details as JSON.
</immich_api>

<architecture_patterns>
## Architecture Patterns

### Recommended Project Structure
```
immich-lib/
├── src/
│   ├── lib.rs           # Library crate root
│   ├── client.rs        # ImmichClient - HTTP client wrapper
│   ├── error.rs         # Custom error types
│   ├── models/          # API response types
│   │   ├── mod.rs
│   │   ├── asset.rs     # AssetResponseDto
│   │   ├── duplicate.rs # DuplicateResponseDto
│   │   └── exif.rs      # ExifResponseDto
│   ├── scoring.rs       # Metadata scoring algorithm
│   └── bin/
│       └── immich-dupes.rs  # CLI binary
├── Cargo.toml
└── tests/
    └── integration.rs
```

### Pattern 1: Typed API Client
**What:** Wrap reqwest::Client with typed methods for each endpoint
**When to use:** Always for API clients
**Example:**
```rust
use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;

pub struct ImmichClient {
    client: reqwest::Client,
    base_url: Url,
}

impl ImmichClient {
    pub fn new(base_url: &str, api_key: &str) -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(api_key)?);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            base_url: Url::parse(base_url)?,
        })
    }

    pub async fn get_duplicates(&self) -> Result<Vec<DuplicateGroup>, Error> {
        let url = self.base_url.join("/api/duplicates")?;
        let response = self.client.get(url).send().await?;
        self.handle_response(response).await
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response
    ) -> Result<T, Error> {
        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(Error::Api { status, body })
        }
    }
}
```

### Pattern 2: Domain Error Types with thiserror
**What:** Define specific error types for different failure modes
**When to use:** Library crates
**Example:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error {status}: {body}")]
    Api {
        status: reqwest::StatusCode,
        body: String
    },

    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),

    #[error("Invalid API key format")]
    InvalidApiKey,

    #[error("Asset not found: {0}")]
    AssetNotFound(String),
}
```

### Pattern 3: Serde Rename for API Compatibility
**What:** Use serde attributes to match API field naming
**When to use:** When Rust naming differs from JSON
**Example:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetResponse {
    pub id: String,
    pub original_file_name: String,
    pub file_created_at: String,
    pub local_date_time: String,
    #[serde(rename = "type")]
    pub asset_type: AssetType,
    pub exif_info: Option<ExifInfo>,
    pub is_trashed: bool,
    pub is_favorite: bool,
}
```

### Anti-Patterns to Avoid
- **Blocking in async context:** Never use `std::thread::sleep` or blocking I/O in async code
- **Ignoring errors:** Always handle or propagate errors with `?`
- **String URLs:** Use `url::Url` type for URL manipulation
- **Hardcoded timeouts:** Make timeouts configurable
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP client | Custom TCP/TLS handling | reqwest | Connection pooling, TLS, redirects, compression |
| JSON parsing | Manual string parsing | serde_json | Type safety, error handling, performance |
| Async runtime | Thread pools | tokio | Efficient scheduling, I/O integration |
| Error types | String errors | thiserror | Display, source chaining, downcasting |
| URL handling | String concatenation | url crate | Proper escaping, path joining, validation |
| CLI parsing | Manual argv parsing | clap | Validation, help generation, completions |

**Key insight:** The Rust ecosystem has mature solutions for all foundational concerns. The real work is in the domain logic (metadata scoring, duplicate selection) - not in reinventing HTTP clients or error handling.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Blocking the Async Runtime
**What goes wrong:** UI/CLI freezes, performance degrades
**Why it happens:** Using `std::fs` or `std::thread::sleep` in async context
**How to avoid:** Use `tokio::fs` and `tokio::time::sleep`
**Warning signs:** Unexplained latency, task starvation

### Pitfall 2: Missing Error Context
**What goes wrong:** "request failed" with no useful info
**Why it happens:** Using `?` without adding context
**How to avoid:** Use `.context()` from anyhow or map errors with thiserror
**Warning signs:** Debugging requires adding println! everywhere

### Pitfall 3: API Rate Limiting
**What goes wrong:** 429 errors when processing many duplicates
**Why it happens:** Immich may rate limit rapid requests
**How to avoid:** Add configurable delays, implement exponential backoff
**Warning signs:** Intermittent failures on large libraries

### Pitfall 4: Large File Downloads OOM
**What goes wrong:** Memory exhaustion downloading large videos
**Why it happens:** Loading entire file into memory
**How to avoid:** Stream to disk with `response.bytes_stream()`
**Warning signs:** Memory usage spikes during downloads

### Pitfall 5: Incorrect DateTime Handling
**What goes wrong:** Timezone confusion, wrong sort order
**Why it happens:** Mixing `fileCreatedAt` (UTC) with `localDateTime` (local)
**How to avoid:** Use `localDateTime` for display, `fileCreatedAt` for comparison
**Warning signs:** Photos appear in wrong chronological order
</common_pitfalls>

<code_examples>
## Code Examples

Verified patterns from official sources:

### Basic reqwest Client Setup
```rust
// Source: reqwest docs, Context7
use reqwest::Client;
use std::time::Duration;

let client = Client::builder()
    .timeout(Duration::from_secs(30))
    .default_headers({
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-api-key", "your-key".parse().unwrap());
        headers
    })
    .build()?;
```

### Async JSON Request
```rust
// Source: reqwest docs, Context7
use serde::Deserialize;

#[derive(Deserialize)]
struct DuplicateGroup {
    #[serde(rename = "duplicateId")]
    duplicate_id: String,
    assets: Vec<Asset>,
}

async fn get_duplicates(client: &Client, base_url: &str) -> Result<Vec<DuplicateGroup>, reqwest::Error> {
    let url = format!("{}/api/duplicates", base_url);
    client.get(&url)
        .send()
        .await?
        .json()
        .await
}
```

### Streaming File Download
```rust
// Source: reqwest docs
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;

async fn download_asset(
    client: &Client,
    base_url: &str,
    asset_id: &str,
    path: &Path
) -> Result<(), Error> {
    let url = format!("{}/api/assets/{}/original", base_url, asset_id);
    let response = client.get(&url).send().await?;

    let mut file = tokio::fs::File::create(path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        file.write_all(&chunk?).await?;
    }

    Ok(())
}
```

### Error Type with thiserror
```rust
// Source: thiserror docs, Context7
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImmichError {
    #[error("HTTP request failed")]
    Http(#[from] reqwest::Error),

    #[error("API returned error {status}: {message}")]
    Api { status: u16, message: String },

    #[error("Failed to parse API response")]
    Parse(#[from] serde_json::Error),
}
```
</code_examples>

<sota_updates>
## State of the Art (2024-2025)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| hyper directly | reqwest | Stable since 2020 | reqwest is the standard high-level choice |
| failure crate | thiserror/anyhow | 2019 | failure is deprecated |
| async-std | tokio dominates | 2021+ | tokio has larger ecosystem, better tooling |
| manual OpenAPI parsing | progenitor | 2023+ | Code generation option for complex APIs |

**New tools/patterns to consider:**
- **reqwest 0.12:** Latest version with improved HTTP/2 support
- **tokio 1.x:** Stable, mature async runtime
- **thiserror 2.0:** Latest with improved derive macro

**Deprecated/outdated:**
- **failure crate:** Use thiserror instead
- **actix-web runtime:** Stick with tokio for client-side
- **hyper directly:** Use reqwest unless you need low-level control
</sota_updates>

<open_questions>
## Open Questions

Things that couldn't be fully resolved:

1. **Rate limiting behavior**
   - What we know: No documented rate limits in Immich API docs
   - What's unclear: Actual limits under load with many assets
   - Recommendation: Implement configurable delays, start conservative

2. **OpenAPI spec completeness**
   - What we know: Immich publishes OpenAPI spec at `/api-json`
   - What's unclear: Whether progenitor can generate working client
   - Recommendation: Hand-written client for now (fewer endpoints, more control)

3. **Asset download behavior for large files**
   - What we know: GET /assets/{id}/original returns binary stream
   - What's unclear: Chunked transfer behavior, timeout needs for large videos
   - Recommendation: Use streaming download, generous timeouts
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- https://api.immich.app/introduction - Official API documentation
- https://api.immich.app/authentication - Auth methods
- https://api.immich.app/endpoints/duplicates - Duplicates endpoint
- https://api.immich.app/endpoints/assets - Assets endpoint
- https://api.immich.app/models/ExifResponseDto - EXIF data structure
- Context7 /seanmonstar/reqwest - reqwest patterns
- Context7 /tokio-rs/tokio - Tokio runtime setup
- Context7 /dtolnay/thiserror - Error handling

### Secondary (MEDIUM confidence)
- WebSearch: Rust API client best practices 2024-2025
- https://github.com/oxidecomputer/progenitor - OpenAPI code generation option

### Tertiary (LOW confidence - needs validation)
- Rate limiting assumptions (not documented)
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Immich REST API + Rust HTTP client
- Ecosystem: reqwest, tokio, serde, thiserror
- Patterns: Typed client, async/await, error handling
- Pitfalls: Async blocking, rate limits, large downloads

**Confidence breakdown:**
- Immich API: HIGH - verified with official docs via Playwright
- Standard stack: HIGH - verified with Context7, widely used
- Architecture: HIGH - from official examples and best practices
- Pitfalls: MEDIUM - some based on experience, not all verified
- Code examples: HIGH - from Context7/official sources

**Research date:** 2025-12-26
**Valid until:** 2026-01-26 (30 days - stable ecosystem)
</metadata>

---

*Phase: 01-foundation*
*Research completed: 2025-12-26*
*Ready for planning: yes*
