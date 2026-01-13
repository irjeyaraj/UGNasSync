# Release Checklist

This document provides a step-by-step checklist for releasing a new version of UGNasSync.

## Pre-Release Checklist

### Code Quality

- [ ] All tests pass
  ```bash
  cargo test
  cargo test --release
  ```

- [ ] Code compiles without errors
  ```bash
  cargo check
  cargo build --release
  ```

- [ ] No critical compiler warnings
  ```bash
  cargo clippy -- -D warnings
  ```

- [ ] Code formatting is correct
  ```bash
  cargo fmt --check
  ```

- [ ] Dependencies are up to date
  ```bash
  cargo update
  cargo audit
  ```

### Documentation

- [ ] CHANGELOG.md updated with all changes
  - [ ] Moved [Unreleased] section to new version
  - [ ] Added release date
  - [ ] Categorized changes (Added, Changed, Fixed, Removed, Security)
  - [ ] Included upgrade notes if applicable

- [ ] README.md is current
  - [ ] Features list is accurate
  - [ ] Installation instructions are correct
  - [ ] Examples work with current version

- [ ] Version numbers updated in:
  - [ ] Cargo.toml (`version = "X.Y.Z"`)
  - [ ] VERSION file
  - [ ] Documentation references (if any)

- [ ] API documentation is current
  ```bash
  cargo doc --no-deps
  ```

- [ ] Example config file is up to date
  - [ ] config.toml.example reflects all options

### Testing

- [ ] Manual testing completed
  - [ ] Test basic sync operation
  - [ ] Test dry-run mode
  - [ ] Test verbose output
  - [ ] Test watch mode
  - [ ] Test each sync type (mirror, one-way, two-way, incremental, backup)
  - [ ] Test SMB mount (if applicable)
  - [ ] Test conflict resolution strategies
  - [ ] Test with invalid config
  - [ ] Test with missing dependencies

- [ ] Integration tests pass
  ```bash
  cargo test --test '*'
  ```

- [ ] Example configuration works
  ```bash
  cp config.toml.example test-config.toml
  # Edit test-config.toml with test values
  ./target/release/ugnassync --config test-config.toml --dry-run
  ```

- [ ] Command-line options work
  ```bash
  ./target/release/ugnassync --help
  ./target/release/ugnassync --version
  ```

### Security

- [ ] No hardcoded credentials in code
- [ ] No sensitive data in logs
- [ ] Dependencies have no known vulnerabilities
  ```bash
  cargo audit
  ```
- [ ] File permissions are secure (600 for credentials)
- [ ] Security best practices documented

### Platform Testing

- [ ] Linux (primary platform)
  - [ ] Debian/Ubuntu
  - [ ] Fedora/RHEL
  - [ ] Arch Linux

- [ ] Dependency availability verified
  - [ ] rsync available
  - [ ] SSH client available
  - [ ] cifs-utils available (for SMB)

## Release Process

### 1. Version Bump

- [ ] Determine version number (MAJOR.MINOR.PATCH)
  - [ ] Review VERSIONING.md for guidelines
  - [ ] Breaking changes = MAJOR bump
  - [ ] New features = MINOR bump
  - [ ] Bug fixes = PATCH bump

- [ ] Update version in Cargo.toml
  ```toml
  [package]
  version = "X.Y.Z"
  ```

- [ ] Update VERSION file
  ```bash
  echo "X.Y.Z" > VERSION
  ```

- [ ] Update Cargo.lock
  ```bash
  cargo update -p UGNasSync
  ```

### 2. Update Changelog

- [ ] Edit CHANGELOG.md
  ```markdown
  ## [X.Y.Z] - YYYY-MM-DD

  ### Added
  - Feature descriptions

  ### Changed
  - Change descriptions

  ### Fixed
  - Bug fix descriptions
  ```

- [ ] Add upgrade notes if needed
- [ ] Link version at bottom of CHANGELOG.md
  ```markdown
  [X.Y.Z]: https://github.com/user/repo/compare/vX.Y.Z-1...vX.Y.Z
  ```

### 3. Final Testing

- [ ] Clean build
  ```bash
  cargo clean
  cargo build --release
  ```

- [ ] Run full test suite
  ```bash
  cargo test --all
  ```

- [ ] Test release binary
  ```bash
  ./target/release/ugnassync --version
  ./target/release/ugnassync --help
  ```

- [ ] Verify binary size is reasonable
  ```bash
  ls -lh target/release/ugnassync
  ```

### 4. Commit and Tag

- [ ] Stage changes
  ```bash
  git add Cargo.toml Cargo.lock VERSION CHANGELOG.md
  git add Documentation/  # If documentation updated
  ```

- [ ] Commit version bump
  ```bash
  git commit -m "chore: bump version to X.Y.Z"
  ```

- [ ] Create annotated tag
  ```bash
  git tag -a vX.Y.Z -m "Release version X.Y.Z"
  ```

- [ ] Verify tag
  ```bash
  git tag -v vX.Y.Z
  git show vX.Y.Z
  ```

### 5. Push Release

- [ ] Push commits
  ```bash
  git push origin main
  ```

- [ ] Push tag
  ```bash
  git push origin vX.Y.Z
  ```

- [ ] Verify on remote
  ```bash
  git ls-remote --tags origin
  ```

### 6. Build Release Artifacts

- [ ] Build release binary
  ```bash
  cargo build --release --target x86_64-unknown-linux-gnu
  ```

- [ ] Strip binary (optional, reduces size)
  ```bash
  strip target/release/ugnassync
  ```

- [ ] Create tarball
  ```bash
  tar czf ugnassync-vX.Y.Z-linux-x86_64.tar.gz \
    -C target/release ugnassync \
    -C ../../ README.md LICENSE.txt \
    -C config.toml.example
  ```

- [ ] Generate checksums
  ```bash
  sha256sum ugnassync-vX.Y.Z-linux-x86_64.tar.gz > checksums.txt
  ```

### 7. Create GitHub Release (if applicable)

- [ ] Go to GitHub releases page
- [ ] Click "Draft a new release"
- [ ] Select tag vX.Y.Z
- [ ] Set release title: "UGNasSync vX.Y.Z"
- [ ] Copy relevant CHANGELOG.md section to description
- [ ] Upload release artifacts:
  - [ ] Binary tarball
  - [ ] Checksums file
- [ ] Mark as pre-release if version < 1.0.0
- [ ] Publish release

### 8. Verify Release

- [ ] Download release artifacts
- [ ] Verify checksums
  ```bash
  sha256sum -c checksums.txt
  ```

- [ ] Test downloaded binary
  ```bash
  tar xzf ugnassync-vX.Y.Z-linux-x86_64.tar.gz
  ./ugnassync --version
  ```

### 9. Documentation Updates

- [ ] Update website (if applicable)
- [ ] Update installation instructions
- [ ] Announce release (if applicable):
  - [ ] Mailing list
  - [ ] Social media
  - [ ] Project website

### 10. Post-Release

- [ ] Create new [Unreleased] section in CHANGELOG.md
  ```markdown
  ## [Unreleased]

  ### Added

  ### Changed

  ### Fixed
  ```

- [ ] Commit changelog update
  ```bash
  git add CHANGELOG.md
  git commit -m "chore: prepare changelog for next release"
  git push origin main
  ```

- [ ] Monitor for issues
  - [ ] Check issue tracker
  - [ ] Monitor logs for error reports
  - [ ] Review user feedback

## Hotfix Release Checklist

For critical bug fixes that need immediate release:

- [ ] Create hotfix branch from tag
  ```bash
  git checkout -b hotfix/X.Y.Z+1 vX.Y.Z
  ```

- [ ] Fix bug with minimal changes
- [ ] Bump PATCH version only
- [ ] Update CHANGELOG.md with fix
- [ ] Test thoroughly
- [ ] Follow steps 4-10 from main release process
- [ ] Merge hotfix back to main
  ```bash
  git checkout main
  git merge hotfix/X.Y.Z+1
  git push origin main
  ```

## Rollback Procedure

If a release has critical issues:

1. **Immediate:**
   - [ ] Mark release as problematic in GitHub
   - [ ] Update documentation with warning
   - [ ] Communicate to users

2. **Short-term:**
   - [ ] Revert problematic commits
   - [ ] Release hotfix with PATCH bump
   - [ ] Update CHANGELOG.md

3. **Never:**
   - Do not delete git tags
   - Do not force-push to main
   - Do not reuse version numbers

## Version-Specific Notes

### v0.1.0 (Initial Release)
- First public release
- Baseline features established
- Documentation complete

### v0.2.0 (SMB Mount Support)
- Test SMB mounting thoroughly
- Verify credentials file security
- Test on multiple Linux distributions
- Confirm mount/unmount cycles work
- Test watch mode with persistent mounts

## Contact

For release questions:
- Maintainer: Immanuel Jeyaraj <irj@sefier.com>
- Repository: [GitHub URL]

## References

- [VERSIONING.md](../VERSIONING.md)
- [CHANGELOG.md](../CHANGELOG.md)
- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)

---

**Document Version:** 1.0
**Last Updated:** 2026-01-13
