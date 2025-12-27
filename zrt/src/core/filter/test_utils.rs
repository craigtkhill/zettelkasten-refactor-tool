use anyhow::Result;
use std::fs::{self, File};
use std::io::Write as _;
use std::path::PathBuf;
use tempfile::TempDir;

pub fn create_test_file(dir: &TempDir, name: &str, content: &str) -> Result<PathBuf> {
    let file_path = dir.path().join(name);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(&file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(file_path)
}

pub fn setup_test_directory() -> Result<TempDir> {
    let dir = TempDir::new()?;

    create_test_file(&dir, "file1.md", "This is file one")?;
    create_test_file(&dir, "file2.md", "This is file two\nWith more words")?;
    create_test_file(&dir, "nested/file3.md", "File in subfolder")?;

    create_test_file(
        &dir,
        "tagged.md",
        "---\ntags: [test, draft]\n---\nThis is a tagged file",
    )?;

    create_test_file(&dir, ".hidden.md", "Hidden file")?;

    Ok(dir)
}
