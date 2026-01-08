# UGNasSync Systemd Configuration

This directory contains systemd service and timer files for automated scheduling of UGNasSync.

## Files

- **ugnassync.service** - Systemd service unit that runs the sync operation (one-time)
- **ugnassync.timer** - Systemd timer unit that schedules the service execution
- **ugnassync-watch.service** - Systemd service for real-time sync (watch mode daemon)

## Installation

### For Scheduled Sync (Timer-based)

1. Copy the service and timer files to systemd directory:
```bash
sudo cp ugnassync.service /etc/systemd/system/
sudo cp ugnassync.timer /etc/systemd/system/
```

2. Reload systemd daemon:
```bash
sudo systemctl daemon-reload
```

3. Enable the timer to start on boot:
```bash
sudo systemctl enable ugnassync.timer
```

4. Start the timer immediately:
```bash
sudo systemctl start ugnassync.timer
```

### For Real-Time Sync (Watch Mode)

1. Copy the watch service file to systemd directory:
```bash
sudo cp ugnassync-watch.service /etc/systemd/system/
```

2. Reload systemd daemon:
```bash
sudo systemctl daemon-reload
```

3. Enable the watch service to start on boot:
```bash
sudo systemctl enable ugnassync-watch.service
```

4. Start the watch service immediately:
```bash
sudo systemctl start ugnassync-watch.service
```

**Note:** You can use both timer-based and watch mode simultaneously. The watch mode will handle real-time changes for profiles with `watch_mode = true`, while the timer will run all enabled profiles on schedule.

## Usage

### Check timer status
```bash
sudo systemctl status ugnassync.timer
```

### List all timers (including next run time)
```bash
systemctl list-timers ugnassync.timer
```

### Check service status
```bash
sudo systemctl status ugnassync.service
```

### View service logs
```bash
sudo journalctl -u ugnassync.service
sudo journalctl -u ugnassync.service -f  # Follow logs in real-time
```

### Run sync manually (without waiting for timer)
```bash
sudo systemctl start ugnassync.service
```

### Stop the timer
```bash
sudo systemctl stop ugnassync.timer
```

### Disable the timer from starting on boot
```bash
sudo systemctl disable ugnassync.timer
```

## Configuration

### Default Schedule
The timer is configured to run:
- Daily at 2:00 AM
- 5 minutes after boot (if last scheduled run was missed)
- With a random delay of up to 10 minutes

### Customizing Schedule
Edit the timer file to change the schedule. Common patterns:

```ini
# Every hour
OnCalendar=hourly

# Every 6 hours
OnCalendar=*-*-* 00,06,12,18:00:00

# Every Monday at 3:00 AM
OnCalendar=Mon *-*-* 03:00:00

# Twice daily (6 AM and 6 PM)
OnCalendar=*-*-* 06,18:00:00
```

After editing, reload systemd and restart the timer:
```bash
sudo systemctl daemon-reload
sudo systemctl restart ugnassync.timer
```

## Customizing Service

### Change User/Group
Edit `ugnassync.service` and modify:
```ini
User=your_username
Group=your_group
```

### Change Config File Location
Edit the `ExecStart` line in `ugnassync.service`:
```ini
ExecStart=/usr/local/bin/ugnassync --config /path/to/your/config.toml
```

### Change Binary Location
If ugnassync is installed elsewhere, update the path in `ExecStart`:
```ini
ExecStart=/usr/bin/ugnassync --config /etc/ugnassync/config.toml
```

## Troubleshooting

### Timer not running
- Check timer is enabled: `systemctl is-enabled ugnassync.timer`
- Check timer is active: `systemctl is-active ugnassync.timer`
- View timer logs: `journalctl -u ugnassync.timer`

### Service failing
- Check service logs: `journalctl -u ugnassync.service -n 50`
- Verify config file path and permissions
- Ensure binary is executable: `chmod +x /usr/local/bin/ugnassync`
- Check network connectivity to NAS

### Permission issues
- Ensure user has read access to source directories
- Ensure log directory exists: `sudo mkdir -p /var/log/ugnassync`
- Set proper ownership: `sudo chown -R root:root /var/log/ugnassync`

## Watch Mode Service

### Check watch service status
```bash
sudo systemctl status ugnassync-watch.service
```

### View watch service logs
```bash
sudo journalctl -u ugnassync-watch.service -f
```

### Stop watch service
```bash
sudo systemctl stop ugnassync-watch.service
```

### Restart watch service (after config changes)
```bash
sudo systemctl restart ugnassync-watch.service
```

### Disable watch service
```bash
sudo systemctl disable ugnassync-watch.service
```

### Watch Mode vs Timer
- **Watch Mode**: Runs continuously, monitors file changes, syncs immediately when changes detected
- **Timer Mode**: Runs at scheduled intervals, syncs all configured profiles
- **Use Cases**:
  - Watch Mode: For critical files that need immediate backup (e.g., active projects, photos)
  - Timer Mode: For periodic backups of large directories where immediate sync isn't necessary
