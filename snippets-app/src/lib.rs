// src/lib.rs
// Library interface for snippets-app to enable testing

pub mod error;
pub mod storage;

// Re-export commonly used types
pub use error::{ConfigError, Result, SnippetError, StorageError};
pub use storage::{
    create_storage, parse_storage_config, JsonStorage, Snippet, SnippetStorage, SqliteStorage,
    StorageType,
};