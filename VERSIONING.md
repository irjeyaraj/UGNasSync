# Versioning Policy

**UGNasSync** follows [Semantic Versioning 2.0.0](https://semver.org/).

## Version Format

Version numbers follow the format: **MAJOR.MINOR.PATCH**

Example: `0.1.0`, `1.2.3`, `2.0.0`

## Semantic Versioning Rules

### MAJOR version (X.0.0)
Incremented when making **incompatible API changes** or **breaking changes**:
- Changes to config file format that require manual migration
- Removal of command-line options or flags
- Changes to default behavior that could break existing workflows
- Removal of features
- Changes to log format that break log parsers

**Example:** `1.5.2` → `2.0.0`

### MINOR version (0.X.0)
Incremented when adding **new functionality** in a **backward-compatible** manner:
- New sync types
- New configuration options (with backward-compatible defaults)
- New command-line flags or options
- New features (e.g., SMB mount support)
- Performance improvements
- Non-breaking enhancements

**Example:** `1.5.2` → `1.6.0`

### PATCH version (0.0.X)
Incremented for **backward-compatible bug fixes**:
- Bug fixes that don't change functionality
- Security patches
- Documentation fixes
- Dependency updates without feature changes
- Performance optimizations without behavior changes

**Example:** `1.5.2` → `1.5.3`

## Pre-1.0.0 Development

While in `0.x.y` versions (pre-1.0):
- The API is not considered stable
- Breaking changes may occur in MINOR versions
- PATCH versions are still for bug fixes only
- Version `1.0.0` will mark the first stable release

**Current Status:** UGNasSync is at version `0.2.0`, indicating early development.

## Version Tracking

Versions are tracked in multiple locations:

1. **Cargo.toml** - Rust package version
   ```toml
   [package]
   version = "0.1.0"
   ```

2. **VERSION** - Plain text version file
   ```
   0.1.0
   ```

3. **CHANGELOG.md** - Detailed version history with changes

4. **Git tags** - Release tags in format `vX.Y.Z`
   ```bash
   git tag v0.1.0
   ```

## Release Process

### 1. Update Version Numbers

Update version in all locations:

```bash
# Update Cargo.toml
vim Cargo.toml  # Change version = "0.1.0" to new version

# Update VERSION file
echo "0.2.0" > VERSION

# Update CHANGELOG.md
vim CHANGELOG.md  # Move [Unreleased] to [0.2.0] - YYYY-MM-DD
```

### 2. Update Changelog

Move unreleased changes to the new version section:

```markdown
## [0.2.0] - 2026-01-20

### Added
- SMB/CIFS mount support
...

## [0.1.0] - 2026-01-13
...
```

### 3. Commit Version Bump

```bash
git add Cargo.toml Cargo.lock VERSION CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
```

### 4. Create Git Tag

```bash
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
git push origin main
```

### 5. Build Release

```bash
cargo build --release
cargo test
```

### 6. Create Release Notes

Create release notes based on CHANGELOG.md for the version.

## Version Compatibility

### Configuration File Compatibility

Configuration files should be forward and backward compatible within the same MAJOR version:

- **Forward compatible:** Newer software can read older config files
- **Backward compatible:** Older software can read newer config files (ignoring unknown fields)

**Implementation:**
- Use `#[serde(default)]` for new optional fields
- Use `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields
- Never remove required fields in MINOR versions
- Provide migration tools for MAJOR version upgrades

### Example: v0.1.0 → v0.2.0 (SMB Mount Feature)

**Backward Compatible:**
```toml
# Old config (v0.1.0) still works in v0.2.0
[nas]
host = "192.168.1.100"
username = "admin"
# No SMB config - software uses defaults
```

**New Features Available:**
```toml
# New config (v0.2.0) with SMB support
[nas]
host = "192.168.1.100"
username = "admin"

[nas.smb]  # New section - optional
enabled = true
share_path = "//192.168.1.100/backups"
```

**Result:** Old configs work without modification; new features opt-in.

## Breaking Change Guidelines

When breaking changes are necessary (MAJOR version bump):

1. **Announce in advance** - Document in changelog as "Deprecated"
2. **Provide migration path** - Document how to upgrade
3. **Migration tool** - Provide tool/script if possible
4. **Clear error messages** - Help users understand what broke
5. **Update documentation** - Reflect all breaking changes

### Example Breaking Change

```markdown
## [2.0.0] - 2026-06-01

### BREAKING CHANGES
- Configuration file format changed from TOML to YAML
- Run `ugnassync migrate-config` to convert old configs
- Old `sync_type` values renamed:
  - "one-way" → "oneway"
  - "two-way" → "twoway"

### Migration
1. Backup your config: `cp config.toml config.toml.backup`
2. Run migration: `ugnassync migrate-config config.toml > config.yaml`
3. Test new config: `ugnassync --config config.yaml --dry-run`
```

## Development Branches

- **main** - Stable releases only (tagged versions)
- **develop** - Active development, unreleased features
- **feature/** - Feature branches for new functionality
- **hotfix/** - Critical bug fixes for releases

## Release Cadence

**Target Release Schedule:**
- **MAJOR:** As needed for breaking changes
- **MINOR:** Monthly or when significant features are ready
- **PATCH:** As needed for critical bugs

**Version 0.x Development:**
- More frequent releases during early development
- May release MINOR versions weekly during active development

## Version Support

**Support Policy:**
- **Latest MAJOR.MINOR:** Full support (features + bug fixes)
- **Previous MINOR:** Security fixes only for 6 months
- **Older versions:** Unsupported (users should upgrade)

**Example (when at v2.3.x):**
- v2.3.x - Full support
- v2.2.x - Security fixes until v2.4.0 + 6 months
- v2.1.x and older - Unsupported

## Checking Version

Users can check the version using:

```bash
# Show version
ugnassync --version

# Output:
# UGNasSync v0.1.0
# Author: Immanuel Jeyaraj <irj@sefier.com>
# Copyright (c) 2025 Sefier AI
# License: GPL-3.0
```

## Version in Code

Version is defined in `Cargo.toml` and accessed via environment variable:

```rust
const VERSION: &str = env!("CARGO_PKG_VERSION");

println!("UGNasSync v{}", VERSION);
```

## Deprecation Policy

Features scheduled for removal:

1. **Mark as deprecated** in MINOR version
2. **Keep functional** for at least 2 MINOR versions
3. **Log deprecation warning** when used
4. **Document alternative** in warning and docs
5. **Remove** in next MAJOR version

**Example:**
```
v1.5.0 - Feature X marked deprecated, use Feature Y
v1.6.0 - Feature X still works, shows warning
v1.7.0 - Feature X still works, shows warning
v2.0.0 - Feature X removed
```

## Version Comparison

UGNasSync versions can be compared:

```bash
# Compare versions
v0.1.0 < v0.2.0 < v1.0.0 < v1.1.0 < v2.0.0

# Pre-release versions (if used)
v1.0.0-alpha < v1.0.0-beta < v1.0.0-rc1 < v1.0.0
```

## References

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Cargo Versioning](https://doc.rust-lang.org/cargo/reference/semver.html)

---

**Document Version:** 1.0
**Last Updated:** 2026-01-13
**Maintained By:** Immanuel Jeyaraj <irj@sefier.com>
