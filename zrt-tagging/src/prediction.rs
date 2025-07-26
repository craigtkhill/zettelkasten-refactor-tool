use anyhow::{Context as _, Result};
use std::collections::HashMap;

use crate::config::Settings;
use crate::embedding::Model as EmbeddingModel;
use crate::extraction::TrainingData;
use crate::model::TagClassifier;

#[derive(Debug, Clone)]
pub struct Prediction {
    pub confidence: f32,
    pub tag: String,
}

pub struct Predictor {
    classifiers: HashMap<String, TagClassifier>,
    embedding_model: EmbeddingModel,
    settings: Settings,
}

impl Predictor {
    /// Creates a new tag predictor
    ///
    /// # Errors
    /// Returns an error if initialization fails
    #[inline]
    pub fn new(settings: Settings) -> Result<Self> {
        let embedding_model =
            EmbeddingModel::new().context("Failed to initialize embedding model")?;

        Ok(Self {
            classifiers: HashMap::new(),
            embedding_model,
            settings,
        })
    }

    /// Predicts tags for the given content
    ///
    /// # Errors
    /// Returns an error if prediction fails
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "Development: complex function"
    )]
    pub fn predict(&self, content: &str) -> Result<Vec<Prediction>> {
        // Generate embedding for the content
        let embedding = self
            .embedding_model
            .embed(content)
            .context("Failed to generate embedding")?;

        let mut predictions = Vec::new();

        // Run prediction through each classifier
        for (tag, classifier) in &self.classifiers {
            let confidence = classifier
                .predict(&embedding)
                .with_context(|| format!("Failed to predict for tag: {tag}"))?;

            // Only include predictions above threshold
            if confidence >= self.settings.confidence_threshold {
                predictions.push(Prediction {
                    tag: tag.clone(),
                    confidence,
                });
            }
        }

        // Sort by confidence (highest first)
        predictions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(core::cmp::Ordering::Equal)
        });

        // Limit to max suggestions
        predictions.truncate(self.settings.max_suggestions);

        Ok(predictions)
    }

    /// Trains the predictor with the given training data
    ///
    /// # Errors
    /// Returns an error if training fails
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "Development: complex training function"
    )]
    pub fn train(&mut self, training_data: &TrainingData) -> Result<()> {
        println!("Starting training with {} notes", training_data.notes.len());

        // Generate embeddings for all notes
        println!("Generating embeddings...");
        let mut embeddings = Vec::new();
        for note in &training_data.notes {
            let embedding = self
                .embedding_model
                .embed(&note.content)
                .with_context(|| format!("Failed to embed note: {}", note.path))?;
            embeddings.push(embedding);
        }

        println!("Generated {} embeddings", embeddings.len());

        // Get all unique tags
        let all_tags = training_data.get_all_tags();
        println!("Found {} unique tags", all_tags.len());

        // Train a binary classifier for each tag
        for tag in &all_tags {
            // Skip excluded tags
            if self.settings.excluded_tags.contains(tag) {
                println!("Skipping excluded tag: {tag}");
                continue;
            }

            // Check minimum examples requirement
            let positive_count = training_data
                .notes
                .iter()
                .filter(|note| note.tags.contains(tag))
                .count();

            if positive_count < self.settings.min_tag_examples {
                println!(
                    "Skipping tag '{tag}' (only {positive_count} examples, need {})",
                    self.settings.min_tag_examples
                );
                continue;
            }

            println!("Training classifier for tag: {tag} ({positive_count} positive examples)");

            // Create labels for this tag
            let labels: Vec<bool> = training_data
                .notes
                .iter()
                .map(|note| note.tags.contains(tag))
                .collect();

            // Create and train classifier
            let mut classifier = TagClassifier::new(EmbeddingModel::embedding_dim())
                .with_context(|| format!("Failed to create classifier for tag: {tag}"))?;

            classifier
                .train(&embeddings, &labels, &self.settings.training)
                .with_context(|| format!("Failed to train classifier for tag: {tag}"))?;

            // Save the trained classifier
            let model_path = self.settings.model_path.join(format!("{tag}.safetensors"));
            classifier
                .save(&model_path)
                .with_context(|| format!("Failed to save classifier for tag: {tag}"))?;

            // Store in memory for immediate use
            self.classifiers.insert(tag.clone(), classifier);

            println!("Successfully trained and saved classifier for: {tag}");
        }

        println!(
            "Training completed! Trained {} classifiers",
            self.classifiers.len()
        );
        Ok(())
    }

    /// Loads trained classifiers from disk
    ///
    /// # Errors
    /// Returns an error if loading fails
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "Development: file I/O function"
    )]
    #[expect(
        clippy::default_numeric_fallback,
        reason = "Development: simple counter"
    )]
    pub fn load_classifiers(&mut self) -> Result<()> {
        let model_dir = &self.settings.model_path;

        if !model_dir.exists() {
            return Err(anyhow::anyhow!(
                "Model directory does not exist: {}",
                model_dir.display()
            ));
        }

        let mut loaded_count = 0;

        // Load all .safetensors files in the model directory
        for entry in std::fs::read_dir(model_dir)
            .with_context(|| format!("Failed to read model directory: {}", model_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.extension() == Some(std::ffi::OsStr::new("safetensors")) {
                if let Some(tag) = path.file_stem().and_then(|s| s.to_str()) {
                    let classifier = TagClassifier::load(&path, EmbeddingModel::embedding_dim())
                        .with_context(|| format!("Failed to load classifier for tag: {tag}"))?;

                    self.classifiers.insert(tag.to_owned(), classifier);
                    loaded_count += 1;
                }
            }
        }

        println!("Loaded {loaded_count} trained classifiers");
        Ok(())
    }
}

impl Default for Predictor {
    #[expect(
        clippy::expect_used,
        reason = "Development: default should panic on failure"
    )]
    #[inline]
    fn default() -> Self {
        Self::new(Settings::default()).expect("Failed to create default predictor")
    }
}
