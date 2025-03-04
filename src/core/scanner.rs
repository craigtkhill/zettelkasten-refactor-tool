// src/core/scanner.rs
use crate::core::ignore::{Patterns, load_ignore_patterns};
use crate::models::{ComparisonStats, FileWordCount, SinglePatternStats, WordCountStats};
use crate::utils::{contains_tag, is_hidden, parse_frontmatter};
use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

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
pub fn scan_directory_single_pattern(dir: &PathBuf, pattern: &str) -> Result<SinglePatternStats> {
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
pub fn scan_directory_two_patterns(
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

fn should_exclude(
    entry: &walkdir::DirEntry,
    exclude_dirs: &[&str],
    ignore_patterns: Option<&Patterns>,
) -> bool {
    // Always check if it's hidden first
    if is_hidden(entry) {
        return true;
    }

    // Check if the path contains any of the excluded directory names
    if let Some(path_str) = entry.path().to_str() {
        for dir in exclude_dirs {
            // Check if it's either the directory itself or a path containing the directory
            if entry.file_type().is_dir() && entry.file_name().to_str() == Some(*dir) {
                return true;
            }
            if path_str.contains(&format!("/{dir}/")) {
                return true;
            }
        }
    }

    // Check ignore patterns
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
    use std::fs::{self, File};
    use std::io::Write as _;
    use tempfile::TempDir;

    fn create_test_file(dir: &TempDir, name: &str, content: &str) -> Result<PathBuf> {
        let file_path = dir.path().join(name);
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
        Ok(file_path)
    }

    fn setup_test_directory() -> Result<TempDir> {
        let dir = TempDir::new()?;

        // Create some test files
        create_test_file(&dir, "file1.md", "This is file one")?;
        create_test_file(&dir, "file2.md", "This is file two\nWith more words")?;
        create_test_file(&dir, "nested/file3.md", "File in subfolder")?;

        // Create a file with frontmatter
        create_test_file(
            &dir,
            "tagged.md",
            "---\ntags: [test, draft]\n---\nThis is a tagged file",
        )?;

        // Create hidden files
        create_test_file(&dir, ".hidden.md", "Hidden file")?;

        Ok(dir)
    }

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

    #[test]
    fn test_scan_directory_single_pattern() -> Result<()> {
        let dir = setup_test_directory()?;

        // Add files with specific tags in frontmatter
        create_test_file(&dir, "todo1.md", "---\ntags: [todo]\n---\nItem one")?;
        create_test_file(&dir, "todo2.md", "---\ntags: [todo]\n---\nItem two")?;
        create_test_file(&dir, "normal.md", "Just regular content")?;

        let stats = scan_directory_single_pattern(&dir.path().to_path_buf(), "todo")?;

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

        let stats = scan_directory_two_patterns(&dir.path().to_path_buf(), "done", "todo")?;

        assert_eq!(stats.total, 8, "Should count all non-hidden files");
        assert_eq!(stats.done, 3, "Should find 3 files with 'done' tag");
        assert_eq!(stats.todo, 2, "Should find 2 files with 'todo' tag");

        Ok(())
    }

    #[test]
    fn test_should_exclude() -> Result<()> {
        let dir = setup_test_directory()?;

        // Test hidden file exclusion
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

        // Test directory exclusion
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
}
