use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub confidence_threshold: f32,
    pub embedding_model: String,
    pub excluded_tags: HashSet<String>,
    pub max_suggestions: usize,
    pub min_tag_examples: usize,
    pub model_path: PathBuf,
    pub training: Training,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Training {
    pub batch_size: usize,
    pub epochs: usize,
    pub learning_rate: f32,
    pub train_split: f32,
}

impl Default for Settings {
    #[inline]
    fn default() -> Self {
        Self {
            confidence_threshold: 0.7,
            embedding_model: "snowflake-arctic-embed-xs".to_owned(),
            excluded_tags: HashSet::new(),
            max_suggestions: 5,
            min_tag_examples: 5,
            model_path: PathBuf::from(".zrt/models"),
            training: Training::default(),
        }
    }
}

impl Default for Training {
    #[inline]
    fn default() -> Self {
        Self {
            batch_size: 32,
            epochs: 10,
            learning_rate: 0.001,
            train_split: 0.8,
        }
    }
}

impl Settings {
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
}
