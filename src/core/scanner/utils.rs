// src/core/scanner/utils.rs
use crate::core::ignore::Patterns;
use crate::utils::is_hidden;

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
    use crate::core::scanner::test_utils::setup_test_directory;
    use anyhow::Result;
    use walkdir::WalkDir;

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
