use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::settings::ZrtConfig;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_should_create_zrt_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        run(Some(temp_dir.path()))?;

        let zrt_exists = temp_dir.path().join(".zrt").exists();
        assert!(zrt_exists);
        Ok(())
    }

    #[test]
    fn test_should_create_config_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        run(Some(temp_dir.path()))?;

        let config_exists = temp_dir.path().join(".zrt/config.toml").exists();
        assert!(config_exists);
        Ok(())
    }

    #[test]
    fn test_should_succeed_when_directory_already_exists() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::fs::create_dir_all(temp_dir.path().join(".zrt"))?;

        let result = run(Some(temp_dir.path()));
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_should_not_overwrite_existing_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        std::fs::create_dir_all(temp_dir.path().join(".zrt"))?;
        let config_path = temp_dir.path().join(".zrt/config.toml");
        std::fs::write(&config_path, "test content")?;

        run(Some(temp_dir.path()))?;

        let content = std::fs::read_to_string(&config_path)?;
        assert_eq!(content, "test content");
        Ok(())
    }
}

/// Initialize ZRT configuration directory and files.
///
/// Creates `.zrt/` directory and `config.toml` with default refactor thresholds.
///
/// # Arguments
///
/// * `base_path` - Optional base directory path. If `None`, uses current directory.
///
/// # Returns
///
/// * `Ok(())` if initialization succeeds
///
/// # Errors
///
/// Returns an error if directory creation or file writing fails.
pub fn run(base_path: Option<&Path>) -> Result<()> {
    let zrt_dir = base_path
        .map(|p| p.join(".zrt"))
        .unwrap_or_else(|| PathBuf::from(".zrt"));

    if zrt_dir.exists() {
        println!("config directory already exists at .zrt/");
        return Ok(());
    }

    std::fs::create_dir_all(&zrt_dir)?;

    let config = ZrtConfig::default();
    config.save_to_file(&zrt_dir.join("config.toml"))?;

    println!("Initialized config directory at .zrt/");

    Ok(())
}
