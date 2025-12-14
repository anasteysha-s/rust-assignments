mod storage;

use chrono::Local;
use clap::Parser;
use std::io::{self, Read};
use storage::{create_storage, Snippet};

/// A simple CLI tool for managing code snippets with timestamps
#[derive(Parser, Debug)]
#[command(name = "snippets-app")]
#[command(about = "Store and manage code snippets with creation timestamps", long_about = None)]
#[command(version = "0.2.0")]
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
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let cli = Cli::parse();

    // Create storage based on SNIPPETS_APP_STORAGE environment variable
    let mut storage = create_storage()?;

    // Execute command based on arguments
    if let Some(name) = cli.name {
        // Create snippet: read from stdin
        let content = read_stdin()?;
        let snippet = Snippet::new(name.clone(), content);
        storage.save_snippet(snippet.clone())?;

        println!("Snippet '{}' saved successfully", name);
        if cli.verbose {
            let local_time = snippet.created_at.with_timezone(&Local);
            println!("Created at: {}", local_time.format("%Y-%m-%d %H:%M:%S"));
        }
    } else if let Some(name) = cli.read {
        // Read snippet
        match storage.get_snippet(&name)? {
            Some(snippet) => {
                if cli.verbose {
                    let local_time = snippet.created_at.with_timezone(&Local);
                    println!("# Snippet: {}", snippet.name);
                    println!("# Created: {}", local_time.format("%Y-%m-%d %H:%M:%S"));
                    println!();
                }
                print!("{}", snippet.content);
            }
            None => {
                eprintln!("Snippet '{}' not found", name);
                std::process::exit(1);
            }
        }
    } else if let Some(name) = cli.delete {
        // Delete snippet
        if storage.delete_snippet(&name)? {
            println!("Snippet '{}' deleted successfully", name);
        } else {
            eprintln!("Snippet '{}' not found", name);
            std::process::exit(1);
        }
    } else if cli.list {
        // List all snippets
        let names = storage.list_snippets()?;
        if names.is_empty() {
            println!("No snippets found");
        } else {
            if cli.verbose {
                println!("Stored snippets (with creation times):");
                println!();
                for name in names {
                    if let Ok(Some(snippet)) = storage.get_snippet(&name) {
                        let local_time = snippet.created_at.with_timezone(&Local);
                        println!(
                            "  {} - {}",
                            name,
                            local_time.format("%Y-%m-%d %H:%M:%S")
                        );
                    }
                }
            } else {
                println!("Stored snippets:");
                for name in names {
                    println!("  - {}", name);
                }
            }
        }
    } else {
        // No command specified
        eprintln!("No command specified. Use --help for usage information.");
        eprintln!();
        eprintln!("Tip: Set SNIPPETS_APP_STORAGE environment variable to choose storage:");
        eprintln!("  JSON:/path/to/snippets.json");
        eprintln!("  SQLITE:/path/to/snippets.db");
        std::process::exit(1);
    }

    Ok(())
}

/// Read all content from stdin
fn read_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
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
    fn test_snippet_has_timestamp() {
        let snippet = Snippet::new("test".to_string(), "content".to_string());
        // Timestamp should be recent (within last minute)
        let now = chrono::Utc::now();
        let diff = (now - snippet.created_at).num_seconds();
        assert!(diff >= 0 && diff < 60);
    }
}