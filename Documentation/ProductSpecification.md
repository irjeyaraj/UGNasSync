# Product Specification Document: UGNasSync

**Author:** Immanuel Jeyaraj <irj@sefier.com>
**Copyright:** (c) 2025 Sefier AI
**License:** GPL-3.0

## 1. Product Overview

**Product Name:** UGNasSync
**Version:** 0.1.0
**Platform:** Cross-platform (Rust-based)
**Purpose:** Automated synchronization tool for backing up local directories to NAS (Network Attached Storage) shares using rsync protocol.

## 2. Core Features

### 2.1 Configuration-Driven Operation
- Read sync configuration from a TOML/YAML/JSON config file
- Store NAS credentials securely in the config file
- Define multiple sync profiles in a single config file
- Support for sync type specification (one-way, two-way, mirror, etc.)

### 2.2 Rsync Integration
- Execute rsync operations with configurable parameters
- Support multiple sync modes:
  - **Mirror:** Exact copy, delete files not in source
  - **One-way:** Copy from local to NAS, preserve extra files
  - **Incremental:** Only copy changed files
  - **Backup:** Keep versions of deleted/modified files

### 2.3 NAS Authentication
- Store and retrieve NAS credentials from config
- Support for:
  - Username/password authentication
  - SSH key-based authentication
  - SMB/CIFS share mounting

### 2.4 Logging System
- Comprehensive logging of all sync operations
- Configurable log levels (debug, info, warn, error)
- Log file rotation to manage disk space
- Timestamped log entries
- Log both to file and console (configurable)

### 2.5 Real-Time Sync (Watch Mode)
- Monitor source directories for file system changes
- Automatically trigger sync operations when changes are detected
- Configurable per sync profile (enable/disable watch mode)
- Debounce period to batch multiple rapid changes
- Support for recursive directory watching

## 3. Configuration File Structure

### Example Config Format (TOML):
```toml
[nas]
host = "192.168.1.100"
port = 22
username = "admin"
password = "encrypted_password_here"  # or use key_path
# key_path = "/path/to/ssh/key"

[logging]
enabled = true
log_file = "/var/log/ugnassync/sync.log"
log_level = "info"  # debug, info, warn, error
console_output = true
file_output = true

# Log rotation settings
rotate_enabled = true
max_file_size_mb = 10
max_files = 5
compress_rotated = true

[[sync_profiles]]
name = "Documents Backup"
local_path = "/home/user/Documents"
remote_path = "/volume1/backups/Documents"
sync_type = "mirror"
enabled = true
exclude = [".git", "*.tmp", "node_modules"]

# Real-time sync settings
watch_mode = false
debounce_seconds = 5

[[sync_profiles]]
name = "Photos Backup"
local_path = "/home/user/Pictures"
remote_path = "/volume1/backups/Pictures"
sync_type = "one-way"
enabled = true

# Enable real-time sync for photos
watch_mode = true
debounce_seconds = 10

[[sync_profiles]]
name = "Project Files Two-Way Sync"
local_path = "/home/user/Projects"
remote_path = "/volume1/sync/Projects"
sync_type = "two-way"
enabled = true

# Conflict resolution strategy
conflict_resolution = "newest"  # skip, overwrite, keep, newest, largest
```

## 4. Sync Types Specification

- **mirror:** Complete synchronization with deletion of extra files on destination
- **one-way:** Copy from source to destination, preserve extra destination files
- **two-way:** Bidirectional synchronization with conflict resolution
- **incremental:** Transfer only modified/new files
- **backup:** Create timestamped copies of changed files before overwriting

### 4.1 Two-Way Sync Conflict Resolution

When the same file is modified in both source and destination, conflicts must be resolved. The behavior is controlled by the `conflict_resolution` setting in each sync profile.

**Available Conflict Resolution Strategies:**

- **skip:** Skip conflicting files, log warning, continue with other files
- **overwrite:** Always overwrite destination with source (source wins)
- **keep:** Keep both versions, rename destination file with timestamp suffix (`.conflict.YYYYMMDD-HHMMSS`)
- **newest:** Keep the file with the most recent modification time
- **largest:** Keep the file with the larger size

**Configuration:** Set `conflict_resolution` parameter in sync profile (only applies to `sync_type = "two-way"`)

**Default:** If not specified, defaults to `skip` for safety

## 5. Technical Requirements

### 5.1 Dependencies
- rsync binary (must be installed on system)
- SSH client for remote connections
- Rust standard library + external crates:
  - `serde` + `toml`/`serde_yaml` for config parsing
  - `tokio` for async operations
  - `clap` for CLI argument parsing
  - `tracing` + `tracing-subscriber` for logging
  - `tracing-appender` for log file rotation
  - `notify` for file system watching (real-time sync)
  - `rusqlite` for sync state database (two-way sync tracking)

### 5.2 Error Handling
- Validate config file before execution
- Handle network failures gracefully
- Log all errors with timestamps
- Retry logic for transient failures
- Conflict detection and resolution for two-way sync
- Report conflicting files in sync summary

### 5.3 Security
- Never log passwords in plain text
- Support encrypted credential storage
- Validate file permissions on config file (warn if world-readable)
- Option to use SSH keys instead of passwords

### 5.4 Logging Implementation Details
- **Log Levels:**
  - `debug`: Detailed diagnostic information (rsync commands, config parsing details)
  - `info`: General informational messages (sync start/end, files transferred)
  - `warn`: Warning messages (skipped files, retries)
  - `error`: Error messages (connection failures, authentication errors)

- **Log Format:**
  ```
  [2026-01-08T10:15:30Z] [INFO] Starting sync profile: Documents Backup
  [2026-01-08T10:15:31Z] [INFO] Connected to NAS: 192.168.1.100
  [2026-01-08T10:15:35Z] [INFO] Transferred 42 files (1.2 GB) in 5.2s
  [2026-01-08T10:15:35Z] [INFO] Sync completed successfully
  ```

- **Log Rotation:**
  - Rotate when log file reaches `max_file_size_mb`
  - Keep up to `max_files` rotated logs
  - Rotated files named: `sync.log.1`, `sync.log.2`, etc.
  - Optional gzip compression for rotated logs
  - Automatic cleanup of oldest logs when limit reached

- **Logged Events:**
  - Application startup/shutdown
  - Configuration file loading and validation
  - NAS connection attempts and results
  - Each sync profile execution (start, progress, completion)
  - File transfer statistics
  - Errors and warnings with context
  - Retry attempts
  - Performance metrics (duration, throughput)
  - Watch mode events (file changes detected, debounce triggers)

### 5.5 Real-Time Sync Implementation
- **File System Monitoring:**
  - Use `notify` crate for cross-platform file system event monitoring
  - Watch for: create, modify, delete, rename events
  - Recursive watching of all subdirectories
  - Respect exclude patterns when watching

- **Debouncing:**
  - Collect file system events for `debounce_seconds` duration
  - After debounce period expires, trigger single sync operation
  - Prevents excessive sync operations during rapid file changes
  - Example: Save operation triggers multiple events, but only one sync occurs

- **Watch Mode Behavior:**
  - Runs in foreground as daemon when any profile has `watch_mode = true`
  - Performs initial sync for all watch-enabled profiles on startup
  - Continues monitoring until interrupted (Ctrl+C or SIGTERM)
  - Each profile with watch mode runs independently
  - Non-watch profiles are skipped in watch mode

- **Event Handling:**
  ```
  [2026-01-08T10:15:30Z] [INFO] Watch mode enabled for profile: Photos Backup
  [2026-01-08T10:15:30Z] [INFO] Monitoring: /home/user/Pictures
  [2026-01-08T10:20:15Z] [DEBUG] File change detected: /home/user/Pictures/vacation.jpg
  [2026-01-08T10:20:20Z] [INFO] Debounce period elapsed, starting sync...
  [2026-01-08T10:20:25Z] [INFO] Transferred 1 file (2.5 MB) in 5.0s
  ```

- **Resource Management:**
  - Limit concurrent sync operations (one per profile at a time)
  - Queue additional changes if sync is already in progress
  - Graceful shutdown on signal interruption

### 5.6 Two-Way Sync and Conflict Resolution Implementation

- **Conflict Detection:**
  - Compare modification timestamps and file sizes between source and destination
  - Detect conflicts when both source and destination have been modified since last sync
  - Maintain sync state database to track last known sync timestamps

- **Conflict Resolution Strategies:**

  **skip:**
  ```
  [2026-01-08T10:15:30Z] [WARN] Conflict detected: /home/user/Projects/file.txt
  [2026-01-08T10:15:30Z] [WARN] Skipping conflicting file per conflict_resolution policy
  ```
  - Leave both files unchanged
  - Log warning with file path
  - Continue with remaining files

  **overwrite:**
  ```
  [2026-01-08T10:15:30Z] [WARN] Conflict detected: /home/user/Projects/file.txt
  [2026-01-08T10:15:30Z] [INFO] Overwriting destination with source (source wins)
  ```
  - Source file overwrites destination
  - No backup created

  **keep:**
  ```
  [2026-01-08T10:15:30Z] [WARN] Conflict detected: /home/user/Projects/file.txt
  [2026-01-08T10:15:30Z] [INFO] Keeping both versions
  [2026-01-08T10:15:30Z] [INFO] Renamed destination: file.txt.conflict.20260108-101530
  ```
  - Rename destination file: `filename.ext.conflict.YYYYMMDD-HHMMSS`
  - Copy source file to destination
  - Both versions preserved

  **newest:**
  ```
  [2026-01-08T10:15:30Z] [WARN] Conflict detected: /home/user/Projects/file.txt
  [2026-01-08T10:15:30Z] [INFO] Keeping newest version (source: 2026-01-08, dest: 2026-01-07)
  ```
  - Compare modification timestamps
  - Keep file with most recent timestamp
  - Overwrite older version

  **largest:**
  ```
  [2026-01-08T10:15:30Z] [WARN] Conflict detected: /home/user/Projects/file.txt
  [2026-01-08T10:15:30Z] [INFO] Keeping largest version (source: 2.5MB, dest: 1.8MB)
  ```
  - Compare file sizes
  - Keep larger file
  - Overwrite smaller version

- **Sync State Database:**
  - Store in `~/.ugnassync/sync_state.db` (SQLite)
  - Track: file path, last sync timestamp, file hash
  - Use to detect three-way conflicts (both modified since last sync)
  - Clean up stale entries periodically

## 6. User Interface

### 6.1 Command-Line Interface
```bash
# Show help
ugnassync --help

# Run with default config
ugnassync

# Specify config file
ugnassync --config /path/to/config.toml

# Dry run mode
ugnassync --dry-run

# Verbose output
ugnassync --verbose

# Run specific profile only
ugnassync --profile "Documents Backup"

# Enable watch mode (real-time sync)
ugnassync --watch

# Show version
ugnassync --version
```

**Help Output Example:**
```
UGNasSync v0.1.0
Automated NAS synchronization tool using rsync

Author: Immanuel Jeyaraj <irj@sefier.com>
Copyright (c) 2025 Sefier AI
License: GPL-3.0

USAGE:
    ugnassync [OPTIONS]

OPTIONS:
    -c, --config <FILE>       Path to config file [default: ./config.toml]
    -p, --profile <NAME>      Run only the specified sync profile
    -d, --dry-run             Simulate sync without making changes
    -v, --verbose             Enable verbose output (overrides config log level)
    -w, --watch               Enable watch mode for real-time sync (runs as daemon)
    -h, --help                Print help information
    -V, --version             Print version information

EXAMPLES:
    ugnassync
    ugnassync --config /etc/ugnassync/config.toml
    ugnassync --profile "Documents Backup" --dry-run
    ugnassync --verbose
    ugnassync --watch --config /etc/ugnassync/config.toml
```

**Version Output Example:**
```
UGNasSync v0.1.0
Author: Immanuel Jeyaraj <irj@sefier.com>
Copyright (c) 2025 Sefier AI
License: GPL-3.0
```

### 6.2 Output
- Progress indication for each sync operation
- Summary statistics (files transferred, bytes, duration)
- Error reporting with actionable messages
- Conflict resolution summary for two-way sync:
  ```
  Sync Summary:
  Profile: Project Files Two-Way Sync
  Files transferred: 42
  Bytes transferred: 15.3 MB
  Conflicts detected: 3
    - Skipped: 1
    - Resolved (newest): 2
  Duration: 12.5s
  Status: Completed with warnings
  ```

## 7. Scheduling and Automation

### 7.1 Systemd Integration
UGNasSync includes systemd service and timer units for automated scheduling on Linux systems.

**Files:** Located in `etc/systemd/`
- `ugnassync.service` - Service unit definition
- `ugnassync.timer` - Timer unit for scheduling
- `README.md` - Installation and usage instructions

**Default Schedule:**
- Runs daily at 2:00 AM
- Executes 5 minutes after boot if last run was missed
- Persistent across reboots
- Random delay of up to 10 minutes to avoid network congestion

**Installation:**
```bash
sudo cp etc/systemd/ugnassync.* /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable ugnassync.timer
sudo systemctl start ugnassync.timer
```

**Management:**
```bash
# Check timer status
systemctl status ugnassync.timer

# View next scheduled runs
systemctl list-timers ugnassync.timer

# View service logs
journalctl -u ugnassync.service

# Run manually
sudo systemctl start ugnassync.service
```

## 8. Future Enhancements (Out of Scope for v0.1)
- GUI interface
- Email notifications on completion/failure
- Bandwidth throttling
- Web dashboard for monitoring
- Windows Task Scheduler support
- Interactive conflict resolution mode (prompt user for each conflict)
- Custom conflict resolution scripts

## 9. Success Criteria
- Successfully read and parse config file
- Authenticate with NAS using provided credentials
- Execute rsync with correct parameters based on sync type
- Handle at least 3 different sync profiles in one execution
- Complete sync of 1GB+ data without errors
- Proper error messages for common failure scenarios
