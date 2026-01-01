// Additional tests for the snippets app

#[cfg(test)]
mod integration_tests {
    use std::fs;
    use std::process::Command;
    use tempfile::NamedTempFile;

    /// Helper function to get the path to the compiled binary
    fn get_binary_path() -> String {
        // In a real scenario, this would point to target/debug/snippets-app
        // For testing purposes, we'll use a placeholder
        "target/debug/snippets-app".to_string()
    }

    #[test]
    #[ignore] // Ignore by default since it requires compiled binary
    fn test_create_and_read_snippet() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage_path = temp_file.path().to_str().unwrap();
        fs::remove_file(storage_path).unwrap();

        let binary = get_binary_path();

        // Create a snippet
        let output = Command::new(&binary)
            .arg("--storage")
            .arg(storage_path)
            .arg("--name")
            .arg("test snippet")
            .stdin(std::process::Stdio::piped())
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());

        // Read the snippet
        let output = Command::new(&binary)
            .arg("--storage")
            .arg(storage_path)
            .arg("--read")
            .arg("test snippet")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
    }

    #[test]
    #[ignore] // Ignore by default since it requires compiled binary
    fn test_delete_snippet() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage_path = temp_file.path().to_str().unwrap();
        fs::remove_file(storage_path).unwrap();

        let binary = get_binary_path();

        // Create a snippet
        Command::new(&binary)
            .arg("--storage")
            .arg(storage_path)
            .arg("--name")
            .arg("to_delete")
            .stdin(std::process::Stdio::piped())
            .output()
            .expect("Failed to execute command");

        // Delete the snippet
        let output = Command::new(&binary)
            .arg("--storage")
            .arg(storage_path)
            .arg("--delete")
            .arg("to_delete")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());

        // Try to read the deleted snippet (should fail)
        let output = Command::new(&binary)
            .arg("--storage")
            .arg(storage_path)
            .arg("--read")
            .arg("to_delete")
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
    }
}