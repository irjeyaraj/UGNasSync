# Changelog

All notable changes to UGNasSync will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-01-13

### Added
- SMB/CIFS mount support for network shares
  - Automatic mounting and unmounting of SMB shares before/after sync
  - Persistent mount support for watch mode
  - Secure credentials file management with 600 permissions
  - Configurable mount options (UID, GID, permissions, caching)
  - Per-profile SMB mount configuration via `use_smb_mount` flag
  - Support for domain authentication
  - Mount point validation and status checking
  - Comprehensive error handling for mount operations
- SMB configuration in `config.toml`:
  - `[nas.smb]` section with share path, mount point, credentials
  - `use_smb_mount` field in sync profiles
- Comprehensive SMB mount documentation (SMB_Mount_Guide.md)
- `dirs` crate dependency for home directory detection
- Complete versioning documentation:
  - CHANGELOG.md following Keep a Changelog format
  - VERSIONING.md with semantic versioning policy
  - VERSION file for version tracking
  - RELEASE_CHECKLIST.md for release process
  - Documentation/README.md as documentation index

### Changed
- Updated `NasConfig` struct to include optional `SmbConfig`
- Updated `SyncProfile` struct to include `use_smb_mount` field
- Modified sync engine to support both SSH/rsync and SMB mount modes
- Enhanced configuration examples with SMB mount scenarios
- Updated README.md with SMB mount feature documentation

### Technical
- New `smb.rs` module implementing mount/unmount operations
- Integration with existing sync workflow in `sync.rs`
- Credentials stored in `~/.ugnassync/smb_credentials/`
- Uses `mount.cifs` for Linux SMB mounting

## [0.1.0] - 2026-01-13

### Added
- Initial release of UGNasSync
- Configuration-driven operation with TOML config files
- Multiple sync types:
  - Mirror (exact copy with deletion)
  - One-way (preserve extra files on destination)
  - Two-way (bidirectional sync with conflict resolution)
  - Incremental (only changed files)
  - Backup (timestamped copies)
- Real-time sync (watch mode) with file system monitoring
- Conflict resolution strategies for two-way sync:
  - Skip conflicting files
  - Overwrite (source wins)
  - Keep both versions
  - Keep newest file
  - Keep largest file
- Comprehensive logging system:
  - Configurable log levels (debug, info, warn, error)
  - File and console output
  - Log rotation with compression
  - Timestamped entries
- SSH-based rsync operations
  - SSH key authentication
  - Password authentication (with warning)
  - Configurable port and credentials
- Per-profile configuration:
  - Multiple sync profiles in single config
  - Exclusion patterns
  - Watch mode per profile
  - Debounce settings
- Command-line interface:
  - `--config` to specify config file
  - `--profile` to run specific profile
  - `--dry-run` for simulation
  - `--verbose` for detailed output
  - `--watch` for real-time sync mode
  - `--version` and `--help` flags
- Systemd integration:
  - Timer unit for scheduled sync
  - Service units for daemon operation
- Rust modules:
  - `config.rs` - Configuration parsing and validation
  - `logging.rs` - Logging setup and management
  - `sync.rs` - Rsync operations and sync engine
  - `watch.rs` - File system monitoring
  - `conflict.rs` - Two-way sync conflict resolution
- Dependencies:
  - serde/toml for config parsing
  - tokio for async operations
  - clap for CLI parsing
  - tracing for logging
  - notify for file watching
  - rusqlite for sync state tracking
- Documentation:
  - Product Specification Document
  - README with usage examples
  - Example configuration file
  - GPL-3.0 license

### Technical Details
- Written in Rust (edition 2021)
- Async/await with tokio runtime
- Cross-platform file system watching
- SQLite database for two-way sync state
- Structured logging with tracing crate

---

## Version History

- **0.2.0** (2026-01-13) - SMB/CIFS mount support
- **0.1.0** (2026-01-13) - Initial release

## Upgrade Notes

### Upgrading to 0.2.0 (from 0.1.0)

**New Dependencies:**
- `dirs = "5.0"` added to Cargo.toml

**Configuration Changes:**
- New optional `[nas.smb]` section for SMB mount configuration
- New `use_smb_mount` field in sync profiles (defaults to `false`)

**Breaking Changes:**
None - all changes are backward compatible.

**New Features:**
- Can now mount SMB/CIFS shares for improved performance
- Mixed mode operation (some profiles use SSH, others use SMB)

**System Requirements:**
- Linux: Install `cifs-utils` package for SMB support
- May require elevated privileges for mounting

## Contributing

See the contribution guidelines for information on how to contribute to UGNasSync.

## License

UGNasSync is licensed under GPL-3.0. See LICENSE.txt for details.

Copyright (c) 2025 Sefier AI
