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
    pub predictor_type: PredictorType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictorType {
    TfIdf,
    MlEmbedding,
    EmbeddingKnn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Training {
    pub batch_size: usize,
    pub epochs: usize,
    pub learning_rate: f32,
    pub train_split: f32,
    pub random_seed: Option<u64>,
}

impl Default for Settings {
    #[inline]
    fn default() -> Self {
        Self {
            confidence_threshold: 0.9,
            embedding_model: "snowflake-arctic-embed-xs".to_owned(),
            excluded_tags: {
                let mut tags = HashSet::new();
                tags.insert("refactored".to_owned());
                tags.insert("to_refactor".to_owned());
                tags
            },
            max_suggestions: 5,
            min_tag_examples: 5,
            model_path: PathBuf::from(".zrt/models"),
            training: Training::default(),
            predictor_type: PredictorType::EmbeddingKnn,
        }
    }
}

impl Default for Training {
    #[inline]
    fn default() -> Self {
        Self {
            batch_size: 16,
            epochs: 1,
            learning_rate: 0.001,
            train_split: 0.8,
            random_seed: Some(42),
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();

        assert_eq!(settings.confidence_threshold, 0.9);
        assert_eq!(settings.embedding_model, "snowflake-arctic-embed-xs");
        assert_eq!(settings.excluded_tags.len(), 2);
        assert!(settings.excluded_tags.contains("refactored"));
        assert!(settings.excluded_tags.contains("to_refactor"));
        assert_eq!(settings.max_suggestions, 5);
        assert_eq!(settings.min_tag_examples, 5);
        assert_eq!(settings.model_path, PathBuf::from(".zrt/models"));

        assert_eq!(settings.training.batch_size, 16);
        assert_eq!(settings.training.epochs, 1);
        assert_eq!(settings.training.learning_rate, 0.001);
        assert_eq!(settings.training.train_split, 0.8);
        assert_eq!(settings.training.random_seed, Some(42));
        assert!(matches!(
            settings.predictor_type,
            PredictorType::EmbeddingKnn
        ));
    }

    #[test]
    fn test_training_default() {
        let training = Training::default();

        assert_eq!(training.batch_size, 16);
        assert_eq!(training.epochs, 1);
        assert_eq!(training.learning_rate, 0.001);
        assert_eq!(training.train_split, 0.8);
        assert_eq!(training.random_seed, Some(42));
    }

    #[test]
    fn test_settings_save_and_load() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("config.toml");

        // Create modified settings
        let mut settings = Settings::default();
        settings.confidence_threshold = 0.8;
        settings.max_suggestions = 10;
        settings.excluded_tags.insert("draft".to_owned());
        settings.excluded_tags.insert("private".to_owned());
        settings.training.epochs = 20;

        // Save settings
        settings.save_to_file(&config_path)?;

        // Load settings back
        let loaded_settings = Settings::load_from_file(&config_path)?;

        assert_eq!(loaded_settings.confidence_threshold, 0.8);
        assert_eq!(loaded_settings.max_suggestions, 10);
        assert!(loaded_settings.excluded_tags.contains("draft"));
        assert!(loaded_settings.excluded_tags.contains("private"));
        assert_eq!(loaded_settings.excluded_tags.len(), 4);
        assert!(loaded_settings.excluded_tags.contains("refactored"));
        assert!(loaded_settings.excluded_tags.contains("to_refactor"));
        assert_eq!(loaded_settings.training.epochs, 20);

        Ok(())
    }

    #[test]
    fn test_settings_load_nonexistent_file() {
        let result = Settings::load_from_file(&PathBuf::from("/nonexistent/config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_settings_save_creates_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("directory")
            .join("config.toml");

        let settings = Settings::default();
        settings.save_to_file(&nested_path)?;

        assert!(nested_path.exists());

        // Verify we can load it back
        let loaded = Settings::load_from_file(&nested_path)?;
        assert_eq!(loaded.confidence_threshold, settings.confidence_threshold);

        Ok(())
    }

    #[test]
    fn test_settings_serialization_format() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("config.toml");

        let mut settings = Settings::default();
        settings.excluded_tags.insert("test".to_owned());

        settings.save_to_file(&config_path)?;

        let content = std::fs::read_to_string(&config_path)?;

        // Check that the TOML contains expected fields
        assert!(content.contains("confidence_threshold"));
        assert!(content.contains("embedding_model"));
        assert!(content.contains("excluded_tags"));
        assert!(content.contains("max_suggestions"));
        assert!(content.contains("min_tag_examples"));
        assert!(content.contains("[training]"));
        assert!(content.contains("batch_size"));
        assert!(content.contains("epochs"));

        Ok(())
    }
}
