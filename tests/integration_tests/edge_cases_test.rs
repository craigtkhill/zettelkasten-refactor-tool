// tests/integration_tests/edge_cases_test.rs
use super::common::{create_ignore_file, setup_test_directory};
use anyhow::Result;
use zrt::{count_files, load_ignore_patterns};

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
