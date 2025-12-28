# iPhone 16:9 Crop Duplicate Discovery

## Problem Statement

When iPhone users take photos, the Photos app sometimes creates two versions:
1. **4:3 Original** - Full sensor capture (e.g., 5712x4284 = 24.5 MP)
2. **16:9 Crop** - Cropped version for wider aspect ratio (e.g., 5712x3213 = 18.4 MP)

Both versions get imported to Immich as separate assets. Immich's CLIP-based duplicate detection does NOT identify these as duplicates because:
- They have different dimensions
- The 16:9 is a crop, not a visual duplicate
- CLIP embeddings differ enough to escape similarity threshold

**Result**: Users have redundant copies wasting storage, with the cropped version being inferior (less pixels, less of the original scene).

## Sample Analysis

Examined pair from user's Immich instance:

| Property | 4:3 Original (Keeper) | 16:9 Crop (Delete) |
|----------|----------------------|-------------------|
| Filename | `IMG_3460.HEIC` | `241223-iPhone 15 Pro Max-923.HEIC` |
| Dimensions | 5712 x 4284 | 5712 x 3213 |
| Megapixels | 24.5 MP | 18.4 MP |
| Aspect Ratio | 4:3 (1.333) | 16:9 (1.777) |
| Live Photo Video Index | `8595185700` | `8595185700` |
| Media Group UUID | Different | Different |
| GPS | Identical | Identical |
| Timestamp | Identical (ms precision) | Identical (ms precision) |
| Camera | iPhone 15 Pro Max | iPhone 15 Pro Max |

**Key Discovery**: The `Live Photo Video Index` EXIF field is **IDENTICAL** on both images! This links them as variants of the same capture moment.

## Pairing Strategy

### Primary Signal: Live Photo Video Index (NOT AVAILABLE)

The ideal pairing signal is `Live Photo Video Index` - a unique identifier linking all variants of a single capture.

**Problem**: Immich API does NOT expose this field.

Checked Immich API `ExifResponseDto` - available fields:
- city, country, dateTimeOriginal, description
- exifImageHeight, exifImageWidth
- exposureTime, fNumber, fileSizeInByte, focalLength
- iso, latitude, lensModel, longitude
- make, model, modifyDate, orientation
- projectionType, rating, state, timeZone

`Live Photo Video Index` is NOT in this list.

### Fallback Signal: Timestamp + Camera Matching

Since the API doesn't expose Live Photo Video Index, we must use indirect pairing:

**Pairing Criteria** (all must match):
1. `dateTimeOriginal` identical (within 1 second tolerance)
2. `make` = "Apple"
3. `model` contains "iPhone"
4. `latitude`/`longitude` identical (if present)
5. One asset has 4:3 aspect ratio, other has 16:9

**Algorithm**:
```
For each iPhone image:
  Calculate aspect ratio from exifImageWidth / exifImageHeight
  Group by (dateTimeOriginal rounded to second, make, model, GPS)
  Within each group:
    If contains exactly one 4:3 AND one 16:9:
      Mark as candidate pair
      4:3 = keeper (more pixels, full capture)
      16:9 = delete (crop)
```

### Aspect Ratio Detection

| Ratio | Width/Height | Tolerance |
|-------|--------------|-----------|
| 4:3 | 1.333 | ±0.01 |
| 16:9 | 1.778 | ±0.01 |

```rust
fn aspect_ratio_type(width: u32, height: u32) -> Option<AspectRatio> {
    let ratio = width as f64 / height as f64;
    if (ratio - 1.333).abs() < 0.01 {
        Some(AspectRatio::FourThree)
    } else if (ratio - 1.778).abs() < 0.01 {
        Some(AspectRatio::SixteenNine)
    } else {
        None
    }
}
```

## Selection Strategy

**Always keep the 4:3 version**:
- More pixels (full sensor readout)
- Complete scene (no cropping)
- Can always crop to 16:9 later if needed
- Cannot recover cropped pixels from 16:9

**Delete the 16:9 version**:
- Derivative of the 4:3
- Information loss (cropped edges)
- Redundant storage

This is NOT the same as the duplicate scoring problem - there's no metadata to compare. Selection is purely dimensional: bigger pixel count wins.

## Edge Cases

### 1. Non-iPhone Images
- Only process `make = "Apple"` and `model` containing "iPhone"
- Skip all other cameras

### 2. Images Without Timestamps
- Skip - cannot pair without `dateTimeOriginal`

### 3. Portrait vs Landscape
- Check both orientations
- 3:4 portrait is the 4:3 equivalent
- 9:16 portrait is the 16:9 equivalent
- Use max(width, height) / min(width, height) for orientation-agnostic ratio

### 4. Multiple Pairs at Same Timestamp
- Unlikely but possible (burst mode)
- Group by GPS as additional discriminator
- If ambiguous, skip and log for manual review

### 5. Missing Dimensions
- Some assets may lack `exifImageWidth`/`exifImageHeight`
- Fall back to analyzing actual file if needed
- Or skip - dimensions are required for this detection

### 6. Already Processed
- Track processed pairs to avoid re-processing
- Use asset IDs, not filenames

## API Requirements

**From Immich API** (`ExifResponseDto` + `AssetResponseDto`):
- `id` - Asset identifier
- `dateTimeOriginal` - Capture timestamp
- `make` - Camera manufacturer
- `model` - Camera model
- `exifImageWidth` - Image width in pixels
- `exifImageHeight` - Image height in pixels
- `latitude` / `longitude` - GPS coordinates (optional)
- `isTrashed` - Skip trashed assets

All required fields are available in the existing API.

## Implementation Recommendations

### Phase Consolidation

Original v1.1 plan assumed:
- Phase 9: Detection via timestamp + iPhone ID
- Phase 10: Pixel analysis for black bar detection
- Phase 11: CLI command

**Revised plan** (simpler):
- **Phase 9: Detection + Selection** - Combined (no pixel analysis needed)
- **Phase 10: CLI Command** - `letterbox` subcommand with analyze/execute workflow
- ~~Phase 11~~ - Eliminated (absorbed into Phase 9/10)

### Simplified Implementation

1. **No pixel analysis needed** - Detection is purely metadata-based
2. **No HEIC parsing** - Dimensions come from Immich API, not file parsing
3. **Reuse existing infrastructure**:
   - Same analyze → JSON → execute workflow
   - Same backup-before-delete pattern
   - Same rate limiting and error handling

### Risk Assessment

**Risk: Timestamp collision**
- Two different photo pairs taken in same second
- Mitigation: Use GPS as additional discriminator
- Mitigation: Skip ambiguous groups, log for manual review

**Risk: Non-crop 16:9 photos**
- User intentionally shot in 16:9 mode (not a crop)
- These would have different `Live Photo Video Index` values
- With timestamp matching, we can't distinguish
- Mitigation: Only flag as candidates, require user review before deletion

**Risk: API rate limiting**
- Need to fetch all iPhone images for grouping
- Mitigation: Use search API with filters, pagination
- Mitigation: Reuse existing rate limiter from immich-lib

## Conclusion

iPhone 16:9 crop detection is **simpler than anticipated**:
- No pixel analysis required
- All data available via existing Immich API
- Pairing via timestamp + camera matching (fallback from ideal Live Photo Video Index)
- Selection deterministic (4:3 always wins)
- Two implementation phases instead of three

**Next Steps**:
1. Plan Phase 9: Detection + Selection module
2. Plan Phase 10: CLI `letterbox` command
3. Update ROADMAP.md to reflect consolidated phases
