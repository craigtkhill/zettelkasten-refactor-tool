pub mod cli;

use anyhow::Result;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::core::filter::utils::should_exclude;
use crate::core::frontmatter::parse_frontmatter;
use crate::core::ignore::load_ignore_patterns;

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

    #[test]
    fn test_should_find_files_with_exactly_one_tag() -> Result<()> {
        // REQ-SEARCH-001
        let dir = TempDir::new()?;
        create_test_file(&dir, "exact.md", "---\ntags: [refactor]\n---\nContent")?;
        create_test_file(&dir, "extra.md", "---\ntags: [refactor, draft]\n---\nContent")?;
        create_test_file(&dir, "none.md", "No tags")?;

        let files = search_exactly(&[dir.path().to_path_buf()], &["refactor"], &[])?;
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("exact.md"));
        Ok(())
    }

    #[test]
    fn test_should_find_files_with_exactly_multiple_tags() -> Result<()> {
        // REQ-SEARCH-002
        let dir = TempDir::new()?;
        create_test_file(&dir, "exact.md", "---\ntags: [refactor, draft]\n---\nContent")?;
        create_test_file(&dir, "partial.md", "---\ntags: [refactor]\n---\nContent")?;
        create_test_file(&dir, "extra.md", "---\ntags: [refactor, draft, wip]\n---\nContent")?;

        let files = search_exactly(&[dir.path().to_path_buf()], &["refactor", "draft"], &[])?;
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("exact.md"));
        Ok(())
    }

    #[test]
    fn test_should_exclude_files_with_additional_tags() -> Result<()> {
        // REQ-SEARCH-003
        let dir = TempDir::new()?;
        create_test_file(&dir, "exact.md", "---\ntags: [refactor]\n---\nContent")?;
        create_test_file(&dir, "extra.md", "---\ntags: [refactor, draft]\n---\nContent")?;

        let files = search_exactly(&[dir.path().to_path_buf()], &["refactor"], &[])?;
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("exact.md"));
        Ok(())
    }

    #[test]
    fn test_should_exclude_files_missing_tags() -> Result<()> {
        // REQ-SEARCH-004
        let dir = TempDir::new()?;
        create_test_file(&dir, "exact.md", "---\ntags: [refactor, draft]\n---\nContent")?;
        create_test_file(&dir, "partial.md", "---\ntags: [refactor]\n---\nContent")?;

        let files = search_exactly(&[dir.path().to_path_buf()], &["refactor", "draft"], &[])?;
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("exact.md"));
        Ok(())
    }

    #[test]
    fn test_should_scan_multiple_directories() -> Result<()> {
        // REQ-SEARCH-005
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;
        create_test_file(&dir1, "file1.md", "---\ntags: [refactor]\n---\nContent")?;
        create_test_file(&dir2, "file2.md", "---\ntags: [refactor]\n---\nContent")?;

        let files = search_exactly(
            &[dir1.path().to_path_buf(), dir2.path().to_path_buf()],
            &["refactor"],
            &[],
        )?;
        assert_eq!(files.len(), 2);
        Ok(())
    }

    #[test]
    fn test_should_default_to_current_directory() -> Result<()> {
        // REQ-SEARCH-006
        // This will be tested via integration test or CLI test
        Ok(())
    }

    #[test]
    fn test_should_exclude_specified_directories() -> Result<()> {
        // REQ-SEARCH-007
        let dir = TempDir::new()?;
        let excluded = dir.path().join("excluded");
        fs::create_dir(&excluded)?;

        create_test_file(&dir, "file1.md", "---\ntags: [refactor]\n---\nContent")?;
        fs::write(
            excluded.join("file2.md"),
            "---\ntags: [refactor]\n---\nContent",
        )?;

        let files = search_exactly(&[dir.path().to_path_buf()], &["refactor"], &["excluded"])?;
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("file1.md"));
        Ok(())
    }

    #[test]
    fn test_should_handle_tag_order_independently() -> Result<()> {
        let dir = TempDir::new()?;
        create_test_file(&dir, "file.md", "---\ntags: [draft, refactor]\n---\nContent")?;

        let files = search_exactly(&[dir.path().to_path_buf()], &["refactor", "draft"], &[])?;
        assert_eq!(files.len(), 1);
        Ok(())
    }

    #[test]
    fn test_yaml_list_format() -> Result<()> {
        let dir = TempDir::new()?;
        create_test_file(&dir, "file.md", "---\ntags:\n  - refactored\n---\nContent")?;

        let files = search_exactly(&[dir.path().to_path_buf()], &["refactored"], &[])?;
        assert_eq!(files.len(), 1, "Should find file with YAML list format tags");
        Ok(())
    }

    #[test]
    fn test_should_find_file_with_no_tags_field_in_frontmatter() -> Result<()> {
        // REQ-SEARCH-013

        // Given
        let dir = TempDir::new()?;
        create_test_file(&dir, "no_tags.md", "---\ntitle: My Note\n---\nContent")?;
        create_test_file(&dir, "has_tags.md", "---\ntags: [draft]\n---\nContent")?;

        // When
        let files = search_missing_tags(&[dir.path().to_path_buf()], &[])?;

        // Then
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("no_tags.md"));
        Ok(())
    }

    #[test]
    fn test_should_find_file_with_no_frontmatter() -> Result<()> {
        // REQ-SEARCH-014

        // Given
        let dir = TempDir::new()?;
        create_test_file(&dir, "no_fm.md", "Just plain content")?;
        create_test_file(&dir, "has_tags.md", "---\ntags: [draft]\n---\nContent")?;

        // When
        let files = search_missing_tags(&[dir.path().to_path_buf()], &[])?;

        // Then
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("no_fm.md"));
        Ok(())
    }

    #[test]
    fn test_should_respect_directories_when_finding_missing_tags() -> Result<()> {
        // REQ-SEARCH-015

        // Given
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;
        create_test_file(&dir1, "a.md", "No frontmatter")?;
        create_test_file(&dir2, "b.md", "No frontmatter")?;

        // When
        let files = search_missing_tags(
            &[dir1.path().to_path_buf(), dir2.path().to_path_buf()],
            &[],
        )?;

        // Then
        assert_eq!(files.len(), 2);
        Ok(())
    }

    #[test]
    fn test_should_respect_exclude_when_finding_missing_tags() -> Result<()> {
        // REQ-SEARCH-015

        // Given
        let dir = TempDir::new()?;
        let excluded = dir.path().join("excluded");
        fs::create_dir(&excluded)?;
        create_test_file(&dir, "a.md", "No frontmatter")?;
        fs::write(excluded.join("b.md"), "No frontmatter")?;

        // When
        let files = search_missing_tags(&[dir.path().to_path_buf()], &["excluded"])?;

        // Then
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("a.md"));
        Ok(())
    }
}

// ============================================
// TYPE DEFINITIONS
// ============================================

// ============================================
// IMPLEMENTATIONS
// ============================================

/// Search for files that have no tags (missing tags field or no frontmatter)
pub fn search_missing_tags(dirs: &[PathBuf], exclude: &[&str]) -> Result<Vec<String>> {
    let mut matching_files = Vec::new();

    for dir in dirs {
        let absolute_dir = if dir.is_absolute() {
            dir.clone()
        } else {
            std::env::current_dir()?.join(dir)
        };

        let ignore_patterns = load_ignore_patterns(&absolute_dir)?;

        for entry in WalkDir::new(&absolute_dir)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !should_exclude(e, exclude, Some(&ignore_patterns)))
        {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let missing = match parse_frontmatter(&content) {
                    Ok(fm) => fm.tags.is_none(),
                    Err(_) => true,
                };
                if missing {
                    matching_files.push(entry.path().display().to_string());
                }
            }
        }
    }

    Ok(matching_files)
}

/// Search for files that have exactly the specified tags (no more, no less)
pub fn search_exactly(dirs: &[PathBuf], tags: &[&str], exclude: &[&str]) -> Result<Vec<String>> {
    let mut matching_files = Vec::new();

    for dir in dirs {
        let absolute_dir = if dir.is_absolute() {
            dir.clone()
        } else {
            std::env::current_dir()?.join(dir)
        };

        let ignore_patterns = load_ignore_patterns(&absolute_dir)?;

        for entry in WalkDir::new(&absolute_dir)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !should_exclude(e, exclude, Some(&ignore_patterns)))
        {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(frontmatter) = parse_frontmatter(&content) {
                    if let Some(file_tags) = frontmatter.tags {
                        if file_tags.len() == tags.len()
                            && tags.iter().all(|tag| file_tags.contains(&tag.to_string()))
                        {
                            matching_files.push(entry.path().display().to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(matching_files)
}
