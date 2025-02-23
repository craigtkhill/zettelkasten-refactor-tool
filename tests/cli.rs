use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;
use zrt::Args; // Note: using the library crate

fn create_test_file(dir: &TempDir, name: &str, content: &str) -> Result<PathBuf> {
    let file_path = dir.path().join(name);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(&file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(file_path)
}

fn setup_test_directory() -> Result<TempDir> {
    let dir = TempDir::new()?;

    // Create some regular files
    create_test_file(&dir, "file1.md", "This is file one with some content")?;
    create_test_file(&dir, "file2.md", "This is file two\nWith more words here")?;

    // Create files with tags
    create_test_file(
        &dir,
        "done.md",
        "---\ntags: [done]\n---\nThis is a completed file",
    )?;
    create_test_file(
        &dir,
        "todo.md",
        "---\ntags: [to_refactor]\n---\nThis needs work",
    )?;
    create_test_file(
        &dir,
        "both.md",
        "---\ntags: [done, to_refactor]\n---\nMixed status",
    )?;

    // Create a file in a subdirectory
    create_test_file(&dir, "subdir/nested.md", "Nested file content")?;

    Ok(dir)
}

#[test]
fn test_count_files() -> Result<()> {
    let dir = setup_test_directory()?;

    let args = Args {
        directory: dir.path().to_path_buf(),
        count: true,
        words: false,
        top: 10,
        exclude: String::from(""),
        filter_out: None,
        pattern: None,
        done_tag: None,
        todo_tag: None,
    };

    zrt::run(args)?;
    Ok(())
}

#[test]
fn test_word_count_with_filter() -> Result<()> {
    let dir = setup_test_directory()?;

    let args = Args {
        directory: dir.path().to_path_buf(),
        count: false,
        words: true,
        top: 2,
        exclude: String::from(""),
        filter_out: Some(String::from("done")),
        pattern: None,
        done_tag: None,
        todo_tag: None,
    };

    zrt::run(args)?;
    Ok(())
}

#[test]
fn test_pattern_search() -> Result<()> {
    let dir = setup_test_directory()?;

    let args = Args {
        directory: dir.path().to_path_buf(),
        count: false,
        words: false,
        top: 10,
        exclude: String::from(""),
        filter_out: None,
        pattern: Some(String::from("to_refactor")),
        done_tag: None,
        todo_tag: None,
    };

    zrt::run(args)?;
    Ok(())
}

#[test]
fn test_done_vs_todo_comparison() -> Result<()> {
    let dir = setup_test_directory()?;

    let args = Args {
        directory: dir.path().to_path_buf(),
        count: false,
        words: false,
        top: 10,
        exclude: String::from(""),
        filter_out: None,
        pattern: None,
        done_tag: Some(String::from("done")),
        todo_tag: Some(String::from("to_refactor")),
    };

    zrt::run(args)?;
    Ok(())
}
