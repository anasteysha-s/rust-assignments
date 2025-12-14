mod error;
mod storage;

use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use std::io::Read;
use storage::{create_storage, Snippet};

/// A simple CLI tool for managing code snippets with timestamps
#[derive(Parser, Debug)]
#[command(name = "snippets-app")]
#[command(about = "Store and manage code snippets with creation timestamps", long_about = None)]
#[command(version = "0.3.0")]
struct Cli {
    /// Name of the snippet to create (reads content from stdin)
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
}

fn main() {
    if let Err(e) = run() {
        // Pretty print the error with full context chain
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Create storage with context
    let mut storage = create_storage()
        .context("Failed to initialize storage. Check SNIPPETS_APP_STORAGE environment variable or file permissions")?;

    // Execute command based on arguments
    if let Some(name) = cli.name {
        handle_create(&mut *storage, name, cli.verbose)?;
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
    verbose: bool,
) -> Result<()> {
    // Read from stdin with context
    let content = read_stdin()
        .context("Failed to read snippet content from stdin")?;

    // Create snippet with validation
    let snippet = Snippet::new(name.clone(), content)
        .context(format!("Failed to create snippet '{}'", name))?;

    // Save snippet
    storage
        .save_snippet(snippet.clone())
        .context(format!("Failed to save snippet '{}'", name))?;

    println!("✓ Snippet '{}' saved successfully", name);
    
    if verbose {
        let local_time = snippet.created_at.with_timezone(&Local);
        println!("  Created at: {}", local_time.format("%Y-%m-%d %H:%M:%S"));
    }

    Ok(())
}

fn handle_read(
    storage: &dyn storage::SnippetStorage,
    name: String,
    verbose: bool,
) -> Result<()> {
    let snippet = storage
        .get_snippet(&name)
        .context(format!("Failed to retrieve snippet '{}'", name))?;

    match snippet {
        Some(snippet) => {
            if verbose {
                let local_time = snippet.created_at.with_timezone(&Local);
                println!("# Snippet: {}", snippet.name);
                println!("# Created: {}", local_time.format("%Y-%m-%d %H:%M:%S"));
                println!();
            }
            print!("{}", snippet.content);
            Ok(())
        }
        None => {
            anyhow::bail!("Snippet '{}' not found", name);
        }
    }
}

fn handle_delete(
    storage: &mut dyn storage::SnippetStorage,
    name: String,
) -> Result<()> {
    let deleted = storage
        .delete_snippet(&name)
        .context(format!("Failed to delete snippet '{}'", name))?;

    if deleted {
        println!("✓ Snippet '{}' deleted successfully", name);
        Ok(())
    } else {
        anyhow::bail!("Snippet '{}' not found", name);
    }
}

fn handle_list(
    storage: &dyn storage::SnippetStorage,
    verbose: bool,
) -> Result<()> {
    let names = storage
        .list_snippets()
        .context("Failed to list snippets")?;

    if names.is_empty() {
        println!("No snippets found");
        return Ok(());
    }

    if verbose {
        println!("Stored snippets (with creation times):");
        println!();
        for name in names {
            match storage.get_snippet(&name) {
                Ok(Some(snippet)) => {
                    let local_time = snippet.created_at.with_timezone(&Local);
                    println!(
                        "  {} - {}",
                        name,
                        local_time.format("%Y-%m-%d %H:%M:%S")
                    );
                }
                Ok(None) => {
                    // Snippet disappeared between list and get
                    println!("  {} - (unavailable)", name);
                }
                Err(e) => {
                    // Log error but continue with other snippets
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
    eprintln!("  snippets-app --read 'my-snippet'");
    eprintln!("  snippets-app --list");
    eprintln!();
    eprintln!("Storage configuration (via SNIPPETS_APP_STORAGE):");
    eprintln!("  JSON:/path/to/snippets.json");
    eprintln!("  SQLITE:/path/to/snippets.db");
}

/// Read all content from stdin
fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;
    
    if buffer.is_empty() {
        anyhow::bail!("No content provided. Please provide snippet content via stdin");
    }
    
    Ok(buffer)
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
}