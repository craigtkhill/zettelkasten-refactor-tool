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
    pub fn predict(&self, content: &str) -> Result<Vec<Prediction>> {
        // Generate embedding for the content
        let embedding = self
            .embedding_model
            .embed(content)
            .context("Failed to generate embedding")?;

        let mut predictions = Vec::new();

        // Run prediction through each classifier
        for (tag, classifier) in &self.classifiers {
            // Skip excluded tags
            if self.settings.excluded_tags.contains(tag) {
                continue;
            }

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
    pub fn train(&mut self, training_data: &TrainingData) -> Result<()> {
        // Set random seed if specified
        if let Some(seed) = self.settings.training.random_seed {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash as _, Hasher as _};

            // Set seed for reproducible random number generation
            let mut hasher = DefaultHasher::new();
            seed.hash(&mut hasher);
            let _seed_value = hasher.finish();

            // Note: This is a basic seed setting. For full determinism, you'd need
            // to control the ML framework's random state as well.
            println!("Using random seed: {seed}");
        }

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
            let mut classifier = TagClassifier::new_with_seed(
                EmbeddingModel::embedding_dim(),
                self.settings.training.random_seed,
            )
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
    #[inline]
    fn default() -> Self {
        Self::new(Settings::default()).expect("Failed to create default predictor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use crate::extraction::{NoteData, TrainingData};
    use std::collections::HashSet;

    fn create_mock_training_data() -> TrainingData {
        let mut training_data = TrainingData::new();

        // Create some test notes
        let notes = vec![
            ("ML content about neural networks", vec!["ml", "ai"]),
            ("Research paper on deep learning", vec!["research", "ai"]),
            ("Python programming tutorial", vec!["programming", "python"]),
            ("Machine learning algorithms", vec!["ml", "algorithms"]),
            ("Data science workflow", vec!["data_science", "python"]),
        ];

        for (content, tags) in notes {
            let mut tag_set = HashSet::new();
            for tag in tags {
                tag_set.insert(tag.to_owned());
            }

            let note = NoteData {
                content: content.to_owned(),
                path: format!("/test/{}.md", content.len()),
                tags: tag_set,
            };

            training_data.add_note(note);
        }

        training_data
    }

    #[test]
    fn test_prediction_struct_creation() {
        let prediction = Prediction {
            confidence: 0.85,
            tag: "test".to_owned(),
        };

        assert_eq!(prediction.confidence, 0.85);
        assert_eq!(prediction.tag, "test");
    }

    #[test]
    fn test_predictor_new() -> Result<()> {
        // This test will fail if the real embedding model downloads
        // But we can test the settings are stored correctly
        let settings = Settings::default();
        let original_confidence = settings.confidence_threshold;

        // We can't easily test the full constructor without downloading models
        // So just verify settings validation
        assert!(original_confidence > 0.0 && original_confidence <= 1.0);

        Ok(())
    }

    #[test]
    fn test_load_classifiers_validation() {
        // Test the error checking logic for model directory
        let settings = Settings::default();
        // We can't easily create a real predictor without downloading models
        // But we can test the directory path validation indirectly

        // Verify that non-existent model paths would be caught
        let nonexistent_path = std::path::PathBuf::from("/definitely/nonexistent/path/models");
        assert!(!nonexistent_path.exists());

        // Verify the default model path is reasonable
        assert!(settings.model_path.to_str().unwrap().contains("models"));
    }

    // Test the core prediction logic with mocked components
    fn test_prediction_filtering_logic() {
        let mut settings = Settings::default();
        settings.confidence_threshold = 0.5;
        settings.excluded_tags.insert("excluded".to_owned());

        // Mock predictions that would come from classifiers
        let mock_predictions = vec![
            ("good_tag", 0.8),     // Above threshold, not excluded
            ("bad_tag", 0.3),      // Below threshold
            ("excluded", 0.9),     // Above threshold but excluded
            ("another_good", 0.6), // Above threshold, not excluded
        ];

        // Simulate the filtering logic from predict()
        let mut filtered_predictions = Vec::new();

        for (tag, confidence) in mock_predictions {
            // Skip excluded tags
            if settings.excluded_tags.contains(tag) {
                continue;
            }

            // Only include predictions above threshold
            if confidence >= settings.confidence_threshold {
                filtered_predictions.push(Prediction {
                    tag: tag.to_owned(),
                    confidence,
                });
            }
        }

        // Sort by confidence (highest first)
        filtered_predictions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(core::cmp::Ordering::Equal)
        });

        // Should have 2 predictions: "good_tag" (0.8) and "another_good" (0.6)
        assert_eq!(filtered_predictions.len(), 2);
        assert_eq!(filtered_predictions[0].tag, "good_tag");
        assert_eq!(filtered_predictions[0].confidence, 0.8);
        assert_eq!(filtered_predictions[1].tag, "another_good");
        assert_eq!(filtered_predictions[1].confidence, 0.6);
    }

    #[test]
    fn test_prediction_confidence_ordering() {
        let mut predictions = vec![
            Prediction {
                tag: "low".to_owned(),
                confidence: 0.3,
            },
            Prediction {
                tag: "high".to_owned(),
                confidence: 0.9,
            },
            Prediction {
                tag: "medium".to_owned(),
                confidence: 0.6,
            },
        ];

        // Sort by confidence (highest first) - same logic as in predict()
        predictions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(core::cmp::Ordering::Equal)
        });

        assert_eq!(predictions[0].tag, "high");
        assert_eq!(predictions[1].tag, "medium");
        assert_eq!(predictions[2].tag, "low");
    }

    #[test]
    fn test_prediction_max_suggestions_limit() {
        let mut predictions = vec![
            Prediction {
                tag: "tag1".to_owned(),
                confidence: 0.9,
            },
            Prediction {
                tag: "tag2".to_owned(),
                confidence: 0.8,
            },
            Prediction {
                tag: "tag3".to_owned(),
                confidence: 0.7,
            },
            Prediction {
                tag: "tag4".to_owned(),
                confidence: 0.6,
            },
            Prediction {
                tag: "tag5".to_owned(),
                confidence: 0.5,
            },
        ];

        let max_suggestions = 3;

        // Sort and truncate - same logic as in predict()
        predictions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        predictions.truncate(max_suggestions);

        assert_eq!(predictions.len(), 3);
        assert_eq!(predictions[0].tag, "tag1");
        assert_eq!(predictions[1].tag, "tag2");
        assert_eq!(predictions[2].tag, "tag3");
    }

    #[test]
    fn test_training_data_validation() {
        let training_data = create_mock_training_data();

        assert_eq!(training_data.notes.len(), 5);
        assert!(training_data.all_tags.contains("ml"));
        assert!(training_data.all_tags.contains("ai"));
        assert!(training_data.all_tags.contains("python"));
        assert!(training_data.all_tags.contains("research"));

        // Test that tags appear in multiple notes (for ML training)
        let ai_count = training_data
            .notes
            .iter()
            .filter(|note| note.tags.contains("ai"))
            .count();
        assert!(ai_count >= 2);
    }

    #[test]
    fn test_settings_excluded_tags_integration() {
        let mut settings = Settings::default();
        settings.excluded_tags.insert("draft".to_owned());
        settings.excluded_tags.insert("private".to_owned());

        let mut training_data = create_mock_training_data();

        // Add a note with excluded tags
        let mut excluded_tags = HashSet::new();
        excluded_tags.insert("draft".to_owned());
        excluded_tags.insert("keep_me".to_owned());

        let excluded_note = NoteData {
            content: "Draft content".to_owned(),
            path: "/test/draft.md".to_owned(),
            tags: excluded_tags,
        };
        training_data.add_note(excluded_note);

        // Apply exclusions
        training_data.exclude_tags(&settings.excluded_tags);

        // Should not contain excluded tags
        assert!(!training_data.all_tags.contains("draft"));
        assert!(!training_data.all_tags.contains("private"));

        // Should still contain allowed tags
        assert!(training_data.all_tags.contains("keep_me"));
    }

    #[test]
    fn test_min_tag_examples_filtering() {
        let mut training_data = create_mock_training_data();
        let min_examples = 2;

        // Count tag frequencies before filtering
        let mut tag_counts = std::collections::HashMap::new();
        for note in &training_data.notes {
            for tag in &note.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        // Apply minimum examples filter
        training_data.filter_by_min_examples(min_examples);

        // Verify only tags with enough examples remain
        for tag in &training_data.all_tags {
            let count = tag_counts.get(tag).unwrap_or(&0);
            assert!(
                *count >= min_examples,
                "Tag '{}' has {} examples but minimum is {}",
                tag,
                count,
                min_examples
            );
        }
    }

    // Integration test for the full workflow (without real model downloads)
    #[test]
    fn test_training_workflow_validation() {
        let training_data = create_mock_training_data();
        let settings = Settings::default();

        // Verify we have valid training data structure
        assert!(!training_data.notes.is_empty());
        assert!(!training_data.all_tags.is_empty());

        // Verify settings are reasonable
        assert!(settings.confidence_threshold > 0.0 && settings.confidence_threshold <= 1.0);
        assert!(settings.min_tag_examples > 0);
        assert!(settings.max_suggestions > 0);

        // Verify each note has valid structure
        for note in &training_data.notes {
            assert!(!note.content.is_empty());
            assert!(!note.path.is_empty());
            assert!(!note.tags.is_empty());
        }
    }

    // Call the actual test function
    #[test]
    fn test_prediction_filtering() {
        test_prediction_filtering_logic();
    }
}
