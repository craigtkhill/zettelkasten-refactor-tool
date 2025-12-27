// src/core/scanner/pattern.rs
use anyhow::Result;
use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::core::ignore::load_ignore_patterns;
use crate::core::scanner::utils::should_exclude;
use crate::models::SinglePatternStats;
use crate::utils::has_only_tag;

/// Scans one or more directories for files containing only a specific tag and no other tags.
///
/// # Arguments
///
/// * `dirs` - The directory paths to scan (empty slice defaults to current directory)
/// * `tag` - The tag to search for as the only tag in files
///
/// # Returns
///
/// * `Ok(SinglePatternStats)` - Statistics about files with only the specified tag
///
/// # Errors
///
/// This function may return an error if:
/// * The directory cannot be accessed or read
/// * File system operations fail during traversal
/// * Files cannot be read as UTF-8 text
/// * The ignore patterns file cannot be parsed
/// * Tag detection encounters an error
#[inline]
pub fn scan_directory_only_tag(dirs: &[PathBuf], tag: &str) -> Result<SinglePatternStats> {
    let directories: Vec<PathBuf> = if dirs.is_empty() {
        vec![env::current_dir()?]
    } else {
        dirs.to_vec()
    };

    let mut combined_stats = SinglePatternStats::new();
    let mut all_matching_files = Vec::new();

    for dir in directories {
        let absolute_dir = if dir.is_absolute() {
            dir.clone()
        } else {
            env::current_dir()?.join(dir)
        };

        let ignore_patterns = load_ignore_patterns(&absolute_dir)?;

        for entry in WalkDir::new(&absolute_dir)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !should_exclude(e, &[], Some(&ignore_patterns)))
        {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            combined_stats.total_files = combined_stats.total_files.saturating_add(1);

            let path = entry.path();
            if let Ok(true) = has_only_tag(path, tag) {
                combined_stats.files_with_pattern = combined_stats.files_with_pattern.saturating_add(1);
                all_matching_files.push(path.to_path_buf());
            }
        }
    }

    // Print the files
    for file in all_matching_files {
        println!("{}", file.display());
    }

    Ok(combined_stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::scanner::test_utils::{create_test_file, setup_test_directory};
    use anyhow::Result;

    #[test]
    fn test_scan_directory_only_tag() -> Result<()> {
        let dir = setup_test_directory()?;

        // Add files with various tag combinations
        create_test_file(
            &dir,
            "only_refactored.md",
            "---\ntags: [refactored]\n---\nContent",
        )?;
        create_test_file(
            &dir,
            "refactored_plus.md",
            "---\ntags: [refactored, reviewed]\n---\nContent",
        )?;
        create_test_file(&dir, "only_other.md", "---\ntags: [draft]\n---\nContent")?;
        create_test_file(&dir, "no_tags.md", "Just content")?;

        let stats = scan_directory_only_tag(&[dir.path().to_path_buf()], "refactored")?;

        assert_eq!(stats.total_files, 8, "Should count all non-hidden files"); // 4 original + 4 new
        assert_eq!(
            stats.files_with_pattern, 1,
            "Should find 1 file with only 'refactored' tag"
        );

        Ok(())
    }

    #[test]
    fn test_scan_directory_only_tag_with_non_utf8_files() -> Result<()> {
        let dir = setup_test_directory()?;

        // Add files with various tag combinations
        create_test_file(
            &dir,
            "only_refactored.md",
            "---\ntags: [refactored]\n---\nContent",
        )?;

        // Create a binary file with invalid UTF-8 bytes
        let binary_path = dir.path().join("binary.md");
        std::fs::write(&binary_path, &[0xFF, 0xFE, 0x00, 0x48, 0x65, 0x6C, 0x6C, 0x6F])?;

        // This should not panic and should skip the non-UTF-8 file
        let stats = scan_directory_only_tag(&[dir.path().to_path_buf()], "refactored")?;

        assert_eq!(stats.total_files, 6, "Should count all files including non-UTF-8"); // 4 original + 2 new
        assert_eq!(
            stats.files_with_pattern, 1,
            "Should find 1 file with only 'refactored' tag, skipping non-UTF-8"
        );

        Ok(())
    }

    // Multi-directory tests for scan_directory_only_tag
    #[test]
    fn test_only_tag_should_include_files_from_all_directories() -> Result<()> {
        let dir1 = setup_test_directory()?;
        let dir2 = setup_test_directory()?;

        create_test_file(
            &dir1,
            "only_refactored1.md",
            "---\ntags: [refactored]\n---\nContent 1",
        )?;
        create_test_file(
            &dir2,
            "only_refactored2.md",
            "---\ntags: [refactored]\n---\nContent 2",
        )?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let stats = scan_directory_only_tag(&dirs, "refactored")?;

        assert_eq!(
            stats.files_with_pattern, 2,
            "Should find files with only 'refactored' tag from both directories"
        );

        Ok(())
    }

    #[test]
    fn test_only_tag_should_count_total_files_from_all_directories() -> Result<()> {
        let dir1 = setup_test_directory()?;
        let dir2 = setup_test_directory()?;

        create_test_file(
            &dir1,
            "extra1.md",
            "---\ntags: [refactored]\n---\nExtra 1",
        )?;
        create_test_file(
            &dir2,
            "extra2.md",
            "---\ntags: [refactored]\n---\nExtra 2",
        )?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let stats = scan_directory_only_tag(&dirs, "refactored")?;

        assert_eq!(
            stats.total_files, 10,
            "Should count all files from both directories"
        );

        Ok(())
    }

    #[test]
    fn test_only_tag_should_exclude_files_with_multiple_tags() -> Result<()> {
        let dir1 = setup_test_directory()?;
        let dir2 = setup_test_directory()?;

        create_test_file(
            &dir1,
            "only_one.md",
            "---\ntags: [refactored]\n---\nOnly refactored",
        )?;
        create_test_file(
            &dir2,
            "multiple.md",
            "---\ntags: [refactored, reviewed]\n---\nMultiple tags",
        )?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let stats = scan_directory_only_tag(&dirs, "refactored")?;

        assert_eq!(
            stats.files_with_pattern, 1,
            "Should only count files with exactly one tag"
        );

        Ok(())
    }

    #[test]
    fn test_only_tag_should_default_to_current_directory_when_empty() -> Result<()> {
        let dir = setup_test_directory()?;

        create_test_file(
            &dir,
            "only_refactored.md",
            "---\ntags: [refactored]\n---\nContent",
        )?;

        // Change to test directory
        let original_dir = env::current_dir()?;
        env::set_current_dir(dir.path())?;

        let stats = scan_directory_only_tag(&[], "refactored")?;

        // Restore original directory
        env::set_current_dir(original_dir)?;

        assert_eq!(
            stats.files_with_pattern, 1,
            "Should scan current directory when dirs is empty"
        );

        Ok(())
    }
}
