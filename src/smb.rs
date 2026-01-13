// UGNasSync - NAS Synchronization Tool
// Copyright (c) 2025 Sefier AI
// Author: Immanuel Jeyaraj <irj@sefier.com>
// License: GPL-3.0

use crate::config::SmbConfig;
use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, error, info, warn};

pub struct SmbMount {
    config: SmbConfig,
    credentials_file: Option<PathBuf>,
    is_mounted: bool,
}

impl SmbMount {
    pub fn new(config: SmbConfig) -> Self {
        Self {
            config,
            credentials_file: None,
            is_mounted: false,
        }
    }

    /// Check if the mount point is already mounted
    fn is_mount_point_active(&self) -> Result<bool> {
        let output = Command::new("mountpoint")
            .arg("-q")
            .arg(&self.config.mount_point)
            .output()
            .context("Failed to check mount point status")?;

        Ok(output.status.success())
    }

    /// Create credentials file for SMB mount
    fn create_credentials_file(&mut self) -> Result<PathBuf> {
        // Create credentials directory
        let creds_dir = dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".ugnassync")
            .join("smb_credentials");

        fs::create_dir_all(&creds_dir)
            .context("Failed to create credentials directory")?;

        // Create temporary credentials file
        let creds_file = creds_dir.join(format!("smb_creds_{}.tmp", std::process::id()));

        let mut file = fs::File::create(&creds_file)
            .context("Failed to create credentials file")?;

        // Write credentials
        writeln!(file, "username={}", self.config.username)?;
        writeln!(file, "password={}", self.config.password)?;
        if !self.config.domain.is_empty() {
            writeln!(file, "domain={}", self.config.domain)?;
        }

        // Set file permissions to 600 (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&creds_file, permissions)
                .context("Failed to set credentials file permissions")?;
        }

        debug!("Created credentials file: {}", creds_file.display());
        self.credentials_file = Some(creds_file.clone());

        Ok(creds_file)
    }

    /// Clean up credentials file
    fn cleanup_credentials_file(&mut self) -> Result<()> {
        if let Some(creds_file) = &self.credentials_file {
            if creds_file.exists() {
                fs::remove_file(creds_file)
                    .context("Failed to remove credentials file")?;
                debug!("Removed credentials file: {}", creds_file.display());
            }
            self.credentials_file = None;
        }
        Ok(())
    }

    /// Mount the SMB share
    pub async fn mount(&mut self) -> Result<()> {
        info!("SMB mount enabled for share: {}", self.config.share_path);
        info!("Checking mount status: {}", self.config.mount_point);

        // Check if already mounted
        if self.is_mount_point_active()? {
            info!("Mount point {} is already mounted", self.config.mount_point);
            self.is_mounted = true;
            return Ok(());
        }

        // Create mount point if it doesn't exist
        let mount_point = Path::new(&self.config.mount_point);
        if !mount_point.exists() {
            info!("Creating mount point: {}", self.config.mount_point);
            fs::create_dir_all(mount_point)
                .context("Failed to create mount point directory")?;
        }

        // Create credentials file
        let creds_file = self.create_credentials_file()?;

        // Build mount command
        let mut cmd = Command::new("mount");
        cmd.arg("-t").arg("cifs");
        cmd.arg(&self.config.share_path);
        cmd.arg(&self.config.mount_point);
        cmd.arg("-o").arg(format!("credentials={}", creds_file.display()));

        // Add custom mount options if specified
        if !self.config.mount_options.is_empty() {
            cmd.arg("-o").arg(&self.config.mount_options);
        }

        info!(
            "Mounting SMB share: {} -> {}",
            self.config.share_path, self.config.mount_point
        );

        debug!("Mount command: {:?}", cmd);

        // Execute mount command
        let output = cmd.output().context("Failed to execute mount command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("SMB mount failed: {}", stderr);

            // Clean up credentials file
            let _ = self.cleanup_credentials_file();

            // Provide helpful error messages
            if stderr.contains("Permission denied") || stderr.contains("permission denied") {
                anyhow::bail!(
                    "Permission denied. Try running with sudo or adding user to required groups.\nError: {}",
                    stderr
                );
            } else if stderr.contains("Host is down") || stderr.contains("Network is unreachable") {
                anyhow::bail!("Network unreachable: {}\nError: {}", self.config.share_path, stderr);
            } else if stderr.contains("mount error(13)") {
                anyhow::bail!("Invalid credentials or authentication failed.\nError: {}", stderr);
            } else {
                anyhow::bail!("Mount failed: {}", stderr);
            }
        }

        info!("SMB share mounted successfully");
        self.is_mounted = true;

        // Clean up credentials file in persistent mode, keep it for auto_unmount
        if !self.config.auto_unmount {
            // Keep credentials file for persistent mount
            debug!("Keeping credentials file for persistent mount");
        }

        Ok(())
    }

    /// Unmount the SMB share
    pub async fn unmount(&mut self) -> Result<()> {
        if !self.is_mounted {
            return Ok(());
        }

        info!("Unmounting SMB share: {}", self.config.mount_point);

        // Check if mount point is busy
        let output = Command::new("umount")
            .arg(&self.config.mount_point)
            .output()
            .context("Failed to execute umount command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Graceful unmount failed: {}", stderr);

            // Try lazy unmount
            info!("Attempting lazy unmount...");
            let output = Command::new("umount")
                .arg("-l")
                .arg(&self.config.mount_point)
                .output()
                .context("Failed to execute lazy umount command")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("Lazy unmount failed: {}", stderr);
                // Don't fail here, just log the error
                warn!("Failed to unmount SMB share, may need manual cleanup");
            } else {
                info!("Lazy unmount successful");
            }
        } else {
            info!("SMB share unmounted successfully");
        }

        // Clean up credentials file
        let _ = self.cleanup_credentials_file();

        self.is_mounted = false;
        Ok(())
    }

    /// Get the mount point path
    pub fn mount_point(&self) -> &str {
        &self.config.mount_point
    }

    /// Check if auto_unmount is enabled
    pub fn should_auto_unmount(&self) -> bool {
        self.config.auto_unmount
    }

    /// Check if the share is currently mounted
    pub fn is_mounted(&self) -> bool {
        self.is_mounted
    }
}

impl Drop for SmbMount {
    fn drop(&mut self) {
        // Clean up credentials file when dropped
        let _ = self.cleanup_credentials_file();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smb_mount_creation() {
        let config = SmbConfig {
            enabled: true,
            share_path: "//192.168.1.100/backups".to_string(),
            mount_point: "/mnt/nas".to_string(),
            domain: "".to_string(),
            username: "admin".to_string(),
            password: "password".to_string(),
            mount_options: "".to_string(),
            auto_unmount: true,
            mount_timeout: 30,
        };

        let mount = SmbMount::new(config);
        assert!(!mount.is_mounted());
        assert_eq!(mount.mount_point(), "/mnt/nas");
    }
}
