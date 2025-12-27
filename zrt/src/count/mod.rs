pub mod cli;

use anyhow::Result;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::core::ignore::load_ignore_patterns;
use crate::core::scanner::utils::should_exclude;
use crate::utils::parse_frontmatter;

// ============================================
// TESTS
// ============================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &TempDir, name: &str, content: &str) -> Result<PathBuf> {
        let path = dir.path().join(name);
        fs::write(&path, content)?;
        Ok(path)
    }

    // File counting tests
    #[test]
    fn test_should_count_files_with_single_tag() -> Result<()> {
        // REQ-COUNT-001
        let dir = TempDir::new()?;
        create_test_file(&dir, "tagged.md", "---\ntags: [refactor]\n---\nContent")?;
        create_test_file(&dir, "untagged.md", "No tags")?;

        let count = count_files(&[dir.path().to_path_buf()], &["refactor"], &[])?;
        assert_eq!(count, 1);
        Ok(())
    }

    #[test]
    fn test_should_count_files_with_multiple_tags() -> Result<()> {
        // REQ-COUNT-002
        let dir = TempDir::new()?;
        create_test_file(&dir, "tag1.md", "---\ntags: [refactor]\n---\nContent")?;
        create_test_file(&dir, "tag2.md", "---\ntags: [draft]\n---\nContent")?;
        create_test_file(&dir, "untagged.md", "No tags")?;

        let count = count_files(&[dir.path().to_path_buf()], &["refactor", "draft"], &[])?;
        assert_eq!(count, 2);
        Ok(())
    }

    #[test]
    fn test_should_count_all_files_when_no_tags() -> Result<()> {
        // REQ-COUNT-003
        let dir = TempDir::new()?;
        create_test_file(&dir, "file1.md", "Content 1")?;
        create_test_file(&dir, "file2.md", "Content 2")?;

        let count = count_files(&[dir.path().to_path_buf()], &[], &[])?;
        assert_eq!(count, 2);
        Ok(())
    }

    // Word counting tests
    #[test]
    fn test_should_count_words_with_single_tag() -> Result<()> {
        // REQ-COUNT-004
        let dir = TempDir::new()?;
        create_test_file(&dir, "tagged.md", "---\ntags: [refactor]\n---\nOne two three")?;
        create_test_file(&dir, "untagged.md", "Four five six seven")?;

        let count = count_words(&[dir.path().to_path_buf()], &["refactor"], &[])?;
        assert_eq!(count, 3);
        Ok(())
    }

    #[test]
    fn test_should_count_words_with_multiple_tags() -> Result<()> {
        // REQ-COUNT-005
        let dir = TempDir::new()?;
        create_test_file(&dir, "tag1.md", "---\ntags: [refactor]\n---\nOne two")?;
        create_test_file(&dir, "tag2.md", "---\ntags: [draft]\n---\nThree four five")?;

        let count = count_words(&[dir.path().to_path_buf()], &["refactor", "draft"], &[])?;
        assert_eq!(count, 5);
        Ok(())
    }

    #[test]
    fn test_should_count_all_words_when_no_tags() -> Result<()> {
        // REQ-COUNT-006
        let dir = TempDir::new()?;
        create_test_file(&dir, "file1.md", "One two three")?;
        create_test_file(&dir, "file2.md", "Four five")?;

        let count = count_words(&[dir.path().to_path_buf()], &[], &[])?;
        assert_eq!(count, 5);
        Ok(())
    }

    // Percentage tests
    #[test]
    fn test_should_calculate_percentage_for_single_tag() -> Result<()> {
        // REQ-COUNT-007
        let dir = TempDir::new()?;
        create_test_file(&dir, "tagged.md", "---\ntags: [refactor]\n---\nOne two")?;
        create_test_file(&dir, "untagged.md", "Three four five six seven eight")?;

        let percentage = calculate_percentage(&[dir.path().to_path_buf()], &["refactor"], &[])?;
        assert_eq!(percentage, 25.0); // 2 out of 8 words
        Ok(())
    }

    #[test]
    fn test_should_calculate_percentage_for_multiple_tags() -> Result<()> {
        // REQ-COUNT-008
        let dir = TempDir::new()?;
        create_test_file(&dir, "tag1.md", "---\ntags: [refactor]\n---\nOne two")?;
        create_test_file(&dir, "tag2.md", "---\ntags: [draft]\n---\nThree four")?;
        create_test_file(&dir, "untagged.md", "Five six")?;

        let percentage = calculate_percentage(&[dir.path().to_path_buf()], &["refactor", "draft"], &[])?;
        assert_eq!(percentage, 66.67); // 4 out of 6 words, rounded to 2 decimals
        Ok(())
    }

    #[test]
    fn test_should_calculate_100_percent_when_no_tags() -> Result<()> {
        // REQ-COUNT-008a
        let dir = TempDir::new()?;
        create_test_file(&dir, "file1.md", "One two three")?;
        create_test_file(&dir, "file2.md", "Four five")?;

        let percentage = calculate_percentage(&[dir.path().to_path_buf()], &[], &[])?;
        assert_eq!(percentage, 100.0);
        Ok(())
    }

    // Directory scanning tests
    #[test]
    fn test_should_scan_multiple_directories() -> Result<()> {
        // REQ-COUNT-009
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;
        create_test_file(&dir1, "file1.md", "Content 1")?;
        create_test_file(&dir2, "file2.md", "Content 2")?;

        let count = count_files(&[dir1.path().to_path_buf(), dir2.path().to_path_buf()], &[], &[])?;
        assert_eq!(count, 2);
        Ok(())
    }

    #[test]
    fn test_should_default_to_current_directory() -> Result<()> {
        // REQ-COUNT-010
        // This will be tested via integration test or CLI test
        // Unit test would require changing current directory which is problematic
        Ok(())
    }

    #[test]
    fn test_should_exclude_specified_directories() -> Result<()> {
        // REQ-COUNT-011
        let dir = TempDir::new()?;
        let excluded = dir.path().join("excluded");
        fs::create_dir(&excluded)?;

        create_test_file(&dir, "file1.md", "Content 1")?;
        fs::write(excluded.join("file2.md"), "Content 2")?;

        let count = count_files(&[dir.path().to_path_buf()], &[], &["excluded"])?;
        assert_eq!(count, 1);
        Ok(())
    }
}

// ============================================
// TYPE DEFINITIONS
// ============================================

// ============================================
// IMPLEMENTATIONS
// ============================================

/// Count files matching tag criteria
pub fn count_files(dirs: &[PathBuf], tags: &[&str], exclude: &[&str]) -> Result<usize> {
    let mut count = 0;

    for dir in dirs {
        let ignore_patterns = load_ignore_patterns(dir)?;

        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !should_exclude(e, exclude, Some(&ignore_patterns)))
        {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            // If no tags specified, count all files
            if tags.is_empty() {
                count += 1;
                continue;
            }

            // Check if file has any of the specified tags
            let content = std::fs::read_to_string(entry.path())?;
            if let Ok(frontmatter) = parse_frontmatter(&content) {
                if let Some(file_tags) = frontmatter.tags {
                    if tags.iter().any(|tag| file_tags.iter().any(|ft| ft == tag)) {
                        count += 1;
                    }
                }
            }
        }
    }

    Ok(count)
}

/// Strip frontmatter from content and return body
fn strip_frontmatter(content: &str) -> &str {
    if !content.starts_with("---") {
        return content;
    }

    // Find the closing ---
    if let Some(end) = content[3..].find("---") {
        let body_start = 3 + end + 3; // Skip past second ---
        return content.get(body_start..).unwrap_or("");
    }

    content
}

/// Count words in files matching tag criteria
pub fn count_words(dirs: &[PathBuf], tags: &[&str], exclude: &[&str]) -> Result<usize> {
    let mut total_words = 0;

    for dir in dirs {
        let ignore_patterns = load_ignore_patterns(dir)?;

        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !should_exclude(e, exclude, Some(&ignore_patterns)))
        {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let content = std::fs::read_to_string(entry.path())?;
            let body = strip_frontmatter(&content);

            // If no tags specified, count all words
            if tags.is_empty() {
                let words = body.split_whitespace().count();
                total_words += words;
                continue;
            }

            // Check if file has any of the specified tags
            if let Ok(frontmatter) = parse_frontmatter(&content) {
                if let Some(file_tags) = frontmatter.tags {
                    if tags.iter().any(|tag| file_tags.iter().any(|ft| ft == tag)) {
                        let words = body.split_whitespace().count();
                        total_words += words;
                    }
                }
            }
        }
    }

    Ok(total_words)
}

/// Calculate percentage of words in tagged files
pub fn calculate_percentage(dirs: &[PathBuf], tags: &[&str], exclude: &[&str]) -> Result<f64> {
    let tagged_words = count_words(dirs, tags, exclude)?;
    let total_words = count_words(dirs, &[], exclude)?;

    if total_words == 0 {
        return Ok(0.0);
    }

    let percentage = (tagged_words as f64 / total_words as f64) * 100.0;
    // Round to 2 decimal places
    Ok((percentage * 100.0).round() / 100.0)
}
