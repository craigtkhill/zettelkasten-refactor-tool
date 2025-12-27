use anyhow::{Result, anyhow};
use serde::Deserialize;

// ============================================
// TESTS
// ============================================
#[cfg(test)]
mod tests {
    use super::*;

    // Parse frontmatter tests
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

    // Frontmatter model tests
    #[test]
    fn test_frontmatter_deserialize() {
        let yaml = "
            tags:
              - tag1
              - tag2
        ";
        let frontmatter: Frontmatter = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(frontmatter.tags.unwrap(), vec!["tag1", "tag2"]);
    }

    #[test]
    fn test_frontmatter_no_tags() {
        let yaml = "{}";
        let frontmatter: Frontmatter = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(frontmatter.tags.is_none());
    }

    // Strip frontmatter tests
    #[test]
    fn test_should_return_body_when_frontmatter_present() {
        // REQ-STRIP-001
        let content = "---\ntags: [refactor]\n---\nBody content";
        assert_eq!(strip_frontmatter(content), "\nBody content");
    }

    #[test]
    fn test_should_return_original_when_no_frontmatter() {
        // REQ-STRIP-002
        let content = "Just body content";
        assert_eq!(strip_frontmatter(content), "Just body content");
    }

    #[test]
    fn test_should_return_original_when_frontmatter_incomplete() {
        // REQ-STRIP-003
        let content = "---\ntags: [refactor]\nNo closing";
        assert_eq!(strip_frontmatter(content), content);
    }

    #[test]
    fn test_should_return_empty_when_only_frontmatter() {
        // REQ-STRIP-004
        let content = "---\ntags: [refactor]\n---";
        assert_eq!(strip_frontmatter(content), "");
    }
}

// ============================================
// TYPE DEFINITIONS
// ============================================

#[derive(Deserialize, Debug, Default)]
pub struct Frontmatter {
    pub tags: Option<Vec<String>>,
}

// ============================================
// IMPLEMENTATIONS
// ============================================

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

/// Strip YAML frontmatter from content and return body only
///
/// Frontmatter is identified by starting with `---` and ending with another `---` line.
/// If no valid frontmatter is found, returns the original content.
pub fn strip_frontmatter(content: &str) -> &str {
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
