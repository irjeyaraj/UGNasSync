# UGNasSync Documentation

This directory contains comprehensive documentation for UGNasSync.

## Documentation Index

### User Documentation

- **[Product Specification](ProductSpecification.md)**
  - Complete product specification document
  - Feature descriptions and technical requirements
  - Configuration examples
  - Implementation details

- **[SMB Mount Guide](SMB_Mount_Guide.md)** ⭐ NEW
  - Complete guide to SMB/CIFS mount support
  - Configuration examples
  - Performance considerations
  - Troubleshooting guide
  - Security best practices

### Project Documentation

- **[Changelog](../CHANGELOG.md)**
  - Version history
  - Feature additions and changes
  - Bug fixes and improvements
  - Upgrade notes

- **[Versioning Policy](../VERSIONING.md)**
  - Semantic versioning rules
  - Release process
  - Version compatibility guidelines
  - Breaking change policy

- **[Release Checklist](RELEASE_CHECKLIST.md)**
  - Pre-release checklist
  - Step-by-step release process
  - Testing procedures
  - Post-release tasks

### Quick Start

New to UGNasSync? Start here:

1. **[README.md](../README.md)** - Overview, features, and basic usage
2. **[config.toml.example](../config.toml.example)** - Example configuration
3. **[Product Specification](ProductSpecification.md)** - Detailed feature documentation

### Configuration Guides

#### Basic Setup
```bash
# 1. Copy example config
cp config.toml.example config.toml

# 2. Edit with your settings
vim config.toml

# 3. Test with dry run
ugnassync --config config.toml --dry-run

# 4. Run actual sync
ugnassync --config config.toml
```

#### SSH/Rsync Mode (Traditional)
```toml
[nas]
host = "192.168.1.100"
port = 22
username = "admin"
key_path = "/home/user/.ssh/id_rsa"

[[sync_profiles]]
name = "Documents Backup"
local_path = "/home/user/Documents"
remote_path = "/volume1/backups/Documents"
sync_type = "mirror"
enabled = true
use_smb_mount = false  # SSH/rsync mode
```

See: [Product Specification - Configuration](ProductSpecification.md#3-configuration-file-structure)

#### SMB Mount Mode (New)
```toml
[nas.smb]
enabled = true
share_path = "//192.168.1.100/backups"
mount_point = "/mnt/nas"
username = "admin"
password = "smb_password"
auto_unmount = true

[[sync_profiles]]
name = "Documents via SMB"
local_path = "/home/user/Documents"
remote_path = "/mnt/nas/Documents"
sync_type = "mirror"
enabled = true
use_smb_mount = true  # SMB mount mode
```

See: [SMB Mount Guide](SMB_Mount_Guide.md)

### Feature Guides

#### Sync Types
- **Mirror** - Exact copy, delete files not in source
- **One-way** - Copy to destination, keep extra files
- **Two-way** - Bidirectional with conflict resolution
- **Incremental** - Only changed files
- **Backup** - Keep versions of changed files

See: [Product Specification - Sync Types](ProductSpecification.md#4-sync-types-specification)

#### Real-Time Sync (Watch Mode)
Monitor directories and sync automatically when files change:

```toml
[[sync_profiles]]
watch_mode = true
debounce_seconds = 5
```

```bash
ugnassync --watch
```

See: [Product Specification - Watch Mode](ProductSpecification.md#28-real-time-sync-watch-mode)

#### Conflict Resolution
For two-way sync, configure how conflicts are handled:

```toml
[[sync_profiles]]
sync_type = "two-way"
conflict_resolution = "newest"  # skip, overwrite, keep, newest, largest
```

See: [Product Specification - Conflict Resolution](ProductSpecification.md#41-two-way-sync-conflict-resolution)

### Advanced Topics

#### Systemd Integration
Run UGNasSync as a scheduled service or daemon:

```bash
# Install timer for scheduled sync
sudo cp etc/systemd/ugnassync.timer /etc/systemd/system/
sudo systemctl enable ugnassync.timer
sudo systemctl start ugnassync.timer

# Install service for watch mode daemon
sudo cp etc/systemd/ugnassync-watch.service /etc/systemd/system/
sudo systemctl enable ugnassync-watch.service
sudo systemctl start ugnassync-watch.service
```

See: [Product Specification - Scheduling](ProductSpecification.md#7-scheduling-and-automation)

#### Logging Configuration
Comprehensive logging with rotation:

```toml
[logging]
enabled = true
log_file = "/var/log/ugnassync/sync.log"
log_level = "info"
console_output = true
file_output = true
rotate_enabled = true
max_file_size_mb = 10
max_files = 5
compress_rotated = true
```

See: [Product Specification - Logging](ProductSpecification.md#24-logging-system)

### Command-Line Reference

```bash
# Basic usage
ugnassync                                    # Use default config
ugnassync --config /path/to/config.toml     # Specify config
ugnassync --profile "Profile Name"          # Run specific profile
ugnassync --dry-run                          # Simulate without changes
ugnassync --verbose                          # Detailed output
ugnassync --watch                            # Real-time sync mode

# Information
ugnassync --help                             # Show help
ugnassync --version                          # Show version
```

### Troubleshooting

#### Common Issues

**Permission Denied (SMB Mount)**
```bash
# Run with sudo
sudo ugnassync

# Or configure user permissions
sudo usermod -aG disk $USER
```
See: [SMB Mount Guide - Troubleshooting](SMB_Mount_Guide.md#troubleshooting)

**Connection Refused (SSH)**
- Check SSH port in config
- Verify SSH access: `ssh user@host`
- Check firewall rules

**Rsync Not Found**
```bash
# Install rsync
sudo apt install rsync        # Debian/Ubuntu
sudo dnf install rsync        # Fedora
```

#### Getting Help

1. Check log files: `/var/log/ugnassync/sync.log`
2. Run with `--verbose` flag
3. Review error messages carefully
4. Check configuration syntax
5. Test connectivity manually

### Development Documentation

For contributors and developers:

#### Project Structure
```
UGNasSync/
├── src/
│   ├── main.rs          # Entry point and CLI
│   ├── config.rs        # Configuration parsing
│   ├── sync.rs          # Sync engine and rsync operations
│   ├── smb.rs           # SMB mount management
│   ├── watch.rs         # File system watching
│   ├── conflict.rs      # Conflict resolution
│   └── logging.rs       # Logging setup
├── Documentation/
│   ├── ProductSpecification.md
│   ├── SMB_Mount_Guide.md
│   ├── RELEASE_CHECKLIST.md
│   └── README.md (this file)
├── etc/systemd/         # Systemd unit files
├── Cargo.toml           # Rust package manifest
├── CHANGELOG.md         # Version history
├── VERSIONING.md        # Versioning policy
└── VERSION              # Current version
```

#### Building from Source
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code
cargo check
cargo clippy
```

#### Release Process
See: [Release Checklist](RELEASE_CHECKLIST.md)

### Version Information

**Current Version:** 0.2.0
**Status:** Stable Release

**Recent Additions:**
- SMB/CIFS mount support (v0.2.0)
- Watch mode for real-time sync
- Conflict resolution for two-way sync
- Comprehensive logging with rotation

**Version History:**
- 0.2.0 (2026-01-13) - SMB/CIFS mount support
- 0.1.0 (2026-01-13) - Initial release
- See [CHANGELOG.md](../CHANGELOG.md) for complete history

**Versioning:**
UGNasSync follows [Semantic Versioning 2.0.0](https://semver.org/)
See [VERSIONING.md](../VERSIONING.md) for details

### Contributing

#### Reporting Issues
When reporting issues, include:
- Version: `ugnassync --version`
- Operating system and version
- Configuration (sanitize credentials!)
- Error messages and log output
- Steps to reproduce

#### Feature Requests
Feature requests are welcome! Please include:
- Use case description
- Expected behavior
- Example configuration
- Benefits to users

### License

UGNasSync is licensed under GPL-3.0.

Copyright (c) 2025 Sefier AI
Author: Immanuel Jeyaraj <irj@sefier.com>

See [LICENSE.txt](../LICENSE.txt) for full license text.

### Additional Resources

- **Example Config:** [config.toml.example](../config.toml.example)
- **Main README:** [README.md](../README.md)
- **License:** [LICENSE.txt](../LICENSE.txt)
- **Changelog:** [CHANGELOG.md](../CHANGELOG.md)

### Document Version

**Version:** 1.0
**Last Updated:** 2026-01-13
**Maintainer:** Immanuel Jeyaraj <irj@sefier.com>

---

**Need help?** Start with the [README.md](../README.md) for quick start, or dive into the [Product Specification](ProductSpecification.md) for comprehensive documentation.
