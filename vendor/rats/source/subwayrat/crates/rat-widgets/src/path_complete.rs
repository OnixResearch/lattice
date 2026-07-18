//! Path completion utilities for filesystem tab completion.

use std::fs;
use std::path::Path;

/// Filesystem tab completion function.
///
/// Returns a list of matching filesystem paths based on the input:
/// - If input ends with '/', completes entries in that directory
/// - Otherwise, completes entries in parent directory that start with the filename prefix
/// - Directory entries get '/' appended
/// - Empty input completes from current directory
/// - Non-existent parent directory returns empty vec
pub fn path_completer(input: &str) -> Vec<String> {
    if input.is_empty() {
        return complete_directory(".", "");
    }

    let (dir, prefix) = if input.ends_with('/') {
        // Input like "/tmp/" - complete all entries in that directory
        (input, "")
    } else {
        // Input like "/tmp/fo" - complete entries in /tmp/ that start with "fo"
        let path = Path::new(input);
        match path.parent() {
            Some(parent) => {
                let parent_str = if parent == Path::new("") {
                    "."
                } else {
                    parent.to_str().unwrap_or(".")
                };
                let prefix = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                (parent_str, prefix)
            }
            None => (".", input),
        }
    };

    complete_directory(dir, prefix)
}

fn complete_directory(dir_str: &str, prefix: &str) -> Vec<String> {
    let dir_path = Path::new(dir_str);

    let Ok(entries) = fs::read_dir(dir_path) else {
        return Vec::new();
    };

    let mut matches = Vec::new();
    let prefix_lower = prefix.to_lowercase();

    for entry in entries {
        let Ok(entry) = entry else { continue };
        let file_name = entry.file_name();
        let Some(name_str) = file_name.to_str() else {
            continue;
        };

        // Case-insensitive prefix matching
        if name_str.to_lowercase().starts_with(&prefix_lower) {
            let mut result = if dir_str == "." {
                name_str.to_string()
            } else if dir_str.ends_with('/') {
                format!("{}{}", dir_str, name_str)
            } else {
                format!("{}/{}", dir_str, name_str)
            };

            // Append '/' to directories
            if entry.file_type().is_ok_and(|ft| ft.is_dir()) {
                result.push('/');
            }

            matches.push(result);
        }
    }

    matches.sort();
    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn empty_input_completes_current_dir() {
        let _results = path_completer("");
        // Should return some entries from current directory
        // We can't predict exactly what, but it shouldn't be empty in a real filesystem
        // For the test, we just check it doesn't panic
    }

    #[test]
    fn nonexistent_dir_returns_empty() {
        let results = path_completer("/this/path/definitely/does/not/exist/");
        assert!(results.is_empty());
    }

    #[test]
    fn nonexistent_parent_returns_empty() {
        let results = path_completer("/this/path/does/not/exist/foo");
        assert!(results.is_empty());
    }

    #[test]
    fn directory_completion() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create test files and directories
        fs::write(temp_path.join("file1.txt"), "").unwrap();
        fs::write(temp_path.join("file2.log"), "").unwrap();
        fs::create_dir(temp_path.join("subdir1")).unwrap();
        fs::create_dir(temp_path.join("subdir2")).unwrap();

        let input = format!("{}/", temp_path.display());
        let results = path_completer(&input);

        assert!(results.len() >= 4);

        // Check that directories have trailing slashes
        let subdir1_path = format!("{}/subdir1/", temp_path.display());
        let subdir2_path = format!("{}/subdir2/", temp_path.display());
        assert!(results.contains(&subdir1_path));
        assert!(results.contains(&subdir2_path));

        // Check that files don't have trailing slashes
        let file1_path = format!("{}/file1.txt", temp_path.display());
        let file2_path = format!("{}/file2.log", temp_path.display());
        assert!(results.contains(&file1_path));
        assert!(results.contains(&file2_path));
    }

    #[test]
    fn prefix_filtering() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create test files with different prefixes
        fs::write(temp_path.join("apple.txt"), "").unwrap();
        fs::write(temp_path.join("application.log"), "").unwrap();
        fs::write(temp_path.join("banana.txt"), "").unwrap();
        fs::create_dir(temp_path.join("apps")).unwrap();

        let input = format!("{}/app", temp_path.display());
        let results = path_completer(&input);

        // Should match "apple.txt", "application.log", and "apps/"
        assert_eq!(results.len(), 3);

        let expected_apple = format!("{}/apple.txt", temp_path.display());
        let expected_app = format!("{}/application.log", temp_path.display());
        let expected_apps = format!("{}/apps/", temp_path.display());

        assert!(results.contains(&expected_apple));
        assert!(results.contains(&expected_app));
        assert!(results.contains(&expected_apps));

        // Should not match "banana.txt"
        let not_expected = format!("{}/banana.txt", temp_path.display());
        assert!(!results.contains(&not_expected));
    }

    #[test]
    fn case_insensitive_matching() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create files with mixed case
        fs::write(temp_path.join("FileA.txt"), "").unwrap();
        fs::write(temp_path.join("fileb.txt"), "").unwrap();
        fs::create_dir(temp_path.join("FolderC")).unwrap();

        let input = format!("{}/fil", temp_path.display());
        let results = path_completer(&input);

        // Should match "FileA.txt" and "fileb.txt" (case-insensitive "fil" prefix)
        // "FolderC" does NOT match "fil"
        assert_eq!(results.len(), 2);

        let expected_a = format!("{}/FileA.txt", temp_path.display());
        let expected_b = format!("{}/fileb.txt", temp_path.display());

        assert!(results.contains(&expected_a));
        assert!(results.contains(&expected_b));
    }

    #[test]
    fn current_directory_shorthand() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create a test file
        fs::write(temp_path.join("test.txt"), "").unwrap();

        // Change to temp directory and test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_path).unwrap();

        let results = path_completer("test");

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(results.contains(&"test.txt".to_string()));
    }

    #[test]
    fn root_directory_completion() {
        // Test completing from root (might not work on all systems, so make it conditional)
        if cfg!(unix) {
            let results = path_completer("/");
            // Should contain typical Unix directories
            // We can't guarantee specific contents, but it shouldn't be empty
            // and entries should start with "/"
            for result in &results {
                assert!(result.starts_with("/"));
                if result.len() > 1 {
                    // Directories should end with /
                    assert!(result.ends_with("/") || !result[1..].contains("/"));
                }
            }
        }
    }
}
