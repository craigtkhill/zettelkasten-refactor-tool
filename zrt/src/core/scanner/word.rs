// src/core/scanner/word.rs
use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::core::ignore::load_ignore_patterns;
use crate::core::scanner::utils::should_exclude;
use crate::models::{FileMetrics, FileWordCount, WordCountStats};
use crate::utils::parse_frontmatter;

/// Calculates word count statistics for files with and without a specific tag.
///
/// # Arguments
///
/// * `dirs` - The directory paths to scan. If empty, defaults to current directory.
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
/// * A directory cannot be accessed or read
/// * File system operations fail during traversal
/// * Files cannot be read as UTF-8 text
/// * The ignore patterns file cannot be parsed
/// * Frontmatter parsing fails
#[inline]
pub fn count_word_stats(dirs: &[PathBuf], exclude_dirs: &[&str], tag: &str) -> Result<WordCountStats> {
    let mut stats = WordCountStats::new();

    // Default to current directory if no directories specified
    let directories: Vec<PathBuf> = if dirs.is_empty() {
        vec![env::current_dir()?]
    } else {
        dirs.to_vec()
    };

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
            .filter_entry(|e| !should_exclude(e, exclude_dirs, Some(&ignore_patterns)))
        {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if let Ok(content) = fs::read_to_string(path) {
                let has_tag;
                let content_without_frontmatter: String;

                if let Ok(frontmatter) = parse_frontmatter(&content) {
                    has_tag = frontmatter
                        .tags
                        .as_ref()
                        .is_some_and(|tags| tags.iter().any(|t| t == tag));
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
                let word_count = u32::try_from(content_without_frontmatter.split_whitespace().count())
                    .unwrap_or(u32::MAX); // Fallback to max value if conversion fails
                stats.total_files = stats.total_files.saturating_add(1);
                stats.total_words = stats.total_words.saturating_add(word_count);

                if has_tag {
                    stats.tagged_files = stats.tagged_files.saturating_add(1);
                    stats.tagged_words = stats.tagged_words.saturating_add(word_count);
                }
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

/// Counts words and lines in files, optionally filtering by thresholds and tags.
///
/// # Arguments
///
/// * `dir` - The directory path to scan
/// * `exclude_dirs` - A list of directory names to exclude from the scan
/// * `filter_tags` - A list of tags to exclude files containing these tags
/// * `thresholds` - Optional (word_threshold, line_threshold) to filter results
///
/// # Returns
///
/// * `Ok(Vec<FileMetrics>)` - A vector of file metrics with word counts, line counts, and tags
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
pub fn count_file_metrics(
    dir: &PathBuf,
    exclude_dirs: &[&str],
    filter_tags: &[&str],
    thresholds: Option<(usize, usize)>,
) -> Result<Vec<FileMetrics>> {
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
            let mut file_tags = Vec::new();
            let content_without_frontmatter: String;

            // Parse frontmatter and extract tags
            if let Ok(frontmatter) = parse_frontmatter(&content) {
                if let Some(tags) = frontmatter.tags {
                    file_tags = tags;
                }

                // Remove frontmatter from content for accurate word/line counting
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
                content_without_frontmatter = content.clone();
            }

            // Skip files that contain any of the filtered tags
            if !filter_tags.is_empty()
                && file_tags
                    .iter()
                    .any(|tag| filter_tags.contains(&tag.as_str()))
            {
                continue;
            }

            let word_count = content_without_frontmatter.split_whitespace().count();
            let line_count = content_without_frontmatter.lines().count();

            let metrics = FileMetrics::new(path.to_path_buf(), word_count, line_count, file_tags);

            // If thresholds are provided, only include files that exceed them
            if let Some((word_threshold, line_threshold)) = thresholds {
                if metrics.exceeds_thresholds(word_threshold, line_threshold) {
                    files.push(metrics);
                }
            } else {
                files.push(metrics);
            }
        }
    }

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
        let stats = count_word_stats(&[temp_dir.path().to_path_buf()], &[], "refactored")?;

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
        );
        Ok(())
    }

    #[test]
    fn test_count_words() -> Result<()> {
        let dir = setup_test_directory()?;
        let files = count_words(&dir.path().to_path_buf(), &[], None)?;
        assert_eq!(files.len(), 4, "Should process all non-hidden files");
        let file2 = files
            .iter()
            .find(|f| f.path.ends_with("file2.md"))
            .expect("file2.md should exist");
        assert_eq!(file2.words, 7, "file2.md should have 7 words");
        let files = count_words(&dir.path().to_path_buf(), &[], Some("draft"))?;
        assert_eq!(files.len(), 3, "Should exclude file with 'draft' tag");

        Ok(())
    }

    #[test]
    fn test_non_utf8_files_are_skipped() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create a valid UTF-8 markdown file
        create_test_file(&temp_dir, "valid.md", "---\ntags: [test]\n---\nValid content")?;
        
        // Create a binary file with invalid UTF-8 bytes
        let binary_path = temp_dir.path().join("binary.md");
        std::fs::write(&binary_path, &[0xFF, 0xFE, 0x00, 0x48, 0x65, 0x6C, 0x6C, 0x6F])?;
        
        // These functions should not panic and should skip the invalid UTF-8 file
        let word_stats = count_word_stats(&[temp_dir.path().to_path_buf()], &[], "test")?;
        assert_eq!(word_stats.total_files, 1, "Should only count UTF-8 files");
        assert_eq!(word_stats.tagged_files, 1, "Should find the tagged UTF-8 file");
        
        let word_counts = count_words(&temp_dir.path().to_path_buf(), &[], None)?;
        assert_eq!(word_counts.len(), 1, "Should only process UTF-8 files");
        
        let file_metrics = count_file_metrics(&temp_dir.path().to_path_buf(), &[], &[], None)?;
        assert_eq!(file_metrics.len(), 1, "Should only process UTF-8 files");
        
        Ok(())
    }

    // REQ-STATS-MULTI-101: Total file count includes files from all specified directories
    #[test]
    fn test_should_count_total_files_from_multiple_directories() -> Result<()> {
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;

        create_test_file(
            &dir1,
            "file1.md",
            "---\ntags: [refactored]\n---\nContent one",
        )?;
        create_test_file(
            &dir2,
            "file2.md",
            "---\ntags: [draft]\n---\nContent two",
        )?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let stats = count_word_stats(&dirs, &[], "refactored")?;

        assert_eq!(stats.total_files, 2, "Should count files from both directories");

        Ok(())
    }

    // REQ-STATS-MULTI-102: Total word count includes words from all specified directories
    #[test]
    fn test_should_sum_total_words_from_multiple_directories() -> Result<()> {
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;

        create_test_file(
            &dir1,
            "file1.md",
            "---\ntags: [refactored]\n---\nThis has four words",
        )?;
        create_test_file(
            &dir2,
            "file2.md",
            "---\ntags: [draft]\n---\nThis has five total words",
        )?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let stats = count_word_stats(&dirs, &[], "refactored")?;

        assert_eq!(stats.total_words, 9, "Should sum words from both directories");

        Ok(())
    }

    // REQ-STATS-MULTI-103: Tagged file count includes tagged files from all specified directories
    #[test]
    fn test_should_count_tagged_files_from_multiple_directories() -> Result<()> {
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;

        create_test_file(
            &dir1,
            "file1.md",
            "---\ntags: [refactored]\n---\nContent",
        )?;
        create_test_file(
            &dir2,
            "file2.md",
            "---\ntags: [refactored]\n---\nContent",
        )?;
        create_test_file(
            &dir2,
            "file3.md",
            "---\ntags: [draft]\n---\nContent",
        )?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let stats = count_word_stats(&dirs, &[], "refactored")?;

        assert_eq!(stats.tagged_files, 2, "Should count tagged files from both directories");

        Ok(())
    }

    // REQ-STATS-MULTI-104: Tagged word count includes tagged words from all specified directories
    #[test]
    fn test_should_sum_tagged_words_from_multiple_directories() -> Result<()> {
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;

        create_test_file(
            &dir1,
            "file1.md",
            "---\ntags: [refactored]\n---\nFive words in this file",
        )?;
        create_test_file(
            &dir2,
            "file2.md",
            "---\ntags: [refactored]\n---\nSix words in this one here",
        )?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let stats = count_word_stats(&dirs, &[], "refactored")?;

        assert_eq!(stats.tagged_words, 11, "Should sum tagged words from both directories");

        Ok(())
    }

    // REQ-STATS-MULTI-105: Percentage calculation uses aggregated totals
    #[test]
    fn test_should_calculate_percentage_from_aggregated_totals() -> Result<()> {
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;

        create_test_file(
            &dir1,
            "file1.md",
            "---\ntags: [refactored]\n---\nten",
        )?;
        create_test_file(
            &dir2,
            "file2.md",
            "---\ntags: [draft]\n---\nten",
        )?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let stats = count_word_stats(&dirs, &[], "refactored")?;

        assert_eq!(stats.total_words, 2);
        assert_eq!(stats.tagged_words, 1);
        assert_eq!(stats.calculate_percentage(), 50.0);

        Ok(())
    }

    // REQ-STATS-MULTI-201: Each directory is scanned for markdown files
    #[test]
    fn test_should_scan_each_directory_for_markdown_files() -> Result<()> {
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;

        create_test_file(&dir1, "file1.md", "Content")?;
        create_test_file(&dir2, "file2.md", "Content")?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let stats = count_word_stats(&dirs, &[], "test")?;

        assert_eq!(stats.total_files, 2, "Should scan both directories");

        Ok(())
    }

    // REQ-STATS-MULTI-203: Exclude patterns apply to all specified directories
    #[test]
    fn test_should_apply_exclude_patterns_to_all_directories() -> Result<()> {
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;

        create_test_file(&dir1, ".git/config", "Content")?;
        create_test_file(&dir1, "file1.md", "Content")?;
        create_test_file(&dir2, ".git/config", "Content")?;
        create_test_file(&dir2, "file2.md", "Content")?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let stats = count_word_stats(&dirs, &[".git"], "test")?;

        assert_eq!(stats.total_files, 2, "Should exclude .git in both directories");

        Ok(())
    }

    // REQ-STATS-MULTI-003: When no directories specified, defaults to current directory
    #[test]
    fn test_should_default_to_current_directory_when_empty() -> Result<()> {
        let stats = count_word_stats(&[], &[], "test")?;
        // Should not panic and should return valid stats
        let _ = stats.total_files;
        Ok(())
    }
}
