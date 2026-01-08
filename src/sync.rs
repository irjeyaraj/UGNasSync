// UGNasSync - NAS Synchronization Tool
// Copyright (c) 2025 Sefier AI
// Author: Immanuel Jeyaraj <irj@sefier.com>
// License: GPL-3.0

use crate::config::{ConflictResolution, NasConfig, SyncProfile, SyncType};
use crate::conflict::ConflictResolver;
use anyhow::{Context, Result};
use std::process::Command;
use std::time::Instant;
use tracing::{debug, error, info, warn};

#[derive(Debug)]
pub struct SyncStats {
    pub files_transferred: u64,
    pub bytes_transferred: u64,
    pub duration_secs: f64,
    pub conflicts_detected: u64,
    pub conflicts_skipped: u64,
    pub conflicts_resolved: u64,
}

impl Default for SyncStats {
    fn default() -> Self {
        Self {
            files_transferred: 0,
            bytes_transferred: 0,
            duration_secs: 0.0,
            conflicts_detected: 0,
            conflicts_skipped: 0,
            conflicts_resolved: 0,
        }
    }
}

pub struct SyncEngine {
    pub(crate) nas_config: NasConfig,
    conflict_resolver: Option<ConflictResolver>,
}

impl SyncEngine {
    pub fn new(nas_config: NasConfig) -> Self {
        let conflict_resolver = ConflictResolver::new().ok();
        Self {
            nas_config,
            conflict_resolver,
        }
    }

    pub async fn sync_profile(&self, profile: &SyncProfile, dry_run: bool) -> Result<SyncStats> {
        info!("Starting sync profile: {}", profile.name);
        let start = Instant::now();

        let mut stats = SyncStats::default();

        // Handle two-way sync with conflict resolution
        if profile.sync_type == SyncType::TwoWay {
            if let Some(resolver) = &self.conflict_resolver {
                let resolution_strategy = profile
                    .conflict_resolution
                    .as_ref()
                    .unwrap_or(&ConflictResolution::Skip);

                info!("Two-way sync with conflict resolution: {:?}", resolution_strategy);
                // Note: Full two-way sync implementation would scan both directories
                // and detect conflicts. For now, we'll do a one-way sync.
                warn!("Two-way sync with conflict resolution is partially implemented");
            }
        }

        // Build rsync command based on sync type
        let mut cmd = self.build_rsync_command(profile, dry_run)?;

        debug!("Executing rsync command: {:?}", cmd);

        // Execute rsync
        let output = cmd
            .output()
            .context("Failed to execute rsync command")?;

        stats.duration_secs = start.elapsed().as_secs_f64();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Rsync failed: {}", stderr);
            anyhow::bail!("Rsync command failed: {}", stderr);
        }

        // Parse rsync output for statistics
        let stdout = String::from_utf8_lossy(&output.stdout);
        stats = self.parse_rsync_output(&stdout, stats);

        info!(
            "Transferred {} files ({:.2} MB) in {:.2}s",
            stats.files_transferred,
            stats.bytes_transferred as f64 / (1024.0 * 1024.0),
            stats.duration_secs
        );

        if dry_run {
            info!("Dry run completed - no files were actually transferred");
        } else {
            info!("Sync completed successfully");
        }

        Ok(stats)
    }

    fn build_rsync_command(&self, profile: &SyncProfile, dry_run: bool) -> Result<Command> {
        let mut cmd = Command::new("rsync");

        // Common rsync flags
        cmd.arg("-az") // archive mode + compression
            .arg("--stats") // show statistics
            .arg("--human-readable");

        if dry_run {
            cmd.arg("--dry-run");
        }

        // Verbose output for debugging
        cmd.arg("-v");

        // Add exclusions
        for exclude in &profile.exclude {
            cmd.arg(format!("--exclude={}", exclude));
        }

        // Sync type specific flags
        match profile.sync_type {
            SyncType::Mirror => {
                cmd.arg("--delete"); // Delete files not in source
            }
            SyncType::OneWay => {
                // No delete flag - preserve extra files on destination
            }
            SyncType::TwoWay => {
                // Two-way sync requires special handling (not directly supported by rsync)
                warn!("Two-way sync requires conflict resolution - using one-way for now");
                // This will be handled by the conflict module
            }
            SyncType::Incremental => {
                cmd.arg("--update"); // Skip files that are newer on destination
            }
            SyncType::Backup => {
                cmd.arg("--backup")
                    .arg("--backup-dir=.backup");
            }
        }

        // Build remote path with SSH
        let remote_path = if self.nas_config.key_path.is_some() {
            let key_path = self.nas_config.key_path.as_ref().unwrap();
            cmd.arg("-e")
                .arg(format!(
                    "ssh -p {} -i {}",
                    self.nas_config.port, key_path
                ));

            format!(
                "{}@{}:{}",
                self.nas_config.username,
                self.nas_config.host,
                profile.remote_path
            )
        } else {
            // Using sshpass for password authentication (requires sshpass to be installed)
            warn!("Using password authentication - consider using SSH keys for better security");

            cmd.arg("-e")
                .arg(format!("ssh -p {}", self.nas_config.port));

            format!(
                "{}@{}:{}",
                self.nas_config.username,
                self.nas_config.host,
                profile.remote_path
            )
        };

        // Add source and destination
        cmd.arg(&profile.local_path)
            .arg(&remote_path);

        Ok(cmd)
    }

    fn parse_rsync_output(&self, output: &str, mut stats: SyncStats) -> SyncStats {
        // Parse rsync statistics from output
        for line in output.lines() {
            if line.contains("Number of regular files transferred:") {
                if let Some(num_str) = line.split(':').nth(1) {
                    if let Ok(num) = num_str.trim().split_whitespace().next().unwrap_or("0").parse::<u64>() {
                        stats.files_transferred = num;
                    }
                }
            } else if line.contains("Total transferred file size:") {
                if let Some(size_str) = line.split(':').nth(1) {
                    if let Some(bytes_str) = size_str.trim().split_whitespace().next() {
                        if let Ok(bytes) = bytes_str.replace(",", "").parse::<u64>() {
                            stats.bytes_transferred = bytes;
                        }
                    }
                }
            }
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_stats_default() {
        let stats = SyncStats::default();
        assert_eq!(stats.files_transferred, 0);
        assert_eq!(stats.bytes_transferred, 0);
    }
}
