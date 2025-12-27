// src/core/strip_frontmatter/mod.rs

// ============================================
// TESTS
// ============================================
#[cfg(test)]
mod tests {
    use super::*;

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

// ============================================
// IMPLEMENTATIONS
// ============================================

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
