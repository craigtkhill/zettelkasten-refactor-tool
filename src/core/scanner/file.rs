// src/core/scanner/file_counter.rs
use anyhow::Result;
use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::core::ignore::load_ignore_patterns;
use crate::core::scanner::utils::should_exclude;

/// Counts the total number of files in a directory and its subdirectories.
///
/// # Arguments
///
/// * `dir` - The directory path to scan
/// * `exclude_dirs` - A list of directory names to exclude from the count
///
/// # Returns
///
/// * `Ok(u64)` - The total number of files found
///
/// # Errors
///
/// This function may return an error if:
/// * The directory cannot be accessed or read
/// * File system operations fail during traversal
/// * The ignore patterns file cannot be parsed
#[inline]
pub fn count_files(dir: &PathBuf, exclude_dirs: &[&str]) -> Result<u64> {
    let absolute_dir = if dir.is_absolute() {
        dir.clone()
    } else {
        env::current_dir()?.join(dir)
    };

    let ignore_patterns = load_ignore_patterns(&absolute_dir)?;
    let mut count: u64 = 0;

    for entry in WalkDir::new(&absolute_dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, exclude_dirs, Some(&ignore_patterns)))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            count = count.saturating_add(1);
        }
    }

    println!("Total files found: {count}");
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::scanner::test_utils::setup_test_directory;
    use anyhow::Result;

    #[test]
    fn test_count_files() -> Result<()> {
        let dir = setup_test_directory()?;

        // Test basic file counting
        let count = count_files(&dir.path().to_path_buf(), &[])?;
        assert_eq!(count, 4, "Should count all non-hidden files"); // 4 visible files

        // Test with exclude dirs
        let count = count_files(&dir.path().to_path_buf(), &["nested"])?;
        assert_eq!(count, 3, "Should exclude files in 'nested' directory");

        Ok(())
    }
}
