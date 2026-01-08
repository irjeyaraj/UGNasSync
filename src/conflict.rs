// UGNasSync - NAS Synchronization Tool
// Copyright (c) 2025 Sefier AI
// Author: Immanuel Jeyaraj <irj@sefier.com>
// License: GPL-3.0

use crate::config::ConflictResolution;
use anyhow::{Context, Result};
use chrono::Local;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

pub struct ConflictResolver {
    db_path: PathBuf,
}

#[derive(Debug)]
pub struct FileMetadata {
    pub path: String,
    pub size: u64,
    pub modified: i64,
    pub hash: String,
}

impl ConflictResolver {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let db_dir = home.join(".ugnassync");

        fs::create_dir_all(&db_dir)
            .context("Failed to create sync state directory")?;

        let db_path = db_dir.join("sync_state.db");

        let resolver = Self { db_path };
        resolver.init_database()?;

        Ok(resolver)
    }

    fn init_database(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)
            .context("Failed to open sync state database")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_state (
                path TEXT PRIMARY KEY,
                size INTEGER NOT NULL,
                modified INTEGER NOT NULL,
                hash TEXT NOT NULL,
                last_sync INTEGER NOT NULL
            )",
            [],
        )
        .context("Failed to create sync_state table")?;

        Ok(())
    }

    pub fn detect_conflict(
        &self,
        local_file: &Path,
        remote_file: &Path,
    ) -> Result<bool> {
        let local_meta = self.get_file_metadata(local_file)?;
        let remote_meta = self.get_file_metadata(remote_file)?;

        // Check if we have a record of last sync
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare("SELECT modified, hash FROM sync_state WHERE path = ?")?;

        let path_str = local_file.to_string_lossy().to_string();
        let last_sync: Option<(i64, String)> = stmt
            .query_row(params![path_str], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .ok();

        if let Some((last_modified, last_hash)) = last_sync {
            // Both files have been modified since last sync
            let local_changed = local_meta.modified > last_modified || local_meta.hash != last_hash;
            let remote_changed = remote_meta.modified > last_modified || remote_meta.hash != last_hash;

            Ok(local_changed && remote_changed)
        } else {
            // No sync record - check if files are different
            Ok(local_meta.hash != remote_meta.hash)
        }
    }

    pub fn resolve_conflict(
        &self,
        local_file: &Path,
        remote_file: &Path,
        strategy: &ConflictResolution,
    ) -> Result<()> {
        warn!("Conflict detected: {}", local_file.display());

        match strategy {
            ConflictResolution::Skip => {
                warn!("Skipping conflicting file per conflict_resolution policy");
                Ok(())
            }
            ConflictResolution::Overwrite => {
                info!("Overwriting destination with source (source wins)");
                fs::copy(local_file, remote_file)
                    .context("Failed to overwrite destination file")?;
                self.update_sync_state(local_file)?;
                Ok(())
            }
            ConflictResolution::Keep => {
                info!("Keeping both versions");
                let timestamp = Local::now().format("%Y%m%d-%H%M%S");
                let conflict_name = format!(
                    "{}.conflict.{}",
                    remote_file.display(),
                    timestamp
                );
                fs::rename(remote_file, &conflict_name)
                    .context("Failed to rename destination file")?;
                info!("Renamed destination: {}", conflict_name);

                fs::copy(local_file, remote_file)
                    .context("Failed to copy source file")?;
                self.update_sync_state(local_file)?;
                Ok(())
            }
            ConflictResolution::Newest => {
                let local_meta = self.get_file_metadata(local_file)?;
                let remote_meta = self.get_file_metadata(remote_file)?;

                if local_meta.modified >= remote_meta.modified {
                    info!("Keeping newest version (source is newer or same)");
                    fs::copy(local_file, remote_file)?;
                    self.update_sync_state(local_file)?;
                } else {
                    info!("Keeping newest version (destination is newer)");
                    fs::copy(remote_file, local_file)?;
                    self.update_sync_state(remote_file)?;
                }
                Ok(())
            }
            ConflictResolution::Largest => {
                let local_meta = self.get_file_metadata(local_file)?;
                let remote_meta = self.get_file_metadata(remote_file)?;

                if local_meta.size >= remote_meta.size {
                    info!("Keeping largest version (source: {} bytes)", local_meta.size);
                    fs::copy(local_file, remote_file)?;
                    self.update_sync_state(local_file)?;
                } else {
                    info!("Keeping largest version (dest: {} bytes)", remote_meta.size);
                    fs::copy(remote_file, local_file)?;
                    self.update_sync_state(remote_file)?;
                }
                Ok(())
            }
        }
    }

    fn get_file_metadata(&self, path: &Path) -> Result<FileMetadata> {
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to read file metadata: {}", path.display()))?;

        let modified = metadata
            .modified()
            .context("Failed to get modification time")?
            .duration_since(std::time::UNIX_EPOCH)
            .context("Invalid modification time")?
            .as_secs() as i64;

        let hash = self.calculate_file_hash(path)?;

        Ok(FileMetadata {
            path: path.to_string_lossy().to_string(),
            size: metadata.len(),
            modified,
            hash,
        })
    }

    fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        let contents = fs::read(path)
            .with_context(|| format!("Failed to read file for hashing: {}", path.display()))?;

        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let result = hasher.finalize();

        Ok(format!("{:x}", result))
    }

    pub fn update_sync_state(&self, path: &Path) -> Result<()> {
        let meta = self.get_file_metadata(path)?;
        let now = Local::now().timestamp();

        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "INSERT OR REPLACE INTO sync_state (path, size, modified, hash, last_sync)
             VALUES (?, ?, ?, ?, ?)",
            params![meta.path, meta.size as i64, meta.modified, meta.hash, now],
        )?;

        debug!("Updated sync state for: {}", path.display());
        Ok(())
    }
}

// Add dirs crate for home directory detection
// Note: This requires adding `dirs = "5.0"` to Cargo.toml
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_resolver_creation() {
        let resolver = ConflictResolver::new();
        assert!(resolver.is_ok());
    }
}
