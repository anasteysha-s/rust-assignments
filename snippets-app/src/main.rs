mod storage;

use clap::Parser;
use std::io::{self, Read};
use storage::{get_default_storage_path, SnippetStorage};

/// A simple CLI tool for managing code snippets
#[derive(Parser, Debug)]
#[command(name = "snippets-app")]
#[command(about = "Store and manage code snippets", long_about = None)]
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

    /// List all snippets
    #[arg(long)]
    list: bool,

    /// Custom storage file path (optional)
    #[arg(long, value_name = "PATH")]
    storage: Option<String>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let cli = Cli::parse();

    // Determine storage path
    let storage_path = cli
        .storage
        .map(|p| p.into())
        .unwrap_or_else(get_default_storage_path);

    // Load storage
    let mut storage = SnippetStorage::load_or_create(&storage_path)?;

    // Execute command based on arguments
    if let Some(name) = cli.name {
        // Create snippet: read from stdin
        let content = read_stdin()?;
        storage.add_snippet(name.clone(), content);
        storage.save(&storage_path)?;
        println!("Snippet '{}' saved successfully", name);
    } else if let Some(name) = cli.read {
        // Read snippet
        match storage.get_snippet(&name) {
            Some(content) => print!("{}", content),
            None => {
                eprintln!("Snippet '{}' not found", name);
                std::process::exit(1);
            }
        }
    } else if let Some(name) = cli.delete {
        // Delete snippet
        if storage.delete_snippet(&name) {
            storage.save(&storage_path)?;
            println!("Snippet '{}' deleted successfully", name);
        } else {
            eprintln!("Snippet '{}' not found", name);
            std::process::exit(1);
        }
    } else if cli.list {
        // List all snippets
        let snippets = storage.list_snippets();
        if snippets.is_empty() {
            println!("No snippets found");
        } else {
            println!("Stored snippets:");
            for name in snippets {
                println!("  - {}", name);
            }
        }
    } else {
        // No command specified
        eprintln!("No command specified. Use --help for usage information.");
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
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_storage_integration() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        fs::remove_file(path).unwrap();

        // Create storage and add snippet
        let mut storage = SnippetStorage::load_or_create(path).unwrap();
        storage.add_snippet("test".to_string(), "println!(\"Hello\");".to_string());
        storage.save(path).unwrap();

        // Load and verify
        let loaded = SnippetStorage::load_or_create(path).unwrap();
        assert_eq!(
            loaded.get_snippet("test"),
            Some(&"println!(\"Hello\");".to_string())
        );
    }

    #[test]
    fn test_multiple_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        fs::remove_file(path).unwrap();

        // Add multiple snippets
        let mut storage = SnippetStorage::load_or_create(path).unwrap();
        storage.add_snippet("rust".to_string(), "fn main() {}".to_string());
        storage.add_snippet("python".to_string(), "print('Hello')".to_string());
        storage.save(path).unwrap();

        // Load and verify count
        let loaded = SnippetStorage::load_or_create(path).unwrap();
        assert_eq!(loaded.count(), 2);

        // Delete one
        let mut loaded = loaded;
        loaded.delete_snippet("rust");
        loaded.save(path).unwrap();

        // Load and verify deletion
        let final_storage = SnippetStorage::load_or_create(path).unwrap();
        assert_eq!(final_storage.count(), 1);
        assert_eq!(final_storage.get_snippet("rust"), None);
        assert_eq!(
            final_storage.get_snippet("python"),
            Some(&"print('Hello')".to_string())
        );
    }
}