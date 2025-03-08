// tests/integration_tests/word_counting_test.rs
use super::common::{create_ignore_file, setup_test_directory};
use anyhow::Result;
use zrt::count_words;

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
