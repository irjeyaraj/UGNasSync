# SMB/CIFS Mount Support Guide

**Author:** Immanuel Jeyaraj <irj@sefier.com>
**Copyright:** (c) 2025 Sefier AI
**License:** GPL-3.0

## Overview

UGNasSync supports mounting SMB/CIFS network shares locally before performing rsync operations. This approach provides several benefits:

- **Better performance** - Local file system access is faster than rsync over network protocols
- **Compatibility** - Works with any NAS that supports SMB/CIFS (Windows shares, Samba, etc.)
- **Flexibility** - Can use either SSH/rsync or SMB mount on a per-profile basis
- **Watch mode support** - Persistent mounts for real-time sync scenarios

## Architecture

### Two Sync Methods

UGNasSync supports two methods for syncing with your NAS:

#### 1. SSH/Rsync (Traditional)
```
Local Dir → rsync over SSH → Remote NAS
```
- Uses rsync protocol over SSH
- Requires SSH access to NAS
- Direct network transfer

#### 2. SMB Mount + Local Rsync (New)
```
SMB Share → Mount locally → rsync (local) → Mounted path
```
- Mounts SMB share as local directory
- Uses rsync on local file system
- Better performance for SMB-native shares

## Configuration

### Basic SMB Configuration

Add SMB configuration to your `config.toml`:

```toml
[nas]
host = "192.168.1.100"
port = 22
username = "admin"
key_path = "/home/user/.ssh/id_rsa"

# SMB/CIFS mount configuration
[nas.smb]
enabled = true
share_path = "//192.168.1.100/backups"  # UNC path to SMB share
mount_point = "/mnt/nas"                 # Local mount point
domain = ""                              # Optional Windows domain
username = "admin"                       # SMB username
password = "smb_password"                # SMB password
mount_options = "uid=1000,gid=1000,file_mode=0644,dir_mode=0755"
auto_unmount = true                      # Unmount after sync
mount_timeout = 30                       # Timeout in seconds
```

### Sync Profile Configuration

Enable SMB mounting for specific profiles:

```toml
[[sync_profiles]]
name = "Documents Backup via SMB"
local_path = "/home/user/Documents"
remote_path = "/mnt/nas/Documents"  # Path relative to mount point
sync_type = "mirror"
enabled = true
use_smb_mount = true                # Enable SMB mounting
watch_mode = false
debounce_seconds = 5
```

### Mixed Configuration (SSH + SMB)

You can use both methods in the same config:

```toml
# Profile using SSH/rsync
[[sync_profiles]]
name = "Photos via SSH"
local_path = "/home/user/Pictures"
remote_path = "/volume1/backups/Pictures"
sync_type = "mirror"
enabled = true
use_smb_mount = false  # Use SSH/rsync

# Profile using SMB mount
[[sync_profiles]]
name = "Documents via SMB"
local_path = "/home/user/Documents"
remote_path = "/mnt/nas/Documents"
sync_type = "mirror"
enabled = true
use_smb_mount = true   # Use SMB mount
```

## Mount Options

The `mount_options` parameter accepts standard CIFS mount options:

### Common Options

```toml
# User/Group mapping
mount_options = "uid=1000,gid=1000"

# File permissions
mount_options = "file_mode=0644,dir_mode=0755"

# Performance tuning
mount_options = "cache=strict,actimeo=60"

# Security options
mount_options = "sec=ntlmv2,vers=3.0"

# Combined options (comma-separated)
mount_options = "uid=1000,gid=1000,file_mode=0644,dir_mode=0755,cache=strict"
```

### Finding Your UID/GID

```bash
# Get your user ID and group ID
id -u  # UID
id -g  # GID

# Example output:
# UID: 1000
# GID: 1000
```

## Mount Lifecycle

### One-Time Sync

When `auto_unmount = true` (default):

1. Check if mount point already mounted
2. Create mount point directory if needed
3. Create secure credentials file (600 permissions)
4. Mount SMB share
5. Execute rsync operation
6. Unmount SMB share
7. Clean up credentials file

### Watch Mode

When `watch_mode = true` or `auto_unmount = false`:

1. Mount SMB share at startup
2. Keep mount persistent during operation
3. Execute rsync on file changes
4. Only unmount on shutdown or error

## Security

### Credential Management

- Credentials stored in `~/.ugnassync/smb_credentials/`
- Files have 600 permissions (owner read/write only)
- Credentials file format:
  ```
  username=admin
  password=smb_password
  domain=WORKGROUP
  ```
- Credentials file deleted after unmount (unless persistent mode)

### Best Practices

1. **Protect config file:**
   ```bash
   chmod 600 config.toml
   ```

2. **Use strong passwords:**
   - Different from SSH password
   - Meet your organization's requirements

3. **Validate mount options:**
   - Ensure appropriate file permissions
   - Consider security implications

4. **Monitor logs:**
   - Check for authentication failures
   - Watch for suspicious mount attempts

## Troubleshooting

### Permission Denied

**Error:** `Permission denied` when mounting

**Solution:**
```bash
# Option 1: Run with sudo
sudo ugnassync --config /path/to/config.toml

# Option 2: Add user to required groups (system-dependent)
sudo usermod -aG disk $USER

# Option 3: Configure sudo without password for mount
sudo visudo
# Add: your_user ALL=(ALL) NOPASSWD: /bin/mount, /bin/umount
```

### Network Unreachable

**Error:** `Host is down` or `Network is unreachable`

**Solution:**
1. Verify NAS IP address: `ping 192.168.1.100`
2. Check firewall rules
3. Ensure SMB ports are open (445, 139)
4. Verify network connectivity

### Invalid Credentials

**Error:** `mount error(13): Permission denied`

**Solution:**
1. Verify username/password in config
2. Check if account is enabled on NAS
3. Verify domain name (if applicable)
4. Test credentials manually:
   ```bash
   smbclient //192.168.1.100/backups -U admin
   ```

### Mount Point Already Mounted

**Error:** Mount point already in use

**Solution:**
```bash
# Check what's mounted
mount | grep /mnt/nas

# Unmount manually if needed
sudo umount /mnt/nas

# Or force unmount
sudo umount -l /mnt/nas
```

### Stale Mount

**Error:** Mount appears active but not accessible

**Solution:**
```bash
# Force unmount
sudo umount -l /mnt/nas

# Remove mount point if needed
sudo rmdir /mnt/nas

# Re-run ugnassync
ugnassync
```

## System Requirements

### Linux

**Required packages:**
```bash
# Debian/Ubuntu
sudo apt install cifs-utils

# Fedora/RHEL/CentOS
sudo dnf install cifs-utils

# Arch Linux
sudo pacman -S cifs-utils

# openSUSE
sudo zypper install cifs-utils
```

**Kernel support:**
- CIFS kernel module (usually included)
- Verify: `lsmod | grep cifs`

### Privileges

Mounting requires elevated privileges. Options:

1. **Run as root/sudo** (simplest)
2. **User mount capabilities** (advanced)
3. **FUSE-based alternatives** (not covered here)

## Performance Considerations

### When to Use SMB Mount

✅ **Use SMB mount when:**
- Your NAS natively uses SMB/CIFS
- You need better performance for large files
- You're syncing to Windows file servers
- Watch mode with frequent updates

❌ **Use SSH/rsync when:**
- Your NAS is optimized for rsync protocol
- You want simpler configuration
- You don't have SMB credentials
- Running on systems without mount privileges

### Performance Tips

1. **Cache settings:**
   ```toml
   mount_options = "cache=strict,actimeo=60"
   ```

2. **Protocol version:**
   ```toml
   mount_options = "vers=3.0"  # Or vers=2.1
   ```

3. **Persistent mounts for watch mode:**
   ```toml
   auto_unmount = false
   ```

4. **Network optimization:**
   - Use wired connections when possible
   - Ensure adequate bandwidth
   - Monitor network latency

## Examples

### Example 1: Basic SMB Backup

```toml
[nas.smb]
enabled = true
share_path = "//192.168.1.100/backups"
mount_point = "/mnt/nas"
username = "backup_user"
password = "secure_password"
mount_options = "uid=1000,gid=1000"
auto_unmount = true

[[sync_profiles]]
name = "Daily Backup"
local_path = "/home/user/Documents"
remote_path = "/mnt/nas/daily"
sync_type = "mirror"
enabled = true
use_smb_mount = true
```

### Example 2: Watch Mode with Persistent Mount

```toml
[nas.smb]
enabled = true
share_path = "//nas.local/live-sync"
mount_point = "/mnt/live"
username = "sync_daemon"
password = "daemon_password"
mount_options = "cache=strict,uid=1000"
auto_unmount = false  # Keep mounted

[[sync_profiles]]
name = "Live Project Sync"
local_path = "/home/user/Projects"
remote_path = "/mnt/live/projects"
sync_type = "two-way"
enabled = true
use_smb_mount = true
watch_mode = true
debounce_seconds = 5
conflict_resolution = "newest"
```

### Example 3: Domain Authentication

```toml
[nas.smb]
enabled = true
share_path = "//fileserver.company.com/shares"
mount_point = "/mnt/corporate"
domain = "COMPANY"
username = "jdoe"
password = "corp_password"
mount_options = "sec=ntlmv2,vers=3.0"
auto_unmount = true
```

## Logging

SMB mount operations are logged with INFO level:

```
[2026-01-13T10:15:30Z] [INFO] SMB mount enabled for share: //192.168.1.100/backups
[2026-01-13T10:15:30Z] [INFO] Checking mount status: /mnt/nas
[2026-01-13T10:15:31Z] [INFO] Mounting SMB share: //192.168.1.100/backups -> /mnt/nas
[2026-01-13T10:15:32Z] [INFO] SMB share mounted successfully
[2026-01-13T10:15:32Z] [INFO] Starting rsync to local mount point
[2026-01-13T10:15:40Z] [INFO] Rsync completed successfully
[2026-01-13T10:15:40Z] [INFO] Unmounting SMB share: /mnt/nas
[2026-01-13T10:15:41Z] [INFO] SMB share unmounted successfully
```

Error scenarios:

```
[2026-01-13T10:15:31Z] [ERROR] SMB mount failed: Permission denied
[2026-01-13T10:15:31Z] [ERROR] Try running with sudo or adding user to required groups
```

## Migration from SSH/Rsync

To migrate an existing profile to use SMB:

1. **Add SMB configuration:**
   ```toml
   [nas.smb]
   enabled = true
   share_path = "//your.nas.ip/share"
   mount_point = "/mnt/nas"
   username = "your_username"
   password = "your_password"
   ```

2. **Update sync profile:**
   ```toml
   [[sync_profiles]]
   name = "Your Profile"
   local_path = "/home/user/data"
   remote_path = "/mnt/nas/data"  # Update to mount point path
   use_smb_mount = true            # Add this line
   ```

3. **Test with dry run:**
   ```bash
   ugnassync --profile "Your Profile" --dry-run --verbose
   ```

4. **Verify and enable:**
   ```bash
   ugnassync --profile "Your Profile"
   ```

## Support

For issues or questions:
- Check logs: `/var/log/ugnassync/sync.log`
- Review [Product Specification](ProductSpecification.md)
- Verify mount manually: `mount | grep cifs`
- Test SMB access: `smbclient //host/share -U username`

## See Also

- [Product Specification Document](ProductSpecification.md)
- [README.md](../README.md)
- Linux `mount.cifs` man page: `man mount.cifs`
- Samba documentation: https://www.samba.org/
