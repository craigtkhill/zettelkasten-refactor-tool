// src/core/scanner.rs
use crate::core::ignore::{IgnorePatterns, load_ignore_patterns};
use crate::models::{ComparisonStats, FileWordCount, SinglePatternStats};
use crate::utils::{contains_tag, is_hidden};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn count_files(dir: &PathBuf, exclude_dirs: &[&str]) -> Result<u64> {
    let absolute_dir = if dir.is_absolute() {
        dir.clone()
    } else {
        std::env::current_dir()?.join(dir)
    };

    let ignore_patterns = load_ignore_patterns(&absolute_dir)?;
    let mut count = 0;

    for entry in WalkDir::new(&absolute_dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, exclude_dirs, Some(&ignore_patterns)))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            count += 1;
        }
    }

    println!("Total files found: {count}");
    Ok(count)
}

pub fn count_words(
    dir: &PathBuf,
    exclude_dirs: &[&str],
    filter_out: Option<&str>,
) -> Result<Vec<FileWordCount>> {
    let absolute_dir = if dir.is_absolute() {
        dir.clone()
    } else {
        std::env::current_dir()?.join(dir)
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
                if let Ok(frontmatter) = crate::utils::parse_frontmatter(&content) {
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

pub fn scan_directory_single_pattern(dir: &PathBuf, pattern: &str) -> Result<SinglePatternStats> {
    let absolute_dir = if dir.is_absolute() {
        dir.clone()
    } else {
        std::env::current_dir()?.join(dir)
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

        stats.total_files += 1;

        let path = entry.path();
        if contains_tag(path, pattern)? {
            stats.files_with_pattern += 1;
        }
    }

    Ok(stats)
}

pub fn scan_directory_two_patterns(
    dir: &PathBuf,
    done_tag: &str,
    todo_tag: &str,
) -> Result<ComparisonStats> {
    let absolute_dir = if dir.is_absolute() {
        dir.clone()
    } else {
        std::env::current_dir()?.join(dir)
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

        stats.total_files += 1;

        let path = entry.path();
        if contains_tag(path, done_tag)? {
            stats.done_files += 1;
        }
        if contains_tag(path, todo_tag)? {
            stats.todo_files += 1;
        }
    }

    Ok(stats)
}

fn should_exclude(
    entry: &walkdir::DirEntry,
    exclude_dirs: &[&str],
    ignore_patterns: Option<&IgnorePatterns>,
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
    use std::io::Write;
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

        assert_eq!(stats.total_files, 8, "Should count all non-hidden files");
        assert_eq!(stats.done_files, 3, "Should find 3 files with 'done' tag");
        assert_eq!(stats.todo_files, 2, "Should find 2 files with 'todo' tag");

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
}
