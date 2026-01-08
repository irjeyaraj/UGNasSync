// UGNasSync - NAS Synchronization Tool
// Copyright (c) 2025 Sefier AI
// Author: Immanuel Jeyaraj <irj@sefier.com>
// License: GPL-3.0

use crate::config::{NasConfig, SyncProfile};
use crate::sync::SyncEngine;
use anyhow::{Context, Result};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

pub struct WatchManager {
    sync_engine: SyncEngine,
}

impl WatchManager {
    pub fn new(nas_config: NasConfig) -> Self {
        Self {
            sync_engine: SyncEngine::new(nas_config),
        }
    }

    pub async fn start_watching(&self, profiles: Vec<&SyncProfile>) -> Result<()> {
        if profiles.is_empty() {
            warn!("No profiles with watch mode enabled");
            return Ok(());
        }

        info!("Starting watch mode for {} profile(s)", profiles.len());

        // Perform initial sync for all watch-enabled profiles
        for profile in &profiles {
            info!("Performing initial sync for: {}", profile.name);
            match self.sync_engine.sync_profile(profile, false).await {
                Ok(stats) => {
                    info!(
                        "Initial sync completed: {} files, {:.2} MB",
                        stats.files_transferred,
                        stats.bytes_transferred as f64 / (1024.0 * 1024.0)
                    );
                }
                Err(e) => {
                    error!("Initial sync failed for {}: {}", profile.name, e);
                }
            }
        }

        // Create watchers for each profile
        let mut handles = Vec::new();

        for profile in profiles {
            let profile_clone = profile.clone();
            let engine = SyncEngine::new(self.sync_engine.nas_config.clone());

            let handle = tokio::spawn(async move {
                if let Err(e) = Self::watch_profile(engine, &profile_clone).await {
                    error!("Watch failed for {}: {}", profile_clone.name, e);
                }
            });

            handles.push(handle);
        }

        // Wait for all watchers
        for handle in handles {
            let _ = handle.await;
        }

        Ok(())
    }

    async fn watch_profile(engine: SyncEngine, profile: &SyncProfile) -> Result<()> {
        info!("Watch mode enabled for profile: {}", profile.name);
        info!("Monitoring: {}", profile.local_path);

        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = Watcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            notify::Config::default(),
        )
        .context("Failed to create file watcher")?;

        // Watch the local path
        watcher
            .watch(Path::new(&profile.local_path), RecursiveMode::Recursive)
            .with_context(|| format!("Failed to watch directory: {}", profile.local_path))?;

        // Debounce handling
        let debounce_duration = Duration::from_secs(profile.debounce_seconds);
        let last_sync = Mutex::new(Instant::now());

        Self::handle_watch_events(engine, profile, rx, debounce_duration, last_sync).await?;

        Ok(())
    }

    async fn handle_watch_events(
        engine: SyncEngine,
        profile: &SyncProfile,
        rx: Receiver<Event>,
        debounce_duration: Duration,
        last_sync: Mutex<Instant>,
    ) -> Result<()> {
        let mut pending_changes = false;

        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(event) => {
                    // Filter events based on exclude patterns
                    let should_process = event.paths.iter().all(|path| {
                        !Self::is_excluded(path, &profile.exclude)
                    });

                    if should_process {
                        debug!("File change detected: {:?}", event.paths);
                        pending_changes = true;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Check if we should trigger sync
                    if pending_changes {
                        let mut last = last_sync.lock().await;
                        let elapsed = last.elapsed();

                        if elapsed >= debounce_duration {
                            info!("Debounce period elapsed, starting sync...");
                            pending_changes = false;
                            *last = Instant::now();
                            drop(last); // Release lock before sync

                            match engine.sync_profile(profile, false).await {
                                Ok(stats) => {
                                    info!(
                                        "Transferred {} file(s) ({:.2} MB) in {:.2}s",
                                        stats.files_transferred,
                                        stats.bytes_transferred as f64 / (1024.0 * 1024.0),
                                        stats.duration_secs
                                    );
                                }
                                Err(e) => {
                                    error!("Sync failed: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    warn!("File watcher disconnected");
                    break;
                }
            }
        }

        Ok(())
    }

    fn is_excluded(path: &Path, exclude_patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in exclude_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_excluded() {
        let path = PathBuf::from("/home/user/test/.git/config");
        let exclude = vec![".git".to_string(), "*.tmp".to_string()];

        assert!(WatchManager::is_excluded(&path, &exclude));
    }

    #[test]
    fn test_is_not_excluded() {
        let path = PathBuf::from("/home/user/test/file.txt");
        let exclude = vec![".git".to_string(), "*.tmp".to_string()];

        assert!(!WatchManager::is_excluded(&path, &exclude));
    }
}
