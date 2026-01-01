mod error;
mod storage;

use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use std::io::Read;
use storage::{create_storage, Snippet};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// A CLI tool for managing code snippets with timestamps, logging, and download support
#[derive(Parser, Debug)]
#[command(name = "snippets-app")]
#[command(about = "Store and manage code snippets with creation timestamps", long_about = None)]
#[command(version = "0.4.0")]
struct Cli {
    /// Name of the snippet to create (reads content from stdin or URL)
    #[arg(long, value_name = "NAME")]
    name: Option<String>,

    /// Name of the snippet to read
    #[arg(long, value_name = "NAME")]
    read: Option<String>,

    /// Name of the snippet to delete
    #[arg(long, value_name = "NAME")]
    delete: Option<String>,

    /// List all snippets (with creation times)
    #[arg(long)]
    list: bool,

    /// Show verbose output including creation timestamps
    #[arg(long, short)]
    verbose: bool,

    /// Download snippet content from URL instead of reading from stdin
    #[arg(long, value_name = "URL")]
    download: Option<String>,
}

fn main() {
    // Initialize logging first, before any other operations
    if let Err(e) = init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        // Continue execution even if logging fails
    }

    info!("snippets-app v0.4.0 starting");

    if let Err(e) = run() {
        error!("Application error: {:#}", e);
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }

    info!("snippets-app finished successfully");
}

/// Initialize tracing subscriber with configuration from environment variables
fn init_logging() -> Result<()> {
    // Get log level from environment variable, default to "info"
    let log_level = std::env::var("SNIPPETS_APP_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

    debug!("Initializing logging with level: {}", log_level);

    // Create env filter
    let env_filter =
        EnvFilter::try_new(&log_level).context(format!("Invalid log level: {}", log_level))?;

    // Check if log file path is specified
    if let Ok(log_path) = std::env::var("SNIPPETS_APP_LOG_PATH") {
        // Log to file
        let file_appender = tracing_appender::rolling::never(
            std::path::Path::new(&log_path)
                .parent()
                .unwrap_or(std::path::Path::new(".")),
            std::path::Path::new(&log_path)
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("snippets-app.log")),
        );

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().with_writer(file_appender).with_ansi(false))
            .init();

        eprintln!("Logging to file: {}", log_path);
    } else {
        // Log to stderr (default)
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().with_writer(std::io::stderr))
            .init();
    }

    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    debug!("Parsed CLI arguments: {:?}", cli);

    // Create storage with context
    let mut storage = create_storage().context(
        "Failed to initialize storage. Check SNIPPETS_APP_STORAGE environment variable or file permissions",
    )?;

    info!("Storage initialized successfully");

    // Execute command based on arguments
    if let Some(name) = cli.name {
        handle_create(&mut *storage, name, cli.download, cli.verbose)?;
    } else if let Some(name) = cli.read {
        handle_read(&*storage, name, cli.verbose)?;
    } else if let Some(name) = cli.delete {
        handle_delete(&mut *storage, name)?;
    } else if cli.list {
        handle_list(&*storage, cli.verbose)?;
    } else {
        print_usage_hint();
        std::process::exit(1);
    }

    Ok(())
}

fn handle_create(
    storage: &mut dyn storage::SnippetStorage,
    name: String,
    download_url: Option<String>,
    verbose: bool,
) -> Result<()> {
    info!("Creating snippet: {}", name);

    // Get content either from URL or stdin
    let content = if let Some(url) = download_url {
        debug!("Downloading content from URL: {}", url);
        download_content(&url).context(format!("Failed to download content from {}", url))?
    } else {
        debug!("Reading content from stdin");
        read_stdin().context("Failed to read snippet content from stdin")?
    };

    debug!("Content length: {} bytes", content.len());

    // Create snippet with validation
    let snippet = Snippet::new(name.clone(), content)
        .context(format!("Failed to create snippet '{}'", name))?;

    // Save snippet
    storage
        .save_snippet(snippet.clone())
        .context(format!("Failed to save snippet '{}'", name))?;

    info!("Snippet '{}' saved successfully", name);
    println!("✓ Snippet '{}' saved successfully", name);

    if verbose {
        let local_time = snippet.created_at.with_timezone(&Local);
        println!("  Created at: {}", local_time.format("%Y-%m-%d %H:%M:%S"));
    }

    Ok(())
}

fn handle_read(storage: &dyn storage::SnippetStorage, name: String, verbose: bool) -> Result<()> {
    info!("Reading snippet: {}", name);

    let snippet = storage
        .get_snippet(&name)
        .context(format!("Failed to retrieve snippet '{}'", name))?;

    match snippet {
        Some(snippet) => {
            debug!("Snippet found, length: {} bytes", snippet.content.len());

            if verbose {
                let local_time = snippet.created_at.with_timezone(&Local);
                println!("# Snippet: {}", snippet.name);
                println!("# Created: {}", local_time.format("%Y-%m-%d %H:%M:%S"));
                println!();
            }
            print!("{}", snippet.content);

            info!("Snippet '{}' retrieved successfully", name);
            Ok(())
        }
        None => {
            warn!("Snippet '{}' not found", name);
            anyhow::bail!("Snippet '{}' not found", name);
        }
    }
}

fn handle_delete(storage: &mut dyn storage::SnippetStorage, name: String) -> Result<()> {
    info!("Deleting snippet: {}", name);

    let deleted = storage
        .delete_snippet(&name)
        .context(format!("Failed to delete snippet '{}'", name))?;

    if deleted {
        info!("Snippet '{}' deleted successfully", name);
        println!("✓ Snippet '{}' deleted successfully", name);
        Ok(())
    } else {
        warn!("Snippet '{}' not found for deletion", name);
        anyhow::bail!("Snippet '{}' not found", name);
    }
}

fn handle_list(storage: &dyn storage::SnippetStorage, verbose: bool) -> Result<()> {
    info!("Listing snippets");

    let names = storage.list_snippets().context("Failed to list snippets")?;

    if names.is_empty() {
        println!("No snippets found");
        info!("No snippets found");
        return Ok(());
    }

    info!("Found {} snippet(s)", names.len());

    if verbose {
        println!("Stored snippets (with creation times):");
        println!();
        for name in names {
            match storage.get_snippet(&name) {
                Ok(Some(snippet)) => {
                    let local_time = snippet.created_at.with_timezone(&Local);
                    println!("  {} - {}", name, local_time.format("%Y-%m-%d %H:%M:%S"));
                }
                Ok(None) => {
                    // Snippet disappeared between list and get
                    warn!("Snippet '{}' disappeared between list and get", name);
                    println!("  {} - (unavailable)", name);
                }
                Err(e) => {
                    // Log error but continue with other snippets
                    error!("Error retrieving snippet '{}': {}", name, e);
                    eprintln!("  {} - Error: {}", name, e);
                }
            }
        }
    } else {
        println!("Stored snippets:");
        for name in names {
            println!("  - {}", name);
        }
    }

    Ok(())
}

fn print_usage_hint() {
    eprintln!("No command specified. Use --help for usage information.");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  echo 'code' | snippets-app --name 'my-snippet'");
    eprintln!("  snippets-app --name 'gist' --download 'https://example.com/snippet.txt'");
    eprintln!("  snippets-app --read 'my-snippet'");
    eprintln!("  snippets-app --list");
    eprintln!();
    eprintln!("Storage configuration (via SNIPPETS_APP_STORAGE):");
    eprintln!("  JSON:/path/to/snippets.json");
    eprintln!("  SQLITE:/path/to/snippets.db");
    eprintln!();
    eprintln!("Logging configuration:");
    eprintln!("  SNIPPETS_APP_LOG_LEVEL=debug|info|warn|error");
    eprintln!("  SNIPPETS_APP_LOG_PATH=/path/to/logfile.log");
}

/// Read all content from stdin
fn read_stdin() -> Result<String> {
    debug!("Reading from stdin");

    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;

    if buffer.is_empty() {
        warn!("Empty content provided via stdin");
        anyhow::bail!("No content provided. Please provide snippet content via stdin");
    }

    debug!("Read {} bytes from stdin", buffer.len());
    Ok(buffer)
}

/// Download content from URL using reqwest
fn download_content(url: &str) -> Result<String> {
    info!("Downloading content from: {}", url);

    let response =
        reqwest::blocking::get(url).context(format!("Failed to send GET request to {}", url))?;

    let status = response.status();
    debug!("Response status: {}", status);

    if !status.is_success() {
        error!("HTTP request failed with status: {}", status);
        anyhow::bail!("HTTP request failed with status: {}", status);
    }

    let content = response
        .text()
        .context("Failed to read response body as text")?;

    if content.is_empty() {
        warn!("Downloaded content is empty from {}", url);
        anyhow::bail!("Downloaded content is empty");
    }

    info!(
        "Successfully downloaded {} bytes from {}",
        content.len(),
        url
    );
    debug!(
        "Content preview: {}...",
        &content.chars().take(100).collect::<String>()
    );

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;

    #[test]
    fn test_create_storage_with_json() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_string_lossy().to_string();

        unsafe {
            env::set_var("SNIPPETS_APP_STORAGE", format!("JSON:{}", path));
        }
        let storage = create_storage();
        assert!(storage.is_ok());
        unsafe {
            env::remove_var("SNIPPETS_APP_STORAGE");
        }
    }

    #[test]
    fn test_create_storage_with_sqlite() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_string_lossy().to_string();

        unsafe {
            env::set_var("SNIPPETS_APP_STORAGE", format!("SQLITE:{}", path));
        }
        let storage = create_storage();
        assert!(storage.is_ok());
        unsafe {
            env::remove_var("SNIPPETS_APP_STORAGE");
        }
    }

    #[test]
    fn test_snippet_validation() {
        // Empty name should fail
        let result = Snippet::new("".to_string(), "content".to_string());
        assert!(result.is_err());

        // Valid name should succeed
        let result = Snippet::new("valid-name".to_string(), "content".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_storage_config() {
        unsafe {
            env::set_var("SNIPPETS_APP_STORAGE", "INVALID_FORMAT");
        }
        let result = create_storage();
        assert!(result.is_err());
        unsafe {
            env::remove_var("SNIPPETS_APP_STORAGE");
        }
    }

    // Note: Download tests would require a mock server
    // For now, we skip them to avoid network dependencies in tests
}
