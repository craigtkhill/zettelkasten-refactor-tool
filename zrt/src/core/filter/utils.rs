use crate::core::patterns::Patterns;

/// Checks if a directory entry is hidden (starts with '.' except for temp directories)
#[inline]
#[must_use]
pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name().to_str().is_some_and(|s| {
        // Don't consider temp directories as hidden
        if s.starts_with(".tmp") {
            return false;
        }
        s.starts_with('.')
    })
}

/// Determines if a directory entry should be excluded from processing based on
/// multiple criteria including:
/// - Whether it's a hidden file/directory
/// - Whether it matches any of the explicitly excluded directories
/// - Whether it matches any patterns in the provided ignore patterns
///
/// # Arguments
/// * `entry` - The directory entry to check
/// * `exclude_dirs` - List of directory names to exclude
/// * `ignore_patterns` - Optional gitignore-style patterns to match against
///
/// # Returns
/// `true` if the entry should be excluded, `false` otherwise
pub fn should_exclude(
    entry: &walkdir::DirEntry,
    exclude_dirs: &[&str],
    ignore_patterns: Option<&Patterns>,
) -> bool {
    if is_hidden(entry) {
        return true;
    }

    if let Some(path_str) = entry.path().to_str() {
        for dir in exclude_dirs {
            if entry.file_type().is_dir() && entry.file_name().to_str() == Some(*dir) {
                return true;
            }
            if path_str.contains(&format!("/{dir}/")) {
                return true;
            }
        }
    }

    if let Some(patterns) = ignore_patterns {
        if patterns.matches(entry.path()) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::filter::test_utils::setup_test_directory;
    use anyhow::Result;
    use walkdir::WalkDir;

    #[test]
    fn test_is_hidden() -> Result<()> {
        use std::fs::File;
        use tempfile::TempDir;

        let temp_dir = TempDir::new()?;

        // Create test files
        File::create(temp_dir.path().join(".hidden"))?;
        File::create(temp_dir.path().join(".tmp_file"))?;
        File::create(temp_dir.path().join("normal.txt"))?;

        // Test each file using WalkDir
        let mut entries: Vec<_> = WalkDir::new(temp_dir.path())
            .into_iter()
            .filter_map(core::result::Result::ok)
            .collect();
        entries.sort_by_key(|e| e.path().to_path_buf());

        // Test hidden file
        let hidden = entries.iter().find(|e| e.file_name() == ".hidden").unwrap();
        assert!(is_hidden(hidden));

        // Test temp file
        let temp = entries
            .iter()
            .find(|e| e.file_name() == ".tmp_file")
            .unwrap();
        assert!(!is_hidden(temp));

        // Test normal file
        let normal = entries
            .iter()
            .find(|e| e.file_name() == "normal.txt")
            .unwrap();
        assert!(!is_hidden(normal));

        Ok(())
    }

    #[test]
    fn test_should_exclude() -> Result<()> {
        let dir = setup_test_directory()?;

        let hidden_entry = WalkDir::new(dir.path())
            .into_iter()
            .find(|e| {
                e.as_ref()
                    .map(|entry| entry.file_name() == ".hidden.md")
                    .unwrap_or(false)
            })
            .expect("Should find .hidden.md")?;

        assert!(
            should_exclude(&hidden_entry, &[], None),
            "Should exclude hidden files"
        );

        let nested_entry = WalkDir::new(dir.path())
            .into_iter()
            .find(|e| {
                e.as_ref()
                    .map(|entry| entry.file_name() == "nested")
                    .unwrap_or(false)
            })
            .expect("Should find nested directory")?;

        assert!(
            should_exclude(&nested_entry, &["nested"], None),
            "Should exclude specified directories"
        );

        Ok(())
    }
}
