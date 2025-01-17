use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use zrt::{
    contains_tag, count_files, count_words, load_ignore_patterns, parse_frontmatter,
    scan_directory_single_pattern, scan_directory_two_patterns,
};

fn create_test_file(dir: &Path, name: &str, content: &str) -> Result<()> {
    let path = dir.join(name);
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn create_ignore_file(dir: &Path, patterns: &[&str]) -> Result<()> {
    let content = patterns.join("\n");
    create_test_file(dir, ".zrtignore", &content)
}

fn setup_test_directory() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    println!("Created temp directory at: {:?}", temp_dir.path());

    // Create standard test files
    create_test_file(
        temp_dir.path(),
        "no_tags.md",
        "# Test File\nThis is a test file with no tags.",
    )?;

    create_test_file(
        temp_dir.path(),
        "todo.md",
        "---\ntags: [to_refactor]\n---\n# Todo\nThis needs work.",
    )?;

    create_test_file(
        temp_dir.path(),
        "done.md",
        "---\ntags: [refactored]\n---\n# Done\nThis is complete.",
    )?;

    create_test_file(
        temp_dir.path(),
        "both.md",
        "---\ntags: [to_refactor, refactored]\n---\n# Both\nConfusing state.",
    )?;

    // Create files that should be ignored
    create_test_file(temp_dir.path(), "ignore_me.tmp", "Temporary file")?;

    create_test_file(temp_dir.path(), "draft/draft.md", "Draft document")?;

    create_test_file(temp_dir.path(), "cache/data.cache", "Cached data")?;

    create_test_file(
        temp_dir.path(),
        "node_modules/package.json",
        r#"{"name": "test"}"#,
    )?;

    // List all files in the directory
    println!("\nListing all files in temp directory:");
    for entry in fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        println!("  {:?}", entry.path());
    }
    println!();

    Ok(temp_dir)
}

#[test]
fn test_basic_file_operations() -> Result<()> {
    let temp_dir = setup_test_directory()?;
    let exclude_dirs = vec![".git"];

    // Test without ignore file
    let count = count_files(&temp_dir.path().to_path_buf(), &exclude_dirs)?;
    assert_eq!(count, 8, "Should count all test files without ignore file");

    // Create and test with ignore file
    create_ignore_file(
        temp_dir.path(),
        &["*.tmp", "draft/", "cache/", "node_modules/"],
    )?;

    let count_with_ignore = count_files(&temp_dir.path().to_path_buf(), &exclude_dirs)?;
    assert_eq!(count_with_ignore, 4, "Should only count non-ignored files");

    Ok(())
}

#[test]
fn test_word_counting() -> Result<()> {
    let temp_dir = setup_test_directory()?;
    let exclude_dirs = vec![".git"];

    // Create ignore file
    create_ignore_file(
        temp_dir.path(),
        &["*.tmp", "draft/", "cache/", "node_modules/"],
    )?;

    // Test word counting with ignore patterns
    let files = count_words(&temp_dir.path().to_path_buf(), &exclude_dirs, None)?;

    assert_eq!(
        files.len(),
        4,
        "Should count words only in non-ignored files"
    );

    // Test with tag filter
    let filtered_files = count_words(
        &temp_dir.path().to_path_buf(),
        &exclude_dirs,
        Some("refactored"),
    )?;

    assert_eq!(
        filtered_files.len(),
        2,
        "Should respect both ignore patterns and tag filters"
    );

    Ok(())
}

#[test]
fn test_ignore_patterns() -> Result<()> {
    let temp_dir = setup_test_directory()?;

    // Test comprehensive set of ignore patterns
    create_ignore_file(
        temp_dir.path(),
        &[
            "# Comment line",
            "*.tmp",
            "draft/",
            "!draft/important.md",
            "cache/",
            "*.{log,cache,tmp}",
            "node_modules/",
            "/absolute_path.md",
            "*.bak",
            "build/**/*.js",
            "*.pdf",
        ],
    )?;

    let patterns = load_ignore_patterns(temp_dir.path())?;

    // Test pattern matching
    assert!(patterns.matches("test.tmp"), "Should match *.tmp pattern");
    assert!(
        patterns.matches("draft/test.md"),
        "Should match draft/ pattern"
    );
    assert!(
        !patterns.matches("draft/important.md"),
        "Should respect negation pattern"
    );
    assert!(
        !patterns.matches("test.md"),
        "Should not match non-ignored file"
    );
    assert!(
        patterns.matches("test.log"),
        "Should match multiple extensions pattern"
    );
    assert!(
        patterns.matches("test.cache"),
        "Should match multiple extensions pattern"
    );
    assert!(
        patterns.matches("node_modules/package.json"),
        "Should match directory pattern"
    );
    assert!(
        patterns.matches("build/src/main.js"),
        "Should match globstar pattern"
    );
    assert!(
        !patterns.matches("src/main.js"),
        "Should not match files outside build/"
    );
    assert!(patterns.matches("test.bak"), "Should match *.bak pattern");

    // Test absolute path matching
    assert!(
        patterns.matches("/absolute_path.md"),
        "Should match absolute path"
    );
    assert!(
        !patterns.matches("subdirectory/absolute_path.md"),
        "Should not match relative path"
    );

    Ok(())
}

#[test]
fn test_scanning_with_ignore() -> Result<()> {
    let temp_dir = setup_test_directory()?;

    create_ignore_file(
        temp_dir.path(),
        &["*.tmp", "draft/", "cache/", "node_modules/"],
    )?;

    // Test single pattern scan
    let single_stats =
        scan_directory_single_pattern(&temp_dir.path().to_path_buf(), "to_refactor")?;

    assert_eq!(
        single_stats.total_files, 4,
        "Should count only non-ignored files"
    );
    assert_eq!(
        single_stats.files_with_pattern, 2,
        "Should find correct number of tagged files"
    );
    assert!((single_stats.calculate_percentage() - 50.0).abs() < f64::EPSILON);

    // Test two pattern scan
    let dual_stats =
        scan_directory_two_patterns(&temp_dir.path().to_path_buf(), "refactored", "to_refactor")?;

    assert_eq!(
        dual_stats.total_files, 4,
        "Should count only non-ignored files"
    );
    assert_eq!(
        dual_stats.done_files, 2,
        "Should find correct number of done files"
    );
    assert_eq!(
        dual_stats.todo_files, 2,
        "Should find correct number of todo files"
    );
    assert!((dual_stats.calculate_percentage() - 50.0).abs() < f64::EPSILON);

    Ok(())
}

#[test]
fn test_edge_cases() -> Result<()> {
    let temp_dir = setup_test_directory()?;

    // Test empty ignore file
    create_ignore_file(temp_dir.path(), &[])?;
    let count_empty_ignore = count_files(&temp_dir.path().to_path_buf(), &[".git"])?;
    assert_eq!(
        count_empty_ignore, 8,
        "Empty ignore file should not exclude any files"
    );

    // Test ignore file with only comments and empty lines
    create_ignore_file(
        temp_dir.path(),
        &["# Comment 1", "", "  # Comment 2  ", "     ", "# Comment 3"],
    )?;
    let count_comment_only = count_files(&temp_dir.path().to_path_buf(), &[".git"])?;
    assert_eq!(
        count_comment_only, 8,
        "Comments should not exclude any files"
    );

    // Test complex patterns
    create_ignore_file(
        temp_dir.path(),
        &[
            "*.{tmp,bak,swp}",
            "**/*.log",
            "!/important/**/*.log",
            ".*/",
            "build/*/temp/",
            "**/node_modules/**",
        ],
    )?;

    let patterns = load_ignore_patterns(temp_dir.path())?;

    assert!(patterns.matches("test.tmp"), "Should match extension group");
    assert!(
        patterns.matches("deep/nested/file.log"),
        "Should match double globstar"
    );
    assert!(
        !patterns.matches("important/logs/app.log"),
        "Should respect negation with globstar"
    );
    assert!(patterns.matches(".git/"), "Should match dot directories");
    assert!(
        patterns.matches("build/debug/temp/"),
        "Should match nested directories"
    );
    assert!(
        patterns.matches("packages/node_modules/file.js"),
        "Should match nested node_modules"
    );

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
