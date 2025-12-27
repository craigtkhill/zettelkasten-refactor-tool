// src/config.rs

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZrtConfig {
    pub refactor: RefactorConfig,

    pub tagging: Option<TaggingConfig>,
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

impl Default for ZrtConfig {
    #[inline]
    fn default() -> Self {
        Self {
            refactor: RefactorConfig::default(),

            tagging: Some(TaggingConfig::default()),
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
            exclude_tags: Vec::new(), // Empty by default
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggingConfig {
    pub enabled: bool,
}

impl Default for TaggingConfig {
    #[inline]
    fn default() -> Self {
        Self { enabled: true }
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

    /// Loads config from default location (.zrt/config.toml) or returns default if not found
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_refactor_config_default() {
        let config = RefactorConfig::default();

        assert_eq!(config.word_threshold, 300);
        assert_eq!(config.line_threshold, 60);
        assert_eq!(config.max_suggestions, 20);
        assert!(config.exclude_tags.is_empty());
        assert!(matches!(config.sort_by, SortBy::Words));
    }

    #[test]
    fn test_zrt_config_default() {
        let config = ZrtConfig::default();

        assert_eq!(config.refactor.word_threshold, 300);
        assert_eq!(config.refactor.line_threshold, 60);
    }

    #[test]
    fn test_config_save_and_load() -> Result<()> {
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
    fn test_config_load_or_default() {
        // This should return default since .zrt/config.toml likely doesn't exist in test env
        let config = ZrtConfig::load_or_default();
        assert_eq!(config.refactor.word_threshold, 300);
    }

    #[test]
    fn test_sort_by_serialization() -> Result<()> {
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
