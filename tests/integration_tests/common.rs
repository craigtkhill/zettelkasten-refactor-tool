// tests/integration_tests/common.rs
use anyhow::Result;
use std::fs;
use std::io::Write as _;
use std::path::Path;
use tempfile::TempDir;

pub fn create_test_file(dir: &Path, name: &str, content: &str) -> Result<()> {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn create_ignore_file(dir: &Path, patterns: &[&str]) -> Result<()> {
    let content = patterns.join("\n");
    create_test_file(dir, ".zrtignore", &content)
}

pub fn setup_test_directory() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    println!("Created temp directory at: {:?}", temp_dir.path());

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

    create_test_file(temp_dir.path(), "ignore_me.tmp", "Temporary file")?;

    create_test_file(temp_dir.path(), "draft/draft.md", "Draft document")?;

    create_test_file(temp_dir.path(), "cache/data.cache", "Cached data")?;

    create_test_file(
        temp_dir.path(),
        "node_modules/package.json",
        r#"{"name": "test"}"#,
    )?;

    println!("\nListing all files in temp directory:");
    for entry in fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        println!("  {:?}", entry.path());
    }
    println!();

    Ok(temp_dir)
}
