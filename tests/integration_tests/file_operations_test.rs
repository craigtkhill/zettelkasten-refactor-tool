// tests/integration_tests/file_operations_test.rs
use super::common::{create_ignore_file, setup_test_directory};
use anyhow::Result;
use zrt::count_files;

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
