use crate::models::Frontmatter;
use anyhow::{Result, anyhow};

#[inline]
#[must_use]
pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name().to_str().is_some_and(|s| {
        // Don't consider temp directories as hidden
        if s.starts_with(".tmp") {
            return false;
        }
        s.starts_with('.')
    })
}

/// Parses YAML frontmatter from markdown content.
///
/// Frontmatter must be enclosed between `---` delimiters at the start of the content.
///
/// # Arguments
///
/// * `content` - The string content to parse
///
/// # Returns
///
/// * `Ok(Frontmatter)` - The parsed frontmatter, or a default empty frontmatter if none exists
///
/// # Errors
///
/// This function may return an error if:
/// * The frontmatter contains invalid YAML syntax
/// * The YAML cannot be deserialized into the Frontmatter struct
#[inline]
pub fn parse_frontmatter(content: &str) -> Result<Frontmatter> {
    let mut content_iter = content.lines();

    // Check for frontmatter delimiter
    if content_iter.next() != Some("---") {
        return Ok(Frontmatter::default());
    }

    // Collect frontmatter content
    let mut frontmatter_str = String::new();
    for line in content_iter {
        if line == "---" {
            break;
        }
        frontmatter_str.push_str(line);
        frontmatter_str.push('\n');
    }

    // Parse YAML
    serde_yaml_ng::from_str(&frontmatter_str)
        .map_err(|e| anyhow!("Failed to parse front matter: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_empty_file() {
        let content = "";
        let result = parse_frontmatter(content).unwrap();
        assert!(result.tags.is_none());
    }

    #[test]
    fn test_parse_frontmatter_no_delimiter() {
        let content = "Some content without frontmatter";
        let result = parse_frontmatter(content).unwrap();
        assert!(result.tags.is_none());
    }

    #[test]
    fn test_parse_frontmatter_with_tags() {
        let content = "---
tags:
  - tag1
  - tag2
---
Content here";
        let result = parse_frontmatter(content).unwrap();
        assert_eq!(result.tags.unwrap(), vec!["tag1", "tag2"]);
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs::File;
        use tempfile::TempDir;
        use walkdir::WalkDir;

        #[test]
        fn test_is_hidden() -> Result<()> {
            let temp_dir = TempDir::new()?;

            // Create test files
            File::create(temp_dir.path().join(".hidden"))?;
            File::create(temp_dir.path().join(".tmp_file"))?;
            File::create(temp_dir.path().join("normal.txt"))?;

            // Test each file using WalkDir
            let mut entries: Vec<_> = WalkDir::new(temp_dir.path())
                .into_iter()
                .filter_map(core::result::Result::ok)
                .collect();
            entries.sort_by_key(|e| e.path().to_path_buf());

            // Test hidden file
            let hidden = entries.iter().find(|e| e.file_name() == ".hidden").unwrap();
            assert!(is_hidden(hidden));

            // Test temp file
            let temp = entries
                .iter()
                .find(|e| e.file_name() == ".tmp_file")
                .unwrap();
            assert!(!is_hidden(temp));

            // Test normal file
            let normal = entries
                .iter()
                .find(|e| e.file_name() == "normal.txt")
                .unwrap();
            assert!(!is_hidden(normal));

            Ok(())
        }
    }
}
