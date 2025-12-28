# Project Milestones: immich-lib

## v1.2 Configuration UX (Shipped: 2025-12-28)

**Delivered:** Config file support with OS-native location, interactive setup prompts, and credential persistence for improved UX.

**Phases completed:** 11 (2 plans total)

**Key accomplishments:**

- Config module with OS-native paths via directories crate (macOS/Linux/Windows)
- TOML configuration format for human-readable settings
- CLI credential resolution chain: CLI args > env vars > config file > error
- Interactive credential prompts with dialoguer (URL validation, hidden API key)
- --save flag for credential persistence after successful commands

**Stats:**

- 9 files created/modified
- 9,004 lines of Rust (total)
- 1 phase, 2 plans
- Same day execution (2025-12-28, ~13 min)

**Git range:** `feat(11-01)` → `feat(11-02)`

**What's next:** Production use with simplified credential management

---

## v1.1 iPhone Letterbox Duplicates (Shipped: 2025-12-28)

**Delivered:** Detect and remove iPhone 4:3/16:9 crop duplicates using timestamp + camera matching, keeping the full 4:3 sensor capture.

**Phases completed:** 8-10 (5 plans total)

**Key accomplishments:**

- Metadata-based letterbox detection via timestamp + camera matching
- Letterbox detection module with AspectRatio enum, LetterboxPair struct, 25 unit tests
- Paginated asset fetching (get_all_assets) for scanning entire Immich library
- Complete CLI workflow: `letterbox analyze`, `letterbox execute`, `letterbox verify`
- Backup-before-delete safety with rate limiting

**Stats:**

- 21 files created/modified
- 6,882 lines of Rust (total)
- 3 phases, 5 plans
- 1 day from start to ship (2025-12-28)

**Git range:** `feat(08-01)` → `feat(10-02)`

**What's next:** Production use against real Immich library with iPhone letterbox duplicates

---

## v1.0 MVP (Shipped: 2025-12-27)

**Delivered:** A Rust CLI tool that safely de-duplicates Immich photos by selecting the highest-quality image (by dimensions) while preserving metadata completeness through consolidation.

**Phases completed:** 1-7 (23 plans total)

**Key accomplishments:**

- Rust API client with authentication, streaming downloads, bulk delete
- Metadata scoring algorithm prioritizing GPS, timezone, camera info
- Conflict detection for GPS/timezone/camera discrepancies
- Two-stage workflow: `analyze` outputs JSON, `execute` processes with backups
- Metadata consolidation transfers GPS/timezone from losers to winners
- 24 unit tests + live instance validation with restore capability

**Stats:**

- 134 files created/modified
- 6,880 lines of Rust
- 7 phases, 23 plans
- 2 days from start to ship (2025-12-26 → 2025-12-27)

**Git range:** `feat(01-01)` → `feat(07-03)`

**What's next:** Production use against real Immich library with 2000+ duplicates

---
