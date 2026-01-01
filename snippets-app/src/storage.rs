use crate::error::{ConfigError, Result, SnippetError, StorageError, StorageResult};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a code snippet with metadata.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Snippet {
    /// The unique name of the snippet (1-255 characters, no null bytes)
    pub name: String,
    
    /// The content/code of the snippet
    pub content: String,
    
    /// When the snippet was created (UTC timestamp)
    pub created_at: DateTime<Utc>,
}

impl Snippet {
    /// Create a new snippet with validation.
    ///
    /// Validates the name and sets the current timestamp.
    ///
    /// # Errors
    ///
    /// Returns error if name is empty, too long (>255 chars), or contains null bytes.
    pub fn new(name: String, content: String) -> Result<Self> {
        Self::validate_name(&name)?;

        Ok(Self {
            name,
            content,
            created_at: Utc::now(),
        })
    }

    /// Validate snippet name
    fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(SnippetError::InvalidName {
                name: name.to_string(),
                reason: "Name cannot be empty".to_string(),
            });
        }

        if name.len() > 255 {
            return Err(SnippetError::InvalidName {
                name: name.to_string(),
                reason: "Name too long (max 255 characters)".to_string(),
            });
        }

        // Check for invalid characters
        if name.contains('\0') {
            return Err(SnippetError::InvalidName {
                name: name.to_string(),
                reason: "Name cannot contain null bytes".to_string(),
            });
        }

        Ok(())
    }
}

/// Storage trait for snippet persistence (using dynamic dispatch)
pub trait SnippetStorage {
    /// Add or update a snippet
    fn save_snippet(&mut self, snippet: Snippet) -> Result<()>;

    /// Get a snippet by name
    fn get_snippet(&self, name: &str) -> Result<Option<Snippet>>;

    /// Delete a snippet by name, returns true if deleted
    fn delete_snippet(&mut self, name: &str) -> Result<bool>;

    /// List all snippet names (sorted alphabetically)
    fn list_snippets(&self) -> Result<Vec<String>>;
}

// JSON STORAGE IMPLEMENTATION

/// JSON file-based storage
pub struct JsonStorage {
    file_path: PathBuf,
    snippets: HashMap<String, Snippet>,
}

impl JsonStorage {
    /// Create a new JSON storage instance.
    ///
    /// Loads existing snippets from the JSON file if it exists. If the file doesn't
    /// exist or is empty, starts with an empty storage. Automatically creates parent
    /// directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON storage file
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::FileSystem`] if:
    /// - Parent directories cannot be created
    /// - File exists but cannot be read
    ///
    /// Returns [`StorageError::Json`] if the file contains invalid JSON.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use snippets_app::JsonStorage;
    ///
    /// // Create with new file
    /// let storage = JsonStorage::new("snippets.json")?;
    ///
    /// // Create with nested path (creates directories)
    /// let storage = JsonStorage::new("/path/to/dir/snippets.json")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let file_path = path.as_ref().to_path_buf();

        // Validate path
        if let Some(parent) = file_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| StorageError::file_system(&file_path, e))?;
            }
        }

        let snippets = if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| StorageError::file_system(&file_path, e))?;

            if content.trim().is_empty() {
                // Empty file, treat as empty storage
                HashMap::new()
            } else {
                serde_json::from_str(&content).map_err(|e| {
                    StorageError::json(format!("reading JSON file '{}'", file_path.display()), e)
                })?
            }
        } else {
            HashMap::new()
        };

        Ok(Self {
            file_path,
            snippets,
        })
    }

    fn persist(&self) -> StorageResult<()> {
        let json = serde_json::to_string_pretty(&self.snippets).map_err(|e| {
            StorageError::json(
                format!("serializing snippets to '{}'", self.file_path.display()),
                e,
            )
        })?;

        // Create parent directories if they don't exist
        if let Some(parent) = self.file_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|e| StorageError::file_system(&self.file_path, e))?;
            }
        }

        fs::write(&self.file_path, json)
            .map_err(|e| StorageError::file_system(&self.file_path, e))?;

        Ok(())
    }
}

impl SnippetStorage for JsonStorage {
    fn save_snippet(&mut self, snippet: Snippet) -> Result<()> {
        self.snippets.insert(snippet.name.clone(), snippet);
        self.persist()?;
        Ok(())
    }

    fn get_snippet(&self, name: &str) -> Result<Option<Snippet>> {
        Ok(self.snippets.get(name).cloned())
    }

    fn delete_snippet(&mut self, name: &str) -> Result<bool> {
        let removed = self.snippets.remove(name).is_some();
        if removed {
            self.persist()?;
        }
        Ok(removed)
    }

    fn list_snippets(&self) -> Result<Vec<String>> {
        let mut names: Vec<String> = self.snippets.keys().cloned().collect();
        names.sort();
        Ok(names)
    }
}

// ============================================================================
// SQLITE STORAGE IMPLEMENTATION
// ============================================================================

/// SQLite database-based storage
pub struct SqliteStorage {
    conn: Connection,
}

impl SqliteStorage {
    /// Create a new SQLite storage instance.
    ///
    /// Opens or creates a SQLite database at the given path. Automatically creates
    /// parent directories if they don't exist. Creates the `snippets` table if it
    /// doesn't already exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the SQLite database file
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Initialization`] if:
    /// - Parent directories cannot be created
    /// - Database file cannot be opened
    ///
    /// Returns [`StorageError::Database`] if the `snippets` table cannot be created.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use snippets_app::SqliteStorage;
    ///
    /// // Create database in current directory
    /// let storage = SqliteStorage::new("snippets.db")?;
    ///
    /// // Create database with nested path (creates directories)
    /// let storage = SqliteStorage::new("/path/to/data/snippets.db")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let db_path = path.as_ref().to_path_buf();

        // Create parent directories if needed
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    StorageError::initialization(
                        "SQLite",
                        &db_path,
                        format!("Cannot create parent directory: {}", e),
                    )
                })?;
            }
        }

        let conn = Connection::open(&db_path).map_err(|e| {
            StorageError::initialization("SQLite", &db_path, format!("Cannot open database: {}", e))
        })?;

        // Create table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS snippets (
                name TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| StorageError::database("creating snippets table", e))?;

        Ok(Self { conn })
    }
}

impl SnippetStorage for SqliteStorage {
    fn save_snippet(&mut self, snippet: Snippet) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO snippets (name, content, created_at) VALUES (?1, ?2, ?3)",
                params![
                    snippet.name,
                    snippet.content,
                    snippet.created_at.to_rfc3339()
                ],
            )
            .map_err(|e| StorageError::database(format!("saving snippet '{}'", snippet.name), e))?;
        Ok(())
    }

    fn get_snippet(&self, name: &str) -> Result<Option<Snippet>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, content, created_at FROM snippets WHERE name = ?1")
            .map_err(|e| StorageError::database("preparing get query", e))?;

        let result = stmt
            .query_row(params![name], |row| {
                let name: String = row.get(0)?;
                let content: String = row.get(1)?;
                let created_at_str: String = row.get(2)?;

                // Parse datetime, return error if invalid
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|_| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid datetime format: {}", created_at_str),
                        )))
                    })?;

                Ok(Snippet {
                    name,
                    content,
                    created_at,
                })
            })
            .optional()
            .map_err(|e| StorageError::database(format!("getting snippet '{}'", name), e))?;

        Ok(result)
    }

    fn delete_snippet(&mut self, name: &str) -> Result<bool> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM snippets WHERE name = ?1", params![name])
            .map_err(|e| StorageError::database(format!("deleting snippet '{}'", name), e))?;

        Ok(rows_affected > 0)
    }

    fn list_snippets(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM snippets ORDER BY name")
            .map_err(|e| StorageError::database("preparing list query", e))?;

        let names = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| StorageError::database("executing list query", e))?
            .collect::<std::result::Result<Vec<String>, _>>()
            .map_err(|e| StorageError::database("collecting snippet names", e))?;

        Ok(names)
    }
}

// ============================================================================
// STORAGE FACTORY
// ============================================================================

/// Storage backend type.
///
/// Specifies which storage implementation to use. Used in configuration
/// parsing to determine the appropriate storage backend.
#[derive(Debug)]
pub enum StorageType {
    /// JSON file-based storage backend
    Json,
    
    /// SQLite database storage backend
    Sqlite,
}

/// Parse storage configuration from environment variable
/// Format: "JSON:/path/to/file.json" or "SQLITE:/path/to/file.sqlite"
pub fn parse_storage_config(config: &str) -> Result<(StorageType, PathBuf)> {
    let parts: Vec<&str> = config.splitn(2, ':').collect();

    if parts.len() != 2 {
        return Err(ConfigError::invalid_format(config).into());
    }

    let storage_type = match parts[0].to_uppercase().as_str() {
        "JSON" => StorageType::Json,
        "SQLITE" => StorageType::Sqlite,
        _ => return Err(ConfigError::unknown_storage_type(parts[0]).into()),
    };

    let path_str = parts[1];
    if path_str.is_empty() {
        return Err(ConfigError::EmptyPath.into());
    }

    let path = PathBuf::from(path_str);

    Ok((storage_type, path))
}

/// Create a storage instance based on environment variable or default
pub fn create_storage() -> Result<Box<dyn SnippetStorage>> {
    match std::env::var("SNIPPETS_APP_STORAGE") {
        Ok(config) => {
            let (storage_type, path) = parse_storage_config(&config)?;
            match storage_type {
                StorageType::Json => Ok(Box::new(JsonStorage::new(path)?)),
                StorageType::Sqlite => Ok(Box::new(SqliteStorage::new(path)?)),
            }
        }
        Err(_) => {
            // Default to JSON storage in home directory
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let default_path = PathBuf::from(home).join(".snippets.json");
            Ok(Box::new(JsonStorage::new(default_path)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_snippet(name: &str, content: &str) -> Snippet {
        Snippet::new(name.to_string(), content.to_string()).unwrap()
    }

    // ========================================================================
    // VALIDATION TESTS
    // ========================================================================

    #[test]
    fn test_empty_snippet_name() {
        let result = Snippet::new("".to_string(), "content".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_snippet_name_too_long() {
        let long_name = "a".repeat(256);
        let result = Snippet::new(long_name, "content".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[test]
    fn test_snippet_name_with_null_byte() {
        let result = Snippet::new("test\0name".to_string(), "content".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null bytes"));
    }

    // ========================================================================
    // JSON STORAGE TESTS
    // ========================================================================

    #[test]
    fn test_json_save_and_get() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        std::fs::remove_file(path).ok();

        let mut storage = JsonStorage::new(path).unwrap();
        let snippet = create_test_snippet("test", "println!(\"Hello\");");

        storage.save_snippet(snippet.clone()).unwrap();
        let retrieved = storage.get_snippet("test").unwrap().unwrap();

        assert_eq!(retrieved.name, snippet.name);
        assert_eq!(retrieved.content, snippet.content);
    }

    #[test]
    fn test_json_not_found() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        std::fs::remove_file(path).ok();

        let storage = JsonStorage::new(path).unwrap();
        let result = storage.get_snippet("nonexistent").unwrap();
        assert!(result.is_none());
    }

    // ========================================================================
    // SQLITE STORAGE TESTS
    // ========================================================================

    #[test]
    fn test_sqlite_save_and_get() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut storage = SqliteStorage::new(path).unwrap();
        let snippet = create_test_snippet("test", "println!(\"Hello\");");

        storage.save_snippet(snippet.clone()).unwrap();
        let retrieved = storage.get_snippet("test").unwrap().unwrap();

        assert_eq!(retrieved.name, snippet.name);
        assert_eq!(retrieved.content, snippet.content);
    }

    // ========================================================================
    // CONFIG PARSING TESTS
    // ========================================================================

    #[test]
    fn test_parse_valid_json_config() {
        let result = parse_storage_config("JSON:/home/user/snippets.json");
        assert!(result.is_ok());
        let (storage_type, _) = result.unwrap();
        assert!(matches!(storage_type, StorageType::Json));
    }

    #[test]
    fn test_parse_invalid_format() {
        let result = parse_storage_config("invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid storage configuration"));
    }

    #[test]
    fn test_parse_empty_path() {
        let result = parse_storage_config("JSON:");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_parse_unknown_type() {
        let result = parse_storage_config("UNKNOWN:/path");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown storage type"));
    }
}
