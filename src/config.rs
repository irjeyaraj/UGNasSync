// UGNasSync - NAS Synchronization Tool
// Copyright (c) 2025 Sefier AI
// Author: Immanuel Jeyaraj <irj@sefier.com>
// License: GPL-3.0

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub nas: NasConfig,
    pub logging: LoggingConfig,
    pub sync_profiles: Vec<SyncProfile>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NasConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smb: Option<SmbConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SmbConfig {
    pub enabled: bool,
    pub share_path: String,
    pub mount_point: String,
    #[serde(default)]
    pub domain: String,
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub mount_options: String,
    #[serde(default = "default_auto_unmount")]
    pub auto_unmount: bool,
    #[serde(default = "default_mount_timeout")]
    pub mount_timeout: u64,
}

fn default_auto_unmount() -> bool {
    true
}

fn default_mount_timeout() -> u64 {
    30
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub log_file: String,
    pub log_level: String,
    pub console_output: bool,
    pub file_output: bool,
    pub rotate_enabled: bool,
    pub max_file_size_mb: u64,
    pub max_files: usize,
    pub compress_rotated: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncProfile {
    pub name: String,
    pub local_path: String,
    pub remote_path: String,
    pub sync_type: SyncType,
    pub enabled: bool,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub watch_mode: bool,
    #[serde(default = "default_debounce_seconds")]
    pub debounce_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_resolution: Option<ConflictResolution>,
    #[serde(default)]
    pub use_smb_mount: bool,
}

fn default_debounce_seconds() -> u64 {
    5
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SyncType {
    Mirror,
    #[serde(rename = "one-way")]
    OneWay,
    #[serde(rename = "two-way")]
    TwoWay,
    Incremental,
    Backup,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConflictResolution {
    Skip,
    Overwrite,
    Keep,
    Newest,
    Largest,
}

impl Default for ConflictResolution {
    fn default() -> Self {
        ConflictResolution::Skip
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        // Validate NAS config
        if self.nas.password.is_none() && self.nas.key_path.is_none() {
            anyhow::bail!("Either password or key_path must be specified in NAS config");
        }

        // Validate sync profiles
        if self.sync_profiles.is_empty() {
            anyhow::bail!("At least one sync profile must be defined");
        }

        for profile in &self.sync_profiles {
            if profile.sync_type == SyncType::TwoWay && profile.conflict_resolution.is_none() {
                tracing::warn!(
                    "Profile '{}' uses two-way sync without conflict_resolution specified. Defaulting to 'skip'.",
                    profile.name
                );
            }
        }

        Ok(())
    }

    pub fn get_enabled_profiles(&self) -> Vec<&SyncProfile> {
        self.sync_profiles
            .iter()
            .filter(|p| p.enabled)
            .collect()
    }

    pub fn get_watch_profiles(&self) -> Vec<&SyncProfile> {
        self.sync_profiles
            .iter()
            .filter(|p| p.enabled && p.watch_mode)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_resolution_default() {
        assert_eq!(ConflictResolution::default(), ConflictResolution::Skip);
    }

    #[test]
    fn test_sync_type_deserialization() {
        let toml_str = r#"
            sync_type = "two-way"
        "#;

        #[derive(Deserialize)]
        struct TestStruct {
            sync_type: SyncType,
        }

        let result: TestStruct = toml::from_str(toml_str).unwrap();
        assert_eq!(result.sync_type, SyncType::TwoWay);
    }
}
