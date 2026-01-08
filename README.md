# UGNasSync

**Author:** Immanuel Jeyaraj <irj@sefier.com>
**Copyright:** (c) 2025 Sefier AI
**License:** GPL-3.0

Automated NAS synchronization tool using rsync protocol.

## Features

- **Configuration-driven operation** - Define multiple sync profiles in a single TOML config file
- **Multiple sync types** - Mirror, one-way, two-way, incremental, and backup modes
- **Real-time sync (watch mode)** - Automatically sync when files change
- **Conflict resolution** - Handle conflicts in two-way sync with configurable strategies
- **Comprehensive logging** - File and console logging with rotation support
- **Systemd integration** - Includes service and timer units for scheduled and daemon operation

## Installation

### Prerequisites

- Rust toolchain (1.70 or later)
- rsync binary installed on your system
- SSH access to your NAS

### Building from source

```bash
git clone <repository-url>
cd UGNasSync
cargo build --release
sudo cp target/release/ugnassync /usr/local/bin/
```

## Configuration

Create a configuration file (see `config.toml.example` for a complete example):

```toml
[nas]
host = "192.168.1.100"
port = 22
username = "admin"
key_path = "/home/user/.ssh/id_rsa"  # Recommended over password

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

[[sync_profiles]]
name = "Documents Backup"
local_path = "/home/user/Documents"
remote_path = "/volume1/backups/Documents"
sync_type = "mirror"
enabled = true
exclude = [".git", "*.tmp", "node_modules"]
watch_mode = false
debounce_seconds = 5
```

## Usage

### Basic Usage

```bash
# Run with default config
ugnassync

# Specify config file
ugnassync --config /etc/ugnassync/config.toml

# Dry run (don't actually transfer files)
ugnassync --dry-run

# Verbose output
ugnassync --verbose

# Run specific profile only
ugnassync --profile "Documents Backup"
```

### Watch Mode (Real-time Sync)

```bash
# Enable watch mode for profiles with watch_mode = true
ugnassync --watch
```

### Show Version and Help

```bash
# Show version
ugnassync --version

# Show help
ugnassync --help
```

## Sync Types

- **mirror** - Complete synchronization with deletion of extra files on destination
- **one-way** - Copy from source to destination, preserve extra destination files
- **two-way** - Bidirectional synchronization with conflict resolution
- **incremental** - Transfer only modified/new files
- **backup** - Create timestamped copies of changed files before overwriting

## Conflict Resolution (Two-Way Sync)

For `sync_type = "two-way"`, specify a `conflict_resolution` strategy:

- **skip** - Skip conflicting files, log warning
- **overwrite** - Source always wins
- **keep** - Keep both versions with timestamp suffix
- **newest** - Keep file with most recent modification time
- **largest** - Keep file with larger size

## Systemd Integration

### Scheduled Sync (Timer)

```bash
# Install service and timer
sudo cp etc/systemd/ugnassync.service /etc/systemd/system/
sudo cp etc/systemd/ugnassync.timer /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable ugnassync.timer
sudo systemctl start ugnassync.timer

# Check status
systemctl status ugnassync.timer
systemctl list-timers ugnassync.timer
```

### Watch Mode Daemon

```bash
# Install watch service
sudo cp etc/systemd/ugnassync-watch.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable ugnassync-watch.service
sudo systemctl start ugnassync-watch.service

# Check status
systemctl status ugnassync-watch.service
journalctl -u ugnassync-watch.service -f
```

## Documentation

See the [Product Specification Document](Documentation/ProductSpecification.md) for complete details.

## License

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, version 3.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
