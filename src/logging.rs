// UGNasSync - NAS Synchronization Tool
// Copyright (c) 2025 Sefier AI
// Author: Immanuel Jeyaraj <irj@sefier.com>
// License: GPL-3.0

use crate::config::LoggingConfig;
use anyhow::{Context, Result};
use std::path::Path;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logging(config: &LoggingConfig, verbose: bool) -> Result<()> {
    if !config.enabled {
        return Ok(());
    }

    // Determine log level
    let level = if verbose {
        Level::DEBUG
    } else {
        parse_log_level(&config.log_level)?
    };

    let filter = EnvFilter::from_default_env()
        .add_directive(level.into())
        .add_directive("ugnassync=trace".parse().unwrap());

    let registry = tracing_subscriber::registry().with(filter);

    // Setup file logging
    if config.file_output {
        let log_path = Path::new(&config.log_file);
        let log_dir = log_path
            .parent()
            .context("Invalid log file path")?;

        // Create log directory if it doesn't exist
        std::fs::create_dir_all(log_dir)
            .with_context(|| format!("Failed to create log directory: {}", log_dir.display()))?;

        let file_name = log_path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid log file name")?;

        // Setup file appender with rotation
        let file_appender = if config.rotate_enabled {
            // Note: tracing-appender doesn't support size-based rotation directly
            // For now, we use daily rotation. Size-based rotation would require
            // a custom implementation or external log rotation tool
            RollingFileAppender::new(Rotation::DAILY, log_dir, file_name)
        } else {
            RollingFileAppender::new(Rotation::NEVER, log_dir, file_name)
        };

        let file_layer = fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(false);

        if config.console_output {
            let console_layer = fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(false);

            registry
                .with(file_layer)
                .with(console_layer)
                .init();
        } else {
            registry
                .with(file_layer)
                .init();
        }
    } else if config.console_output {
        let console_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .with_target(false);

        registry
            .with(console_layer)
            .init();
    } else {
        // At least one output should be enabled
        anyhow::bail!("Either console_output or file_output must be enabled");
    }

    Ok(())
}

fn parse_log_level(level_str: &str) -> Result<Level> {
    match level_str.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => anyhow::bail!("Invalid log level: {}. Valid levels: trace, debug, info, warn, error", level_str),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_level() {
        assert!(matches!(parse_log_level("debug").unwrap(), Level::DEBUG));
        assert!(matches!(parse_log_level("INFO").unwrap(), Level::INFO));
        assert!(matches!(parse_log_level("warn").unwrap(), Level::WARN));
        assert!(parse_log_level("invalid").is_err());
    }
}
