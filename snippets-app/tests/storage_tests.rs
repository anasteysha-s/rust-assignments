// tests/storage_tests.rs

#[cfg(test)]
mod storage_tests {
    use chrono::Utc;
    use snippets_app::storage::{
        create_storage, parse_storage_config, JsonStorage, Snippet, SnippetStorage, SqliteStorage,
        StorageType,
    };
    use std::env;
    use std::fs;
    use std::path::Path;
    use tempfile::{NamedTempFile, TempDir};

    // ========================================================================
    // SNIPPET TESTS
    // ========================================================================

    #[test]
    fn test_snippet_new_valid() {
        let snippet = Snippet::new("test".to_string(), "content".to_string()).unwrap();
        assert_eq!(snippet.name, "test");
        assert_eq!(snippet.content, "content");
        assert!(snippet.created_at <= Utc::now());
    }

    #[test]
    fn test_snippet_empty_name() {
        let result = Snippet::new("".to_string(), "content".to_string());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn test_snippet_name_too_long() {
        let long_name = "a".repeat(256);
        let result = Snippet::new(long_name, "content".to_string());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("too long"));
    }

    #[test]
    fn test_snippet_name_with_null_byte() {
        let result = Snippet::new("test\0name".to_string(), "content".to_string());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null bytes"));
    }

    #[test]
    fn test_snippet_max_valid_name_length() {
        let name = "a".repeat(255);
        let result = Snippet::new(name.clone(), "content".to_string());
        assert!(result.is_ok());
        let snippet = result.unwrap();
        assert_eq!(snippet.name.len(), 255);
    }

    #[test]
    fn test_snippet_empty_content_allowed() {
        let result = Snippet::new("test".to_string(), "".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_snippet_special_characters_in_name() {
        let names = vec!["test-name", "test_name", "test.name", "test name", "test123"];
        for name in names {
            let result = Snippet::new(name.to_string(), "content".to_string());
            assert!(result.is_ok(), "Name '{}' should be valid", name);
        }
    }

    // ========================================================================
    // JSON STORAGE TESTS
    // ========================================================================

    #[test]
    fn test_json_storage_new() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        fs::remove_file(path).ok();

        let storage = JsonStorage::new(path);
        assert!(storage.is_ok());
    }

    #[test]
    fn test_json_storage_save_and_get() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        fs::remove_file(path).ok();

        let mut storage = JsonStorage::new(path).unwrap();
        let snippet = Snippet::new("test".to_string(), "println!(\"Hello\");".to_string()).unwrap();

        storage.save_snippet(snippet.clone()).unwrap();
        let retrieved = storage.get_snippet("test").unwrap().unwrap();

        assert_eq!(retrieved.name, snippet.name);
        assert_eq!(retrieved.content, snippet.content);
        assert_eq!(retrieved.created_at, snippet.created_at);
    }

    #[test]
    fn test_json_storage_get_nonexistent() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        fs::remove_file(path).ok();

        let storage = JsonStorage::new(path).unwrap();
        let result = storage.get_snippet("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_json_storage_update_snippet() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        fs::remove_file(path).ok();

        let mut storage = JsonStorage::new(path).unwrap();
        let snippet1 = Snippet::new("test".to_string(), "content1".to_string()).unwrap();
        storage.save_snippet(snippet1).unwrap();

        let snippet2 = Snippet::new("test".to_string(), "content2".to_string()).unwrap();
        storage.save_snippet(snippet2.clone()).unwrap();

        let retrieved = storage.get_snippet("test").unwrap().unwrap();
        assert_eq!(retrieved.content, "content2");
    }

    #[test]
    fn test_json_storage_delete_existing() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        fs::remove_file(path).ok();

        let mut storage = JsonStorage::new(path).unwrap();
        let snippet = Snippet::new("test".to_string(), "content".to_string()).unwrap();
        storage.save_snippet(snippet).unwrap();

        let deleted = storage.delete_snippet("test").unwrap();
        assert!(deleted);

        let result = storage.get_snippet("test").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_json_storage_delete_nonexistent() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        fs::remove_file(path).ok();

        let mut storage = JsonStorage::new(path).unwrap();
        let deleted = storage.delete_snippet("nonexistent").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_json_storage_list_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        fs::remove_file(path).ok();

        let storage = JsonStorage::new(path).unwrap();
        let names = storage.list_snippets().unwrap();
        assert!(names.is_empty());
    }

    #[test]
    fn test_json_storage_list_multiple() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        fs::remove_file(path).ok();

        let mut storage = JsonStorage::new(path).unwrap();

        let snippets = vec!["zebra", "apple", "middle"];
        for name in &snippets {
            let snippet = Snippet::new(name.to_string(), "content".to_string()).unwrap();
            storage.save_snippet(snippet).unwrap();
        }

        let names = storage.list_snippets().unwrap();
        assert_eq!(names.len(), 3);
        // Should be sorted alphabetically
        assert_eq!(names, vec!["apple", "middle", "zebra"]);
    }

    #[test]
    fn test_json_storage_persistence() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        fs::remove_file(&path).ok();

        {
            let mut storage = JsonStorage::new(&path).unwrap();
            let snippet = Snippet::new("test".to_string(), "content".to_string()).unwrap();
            storage.save_snippet(snippet).unwrap();
        }

        // Reload from file
        let storage = JsonStorage::new(&path).unwrap();
        let retrieved = storage.get_snippet("test").unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_json_storage_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        fs::write(path, "").unwrap();

        let storage = JsonStorage::new(path);
        assert!(storage.is_ok());
    }

    #[test]
    fn test_json_storage_invalid_json() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        fs::write(path, "invalid json {").unwrap();

        let storage = JsonStorage::new(path);
        assert!(storage.is_err());
    }

    #[test]
    fn test_json_storage_nested_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("a/b/c/snippets.json");

        let storage = JsonStorage::new(&nested_path);
        assert!(storage.is_ok());
        assert!(nested_path.parent().unwrap().exists());
    }

    // ========================================================================
    // SQLITE STORAGE TESTS
    // ========================================================================

    #[test]
    fn test_sqlite_storage_new() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = SqliteStorage::new(temp_file.path());
        assert!(storage.is_ok());
    }

    #[test]
    fn test_sqlite_storage_save_and_get() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut storage = SqliteStorage::new(temp_file.path()).unwrap();

        let snippet = Snippet::new("test".to_string(), "println!(\"Hello\");".to_string()).unwrap();
        storage.save_snippet(snippet.clone()).unwrap();

        let retrieved = storage.get_snippet("test").unwrap().unwrap();
        assert_eq!(retrieved.name, snippet.name);
        assert_eq!(retrieved.content, snippet.content);
    }

    #[test]
    fn test_sqlite_storage_get_nonexistent() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = SqliteStorage::new(temp_file.path()).unwrap();

        let result = storage.get_snippet("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_sqlite_storage_update_snippet() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut storage = SqliteStorage::new(temp_file.path()).unwrap();

        let snippet1 = Snippet::new("test".to_string(), "content1".to_string()).unwrap();
        storage.save_snippet(snippet1).unwrap();

        let snippet2 = Snippet::new("test".to_string(), "content2".to_string()).unwrap();
        storage.save_snippet(snippet2).unwrap();

        let retrieved = storage.get_snippet("test").unwrap().unwrap();
        assert_eq!(retrieved.content, "content2");
    }

    #[test]
    fn test_sqlite_storage_delete_existing() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut storage = SqliteStorage::new(temp_file.path()).unwrap();

        let snippet = Snippet::new("test".to_string(), "content".to_string()).unwrap();
        storage.save_snippet(snippet).unwrap();

        let deleted = storage.delete_snippet("test").unwrap();
        assert!(deleted);

        let result = storage.get_snippet("test").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_sqlite_storage_delete_nonexistent() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut storage = SqliteStorage::new(temp_file.path()).unwrap();

        let deleted = storage.delete_snippet("nonexistent").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_sqlite_storage_list_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = SqliteStorage::new(temp_file.path()).unwrap();

        let names = storage.list_snippets().unwrap();
        assert!(names.is_empty());
    }

    #[test]
    fn test_sqlite_storage_list_multiple() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut storage = SqliteStorage::new(temp_file.path()).unwrap();

        let snippets = vec!["zebra", "apple", "middle"];
        for name in &snippets {
            let snippet = Snippet::new(name.to_string(), "content".to_string()).unwrap();
            storage.save_snippet(snippet).unwrap();
        }

        let names = storage.list_snippets().unwrap();
        assert_eq!(names.len(), 3);
        // Should be sorted alphabetically
        assert_eq!(names, vec!["apple", "middle", "zebra"]);
    }

    #[test]
    fn test_sqlite_storage_large_content() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut storage = SqliteStorage::new(temp_file.path()).unwrap();

        let large_content = "x".repeat(1_000_000); // 1MB
        let snippet = Snippet::new("large".to_string(), large_content.clone()).unwrap();
        storage.save_snippet(snippet).unwrap();

        let retrieved = storage.get_snippet("large").unwrap().unwrap();
        assert_eq!(retrieved.content.len(), 1_000_000);
    }

    #[test]
    fn test_sqlite_storage_special_characters() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut storage = SqliteStorage::new(temp_file.path()).unwrap();

        let content = "Special: 'quotes', \"double\", `backticks`, \n newlines \t tabs";
        let snippet = Snippet::new("special".to_string(), content.to_string()).unwrap();
        storage.save_snippet(snippet).unwrap();

        let retrieved = storage.get_snippet("special").unwrap().unwrap();
        assert_eq!(retrieved.content, content);
    }

    #[test]
    fn test_sqlite_storage_nested_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("a/b/c/snippets.db");

        let storage = SqliteStorage::new(&nested_path);
        assert!(storage.is_ok());
        assert!(nested_path.parent().unwrap().exists());
    }

    // ========================================================================
    // CONFIG PARSING TESTS
    // ========================================================================

    #[test]
    fn test_parse_storage_config_json_valid() {
        let result = parse_storage_config("JSON:/home/user/snippets.json");
        assert!(result.is_ok());

        let (storage_type, path) = result.unwrap();
        assert!(matches!(storage_type, StorageType::Json));
        assert_eq!(path.to_string_lossy(), "/home/user/snippets.json");
    }

    #[test]
    fn test_parse_storage_config_sqlite_valid() {
        let result = parse_storage_config("SQLITE:/tmp/snippets.db");
        assert!(result.is_ok());

        let (storage_type, path) = result.unwrap();
        assert!(matches!(storage_type, StorageType::Sqlite));
        assert_eq!(path.to_string_lossy(), "/tmp/snippets.db");
    }

    #[test]
    fn test_parse_storage_config_case_insensitive() {
        let configs = vec!["json:/path", "Json:/path", "JSON:/path", "JsOn:/path"];
        for config in configs {
            let result = parse_storage_config(config);
            assert!(result.is_ok(), "Config '{}' should be valid", config);
            assert!(matches!(result.unwrap().0, StorageType::Json));
        }
    }

    #[test]
    fn test_parse_storage_config_invalid_format_no_colon() {
        let result = parse_storage_config("JSON/path/to/file");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid storage configuration"));
    }

    #[test]
    fn test_parse_storage_config_invalid_format_empty() {
        let result = parse_storage_config("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_storage_config_unknown_type() {
        let result = parse_storage_config("REDIS:/path");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown storage type"));
    }

    #[test]
    fn test_parse_storage_config_empty_path() {
        let result = parse_storage_config("JSON:");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_parse_storage_config_with_colon_in_path() {
        // Windows paths like C:\path
        let result = parse_storage_config("JSON:C:\\Users\\test\\snippets.json");
        assert!(result.is_ok());
    }

    // ========================================================================
    // FACTORY TESTS
    // ========================================================================

    #[test]
    fn test_create_storage_default() {
        unsafe {
            env::remove_var("SNIPPETS_APP_STORAGE");
        }

        let storage = create_storage();
        assert!(storage.is_ok());
    }

    #[test]
    fn test_create_storage_json_from_env() {
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
    fn test_create_storage_sqlite_from_env() {
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
    fn test_create_storage_invalid_config() {
        unsafe {
            env::set_var("SNIPPETS_APP_STORAGE", "INVALID_FORMAT");
        }

        let storage = create_storage();
        assert!(storage.is_err());

        unsafe {
            env::remove_var("SNIPPETS_APP_STORAGE");
        }
    }

    // ========================================================================
    // CROSS-STORAGE COMPATIBILITY TESTS
    // ========================================================================

    #[test]
    fn test_json_and_sqlite_compatibility() {
        let temp_json = NamedTempFile::new().unwrap();
        let temp_sqlite = NamedTempFile::new().unwrap();

        let mut json_storage = JsonStorage::new(temp_json.path()).unwrap();
        let mut sqlite_storage = SqliteStorage::new(temp_sqlite.path()).unwrap();

        let snippet = Snippet::new("test".to_string(), "content".to_string()).unwrap();

        // Save to both
        json_storage.save_snippet(snippet.clone()).unwrap();
        sqlite_storage.save_snippet(snippet.clone()).unwrap();

        // Retrieve from both
        let json_retrieved = json_storage.get_snippet("test").unwrap().unwrap();
        let sqlite_retrieved = sqlite_storage.get_snippet("test").unwrap().unwrap();

        // Should be equal
        assert_eq!(json_retrieved.name, sqlite_retrieved.name);
        assert_eq!(json_retrieved.content, sqlite_retrieved.content);
    }

    // ========================================================================
    // EDGE CASES
    // ========================================================================

    #[test]
    fn test_unicode_content() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut storage = JsonStorage::new(temp_file.path()).unwrap();

        let content = "Hello 世界 🦀 Rust!";
        let snippet = Snippet::new("unicode".to_string(), content.to_string()).unwrap();
        storage.save_snippet(snippet).unwrap();

        let retrieved = storage.get_snippet("unicode").unwrap().unwrap();
        assert_eq!(retrieved.content, content);
    }

    #[test]
    fn test_multiline_content() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut storage = JsonStorage::new(temp_file.path()).unwrap();

        let content = "line 1\nline 2\nline 3\n\nline 5";
        let snippet = Snippet::new("multiline".to_string(), content.to_string()).unwrap();
        storage.save_snippet(snippet).unwrap();

        let retrieved = storage.get_snippet("multiline").unwrap().unwrap();
        assert_eq!(retrieved.content, content);
    }

    #[test]
    fn test_concurrent_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Simulate multiple operations
        let mut storage = JsonStorage::new(&path).unwrap();

        for i in 0..10 {
            let snippet =
                Snippet::new(format!("test{}", i), format!("content{}", i)).unwrap();
            storage.save_snippet(snippet).unwrap();
        }

        let names = storage.list_snippets().unwrap();
        assert_eq!(names.len(), 10);
    }
}