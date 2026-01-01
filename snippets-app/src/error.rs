//! Error types for the snippets application.
//!
//! This module provides structured error handling using `thiserror` for custom error types.
//! Errors are organized hierarchically:
//!
//! - [`SnippetError`] - Top-level application errors
//! - [`StorageError`] - Storage-specific errors
//! - [`ConfigError`] - Configuration parsing errors

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for the snippets application.
///
/// This is the top-level error type that encompasses all possible errors
/// that can occur in the application. Other error types automatically
/// convert into this type via the `#[from]` attribute.
///
/// # Examples
///
/// ```
/// use snippets_app::SnippetError;
///
/// let error = SnippetError::InvalidName {
///     name: "".to_string(),
///     reason: "Name cannot be empty".to_string(),
/// };
/// assert!(error.to_string().contains("Invalid snippet name"));
/// ```
#[derive(Error, Debug)]
pub enum SnippetError {
    /// Error when snippet is not found.
    ///
    /// This occurs when trying to read or delete a snippet that doesn't exist.
    #[allow(dead_code)]
    #[error("Snippet '{name}' not found")]
    NotFound {
        /// The name of the snippet that was not found
        name: String,
    },

    /// Error when snippet already exists.
    ///
    /// This could be used in future when enforcing uniqueness constraints.
    #[allow(dead_code)]
    #[error("Snippet '{name}' already exists")]
    AlreadyExists {
        /// The name of the duplicate snippet
        name: String,
    },

    /// Error when snippet name is invalid.
    ///
    /// Validation rules:
    /// - Name cannot be empty
    /// - Name cannot exceed 255 characters
    /// - Name cannot contain null bytes
    #[error("Invalid snippet name: {name}. Reason: {reason}")]
    InvalidName {
        /// The invalid name that was provided
        name: String,
        /// Explanation of why the name is invalid
        reason: String,
    },

    /// Storage-related errors.
    ///
    /// Wraps all errors from the storage layer.
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Configuration errors.
    ///
    /// Wraps all configuration parsing and validation errors.
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// IO errors with context.
    ///
    /// Wraps standard IO errors with additional context.
    #[error("IO error: {source}")]
    Io {
        /// The underlying IO error
        #[from]
        source: std::io::Error,
    },
}

/// Storage-specific errors.
///
/// These errors occur during storage operations like reading,
/// writing, or initializing storage backends.
#[derive(Error, Debug)]
pub enum StorageError {
    /// JSON serialization/deserialization errors.
    ///
    /// Occurs when parsing or generating JSON data.
    #[error("JSON error in {context}: {source}")]
    Json {
        /// Description of the operation that failed
        context: String,
        /// The underlying serde_json error
        source: serde_json::Error,
    },

    /// SQLite database errors.
    ///
    /// Occurs during database operations.
    #[error("Database error in {operation}: {source}")]
    Database {
        /// Description of the database operation
        operation: String,
        /// The underlying rusqlite error
        source: rusqlite::Error,
    },

    /// File system errors with path context.
    ///
    /// Occurs when reading or writing files.
    #[error("File system error for '{path}': {source}")]
    FileSystem {
        /// The file path that caused the error
        path: PathBuf,
        /// The underlying IO error
        source: std::io::Error,
    },

    /// Storage initialization errors.
    ///
    /// Occurs when setting up a storage backend fails.
    #[error("Failed to initialize {storage_type} storage at '{path}': {reason}")]
    Initialization {
        /// The type of storage (JSON, SQLite)
        storage_type: String,
        /// The path where initialization failed
        path: PathBuf,
        /// Description of why initialization failed
        reason: String,
    },

    /// Data corruption errors.
    ///
    /// Occurs when data validation fails.
    #[allow(dead_code)]
    #[error("Data corruption detected in {location}: {details}")]
    Corruption {
        /// Where the corruption was detected
        location: String,
        /// Details about the corruption
        details: String,
    },
}

/// Configuration-related errors.
///
/// These errors occur when parsing or validating configuration
/// from environment variables or command-line arguments.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Invalid storage configuration format.
    ///
    /// Expected format: `TYPE:/path` (e.g., `JSON:/path/to/file.json`)
    #[error("Invalid storage configuration: '{config}'. Expected format: 'TYPE:/path' (e.g., 'JSON:/path/to/file.json' or 'SQLITE:/path/to/file.db')")]
    InvalidFormat {
        /// The invalid configuration string
        config: String,
    },

    /// Unknown storage type.
    ///
    /// Supported types are JSON and SQLITE.
    #[error("Unknown storage type: '{storage_type}'. Supported types: JSON, SQLITE")]
    UnknownStorageType {
        /// The unknown storage type that was provided
        storage_type: String,
    },

    /// Empty or missing path.
    #[error("Storage path cannot be empty")]
    EmptyPath,

    /// Invalid path.
    #[allow(dead_code)]
    #[error("Invalid storage path: '{path}'. Reason: {reason}")]
    InvalidPath {
        /// The invalid path
        path: String,
        /// Why the path is invalid
        reason: String,
    },

    /// Missing environment variable.
    #[allow(dead_code)]
    #[error("Environment variable '{var_name}' not set")]
    MissingEnvVar {
        /// The name of the missing environment variable
        var_name: String,
    },
}

/// Type alias for Result with [`SnippetError`].
///
/// This is the standard Result type used throughout the application.
pub type Result<T> = std::result::Result<T, SnippetError>;

/// Type alias for storage-specific results.
///
/// Used internally in the storage module.
pub type StorageResult<T> = std::result::Result<T, StorageError>;

impl StorageError {
    /// Create a JSON error with context.
    ///
    /// # Examples
    ///
    /// ```
    /// use snippets_app::StorageError;
    ///
    /// let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
    /// let error = StorageError::json("deserializing snippet", json_err);
    /// assert!(error.to_string().contains("deserializing snippet"));
    /// ```
    pub fn json(context: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Json {
            context: context.into(),
            source,
        }
    }

    /// Create a database error with operation context.
    ///
    /// # Examples
    ///
    /// ```
    /// use snippets_app::StorageError;
    ///
    /// let db_err = rusqlite::Error::InvalidQuery;
    /// let error = StorageError::database("inserting snippet", db_err);
    /// assert!(error.to_string().contains("inserting snippet"));
    /// ```
    pub fn database(operation: impl Into<String>, source: rusqlite::Error) -> Self {
        Self::Database {
            operation: operation.into(),
            source,
        }
    }

    /// Create a file system error with path context.
    ///
    /// # Examples
    ///
    /// ```
    /// use snippets_app::StorageError;
    /// use std::io;
    /// use std::path::PathBuf;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
    /// let error = StorageError::file_system("/tmp/file.json", io_err);
    /// assert!(error.to_string().contains("/tmp/file.json"));
    /// ```
    pub fn file_system(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::FileSystem {
            path: path.into(),
            source,
        }
    }

    /// Create an initialization error.
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

    /// Create a corruption error.
    #[allow(dead_code)]
    pub fn corruption(location: impl Into<String>, details: impl Into<String>) -> Self {
        Self::Corruption {
            location: location.into(),
            details: details.into(),
        }
    }
}

impl ConfigError {
    /// Create an invalid format error.
    pub fn invalid_format(config: impl Into<String>) -> Self {
        Self::InvalidFormat {
            config: config.into(),
        }
    }

    /// Create an unknown storage type error.
    pub fn unknown_storage_type(storage_type: impl Into<String>) -> Self {
        Self::UnknownStorageType {
            storage_type: storage_type.into(),
        }
    }

    /// Create an invalid path error.
    #[allow(dead_code)]
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
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
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
