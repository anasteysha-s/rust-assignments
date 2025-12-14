use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// Represents a code snippet with metadata
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Snippet {
    pub name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Snippet {
    pub fn new(name: String, content: String) -> Self {
        Self {
            name,
            content,
            created_at: Utc::now(),
        }
    }
}

/// Storage trait for snippet persistence (using dynamic dispatch)
pub trait SnippetStorage {
    /// Add or update a snippet
    fn save_snippet(&mut self, snippet: Snippet) -> io::Result<()>;

    /// Get a snippet by name
    fn get_snippet(&self, name: &str) -> io::Result<Option<Snippet>>;

    /// Delete a snippet by name
    fn delete_snippet(&mut self, name: &str) -> io::Result<bool>;

    /// List all snippet names (sorted alphabetically)
    fn list_snippets(&self) -> io::Result<Vec<String>>;
}

// ============================================================================
// JSON STORAGE IMPLEMENTATION
// ============================================================================

/// JSON file-based storage
pub struct JsonStorage {
    file_path: String,
    snippets: HashMap<String, Snippet>,
}

impl JsonStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file_path = path.as_ref().to_string_lossy().to_string();
        let snippets = if Path::new(&file_path).exists() {
            let content = fs::read_to_string(&file_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new())
        } else {
            HashMap::new()
        };

        Ok(Self {
            file_path,
            snippets,
        })
    }

    fn persist(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.snippets)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Create parent directories if they don't exist
        if let Some(parent) = Path::new(&self.file_path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.file_path, json)
    }
}

impl SnippetStorage for JsonStorage {
    fn save_snippet(&mut self, snippet: Snippet) -> io::Result<()> {
        self.snippets.insert(snippet.name.clone(), snippet);
        self.persist()
    }

    fn get_snippet(&self, name: &str) -> io::Result<Option<Snippet>> {
        Ok(self.snippets.get(name).cloned())
    }

    fn delete_snippet(&mut self, name: &str) -> io::Result<bool> {
        let removed = self.snippets.remove(name).is_some();
        if removed {
            self.persist()?;
        }
        Ok(removed)
    }

    fn list_snippets(&self) -> io::Result<Vec<String>> {
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
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Create table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS snippets (
                name TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(Self { conn })
    }

    fn sql_error_to_io_error(e: rusqlite::Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

impl SnippetStorage for SqliteStorage {
    fn save_snippet(&mut self, snippet: Snippet) -> io::Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO snippets (name, content, created_at) VALUES (?1, ?2, ?3)",
                params![
                    snippet.name,
                    snippet.content,
                    snippet.created_at.to_rfc3339()
                ],
            )
            .map_err(Self::sql_error_to_io_error)?;
        Ok(())
    }

    fn get_snippet(&self, name: &str) -> io::Result<Option<Snippet>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, content, created_at FROM snippets WHERE name = ?1")
            .map_err(Self::sql_error_to_io_error)?;

        let result = stmt
            .query_row(params![name], |row| {
                let name: String = row.get(0)?;
                let content: String = row.get(1)?;
                let created_at_str: String = row.get(2)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Snippet {
                    name,
                    content,
                    created_at,
                })
            })
            .optional()
            .map_err(Self::sql_error_to_io_error)?;

        Ok(result)
    }

    fn delete_snippet(&mut self, name: &str) -> io::Result<bool> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM snippets WHERE name = ?1", params![name])
            .map_err(Self::sql_error_to_io_error)?;

        Ok(rows_affected > 0)
    }

    fn list_snippets(&self) -> io::Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM snippets ORDER BY name")
            .map_err(Self::sql_error_to_io_error)?;

        let names = stmt
            .query_map([], |row| row.get(0))
            .map_err(Self::sql_error_to_io_error)?
            .collect::<SqlResult<Vec<String>>>()
            .map_err(Self::sql_error_to_io_error)?;

        Ok(names)
    }
}

// ============================================================================
// STORAGE FACTORY
// ============================================================================

#[derive(Debug)]
pub enum StorageType {
    Json,
    Sqlite,
}

/// Parse storage configuration from environment variable
/// Format: "JSON:/path/to/file.json" or "SQLITE:/path/to/file.sqlite"
pub fn parse_storage_config(config: &str) -> io::Result<(StorageType, String)> {
    let parts: Vec<&str> = config.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid storage config format. Expected: 'JSON:/path' or 'SQLITE:/path'",
        ));
    }

    let storage_type = match parts[0].to_uppercase().as_str() {
        "JSON" => StorageType::Json,
        "SQLITE" => StorageType::Sqlite,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown storage type: {}. Expected 'JSON' or 'SQLITE'", parts[0]),
            ));
        }
    };

    let path = parts[1].to_string();
    if path.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Path cannot be empty",
        ));
    }

    Ok((storage_type, path))
}

/// Create a storage instance based on environment variable or default
pub fn create_storage() -> io::Result<Box<dyn SnippetStorage>> {
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
            let default_path = format!("{}/.snippets.json", home);
            Ok(Box::new(JsonStorage::new(default_path)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_snippet(name: &str, content: &str) -> Snippet {
        Snippet::new(name.to_string(), content.to_string())
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
    fn test_json_delete() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        std::fs::remove_file(path).ok();

        let mut storage = JsonStorage::new(path).unwrap();
        let snippet = create_test_snippet("test", "content");

        storage.save_snippet(snippet).unwrap();
        assert!(storage.delete_snippet("test").unwrap());
        assert!(storage.get_snippet("test").unwrap().is_none());
    }

    #[test]
    fn test_json_list() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        std::fs::remove_file(path).ok();

        let mut storage = JsonStorage::new(path).unwrap();

        storage.save_snippet(create_test_snippet("zebra", "z")).unwrap();
        storage.save_snippet(create_test_snippet("alpha", "a")).unwrap();
        storage.save_snippet(create_test_snippet("beta", "b")).unwrap();

        let names = storage.list_snippets().unwrap();
        assert_eq!(names, vec!["alpha", "beta", "zebra"]);
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

    #[test]
    fn test_sqlite_delete() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut storage = SqliteStorage::new(path).unwrap();
        let snippet = create_test_snippet("test", "content");

        storage.save_snippet(snippet).unwrap();
        assert!(storage.delete_snippet("test").unwrap());
        assert!(storage.get_snippet("test").unwrap().is_none());
    }

    #[test]
    fn test_sqlite_list() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut storage = SqliteStorage::new(path).unwrap();

        storage.save_snippet(create_test_snippet("zebra", "z")).unwrap();
        storage.save_snippet(create_test_snippet("alpha", "a")).unwrap();
        storage.save_snippet(create_test_snippet("beta", "b")).unwrap();

        let names = storage.list_snippets().unwrap();
        assert_eq!(names, vec!["alpha", "beta", "zebra"]);
    }

    #[test]
    fn test_sqlite_update() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut storage = SqliteStorage::new(path).unwrap();
        let snippet1 = create_test_snippet("test", "old content");
        storage.save_snippet(snippet1).unwrap();

        let snippet2 = create_test_snippet("test", "new content");
        storage.save_snippet(snippet2).unwrap();

        let retrieved = storage.get_snippet("test").unwrap().unwrap();
        assert_eq!(retrieved.content, "new content");
    }

    // ========================================================================
    // CONFIG PARSING TESTS
    // ========================================================================

    #[test]
    fn test_parse_json_config() {
        let (storage_type, path) = parse_storage_config("JSON:/home/user/snippets.json").unwrap();
        assert!(matches!(storage_type, StorageType::Json));
        assert_eq!(path, "/home/user/snippets.json");
    }

    #[test]
    fn test_parse_sqlite_config() {
        let (storage_type, path) =
            parse_storage_config("SQLITE:/home/user/snippets.db").unwrap();
        assert!(matches!(storage_type, StorageType::Sqlite));
        assert_eq!(path, "/home/user/snippets.db");
    }

    #[test]
    fn test_parse_case_insensitive() {
        let (storage_type, _) = parse_storage_config("json:/path").unwrap();
        assert!(matches!(storage_type, StorageType::Json));

        let (storage_type, _) = parse_storage_config("sqlite:/path").unwrap();
        assert!(matches!(storage_type, StorageType::Sqlite));
    }

    #[test]
    fn test_parse_invalid_format() {
        assert!(parse_storage_config("invalid").is_err());
        assert!(parse_storage_config("JSON").is_err());
    }

    #[test]
    fn test_parse_unknown_type() {
        assert!(parse_storage_config("UNKNOWN:/path").is_err());
    }

    #[test]
    fn test_parse_empty_path() {
        assert!(parse_storage_config("JSON:").is_err());
    }

    // ========================================================================
    // CROSS-STORAGE COMPATIBILITY TESTS
    // ========================================================================

    #[test]
    fn test_timestamp_preserved() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut storage = SqliteStorage::new(path).unwrap();
        let snippet = create_test_snippet("test", "content");
        let original_time = snippet.created_at;

        storage.save_snippet(snippet).unwrap();
        let retrieved = storage.get_snippet("test").unwrap().unwrap();

        // Timestamps should be very close (within 1 second due to serialization)
        let diff = (retrieved.created_at - original_time).num_seconds().abs();
        assert!(diff < 2);
    }
}