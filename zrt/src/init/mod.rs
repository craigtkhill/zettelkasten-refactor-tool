use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ============================================
// TESTS
// ============================================
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

    #[test]
    fn test_should_have_default_refactor_config() {
        let config = RefactorConfig::default();

        assert_eq!(config.word_threshold, 300);
        assert_eq!(config.line_threshold, 60);
        assert_eq!(config.max_suggestions, 20);
        assert!(config.exclude_tags.is_empty());
        assert!(matches!(config.sort_by, SortBy::Words));
    }

    #[test]
    fn test_should_have_default_zrt_config() {
        let config = ZrtConfig::default();

        assert_eq!(config.refactor.word_threshold, 300);
        assert_eq!(config.refactor.line_threshold, 60);
    }

    #[test]
    fn test_should_save_and_load_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("config.toml");

        let mut config = ZrtConfig::default();
        config.refactor.word_threshold = 500;
        config.refactor.exclude_tags = vec!["draft".to_owned(), "private".to_owned()];

        config.save_to_file(&config_path)?;

        let loaded_config = ZrtConfig::load_from_file(&config_path)?;

        assert_eq!(loaded_config.refactor.word_threshold, 500);
        assert_eq!(loaded_config.refactor.exclude_tags.len(), 2);
        assert!(
            loaded_config
                .refactor
                .exclude_tags
                .contains(&"draft".to_owned())
        );
        assert!(
            loaded_config
                .refactor
                .exclude_tags
                .contains(&"private".to_owned())
        );

        Ok(())
    }

    #[test]
    fn test_should_load_or_default_config() {
        let config = ZrtConfig::load_or_default();
        assert_eq!(config.refactor.word_threshold, 300);
    }

    #[test]
    fn test_should_serialize_sort_by_as_lowercase() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("config.toml");

        let mut config = ZrtConfig::default();
        config.refactor.sort_by = SortBy::Lines;

        config.save_to_file(&config_path)?;

        let content = std::fs::read_to_string(&config_path)?;
        assert!(content.contains("sort_by = \"lines\""));

        let loaded_config = ZrtConfig::load_from_file(&config_path)?;
        assert!(matches!(loaded_config.refactor.sort_by, SortBy::Lines));

        Ok(())
    }
}

// ============================================
// TYPE DEFINITIONS
// ============================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZrtConfig {
    pub refactor: RefactorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorConfig {
    pub word_threshold: usize,
    pub line_threshold: usize,
    pub max_suggestions: usize,
    pub exclude_tags: Vec<String>,
    pub sort_by: SortBy,
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    Words,
    Lines,
}

// ============================================
// IMPLEMENTATIONS
// ============================================
impl Default for ZrtConfig {
    #[inline]
    fn default() -> Self {
        Self {
            refactor: RefactorConfig::default(),
        }
    }
}

impl Default for RefactorConfig {
    #[inline]
    fn default() -> Self {
        Self {
            word_threshold: 300,
            line_threshold: 60,
            max_suggestions: 20,
            exclude_tags: Vec::new(),
            sort_by: SortBy::Words,
        }
    }
}

impl Default for SortBy {
    #[inline]
    fn default() -> Self {
        Self::Words
    }
}

impl ZrtConfig {
    /// Loads configuration from a TOML file
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed
    #[inline]
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))
    }

    /// Saves configuration to a TOML file
    ///
    /// # Errors
    /// Returns an error if the file cannot be written or serialized
    #[inline]
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let content = toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))
    }

    #[inline]
    pub fn load_or_default() -> Self {
        let config_path = PathBuf::from(".zrt/config.toml");
        if config_path.exists() {
            Self::load_from_file(&config_path).unwrap_or_else(|_| {
                eprintln!("Warning: Failed to parse .zrt/config.toml, using defaults");
                Self::default()
            })
        } else {
            Self::default()
        }
    }
}

// ============================================
// PUBLIC FUNCTIONS
// ============================================
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
