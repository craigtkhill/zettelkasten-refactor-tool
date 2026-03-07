pub mod cli;

use anyhow::Result;
use std::collections::HashMap;
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
    use anyhow::Result;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &TempDir, name: &str, content: &str) -> Result<PathBuf> {
        let path = dir.path().join(name);
        fs::write(&path, content)?;
        Ok(path)
    }

    #[test]
    fn test_should_count_tag_frequency() -> Result<()> {
        // REQ-TAGS-001

        // Given
        let dir = TempDir::new()?;
        create_test_file(&dir, "a.md", "---\ntags: [writing]\n---\nContent")?;
        create_test_file(&dir, "b.md", "---\ntags: [writing, ideas]\n---\nContent")?;
        create_test_file(&dir, "c.md", "---\ntags: [ideas]\n---\nContent")?;

        // When
        let results = count_tags(&[dir.path().to_path_buf()], &[], &[])?;

        // Then
        let writing_count = results.iter().find(|(t, _)| t == "writing").map(|(_, c)| *c);
        assert_eq!(writing_count, Some(2));
        Ok(())
    }

    #[test]
    fn test_should_sort_tags_by_frequency_descending() -> Result<()> {
        // REQ-TAGS-002

        // Given
        let dir = TempDir::new()?;
        create_test_file(&dir, "a.md", "---\ntags: [writing]\n---")?;
        create_test_file(&dir, "b.md", "---\ntags: [writing, ideas]\n---")?;
        create_test_file(&dir, "c.md", "---\ntags: [ideas]\n---")?;
        create_test_file(&dir, "d.md", "---\ntags: [ideas]\n---")?;

        // When
        let results = count_tags(&[dir.path().to_path_buf()], &[], &[])?;

        // Then
        assert_eq!(results[0].0, "ideas");
        Ok(())
    }

    #[test]
    fn test_should_exclude_specified_tags() -> Result<()> {
        // REQ-TAGS-004

        // Given
        let dir = TempDir::new()?;
        create_test_file(&dir, "a.md", "---\ntags: [writing, refactored]\n---")?;

        // When
        let results = count_tags(&[dir.path().to_path_buf()], &["refactored"], &[])?;

        // Then
        assert!(!results.iter().any(|(t, _)| t == "refactored"));
        Ok(())
    }

    #[test]
    fn test_should_scan_multiple_directories() -> Result<()> {
        // REQ-TAGS-006

        // Given
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;
        create_test_file(&dir1, "a.md", "---\ntags: [writing]\n---")?;
        create_test_file(&dir2, "b.md", "---\ntags: [writing]\n---")?;

        // When
        let results = count_tags(
            &[dir1.path().to_path_buf(), dir2.path().to_path_buf()],
            &[],
            &[],
        )?;

        // Then
        let writing_count = results.iter().find(|(t, _)| t == "writing").map(|(_, c)| *c);
        assert_eq!(writing_count, Some(2));
        Ok(())
    }

    #[test]
    fn test_should_exclude_directories() -> Result<()> {
        // REQ-TAGS-007

        // Given
        let dir = TempDir::new()?;
        let excluded = dir.path().join("excluded");
        fs::create_dir(&excluded)?;
        create_test_file(&dir, "a.md", "---\ntags: [writing]\n---")?;
        fs::write(excluded.join("b.md"), "---\ntags: [ideas]\n---")?;

        // When
        let results = count_tags(&[dir.path().to_path_buf()], &[], &["excluded"])?;

        // Then
        assert!(!results.iter().any(|(t, _)| t == "ideas"));
        Ok(())
    }
}

// ============================================
// IMPLEMENTATIONS
// ============================================

/// Count tag frequency across all markdown files in the given directories.
/// Returns tags sorted by frequency descending, excluding any tags in `exclude_tags`.
pub fn count_tags(
    dirs: &[PathBuf],
    exclude_tags: &[&str],
    exclude_dirs: &[&str],
) -> Result<Vec<(String, usize)>> {
    let mut counts: HashMap<String, usize> = HashMap::new();

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
            .filter_entry(|e| !should_exclude(e, exclude_dirs, Some(&ignore_patterns)))
        {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(frontmatter) = parse_frontmatter(&content) {
                    if let Some(tags) = frontmatter.tags {
                        for tag in tags {
                            if !exclude_tags.contains(&tag.as_str()) {
                                *counts.entry(tag).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    let mut result: Vec<(String, usize)> = counts.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    Ok(result)
}
