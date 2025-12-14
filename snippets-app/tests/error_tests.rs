// tests/error_tests.rs

#[cfg(test)]
mod error_tests {
    use snippets_app::error::{ConfigError, SnippetError, StorageError};
    use std::path::PathBuf;

    // ========================================================================
    // SNIPPET ERROR TESTS
    // ========================================================================

    #[test]
    fn test_not_found_error_display() {
        let error = SnippetError::NotFound {
            name: "test-snippet".to_string(),
        };
        assert_eq!(error.to_string(), "Snippet 'test-snippet' not found");
    }

    #[test]
    fn test_already_exists_error_display() {
        let error = SnippetError::AlreadyExists {
            name: "duplicate".to_string(),
        };
        assert_eq!(error.to_string(), "Snippet 'duplicate' already exists");
    }

    #[test]
    fn test_invalid_name_error_display() {
        let error = SnippetError::InvalidName {
            name: "".to_string(),
            reason: "Name cannot be empty".to_string(),
        };
        assert!(error.to_string().contains("Invalid snippet name"));
        assert!(error.to_string().contains("Name cannot be empty"));
    }

    #[test]
    fn test_storage_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let storage_err = StorageError::json("test operation", json_err);
        let snippet_err: SnippetError = storage_err.into();

        assert!(snippet_err.to_string().contains("Storage error"));
    }

    #[test]
    fn test_config_error_conversion() {
        let config_err = ConfigError::EmptyPath;
        let snippet_err: SnippetError = config_err.into();

        assert!(snippet_err.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let snippet_err: SnippetError = io_err.into();

        assert!(snippet_err.to_string().contains("IO error"));
    }

    // ========================================================================
    // STORAGE ERROR TESTS
    // ========================================================================

    #[test]
    fn test_storage_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let error = StorageError::json("deserializing snippet", json_err);

        let error_str = error.to_string();
        assert!(error_str.contains("JSON error"));
        assert!(error_str.contains("deserializing snippet"));
    }

    #[test]
    fn test_storage_database_error() {
        let db_err = rusqlite::Error::InvalidQuery;
        let error = StorageError::database("inserting snippet", db_err);

        let error_str = error.to_string();
        assert!(error_str.contains("Database error"));
        assert!(error_str.contains("inserting snippet"));
    }

    #[test]
    fn test_storage_file_system_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let path = PathBuf::from("/restricted/file.json");
        let error = StorageError::file_system(path, io_err);

        let error_str = error.to_string();
        assert!(error_str.contains("File system error"));
        assert!(error_str.contains("/restricted/file.json"));
    }

    #[test]
    fn test_storage_initialization_error() {
        let error = StorageError::initialization(
            "SQLite",
            PathBuf::from("/tmp/test.db"),
            "Cannot create database",
        );

        let error_str = error.to_string();
        assert!(error_str.contains("Failed to initialize SQLite storage"));
        assert!(error_str.contains("/tmp/test.db"));
        assert!(error_str.contains("Cannot create database"));
    }

    // ========================================================================
    // CONFIG ERROR TESTS
    // ========================================================================

    #[test]
    fn test_config_invalid_format_error() {
        let error = ConfigError::invalid_format("BADFORMAT");

        let error_str = error.to_string();
        assert!(error_str.contains("Invalid storage configuration"));
        assert!(error_str.contains("BADFORMAT"));
        assert!(error_str.contains("TYPE:/path"));
    }

    #[test]
    fn test_config_unknown_storage_type_error() {
        let error = ConfigError::unknown_storage_type("REDIS");

        let error_str = error.to_string();
        assert!(error_str.contains("Unknown storage type"));
        assert!(error_str.contains("REDIS"));
        assert!(error_str.contains("JSON, SQLITE"));
    }

    #[test]
    fn test_config_empty_path_error() {
        let error = ConfigError::EmptyPath;

        let error_str = error.to_string();
        assert!(error_str.contains("Storage path cannot be empty"));
    }

    // ========================================================================
    // ERROR HELPER METHODS TESTS
    // ========================================================================

    #[test]
    fn test_storage_error_json_helper() {
        let json_err = serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err();
        let error = StorageError::json("parsing", json_err);

        assert!(error.to_string().contains("parsing"));
    }

    #[test]
    fn test_storage_error_database_helper() {
        let db_err = rusqlite::Error::InvalidQuery;
        let error = StorageError::database("query execution", db_err);

        assert!(error.to_string().contains("query execution"));
    }

    #[test]
    fn test_storage_error_file_system_helper() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let error = StorageError::file_system("/path/to/file", io_err);

        assert!(error.to_string().contains("/path/to/file"));
    }

    #[test]
    fn test_config_error_invalid_format_helper() {
        let error = ConfigError::invalid_format("WRONG");

        assert!(error.to_string().contains("WRONG"));
    }

    #[test]
    fn test_config_error_unknown_storage_type_helper() {
        let error = ConfigError::unknown_storage_type("MONGODB");

        assert!(error.to_string().contains("MONGODB"));
    }

    // ========================================================================
    // ERROR CHAIN TESTS
    // ========================================================================

    #[test]
    fn test_error_source_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let storage_err = StorageError::file_system("/test/path", io_err);
        let snippet_err: SnippetError = storage_err.into();

        // Verify error chain exists
        let error_str = format!("{:?}", snippet_err);
        assert!(error_str.contains("Storage"));
    }

    #[test]
    fn test_multiple_error_conversions() {
        // Test that all error types can convert properly
        let json_err = serde_json::from_str::<serde_json::Value>("bad").unwrap_err();
        let storage_err = StorageError::json("test", json_err);
        let _snippet_err: SnippetError = storage_err.into();

        let config_err = ConfigError::EmptyPath;
        let _snippet_err2: SnippetError = config_err.into();

        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let _snippet_err3: SnippetError = io_err.into();
    }
}