// tests/integration_tests/ignore_patterns_test.rs
use super::common::{create_ignore_file, setup_test_directory};
use anyhow::Result;
use zrt::load_ignore_patterns;

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
            "/absolute_path.md", // This means 'absolute_path.md' at root only
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
        patterns.matches("absolute_path.md"),
        "Should match absolute path at root"
    );
    assert!(
        !patterns.matches("subdirectory/absolute_path.md"),
        "Should not match absolute path in subdirectory"
    );

    Ok(())
}
