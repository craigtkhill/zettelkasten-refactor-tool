// src/core/scanner/pattern_scanner.rs
use anyhow::Result;
use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::core::ignore::load_ignore_patterns;
use crate::core::scanner::utils::should_exclude;
use crate::models::{ComparisonStats, SinglePatternStats};
use crate::utils::contains_tag;

/// Scans a directory for files containing a specific pattern/tag.
///
/// # Arguments
///
/// * `dir` - The directory path to scan
/// * `pattern` - The pattern or tag to search for in files
///
/// # Returns
///
/// * `Ok(SinglePatternStats)` - Statistics about files with the specified pattern
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
pub fn scan_directory_single(dir: &PathBuf, pattern: &str) -> Result<SinglePatternStats> {
    let absolute_dir = if dir.is_absolute() {
        dir.clone()
    } else {
        env::current_dir()?.join(dir)
    };

    let ignore_patterns = load_ignore_patterns(&absolute_dir)?;
    let mut stats = SinglePatternStats::new();

    for entry in WalkDir::new(&absolute_dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, &[], Some(&ignore_patterns)))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        stats.total_files = stats.total_files.saturating_add(1);

        let path = entry.path();
        if contains_tag(path, pattern)? {
            stats.files_with_pattern = stats.files_with_pattern.saturating_add(1);
        }
    }

    Ok(stats)
}

/// Scans a directory for files containing two different patterns/tags.
///
/// # Arguments
///
/// * `dir` - The directory path to scan
/// * `done_tag` - The first tag to search for in files
/// * `todo_tag` - The second tag to search for in files
///
/// # Returns
///
/// * `Ok(ComparisonStats)` - Statistics comparing the presence of both tags
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
pub fn scan_directory_two(
    dir: &PathBuf,
    done_tag: &str,
    todo_tag: &str,
) -> Result<ComparisonStats> {
    let absolute_dir = if dir.is_absolute() {
        dir.clone()
    } else {
        env::current_dir()?.join(dir)
    };

    let ignore_patterns = load_ignore_patterns(&absolute_dir)?;
    let mut stats = ComparisonStats::new();

    for entry in WalkDir::new(&absolute_dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, &[], Some(&ignore_patterns)))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        stats.total = stats.total.saturating_add(1);

        let path = entry.path();
        if contains_tag(path, done_tag)? {
            stats.done = stats.done.saturating_add(1);
        }
        if contains_tag(path, todo_tag)? {
            stats.todo = stats.todo.saturating_add(1);
        }
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::scanner::test_utils::{create_test_file, setup_test_directory};
    use anyhow::Result;

    #[test]
    fn test_scan_directory_single_pattern() -> Result<()> {
        let dir = setup_test_directory()?;

        // Add files with specific tags in frontmatter
        create_test_file(&dir, "todo1.md", "---\ntags: [todo]\n---\nItem one")?;
        create_test_file(&dir, "todo2.md", "---\ntags: [todo]\n---\nItem two")?;
        create_test_file(&dir, "normal.md", "Just regular content")?;

        let stats = scan_directory_single(&dir.path().to_path_buf(), "todo")?;

        assert_eq!(stats.total_files, 7, "Should count all non-hidden files");
        assert_eq!(
            stats.files_with_pattern, 2,
            "Should find 2 files with 'todo' tag"
        );

        Ok(())
    }

    #[test]
    fn test_scan_directory_two_patterns() -> Result<()> {
        let dir = setup_test_directory()?;

        // Add files with done/todo tags in front-matter
        create_test_file(&dir, "done1.md", "---\ntags: [done]\n---\nTask one")?;
        create_test_file(&dir, "done2.md", "---\ntags: [done]\n---\nTask two")?;
        create_test_file(&dir, "todo1.md", "---\ntags: [todo]\n---\nTask three")?;
        create_test_file(&dir, "both.md", "---\ntags: [done, todo]\n---\nTask four")?;

        let stats = scan_directory_two(&dir.path().to_path_buf(), "done", "todo")?;

        assert_eq!(stats.total, 8, "Should count all non-hidden files");
        assert_eq!(stats.done, 3, "Should find 3 files with 'done' tag");
        assert_eq!(stats.todo, 2, "Should find 2 files with 'todo' tag");

        Ok(())
    }
}
