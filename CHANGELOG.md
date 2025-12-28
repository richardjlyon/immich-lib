# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [1.2.0] - 2025-12-28

### Added
- **Configuration File Support** - Credentials saved to OS-native config locations
  - macOS: `~/Library/Application Support/immich-dupes/config.toml`
  - Linux: `~/.config/immich-dupes/config.toml`
  - Windows: `AppData/Roaming/immich-dupes/config.toml`
- **Interactive Prompts** - URL and API key prompts when credentials not provided
- **`--save` Flag** - Persist credentials after successful command execution
- **Credential Resolution Chain** - CLI args > env vars > config file > interactive prompt

## [1.1.0] - 2025-12-28

### Added
- **iPhone Letterbox Detection** - New `letterbox` subcommand to find and remove 4:3/16:9 crop pairs
  - `letterbox analyze` - Scans all assets for iPhone letterbox pairs
  - `letterbox execute` - Downloads 16:9 backups and deletes them from Immich
  - `letterbox verify` - Validates keepers exist and deletes are removed
- Progress bar for letterbox execution (matches duplicate removal UX)

### Fixed
- Fixed `get_all_assets` API call - changed from non-existent `GET /api/assets` to correct `POST /api/search/metadata` endpoint

## [1.0.0] - 2025-12-27

### Added
- Initial release
- **Duplicate Analysis** - Scans Immich duplicate groups and scores by metadata completeness
- **Smart Selection** - Selects winners by largest dimensions (width × height)
- **Metadata Consolidation** - Transfers GPS, timezone, description from losers to winners
- **Safe Execution** - Downloads backups before any deletions
- **Verification** - Validates winners exist and losers are deleted post-execution
- **Restore** - Re-uploads backed-up files if needed
- Rate limiting and concurrent operation controls
- Conflict detection for groups needing manual review
