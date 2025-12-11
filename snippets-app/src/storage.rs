use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Represents a code snippet with its content
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Snippet {
    pub name: String,
    pub content: String,
}

/// Storage for managing code snippets
#[derive(Debug, Serialize, Deserialize)]
pub struct SnippetStorage {
    snippets: HashMap<String, String>,
}

impl SnippetStorage {
    /// Create a new empty storage
    pub fn new() -> Self {
        Self {
            snippets: HashMap::new(),
        }
    }

    /// Load storage from file, or create new if file doesn't exist
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let storage: SnippetStorage = serde_json::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(storage)
        } else {
            Ok(Self::new())
        }
    }

    /// Save storage to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, json)
    }

    /// Add or update a snippet
    pub fn add_snippet(&mut self, name: String, content: String) {
        self.snippets.insert(name, content);
    }

    /// Get a snippet by name
    pub fn get_snippet(&self, name: &str) -> Option<&String> {
        self.snippets.get(name)
    }

    /// Delete a snippet by name
    pub fn delete_snippet(&mut self, name: &str) -> bool {
        self.snippets.remove(name).is_some()
    }

    /// List all snippet names
    pub fn list_snippets(&self) -> Vec<&String> {
        let mut names: Vec<&String> = self.snippets.keys().collect();
        names.sort();
        names
    }

    /// Get the number of snippets
    pub fn count(&self) -> usize {
        self.snippets.len()
    }
}

impl Default for SnippetStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the default storage path
pub fn get_default_storage_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".snippets.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_new_storage() {
        let storage = SnippetStorage::new();
        assert_eq!(storage.count(), 0);
    }

    #[test]
    fn test_add_snippet() {
        let mut storage = SnippetStorage::new();
        storage.add_snippet("test".to_string(), "content".to_string());
        
        assert_eq!(storage.count(), 1);
        assert_eq!(storage.get_snippet("test"), Some(&"content".to_string()));
    }

    #[test]
    fn test_get_nonexistent_snippet() {
        let storage = SnippetStorage::new();
        assert_eq!(storage.get_snippet("nonexistent"), None);
    }

    #[test]
    fn test_delete_snippet() {
        let mut storage = SnippetStorage::new();
        storage.add_snippet("test".to_string(), "content".to_string());
        
        assert!(storage.delete_snippet("test"));
        assert_eq!(storage.count(), 0);
        assert_eq!(storage.get_snippet("test"), None);
    }

    #[test]
    fn test_delete_nonexistent_snippet() {
        let mut storage = SnippetStorage::new();
        assert!(!storage.delete_snippet("nonexistent"));
    }

    #[test]
    fn test_update_snippet() {
        let mut storage = SnippetStorage::new();
        storage.add_snippet("test".to_string(), "old content".to_string());
        storage.add_snippet("test".to_string(), "new content".to_string());
        
        assert_eq!(storage.count(), 1);
        assert_eq!(storage.get_snippet("test"), Some(&"new content".to_string()));
    }

    #[test]
    fn test_list_snippets() {
        let mut storage = SnippetStorage::new();
        storage.add_snippet("zebra".to_string(), "z".to_string());
        storage.add_snippet("alpha".to_string(), "a".to_string());
        storage.add_snippet("beta".to_string(), "b".to_string());
        
        let names = storage.list_snippets();
        assert_eq!(names.len(), 3);
        assert_eq!(names, vec![&"alpha".to_string(), &"beta".to_string(), &"zebra".to_string()]);
    }

    #[test]
    fn test_save_and_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Create and save storage
        let mut storage = SnippetStorage::new();
        storage.add_snippet("test1".to_string(), "content1".to_string());
        storage.add_snippet("test2".to_string(), "content2".to_string());
        storage.save(path).unwrap();

        // Load storage
        let loaded_storage = SnippetStorage::load_or_create(path).unwrap();
        assert_eq!(loaded_storage.count(), 2);
        assert_eq!(loaded_storage.get_snippet("test1"), Some(&"content1".to_string()));
        assert_eq!(loaded_storage.get_snippet("test2"), Some(&"content2".to_string()));
    }

    #[test]
    fn test_load_or_create_nonexistent() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        // Delete the file so it doesn't exist
        std::fs::remove_file(path).unwrap();

        // Should create new storage
        let storage = SnippetStorage::load_or_create(path).unwrap();
        assert_eq!(storage.count(), 0);
    }
}