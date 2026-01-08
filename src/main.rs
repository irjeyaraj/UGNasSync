// UGNasSync - NAS Synchronization Tool
// Copyright (c) 2025 Sefier AI
// Author: Immanuel Jeyaraj <irj@sefier.com>
// License: GPL-3.0

mod config;
mod conflict;
mod logging;
mod sync;
mod watch;

use anyhow::Result;
use clap::Parser;
use config::Config;
use std::path::PathBuf;
use sync::SyncEngine;
use tracing::{error, info};
use watch::WatchManager;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = "Immanuel Jeyaraj <irj@sefier.com>";
const COPYRIGHT: &str = "Copyright (c) 2025 Sefier AI";
const LICENSE: &str = "GPL-3.0";

#[derive(Parser)]
#[command(
    name = "ugnassync",
    version = VERSION,
    author = AUTHORS,
    about = "Automated NAS synchronization tool using rsync",
    long_about = format!(
        "UGNasSync v{}\nAutomated NAS synchronization tool using rsync\n\nAuthor: {}\n{}\nLicense: {}",
        VERSION, AUTHORS, COPYRIGHT, LICENSE
    )
)]
struct Cli {
    /// Path to config file
    #[arg(short, long, default_value = "./config.toml")]
    config: PathBuf,

    /// Run only the specified sync profile
    #[arg(short, long)]
    profile: Option<String>,

    /// Simulate sync without making changes
    #[arg(short, long)]
    dry_run: bool,

    /// Enable verbose output (overrides config log level)
    #[arg(short, long)]
    verbose: bool,

    /// Enable watch mode for real-time sync (runs as daemon)
    #[arg(short, long)]
    watch: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config = Config::from_file(&cli.config)?;

    // Initialize logging
    logging::init_logging(&config.logging, cli.verbose)?;

    info!("UGNasSync v{}", VERSION);
    info!("{}", COPYRIGHT);
    info!("License: {}", LICENSE);
    info!("Starting UGNasSync...");

    if cli.watch {
        // Watch mode
        let watch_profiles = config.get_watch_profiles();

        if watch_profiles.is_empty() {
            error!("No profiles with watch_mode enabled found in config");
            anyhow::bail!("No watch-enabled profiles configured");
        }

        info!("Running in watch mode");
        let watch_manager = WatchManager::new(config.nas.clone());
        watch_manager.start_watching(watch_profiles).await?;
    } else {
        // One-time sync mode
        let profiles = if let Some(profile_name) = &cli.profile {
            // Run specific profile
            config
                .get_enabled_profiles()
                .into_iter()
                .filter(|p| &p.name == profile_name)
                .collect::<Vec<_>>()
        } else {
            // Run all enabled profiles (excluding watch-only profiles)
            config.get_enabled_profiles()
        };

        if profiles.is_empty() {
            error!("No enabled profiles found");
            anyhow::bail!("No profiles to sync");
        }

        info!("Found {} profile(s) to sync", profiles.len());

        let sync_engine = SyncEngine::new(config.nas.clone());

        for profile in profiles {
            info!("Processing profile: {}", profile.name);

            match sync_engine.sync_profile(profile, cli.dry_run).await {
                Ok(stats) => {
                    println!("\nSync Summary:");
                    println!("Profile: {}", profile.name);
                    println!("Files transferred: {}", stats.files_transferred);
                    println!(
                        "Bytes transferred: {:.2} MB",
                        stats.bytes_transferred as f64 / (1024.0 * 1024.0)
                    );

                    if stats.conflicts_detected > 0 {
                        println!("Conflicts detected: {}", stats.conflicts_detected);
                        println!("  - Skipped: {}", stats.conflicts_skipped);
                        println!("  - Resolved: {}", stats.conflicts_resolved);
                    }

                    println!("Duration: {:.2}s", stats.duration_secs);
                    println!(
                        "Status: {}",
                        if stats.conflicts_skipped > 0 {
                            "Completed with warnings"
                        } else {
                            "Completed successfully"
                        }
                    );
                }
                Err(e) => {
                    error!("Failed to sync profile {}: {}", profile.name, e);
                }
            }
        }

        info!("All sync operations completed");
    }

    Ok(())
}
