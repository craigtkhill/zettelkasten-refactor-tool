use zrt::{
    count_files,
    count_words,
    scan_directory_single_pattern,
    scan_directory_two_patterns,
    parse_frontmatter,
    contains_tag
};
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use std::io::Write;
use anyhow::Result;

fn create_test_file(dir: &Path, name: &str, content: &str) -> Result<()> {
    let path = dir.join(name);
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn setup_test_directory() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    println!("Created temp directory at: {:?}", temp_dir.path());

    // Create a file with no tags
    create_test_file(
        temp_dir.path(),
        "no_tags.md",
        "# Test File\nThis is a test file with no tags."
    )?;
    println!("Created no_tags.md");

    // Create a file with to_refactor tag
    create_test_file(
        temp_dir.path(),
        "todo.md",
        "---\ntags: [to_refactor]\n---\n# Todo\nThis needs work."
    )?;
    println!("Created todo.md");

    // Create a file with refactored tag
    create_test_file(
        temp_dir.path(),
        "done.md",
        "---\ntags: [refactored]\n---\n# Done\nThis is complete."
    )?;
    println!("Created done.md");

    // Create a file with both tags
    create_test_file(
        temp_dir.path(),
        "both.md",
        "---\ntags: [to_refactor, refactored]\n---\n# Both\nConfusing state."
    )?;
    println!("Created both.md");

    // List all files in the directory
    println!("\nListing all files in temp directory:");
    for entry in std::fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        println!("  {:?}", entry.path());
    }
    println!("");

    Ok(temp_dir)
}

#[test]
fn test_count_files() -> Result<()> {
    let temp_dir = setup_test_directory()?;
    let exclude_dirs = vec![".git"];

    let count = count_files(&temp_dir.path().to_path_buf(), &exclude_dirs)?;
    assert_eq!(count, 4, "Should count all test files");

    Ok(())
}

#[test]
fn test_word_count() -> Result<()> {
    let temp_dir = setup_test_directory()?;
    let exclude_dirs = vec![".git"];

    let files = count_words(
        &temp_dir.path().to_path_buf(),
        &exclude_dirs,
        None
    )?;

    assert_eq!(files.len(), 4, "Should count words in all files");

    // Test with filter
    let filtered_files = count_words(
        &temp_dir.path().to_path_buf(),
        &exclude_dirs,
        Some("refactored")
    )?;

    assert_eq!(filtered_files.len(), 2, "Should exclude files with 'refactored' tag");

    Ok(())
}

#[test]
fn test_single_pattern_scan() -> Result<()> {
    let temp_dir = setup_test_directory()?;

    let stats = scan_directory_single_pattern(
        &temp_dir.path().to_path_buf(),
        "to_refactor"
    )?;

    assert_eq!(stats.total_files, 4, "Should count all files");
    assert_eq!(stats.files_with_pattern, 2, "Should find two files with to_refactor tag");
    assert!((stats.calculate_percentage() - 50.0).abs() < f64::EPSILON);

    Ok(())
}

#[test]
fn test_two_pattern_scan() -> Result<()> {
    let temp_dir = setup_test_directory()?;

    let stats = scan_directory_two_patterns(
        &temp_dir.path().to_path_buf(),
        "refactored",
        "to_refactor"
    )?;

    assert_eq!(stats.total_files, 4, "Should count all files");
    assert_eq!(stats.done_files, 2, "Should find two files with refactored tag");
    assert_eq!(stats.todo_files, 2, "Should find two files with to_refactor tag");
    assert!((stats.calculate_percentage() - 50.0).abs() < f64::EPSILON);

    Ok(())
}

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