# Phase 5: Execution Stage - Research

**Researched:** 2025-12-26
**Domain:** Immich API bulk operations (download/delete) with Rust async patterns
**Confidence:** HIGH

<research_summary>
## Summary

Researched the Immich API endpoints for downloading and deleting assets, along with Rust ecosystem libraries for rate limiting, progress reporting, and async bulk operations.

Key finding: The Immich API provides simple, stable endpoints for individual asset download (`GET /assets/{id}/original`) and bulk delete (`DELETE /assets` with `ids[]`). Rate limiting should be implemented client-side using the `governor` crate with `tokio::sync::Semaphore` for concurrency control.

**Primary recommendation:** Use `governor` for rate limiting (requests/second), `tokio::sync::Semaphore` for concurrency limiting, and `indicatif` for progress bars. Download assets individually (not via archive endpoint) to preserve original filenames and enable per-file progress tracking.

**Critical finding on metadata:** Immich stores EXIF metadata in its database, but the API only allows updating a subset of fields. Before deleting duplicates, we should consolidate metadata from losers to winner where possible. See "Metadata Consolidation" section below.
</research_summary>

<metadata_consolidation>
## Metadata Consolidation (CRITICAL)

When deleting duplicate assets, metadata on "loser" assets may be lost forever. Immich stores metadata in its database separately from the file, and provides `PUT /assets/{id}` to update some fields.

### What CAN Be Consolidated via API

| Field | API Parameter | Notes |
|-------|---------------|-------|
| GPS Coordinates | `latitude`, `longitude` | Can copy from loser if winner lacks GPS |
| Original Date/Time | `dateTimeOriginal` | Can update winner's timestamp |
| Description | `description` | Can merge/copy descriptions |
| Rating | `rating` | Can copy star rating |

### What CANNOT Be Consolidated via API

| Field | Why It Matters | Impact |
|-------|----------------|--------|
| Camera Make/Model | `make`, `model` | **Lost forever** - identifies source device |
| Lens Model | `lensModel` | **Lost forever** - important for photographers |
| Exposure Settings | `exposureTime`, `fNumber`, `focalLength`, `iso` | **Lost forever** - technical capture info |
| Timezone | `timeZone` | Cannot update directly; may affect time display |

### Recommended Execution Workflow

```
Phase 1: Metadata Consolidation (before any deletion)
   For each duplicate group:
   1. Identify winner and losers
   2. Check if any loser has metadata winner lacks:
      - GPS: If winner.latitude is null but loser has GPS → update winner
      - DateTime: If winner.dateTimeOriginal is null → copy from loser
      - Description: If winner.description is empty → copy from loser
   3. Call PUT /assets/{winner_id} with consolidated metadata
   4. Log consolidation actions

Phase 2: Download Losers (backup)
   For each loser asset:
   1. Download original file to backup directory
   2. Verify download completed successfully

Phase 3: Delete Losers
   For successfully downloaded losers only:
   1. Call DELETE /assets with loser IDs
   2. Use force=false (trash) by default for safety
```

### API Endpoint for Metadata Update

```rust
// PUT /assets/{id} - Update asset metadata
impl ImmichClient {
    pub async fn update_asset_metadata(
        &self,
        asset_id: &str,
        latitude: Option<f64>,
        longitude: Option<f64>,
        date_time_original: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        let url = self.base_url.join(&format!("/api/assets/{}", asset_id))?;

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct UpdateRequest<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            latitude: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            longitude: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            date_time_original: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
        }

        let response = self.client
            .put(url)
            .json(&UpdateRequest {
                latitude,
                longitude,
                date_time_original,
                description,
            })
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ImmichError::Api {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            })
        }
    }
}
```

### Impact on Scoring Algorithm

Given that camera info (make/model) CANNOT be consolidated, groups where:
- Winner lacks camera info
- But a loser HAS camera info

Should be flagged as **needing manual review** in the conflict detection, since deleting the loser means permanent loss of that metadata.

Consider adding to `MetadataConflict` enum:
```rust
pub enum MetadataConflict {
    // ... existing variants ...

    /// Loser has camera info that winner lacks - cannot be consolidated
    UnconsolidatableCameraInfo {
        loser_id: String,
        make: Option<String>,
        model: Option<String>,
    },
}
```
</metadata_consolidation>

<standard_stack>
## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.12 | HTTP client (already in use) | Async streaming downloads, proven reliability |
| tokio | 1.x | Async runtime (already in use) | Semaphore for concurrency control |
| governor | 0.6 | Client-side rate limiting | GCRA algorithm, async-native, well-maintained |
| indicatif | 0.17 | Progress bars | Standard for Rust CLI apps, multi-bar support |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| futures | 0.3 | Stream utilities | `futures::stream::iter` for async iteration |
| tokio-util | 0.7 | IO utilities | `tokio_util::io::StreamReader` for streaming downloads |
| nonzero_ext | 0.3 | NonZero literals | Required by governor for quota config |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| governor | tower RateLimit | tower is for servers, governor is simpler for clients |
| indicatif | console | indicatif has better multi-progress support |
| Individual downloads | POST /download/archive | Archive requires extra API call, harder to track per-file |

**Installation:**
```bash
cargo add governor indicatif futures nonzero_ext
```
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Project Structure
```
src/
├── client.rs           # Add download_asset(), delete_assets() methods
├── executor.rs         # NEW: Execution logic with rate limiting
├── models/
│   └── execution.rs    # NEW: ExecutionReport, DownloadResult types
└── bin/
    └── immich-dupes.rs # Add execute subcommand
```

### Pattern 1: Rate-Limited Concurrent Downloads
**What:** Combine `governor` for rate limiting with `Semaphore` for concurrency
**When to use:** Bulk API operations where you need both requests/sec AND max concurrent
**Example:**
```rust
use governor::{Quota, RateLimiter};
use tokio::sync::Semaphore;
use std::sync::Arc;
use nonzero_ext::nonzero;

struct Executor {
    client: ImmichClient,
    rate_limiter: RateLimiter</* ... */>,
    concurrency: Arc<Semaphore>,
}

impl Executor {
    fn new(client: ImmichClient, requests_per_sec: u32, max_concurrent: usize) -> Self {
        Self {
            client,
            rate_limiter: RateLimiter::direct(Quota::per_second(nonzero!(requests_per_sec))),
            concurrency: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    async fn download_asset(&self, asset_id: &str, path: &Path) -> Result<()> {
        // Wait for rate limit
        self.rate_limiter.until_ready().await;

        // Acquire concurrency permit
        let _permit = self.concurrency.acquire().await?;

        // Perform download
        self.client.download_asset(asset_id, path).await
    }
}
```

### Pattern 2: Streaming Download with Progress
**What:** Stream response bytes to file while updating progress bar
**When to use:** Large file downloads where memory efficiency matters
**Example:**
```rust
use indicatif::{ProgressBar, ProgressStyle};
use tokio::io::AsyncWriteExt;
use futures::StreamExt;

async fn download_with_progress(
    response: reqwest::Response,
    path: &Path,
    pb: &ProgressBar,
) -> Result<()> {
    let total_size = response.content_length().unwrap_or(0);
    pb.set_length(total_size);

    let mut file = tokio::fs::File::create(path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
    }

    file.flush().await?;
    Ok(())
}
```

### Pattern 3: Graceful Partial Failure Handling
**What:** Continue on individual failures, collect results for reporting
**When to use:** Bulk operations where some items may fail
**Example:**
```rust
#[derive(Debug)]
enum OperationResult {
    Success { id: String, path: PathBuf },
    Failed { id: String, error: String },
    Skipped { id: String, reason: String },
}

async fn execute_downloads(
    assets: Vec<AssetInfo>,
    executor: &Executor,
) -> Vec<OperationResult> {
    let mut results = Vec::with_capacity(assets.len());

    for asset in assets {
        let result = match executor.download_asset(&asset.id, &asset.path).await {
            Ok(()) => OperationResult::Success {
                id: asset.id,
                path: asset.path
            },
            Err(e) => OperationResult::Failed {
                id: asset.id,
                error: e.to_string()
            },
        };
        results.push(result);
    }

    results
}
```

### Anti-Patterns to Avoid
- **Unbounded concurrency:** Don't spawn unlimited tasks; always use Semaphore
- **Ignoring rate limits:** Immich servers may have rate limits; respect them
- **In-memory buffering:** Don't load entire file into memory; stream to disk
- **Silent failures:** Always report what failed and why
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rate limiting | Custom token bucket | `governor` crate | GCRA algorithm is complex, edge cases abound |
| Progress bars | println!() updates | `indicatif` | Terminal handling, multi-bar, ETA calculation |
| Concurrency control | Custom counters | `tokio::sync::Semaphore` | Async-safe, no race conditions |
| Retry logic | Custom loops | Consider `backoff` crate | Exponential backoff is tricky to get right |
| Streaming downloads | Read all to Vec<u8> | `bytes_stream()` + async write | Memory efficiency for large files |

**Key insight:** Bulk operations at scale (2000+ assets) require proper async patterns. The naive approach of spawning unlimited concurrent downloads will overwhelm both client and server. Using proper primitives (Semaphore + governor) prevents resource exhaustion.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Memory Exhaustion on Large Downloads
**What goes wrong:** Loading entire file response into memory before writing
**Why it happens:** Using `.bytes()` instead of `.bytes_stream()`
**How to avoid:** Always stream response body directly to file
**Warning signs:** Process memory grows with file sizes, OOM on large videos

### Pitfall 2: Overwhelming the Server
**What goes wrong:** Too many concurrent requests cause 429/503 errors
**Why it happens:** No rate limiting or concurrency control
**How to avoid:** Use `governor` for rate limiting, `Semaphore` for concurrency (recommend 5-10 concurrent, 10-20 req/sec)
**Warning signs:** Sporadic failures, increasing error rate during execution

### Pitfall 3: Partial Execution State Loss
**What goes wrong:** Crash mid-execution loses track of what was completed
**Why it happens:** Not persisting progress
**How to avoid:** Write results to file as they complete, or use the analysis JSON as a checklist
**Warning signs:** Re-running downloads already-downloaded files

### Pitfall 4: Disk Space Exhaustion
**What goes wrong:** Downloads fill disk before deletes happen
**Why it happens:** Not checking available space before starting
**How to avoid:** Calculate total download size from analysis, check disk space first
**Warning signs:** Write errors mid-execution, corrupted partial files

### Pitfall 5: Original Filename Collisions
**What goes wrong:** Multiple assets have same `originalFileName`
**Why it happens:** Different folders had files with same name
**How to avoid:** Include asset ID in download filename (e.g., `{id}_{originalFileName}`)
**Warning signs:** Files overwriting each other, fewer files than expected

### Pitfall 6: Delete Without Successful Download
**What goes wrong:** Deleting asset before confirming download succeeded
**Why it happens:** Not checking download result before delete
**How to avoid:** Two-phase: download ALL first, then delete only successfully downloaded
**Warning signs:** Data loss, missing backups
</common_pitfalls>

<code_examples>
## Code Examples

### Download Asset (ImmichClient method)
```rust
// Source: Immich API docs + reqwest streaming pattern
impl ImmichClient {
    pub async fn download_asset(&self, asset_id: &str, path: &Path) -> Result<u64> {
        let url = self.base_url.join(&format!("/api/assets/{}/original", asset_id))?;

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(ImmichError::Api {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        let mut file = tokio::fs::File::create(path).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
        }

        file.flush().await?;
        Ok(downloaded)
    }
}
```

### Delete Assets (bulk)
```rust
// Source: Immich API docs
impl ImmichClient {
    pub async fn delete_assets(&self, asset_ids: &[String], force: bool) -> Result<()> {
        let url = self.base_url.join("/api/assets")?;

        #[derive(Serialize)]
        struct DeleteRequest<'a> {
            ids: &'a [String],
            force: bool,
        }

        let response = self.client
            .delete(url)
            .json(&DeleteRequest { ids: asset_ids, force })
            .send()
            .await?;

        if response.status() == 204 {
            Ok(())
        } else {
            Err(ImmichError::Api {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            })
        }
    }
}
```

### Progress Bar Setup
```rust
// Source: indicatif docs
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

fn create_progress_bars(total_files: u64) -> (MultiProgress, ProgressBar, ProgressBar) {
    let multi = MultiProgress::new();

    let overall = multi.add(ProgressBar::new(total_files));
    overall.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} files ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let current = multi.add(ProgressBar::new(0));
    current.set_style(ProgressStyle::default_bar()
        .template("  {msg} [{bar:30.yellow/blue}] {bytes}/{total_bytes}")
        .unwrap());

    (multi, overall, current)
}
```
</code_examples>

<sota_updates>
## State of the Art (2024-2025)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| POST /asset/download (deprecated) | GET /assets/{id}/original | v1.99+ | Use new endpoint for downloads |
| Callback-based async | async/await native | Rust 1.39+ | Cleaner code, easier error handling |
| Manual rate limiting | governor crate | 2022+ | Reliable GCRA implementation |

**New tools/patterns to consider:**
- **tokio::task::JoinSet:** Better than manual Vec<JoinHandle> for managing concurrent tasks
- **governor async methods:** `until_ready().await` is cleaner than checking in a loop

**Deprecated/outdated:**
- **POST /asset/download:** Use `GET /assets/{id}/original` instead
- **futures 0.1 style:** Use async/await, not combinators
</sota_updates>

<open_questions>
## Open Questions

1. **Immich rate limits**
   - What we know: No documented rate limits in Immich API docs
   - What's unclear: Whether self-hosted instances have any default limits
   - Recommendation: Start conservative (10 req/sec, 5 concurrent), make configurable

2. **Trash retention period**
   - What we know: `force=false` sends to trash, items auto-delete after configured period
   - What's unclear: Default retention period varies by installation
   - Recommendation: Default to `force=false` (trash), let user override with `--force`

3. **Large video handling**
   - What we know: Some assets may be multi-GB videos
   - What's unclear: Whether Immich has any special handling for large files
   - Recommendation: Stream all downloads, add timeout override for large files

4. **Non-consolidatable metadata loss**
   - What we know: Camera make/model, lens info, exposure settings CANNOT be updated via API
   - What's unclear: Whether this is a deliberate API limitation or could change
   - Recommendation: Flag groups where winner lacks camera info but loser has it; may need manual review or different winner selection
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- https://api.immich.app/endpoints/assets - Asset endpoints including download
- https://api.immich.app/endpoints/assets/deleteAssets - Bulk delete endpoint
- https://api.immich.app/endpoints/assets/updateAsset - Asset metadata update (PUT)
- https://api.immich.app/models/ExifResponseDto - EXIF data model (what's stored)
- https://api.immich.app/endpoints/trash - Trash management endpoints
- /boinkor-net/governor (Context7) - Rate limiting patterns
- /websites/docs_rs-indicatif-latest-indicatif-index.html (Context7) - Progress bar patterns
- /tokio-rs/tokio (Context7) - Semaphore and async patterns
- /seanmonstar/reqwest (Context7) - Streaming download patterns

### Secondary (MEDIUM confidence)
- https://zuplo.com/learning-center/immich-api - Best practices overview
- GitHub discussions on Immich API usage patterns

### Tertiary (LOW confidence - needs validation)
- None - all findings verified against official sources
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Immich REST API for assets
- Ecosystem: Rust async (tokio), rate limiting (governor), progress (indicatif)
- Patterns: Streaming downloads, rate-limited concurrency, partial failure handling
- Pitfalls: Memory, disk space, rate limiting, filename collisions

**Confidence breakdown:**
- API endpoints: HIGH - verified from official Immich API docs
- Rust libraries: HIGH - verified from crates.io/Context7
- Architecture patterns: HIGH - standard Rust async patterns
- Pitfalls: HIGH - derived from documented best practices

**Research date:** 2025-12-26
**Valid until:** 2026-01-26 (30 days - Immich API stable, Rust ecosystem stable)
</metadata>

---

*Phase: 05-execution-stage*
*Research completed: 2025-12-26*
*Ready for planning: yes*
