//! # Snippets App
//!
//! A CLI tool for managing code snippets with timestamps, logging, and download support.
//!
//! ## Features
//!
//! - **Storage**: JSON and SQLite backends
//! - **Logging**: Structured logging with tracing
//! - **Download**: Fetch snippets from URLs
//!
//! ## Quick Start
//!
//! ```no_run
//! use snippets_app::{Snippet, JsonStorage, SnippetStorage};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut storage = JsonStorage::new("snippets.json")?;
//! let snippet = Snippet::new("hello".to_string(), "code".to_string())?;
//! storage.save_snippet(snippet)?;
//! # Ok(())
//! # }
//! ```

// ============================================================================
// LINT CONFIGURATION - Add these lines
// ============================================================================

#![warn(missing_docs)]
#![warn(broken_intra_doc_links)]
#![warn(missing_crate_level_docs)]
#![warn(unreachable_pub)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::similar_names)]

// ============================================================================
// LIBRARY CODE
// ============================================================================

/// Error types for the snippets application.
pub mod error;

/// Storage implementations for snippet persistence.
pub mod storage;

pub use error::{ConfigError, Result, SnippetError, StorageError};
pub use storage::{
    create_storage, parse_storage_config, JsonStorage, Snippet, SnippetStorage, SqliteStorage,
    StorageType,
};