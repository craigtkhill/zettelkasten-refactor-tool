// src/utils.rs
use crate::models::{FileWordCount, Frontmatter};
use anyhow::{Result, anyhow};
use std::fs;
use std::io;
use std::path::Path;

/// Checks if a file contains a specific tag in its frontmatter.
///
/// # Arguments
///
/// * `path` - Path to the file to check
/// * `tag` - The tag to search for
///
/// # Returns
///
/// * `Ok(bool)` - True if the file contains the tag, false otherwise
///
/// # Errors
///
/// This function may return an error if:
/// * The file cannot be read
/// * File system operations fail
#[inline]
pub fn contains_tag(path: &Path, tag: &str) -> io::Result<bool> {
    let content = fs::read_to_string(path)?;

    match parse_frontmatter(&content) {
        Ok(frontmatter) => Ok(frontmatter
            .tags
            .is_some_and(|tags| tags.iter().any(|t| t == tag))),
        Err(_) => Ok(false), // If parsing fails, assume no tags
    }
}

/// Checks if a file contains only the specified tag and no other tags.
///
/// # Arguments
///
/// * `path` - Path to the file to check
/// * `tag` - The tag to search for as the only tag
///
/// # Returns
///
/// * `Ok(bool)` - True if the file contains only the specified tag, false otherwise
///
/// # Errors
///
/// This function may return an error if:
/// * The file cannot be read
/// * File system operations fail
#[inline]
pub fn has_only_tag(path: &Path, tag: &str) -> io::Result<bool> {
    let content = fs::read_to_string(path)?;

    match parse_frontmatter(&content) {
        Ok(frontmatter) => frontmatter.tags.map_or(Ok(false), |tags| {
            Ok(tags.len() == 1 && tags.contains(&tag.to_owned()))
        }),
        Err(_) => Ok(false),
    }
}

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

#[inline]
pub fn print_top_files(files: &[FileWordCount], top: usize) {
    for file in files.iter().take(top) {
        println!("{:8} words  {}", file.words, file.path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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

    #[test]
    fn test_print_top_files() {
        let files = vec![
            FileWordCount {
                path: PathBuf::from("test.txt"),
                words: 100,
            },
            FileWordCount {
                path: PathBuf::from("test2.txt"),
                words: 50,
            },
        ];

        // Here we could capture stdout to verify the output format
        print_top_files(&files, 1);
    }
}

#[cfg(test)]
mod file_tests {
    use super::*;
    use std::fs::File;
    use std::io::Write as _;
    use tempfile::TempDir;

    #[test]
    fn test_contains_tag() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.md");
        let content = "---
tags:
  - test_tag
---
Content";

        let mut file = File::create(&file_path)?;
        file.write_all(content.as_bytes())?;

        assert!(contains_tag(&file_path, "test_tag")?);
        Ok(())
    }

    #[test]
    fn test_contains_tag_no_tags() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.md");
        let content = "Just content, no frontmatter";

        let mut file = File::create(&file_path)?;
        file.write_all(content.as_bytes())?;

        assert!(!contains_tag(&file_path, "test_tag")?);
        Ok(())
    }

    #[test]
    fn test_has_only_tag() -> Result<()> {
        let temp_dir = TempDir::new()?;

        let single_tag_file = temp_dir.path().join("single.md");
        let content = "---\ntags: [refactored]\n---\nContent";
        let mut file = File::create(&single_tag_file)?;
        file.write_all(content.as_bytes())?;

        let multi_tag_file = temp_dir.path().join("multi.md");
        let content = "---\ntags: [refactored, reviewed]\n---\nContent";
        let mut file = File::create(&multi_tag_file)?;
        file.write_all(content.as_bytes())?;

        let no_tag_file = temp_dir.path().join("none.md");
        let content = "Just content, no frontmatter";
        let mut file = File::create(&no_tag_file)?;
        file.write_all(content.as_bytes())?;

        assert!(has_only_tag(&single_tag_file, "refactored")?);
        assert!(!has_only_tag(&multi_tag_file, "refactored")?);
        assert!(!has_only_tag(&no_tag_file, "refactored")?);

        Ok(())
    }
}
