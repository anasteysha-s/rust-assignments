#!/bin/bash

# Integration test script for snippets-app
# This demonstrates the expected behavior

set -e

echo "=== Snippets App Integration Tests ==="
echo ""

# Use a temporary storage file for testing
STORAGE_FILE="/tmp/test_snippets_$$.json"
APP="./target/release/snippets-app"

# Clean up function
cleanup() {
    rm -f "$STORAGE_FILE"
}
trap cleanup EXIT

echo "1. Creating a snippet from stdin..."
echo 'if let Some(local_time) = self.local_time else { }' | $APP --storage "$STORAGE_FILE" --name "Cool Rust pattern"
echo ""

echo "2. Creating another snippet..."
echo 'fn main() { println!("Hello, World!"); }' | $APP --storage "$STORAGE_FILE" --name "Hello World"
echo ""

echo "3. Listing all snippets..."
$APP --storage "$STORAGE_FILE" --list
echo ""

echo "4. Reading a snippet..."
echo "Content of 'Cool Rust pattern':"
$APP --storage "$STORAGE_FILE" --read "Cool Rust pattern"
echo ""
echo ""

echo "5. Reading another snippet..."
echo "Content of 'Hello World':"
$APP --storage "$STORAGE_FILE" --read "Hello World"
echo ""
echo ""

echo "6. Deleting a snippet..."
$APP --storage "$STORAGE_FILE" --delete "Hello World"
echo ""

echo "7. Listing snippets after deletion..."
$APP --storage "$STORAGE_FILE" --list
echo ""

echo "8. Trying to read deleted snippet (should fail)..."
if $APP --storage "$STORAGE_FILE" --read "Hello World" 2>/dev/null; then
    echo "ERROR: Should have failed!"
    exit 1
else
    echo "Correctly failed to read deleted snippet"
fi
echo ""

echo "=== All tests passed! ==="