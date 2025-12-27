use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::core::filter::utils::should_exclude;
use crate::core::frontmatter::{parse_frontmatter, strip_frontmatter};
use crate::core::ignore::load_ignore_patterns;
use crate::wordcount::models::{FileMetrics, FileWordCount};

/// Counts words in all files within one or more directories and their subdirectories.
///
/// # Arguments
///
/// * `dirs` - The directory paths to scan. If empty, defaults to current directory.
/// * `exclude_dirs` - A list of directory names to exclude from the scan
/// * `filter_out` - Optional tag to exclude files containing this tag
///
/// # Returns
///
/// * `Ok(Vec<FileWordCount>)` - A vector of file paths and their word counts, sorted by word count descending
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
pub fn count_words(
    dirs: &[PathBuf],
    exclude_dirs: &[&str],
    filter_out: Option<&str>,
) -> Result<Vec<FileWordCount>> {
    let mut files = Vec::new();

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
                if let Some(tag) = filter_out {
                    if let Ok(frontmatter) = parse_frontmatter(&content) {
                        if let Some(tags) = frontmatter.tags {
                            if tags.iter().any(|t| t == tag) {
                                continue;
                            }
                        }
                    }
                }

                let body = strip_frontmatter(&content);
                let word_count = body.split_whitespace().count();
                files.push(FileWordCount {
                    path: path.to_path_buf(),
                    words: word_count,
                });
            }
        }
    }

    files.sort_by(|a, b| b.words.cmp(&a.words));
    Ok(files)
}

/// Counts words and lines in files, optionally filtering by thresholds and tags.
///
/// # Arguments
///
/// * `dirs` - The directory paths to scan. If empty, defaults to current directory.
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
/// * A directory cannot be accessed or read
/// * File system operations fail during traversal
/// * Files cannot be read as UTF-8 text
/// * The ignore patterns file cannot be parsed
/// * Frontmatter parsing fails
#[inline]
pub fn count_file_metrics(
    dirs: &[PathBuf],
    exclude_dirs: &[&str],
    filter_tags: &[&str],
    thresholds: Option<(usize, usize)>,
) -> Result<Vec<FileMetrics>> {
    let mut files = Vec::new();

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

                let metrics = FileMetrics::new(path.to_path_buf(), word_count, line_count);

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
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::filter::test_utils::{create_test_file, setup_test_directory};
    use anyhow::Result;
    use tempfile::TempDir;

    #[test]
    fn test_count_words() -> Result<()> {
        let dir = setup_test_directory()?;
        let files = count_words(&[dir.path().to_path_buf()], &[], None)?;
        assert_eq!(files.len(), 4, "Should process all non-hidden files");
        let file2 = files
            .iter()
            .find(|f| f.path.ends_with("file2.md"))
            .expect("file2.md should exist");
        assert_eq!(file2.words, 7, "file2.md should have 7 words");
        let files = count_words(&[dir.path().to_path_buf()], &[], Some("draft"))?;
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
        let word_counts = count_words(&[temp_dir.path().to_path_buf()], &[], None)?;
        assert_eq!(word_counts.len(), 1, "Should only process UTF-8 files");

        let file_metrics = count_file_metrics(&[temp_dir.path().to_path_buf()], &[], &[], None)?;
        assert_eq!(file_metrics.len(), 1, "Should only process UTF-8 files");

        Ok(())
    }

    // REQ-WC-MULTI-101: Results include files from all specified directories
    #[test]
    fn test_wordcount_should_include_files_from_all_directories() -> Result<()> {
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;

        create_test_file(&dir1, "file1.md", "---\ntags: [test]\n---\nContent one")?;
        create_test_file(&dir2, "file2.md", "---\ntags: [test]\n---\nContent two")?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let files = count_words(&dirs, &[], None)?;

        assert_eq!(files.len(), 2, "Should include files from both directories");

        Ok(())
    }

    // REQ-WC-MULTI-102: Files are sorted by word count across all directories
    #[test]
    fn test_wordcount_should_sort_files_across_all_directories() -> Result<()> {
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;

        create_test_file(&dir1, "small.md", "Two words")?;
        create_test_file(&dir2, "large.md", "One two three four five six")?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let files = count_words(&dirs, &[], None)?;

        assert_eq!(files.len(), 2);
        assert!(files[0].words > files[1].words, "Files should be sorted by word count descending");

        Ok(())
    }

    // REQ-WC-MULTI-201: Each directory is scanned for markdown files
    #[test]
    fn test_wordcount_should_scan_each_directory() -> Result<()> {
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;

        create_test_file(&dir1, "file1.md", "Content")?;
        create_test_file(&dir2, "file2.md", "Content")?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let files = count_words(&dirs, &[], None)?;

        assert_eq!(files.len(), 2, "Should scan both directories");

        Ok(())
    }

    // REQ-WC-MULTI-202: Filter tags apply to all specified directories
    #[test]
    fn test_wordcount_filter_applies_to_all_directories() -> Result<()> {
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;

        create_test_file(&dir1, "file1.md", "---\ntags: [filtered]\n---\nContent")?;
        create_test_file(&dir1, "file2.md", "---\ntags: [keep]\n---\nContent")?;
        create_test_file(&dir2, "file3.md", "---\ntags: [filtered]\n---\nContent")?;
        create_test_file(&dir2, "file4.md", "---\ntags: [keep]\n---\nContent")?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let files = count_words(&dirs, &[], Some("filtered"))?;

        assert_eq!(files.len(), 2, "Should filter out tagged files from both directories");

        Ok(())
    }

    // REQ-WC-MULTI-203: Exclude patterns apply to all specified directories
    #[test]
    fn test_wordcount_exclude_applies_to_all_directories() -> Result<()> {
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;

        create_test_file(&dir1, ".git/config", "Content")?;
        create_test_file(&dir1, "file1.md", "Content")?;
        create_test_file(&dir2, ".git/config", "Content")?;
        create_test_file(&dir2, "file2.md", "Content")?;

        let dirs = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
        let files = count_words(&dirs, &[".git"], None)?;

        assert_eq!(files.len(), 2, "Should exclude .git in both directories");

        Ok(())
    }

    // REQ-WC-MULTI-003: When no directories specified, defaults to current directory
    #[test]
    fn test_wordcount_should_default_to_current_directory() -> Result<()> {
        let files = count_words(&[], &[], None)?;
        // Should not panic and should return valid results
        let _ = files.len();
        Ok(())
    }
}
