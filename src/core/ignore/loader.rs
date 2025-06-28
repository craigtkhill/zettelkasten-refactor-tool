// src/core/ignore/loader.rs
use crate::core::ignore::Patterns;
use anyhow::{Context as _, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Loads ignore patterns from .zrtignore files starting from the given directory
/// and recursively checking parent directories until a file is found.
///
/// # Arguments
///
/// * `dir` - The starting directory to search for .zrtignore files
///
/// # Returns
///
/// * `Ok(Patterns)` containing the loaded patterns
///
/// # Errors
///
/// This function may return an error if:
/// * The .zrtignore file exists but cannot be read
/// * The file contains invalid pattern syntax
/// * File system operations fail during the search
#[inline]
pub fn load_ignore_patterns(dir: &Path) -> Result<Patterns> {
    let mut patterns = Patterns::new(PathBuf::new());

    let mut current_dir = dir.to_path_buf();

    let mut visited = HashSet::new();

    while !visited.contains(&current_dir) {
        visited.insert(current_dir.clone());

        let ignore_file = current_dir.join(".zrtignore");

        if ignore_file.exists() {
            let content = fs::read_to_string(&ignore_file).with_context(|| {
                format!("Failed to read .zrtignore file: {}", ignore_file.display())
            })?;

            for line in content.lines() {
                patterns.add_pattern(line)?;
            }

            break;
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    Ok(patterns)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_relative_path_matching() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;

        // Create a .zrtignore file with a specific pattern
        let ignore_file = temp_dir.path().join(".zrtignore");
        std::fs::write(&ignore_file, "ignore_me.tmp\n")?;

        // Load patterns
        let patterns = load_ignore_patterns(temp_dir.path())?;

        // Test with relative path
        let relative_path = PathBuf::from("ignore_me.tmp");

        assert!(
            patterns.matches(&relative_path),
            "Should match relative path"
        );

        Ok(())
    }

    #[test]
    fn test_load_ignore_patterns() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let ignore_file = temp_dir.path().join(".zrtignore");
        std::fs::write(
            &ignore_file,
            "*.txt\n!important.txt\n# comment\n\n/src/generated/*.rs",
        )?;

        let patterns = load_ignore_patterns(temp_dir.path())?;
        assert!(patterns.matches("file.txt"));
        assert!(!patterns.matches("important.txt"));
        assert!(patterns.matches("src/generated/test.rs"));
        assert!(!patterns.matches("src/main.rs"));
        Ok(())
    }

    #[test]
    fn test_todo_chores_ignore() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;

        let ignore_file = temp_dir.path().join(".zrtignore");
        std::fs::write(
            &ignore_file,
            "ARCHIVE/\nCALENDAR/\nDRAWINGS/\nIMAGES/\n.git/\nTODO-CHORES.md\n",
        )?;

        let todo_file = temp_dir.path().join("TODO-CHORES.md");
        std::fs::write(&todo_file, "Test content")?;

        let other_file = temp_dir.path().join("OTHER-FILE.md");
        std::fs::write(&other_file, "Other content")?;

        let patterns = load_ignore_patterns(temp_dir.path())?;

        assert!(
            patterns.matches(&todo_file),
            "TODO-CHORES.md should match the ignore pattern"
        );

        assert!(
            !patterns.matches(&other_file),
            "OTHER-FILE.md should not match any ignore pattern"
        );

        Ok(())
    }
}
