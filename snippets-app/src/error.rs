// error.rs - Custom error types using thiserror

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for the snippets application
#[derive(Error, Debug)]
pub enum SnippetError {
    /// Error when snippet is not found
    #[error("Snippet '{name}' not found")]
    NotFound { name: String },

    /// Error when snippet already exists
    #[error("Snippet '{name}' already exists")]
    AlreadyExists { name: String },

    /// Error when snippet name is invalid
    #[error("Invalid snippet name: {name}. Reason: {reason}")]
    InvalidName { name: String, reason: String },

    /// Storage-related errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// IO errors with context
    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

/// Storage-specific errors
#[derive(Error, Debug)]
pub enum StorageError {
    /// JSON serialization/deserialization errors
    #[error("JSON error in {context}: {source}")]
    Json {
        context: String,
        source: serde_json::Error,
    },

    /// SQLite database errors
    #[error("Database error in {operation}: {source}")]
    Database {
        operation: String,
        source: rusqlite::Error,
    },

    /// File system errors with path context
    #[error("File system error for '{path}': {source}")]
    FileSystem {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Storage initialization errors
    #[error("Failed to initialize {storage_type} storage at '{path}': {reason}")]
    Initialization {
        storage_type: String,
        path: PathBuf,
        reason: String,
    },

    /// Data corruption errors
    #[error("Data corruption detected in {location}: {details}")]
    Corruption { location: String, details: String },
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Invalid storage configuration format
    #[error("Invalid storage configuration: '{config}'. Expected format: 'TYPE:/path' (e.g., 'JSON:/path/to/file.json' or 'SQLITE:/path/to/file.db')")]
    InvalidFormat { config: String },

    /// Unknown storage type
    #[error("Unknown storage type: '{storage_type}'. Supported types: JSON, SQLITE")]
    UnknownStorageType { storage_type: String },

    /// Empty or missing path
    #[error("Storage path cannot be empty")]
    EmptyPath,

    /// Invalid path
    #[error("Invalid storage path: '{path}'. Reason: {reason}")]
    InvalidPath { path: String, reason: String },

    /// Missing environment variable
    #[error("Environment variable '{var_name}' not set")]
    MissingEnvVar { var_name: String },
}

/// Type alias for Result with SnippetError
pub type Result<T> = std::result::Result<T, SnippetError>;

/// Type alias for storage-specific results
pub type StorageResult<T> = std::result::Result<T, StorageError>;

impl StorageError {
    /// Create a JSON error with context
    pub fn json(context: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Json {
            context: context.into(),
            source,
        }
    }

    /// Create a database error with operation context
    pub fn database(operation: impl Into<String>, source: rusqlite::Error) -> Self {
        Self::Database {
            operation: operation.into(),
            source,
        }
    }

    /// Create a file system error with path context
    pub fn file_system(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::FileSystem {
            path: path.into(),
            source,
        }
    }

    /// Create an initialization error
    pub fn initialization(
        storage_type: impl Into<String>,
        path: impl Into<PathBuf>,
        reason: impl Into<String>,
    ) -> Self {
        Self::Initialization {
            storage_type: storage_type.into(),
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Create a corruption error
    pub fn corruption(location: impl Into<String>, details: impl Into<String>) -> Self {
        Self::Corruption {
            location: location.into(),
            details: details.into(),
        }
    }
}

impl ConfigError {
    /// Create an invalid format error
    pub fn invalid_format(config: impl Into<String>) -> Self {
        Self::InvalidFormat {
            config: config.into(),
        }
    }

    /// Create an unknown storage type error
    pub fn unknown_storage_type(storage_type: impl Into<String>) -> Self {
        Self::UnknownStorageType {
            storage_type: storage_type.into(),
        }
    }

    /// Create an invalid path error
    pub fn invalid_path(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidPath {
            path: path.into(),
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_not_found_error() {
        let err = SnippetError::NotFound {
            name: "test".to_string(),
        };
        assert_eq!(err.to_string(), "Snippet 'test' not found");
    }

    #[test]
    fn test_storage_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid")
            .unwrap_err();
        let err = StorageError::json("deserializing snippet", json_err);
        assert!(err.to_string().contains("JSON error"));
        assert!(err.to_string().contains("deserializing snippet"));
    }

    #[test]
    fn test_config_invalid_format_error() {
        let err = ConfigError::invalid_format("BADFORMAT");
        assert!(err.to_string().contains("Invalid storage configuration"));
        assert!(err.to_string().contains("BADFORMAT"));
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let snippet_err: SnippetError = io_err.into();
        assert!(snippet_err.to_string().contains("IO error"));
    }
}