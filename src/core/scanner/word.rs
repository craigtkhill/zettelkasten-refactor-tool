// src/core/scanner/word_counter.rs

use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::core::ignore::load_ignore_patterns;
use crate::core::scanner::utils::should_exclude;
use crate::models::{FileWordCount, WordCountStats};
use crate::utils::parse_frontmatter;

/// Calculates word count statistics for files with and without a specific tag.
///
/// # Arguments
///
/// * `dir` - The directory path to scan
/// * `exclude_dirs` - A list of directory names to exclude from the scan
/// * `tag` - The tag to identify files for separate statistics
///
/// # Returns
///
/// * `Ok(WordCountStats)` - Word count statistics for tagged and untagged files
///
/// # Errors
///
/// This function may return an error if:
/// * The directory cannot be accessed or read
/// * File system operations fail during traversal
/// * Files cannot be read as UTF-8 text
/// * The ignore patterns file cannot be parsed
/// * Frontmatter parsing fails
#[inline]
pub fn count_word_stats(dir: &PathBuf, exclude_dirs: &[&str], tag: &str) -> Result<WordCountStats> {
    let absolute_dir = if dir.is_absolute() {
        dir.clone()
    } else {
        env::current_dir()?.join(dir)
    };

    let ignore_patterns = load_ignore_patterns(&absolute_dir)?;
    let mut stats = WordCountStats::new();

    for entry in WalkDir::new(&absolute_dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, exclude_dirs, Some(&ignore_patterns)))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if let Ok(content) = fs::read_to_string(path) {
            // Parse frontmatter to check for tags
            let has_tag;
            let content_without_frontmatter: String;

            if let Ok(frontmatter) = parse_frontmatter(&content) {
                // Check if the file has the specified tag
                has_tag = frontmatter
                    .tags
                    .as_ref()
                    .is_some_and(|tags| tags.iter().any(|t| t == tag));

                // Extract content without frontmatter
                let lines: Vec<&str> = content.lines().collect();
                if lines.len() > 2 && lines.first().is_some_and(|line| *line == "---") {
                    if let Some(end_index) = lines.iter().skip(1).position(|&line| line == "---") {
                        content_without_frontmatter =
                            lines.get(end_index.saturating_add(2)..).map_or_else(
                                || content.clone(),
                                |content_slice| content_slice.join("\n"),
                            );
                    } else {
                        content_without_frontmatter = content.clone();
                    }
                } else {
                    content_without_frontmatter = content.clone();
                }
            } else {
                has_tag = false;
                content_without_frontmatter = content.clone();
            }

            // Count words in the content (excluding frontmatter)
            let word_count = u64::try_from(content_without_frontmatter.split_whitespace().count())
                .unwrap_or(u64::MAX); // Fallback to max value if conversion fails

            // Update the stats
            stats.total_files = stats.total_files.saturating_add(1);
            stats.total_words = stats.total_words.saturating_add(word_count);

            if has_tag {
                stats.tagged_files = stats.tagged_files.saturating_add(1);
                stats.tagged_words = stats.tagged_words.saturating_add(word_count);
            }
        }
    }

    Ok(stats)
}

/// Counts words in all files within a directory and its subdirectories.
///
/// # Arguments
///
/// * `dir` - The directory path to scan
/// * `exclude_dirs` - A list of directory names to exclude from the scan
/// * `filter_out` - Optional tag to exclude files containing this tag
///
/// # Returns
///
/// * `Ok(Vec<FileWordCount>)` - A vector of file paths and their word counts
///
/// # Errors
///
/// This function may return an error if:
/// * The directory cannot be accessed or read
/// * File system operations fail during traversal
/// * Files cannot be read as UTF-8 text
/// * The ignore patterns file cannot be parsed
/// * Frontmatter parsing fails
#[inline]
pub fn count_words(
    dir: &PathBuf,
    exclude_dirs: &[&str],
    filter_out: Option<&str>,
) -> Result<Vec<FileWordCount>> {
    let absolute_dir = if dir.is_absolute() {
        dir.clone()
    } else {
        env::current_dir()?.join(dir)
    };

    let ignore_patterns = load_ignore_patterns(&absolute_dir)?;
    let mut files = Vec::new();

    for entry in WalkDir::new(&absolute_dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, exclude_dirs, Some(&ignore_patterns)))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if let Ok(content) = fs::read_to_string(path) {
            // Skip file if it contains the filter_out tag
            if let Some(tag) = filter_out {
                if let Ok(frontmatter) = parse_frontmatter(&content) {
                    if let Some(tags) = frontmatter.tags {
                        if tags.iter().any(|t| t == tag) {
                            continue;
                        }
                    }
                }
            }

            let word_count = content.split_whitespace().count();
            files.push(FileWordCount {
                path: path.to_path_buf(),
                words: word_count,
            });
        }
    }

    files.sort_by(|a, b| b.words.cmp(&a.words));
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::scanner::test_utils::{create_test_file, setup_test_directory};
    use anyhow::Result;
    use tempfile::TempDir;

    #[test]
    fn test_count_word_stats() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create files with and without the refactored tag
        create_test_file(
            &temp_dir,
            "file1.md",
            "---\ntags: [refactored]\n---\nThis file has five words",
        )?;
        create_test_file(
            &temp_dir,
            "file2.md",
            "---\ntags: [other]\n---\nThis file has four words",
        )?;
        create_test_file(
            &temp_dir,
            "file3.md",
            "---\ntags: [refactored]\n---\nThis file has five more words",
        )?;
        create_test_file(&temp_dir, "file4.md", "No tags in this file")?;

        // Test word stats counting
        let stats = count_word_stats(&temp_dir.path().to_path_buf(), &[], "refactored")?;

        assert_eq!(stats.total_files, 4, "Should count all 4 files");
        assert_eq!(
            stats.tagged_files, 2,
            "Should find 2 files with 'refactored' tag"
        );
        assert_eq!(stats.tagged_words, 11, "Tagged files have 11 words total"); // Updated to 11
        assert_eq!(stats.total_words, 21, "All files have 21 words total"); // Updated to 21
        assert_eq!(
            stats.calculate_percentage(),
            (11.0 / 21.0) * 100.0,
            "Percentage calculation should be correct"
        ); // Updated

        Ok(())
    }

    #[test]
    fn test_count_words() -> Result<()> {
        let dir = setup_test_directory()?;

        // Test basic word counting
        let files = count_words(&dir.path().to_path_buf(), &[], None)?;
        assert_eq!(files.len(), 4, "Should process all non-hidden files");

        // Find file2.md (should have 7 words)
        let file2 = files
            .iter()
            .find(|f| f.path.ends_with("file2.md"))
            .expect("file2.md should exist");
        assert_eq!(file2.words, 7, "file2.md should have 7 words");

        // Test with filter_out tag
        let files = count_words(&dir.path().to_path_buf(), &[], Some("draft"))?;
        assert_eq!(files.len(), 3, "Should exclude file with 'draft' tag");

        Ok(())
    }
}
