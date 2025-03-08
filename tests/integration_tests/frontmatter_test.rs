// tests/integration_tests/frontmatter_test.rs
use super::common::setup_test_directory;
use anyhow::Result;
use zrt::{contains_tag, parse_frontmatter};

#[test]
fn test_frontmatter_parsing() -> Result<()> {
    let content = "\
---
tags: [tag1, tag2]
---
# Content
Some content here.";

    let frontmatter = parse_frontmatter(content)?;
    assert!(frontmatter.tags.is_some());
    assert_eq!(frontmatter.tags.unwrap(), vec!["tag1", "tag2"]);

    // Test with no frontmatter
    let content_no_fm = "# Just content\nNo frontmatter here.";
    let frontmatter = parse_frontmatter(content_no_fm)?;
    assert!(frontmatter.tags.is_none());

    Ok(())
}

#[test]
fn test_contains_tag() -> Result<()> {
    let temp_dir = setup_test_directory()?;

    let todo_path = temp_dir.path().join("todo.md");
    assert!(contains_tag(&todo_path, "to_refactor")?);

    let no_tags_path = temp_dir.path().join("no_tags.md");
    assert!(!contains_tag(&no_tags_path, "to_refactor")?);

    Ok(())
}
