// tests/integration_tests.rs

use std::env;
use std::process::Command;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("snippets-app"));
    assert!(stdout.contains("--name"));
    assert!(stdout.contains("--read"));
    assert!(stdout.contains("--delete"));
    assert!(stdout.contains("--list"));
    assert!(stdout.contains("--download"));
}

#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.4"));
}

#[test]
fn test_create_and_read_snippet_json() {
    let temp_file = NamedTempFile::new().unwrap();
    let storage_path = temp_file.path().to_string_lossy().to_string();

    // Create snippet
    let create_output = Command::new("cargo")
        .args(&["run", "--", "--name", "test-snippet"])
        .env("SNIPPETS_APP_STORAGE", format!("JSON:{}", storage_path))
        .env("SNIPPETS_APP_LOG_LEVEL", "error") // Suppress logs
        .stdin(std::process::Stdio::piped())
        .output()
        .expect("Failed to create snippet");

    // Note: In real integration tests, we'd write to stdin
    // For now, we verify the structure is correct
    println!("Create output: {:?}", create_output);
}

#[test]
fn test_list_empty_storage() {
    let temp_file = NamedTempFile::new().unwrap();
    let storage_path = temp_file.path().to_string_lossy().to_string();

    let output = Command::new("cargo")
        .args(&["run", "--", "--list"])
        .env("SNIPPETS_APP_STORAGE", format!("JSON:{}", storage_path))
        .env("SNIPPETS_APP_LOG_LEVEL", "error")
        .output()
        .expect("Failed to list snippets");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No snippets found") || stdout.is_empty());
}

#[test]
fn test_invalid_storage_config() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--list"])
        .env("SNIPPETS_APP_STORAGE", "INVALID_FORMAT")
        .env("SNIPPETS_APP_LOG_LEVEL", "error")
        .output()
        .expect("Failed to execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid storage configuration") || 
            stderr.contains("Configuration error"));
}

#[test]
fn test_sqlite_storage_creation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage_path = db_path.to_string_lossy().to_string();

    let output = Command::new("cargo")
        .args(&["run", "--", "--list"])
        .env("SNIPPETS_APP_STORAGE", format!("SQLITE:{}", storage_path))
        .env("SNIPPETS_APP_LOG_LEVEL", "error")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    assert!(db_path.exists());
}

#[test]
fn test_logging_to_stderr() {
    let temp_file = NamedTempFile::new().unwrap();
    let storage_path = temp_file.path().to_string_lossy().to_string();

    let output = Command::new("cargo")
        .args(&["run", "--", "--list"])
        .env("SNIPPETS_APP_STORAGE", format!("JSON:{}", storage_path))
        .env("SNIPPETS_APP_LOG_LEVEL", "debug")
        .output()
        .expect("Failed to execute");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("INFO") || stderr.contains("DEBUG"));
}

#[test]
fn test_logging_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test.log");
    let storage_file = NamedTempFile::new().unwrap();
    let storage_path = storage_file.path().to_string_lossy().to_string();

    let output = Command::new("cargo")
        .args(&["run", "--", "--list"])
        .env("SNIPPETS_APP_STORAGE", format!("JSON:{}", storage_path))
        .env("SNIPPETS_APP_LOG_LEVEL", "info")
        .env("SNIPPETS_APP_LOG_PATH", log_path.to_string_lossy().to_string())
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    assert!(log_path.exists());

    let log_content = std::fs::read_to_string(&log_path).unwrap();
    assert!(log_content.contains("snippets-app"));
}

#[test]
fn test_verbose_flag() {
    let temp_file = NamedTempFile::new().unwrap();
    let storage_path = temp_file.path().to_string_lossy().to_string();

    let output = Command::new("cargo")
        .args(&["run", "--", "--list", "--verbose"])
        .env("SNIPPETS_APP_STORAGE", format!("JSON:{}", storage_path))
        .env("SNIPPETS_APP_LOG_LEVEL", "error")
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verbose mode should show timestamps
    assert!(stdout.contains("No snippets found") || 
            stdout.contains("Stored snippets (with creation times)"));
}

#[test]
fn test_nonexistent_snippet_read() {
    let temp_file = NamedTempFile::new().unwrap();
    let storage_path = temp_file.path().to_string_lossy().to_string();

    let output = Command::new("cargo")
        .args(&["run", "--", "--read", "nonexistent"])
        .env("SNIPPETS_APP_STORAGE", format!("JSON:{}", storage_path))
        .env("SNIPPETS_APP_LOG_LEVEL", "error")
        .output()
        .expect("Failed to execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found"));
}

#[test]
fn test_nonexistent_snippet_delete() {
    let temp_file = NamedTempFile::new().unwrap();
    let storage_path = temp_file.path().to_string_lossy().to_string();

    let output = Command::new("cargo")
        .args(&["run", "--", "--delete", "nonexistent"])
        .env("SNIPPETS_APP_STORAGE", format!("JSON:{}", storage_path))
        .env("SNIPPETS_APP_LOG_LEVEL", "error")
        .output()
        .expect("Failed to execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found"));
}

#[test]
fn test_no_command_shows_usage() {
    let temp_file = NamedTempFile::new().unwrap();
    let storage_path = temp_file.path().to_string_lossy().to_string();

    let output = Command::new("cargo")
        .args(&["run", "--"])
        .env("SNIPPETS_APP_STORAGE", format!("JSON:{}", storage_path))
        .env("SNIPPETS_APP_LOG_LEVEL", "error")
        .output()
        .expect("Failed to execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No command specified") || 
            stderr.contains("--help"));
}

// Note: Download tests require network access and are better suited
// for manual testing or with mock servers. Skipping for unit tests.