use anyhow::{Context as _, Result};

use crate::config::{PredictorType, Settings};
use crate::embedding_knn::Predictor as EmbeddingKnnPredictor;
use crate::extraction::TrainingData;
use crate::prediction::{Prediction, Predictor as MlPredictor};
use crate::tfidf::{Document, TfIdfPredictor};

/// Unified predictor that can use TF-IDF, ML embedding, or Embedding KNN approach
pub struct UnifiedPredictor {
    settings: Settings,
    tfidf_predictor: Option<TfIdfPredictor>,
    ml_predictor: Option<MlPredictor>,
    knn_predictor: Option<EmbeddingKnnPredictor>,
}

impl UnifiedPredictor {
    /// Create a new unified predictor
    pub fn new(settings: Settings) -> Result<Self> {
        Ok(Self {
            settings,
            tfidf_predictor: None,
            ml_predictor: None,
            knn_predictor: None,
        })
    }

    /// Train the predictor with training data
    pub fn train(&mut self, training_data: &TrainingData) -> Result<()> {
        match self.settings.predictor_type {
            PredictorType::TfIdf => {
                println!("Training TF-IDF predictor...");
                let mut tfidf_predictor = TfIdfPredictor::new();

                // Convert TrainingData to TF-IDF Documents
                let documents: Vec<Document> = training_data
                    .notes
                    .iter()
                    .map(|note| Document {
                        content: note.content.clone(),
                        tags: note.tags.clone(),
                    })
                    .collect();

                tfidf_predictor
                    .train(&documents)
                    .context("Failed to train TF-IDF predictor")?;

                // Save the trained TF-IDF model
                let model_path = self.settings.model_path.join("tfidf_model.json");
                tfidf_predictor
                    .save(&model_path)
                    .context("Failed to save TF-IDF model")?;

                self.tfidf_predictor = Some(tfidf_predictor);
                println!("TF-IDF training completed and model saved!");
            }
            PredictorType::MlEmbedding => {
                println!("Training ML embedding predictor...");
                let mut ml_predictor = MlPredictor::new(self.settings.clone())?;
                ml_predictor
                    .train(training_data)
                    .context("Failed to train ML predictor")?;

                self.ml_predictor = Some(ml_predictor);
                println!("ML training completed successfully!");
            }
            PredictorType::EmbeddingKnn => {
                println!("Training Embedding KNN predictor...");
                let mut knn_predictor = EmbeddingKnnPredictor::new()?;

                // Convert TrainingData to the format expected by KNN predictor
                let notes: Vec<(String, String, std::collections::HashSet<String>)> = training_data
                    .notes
                    .iter()
                    .map(|note| (note.path.clone(), note.content.clone(), note.tags.clone()))
                    .collect();

                // Use cached training for much better performance
                knn_predictor
                    .train_with_cache(&notes, &self.settings.model_path)
                    .context("Failed to train KNN predictor with cache")?;

                // Save the trained KNN model
                let model_path = self.settings.model_path.join("knn_model.json");
                knn_predictor
                    .save(&model_path)
                    .context("Failed to save KNN model")?;

                self.knn_predictor = Some(knn_predictor);
                println!("Embedding KNN training completed and model saved!");
            }
        }
        Ok(())
    }

    /// Predict tags for content
    pub fn predict(&self, content: &str) -> Result<Vec<Prediction>> {
        let predictions = match self.settings.predictor_type {
            PredictorType::TfIdf => {
                let tfidf_predictor = self
                    .tfidf_predictor
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("TF-IDF predictor not trained"))?;

                let tfidf_predictions = tfidf_predictor.predict(
                    content,
                    self.settings.confidence_threshold,
                    self.settings.max_suggestions,
                )?;

                // Convert TfIdfPrediction to Prediction
                tfidf_predictions
                    .into_iter()
                    .filter(|p| !self.settings.excluded_tags.contains(&p.tag))
                    .map(|p| Prediction {
                        tag: p.tag,
                        confidence: p.confidence,
                    })
                    .collect()
            }
            PredictorType::MlEmbedding => {
                let ml_predictor = self
                    .ml_predictor
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("ML predictor not trained"))?;

                ml_predictor.predict(content)?
            }
            PredictorType::EmbeddingKnn => {
                let knn_predictor = self
                    .knn_predictor
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("KNN predictor not trained"))?;

                let knn_predictions = knn_predictor.predict(
                    content,
                    5, // K neighbors
                    self.settings.confidence_threshold,
                    self.settings.max_suggestions,
                )?;

                // Convert KnnPrediction to Prediction
                knn_predictions
                    .into_iter()
                    .filter(|p| !self.settings.excluded_tags.contains(&p.tag))
                    .map(|p| Prediction {
                        tag: p.tag,
                        confidence: p.confidence,
                    })
                    .collect()
            }
        };

        Ok(predictions)
    }

    /// Predict tags for multiple files using cached embeddings (efficient for EmbeddingKnn)
    pub fn predict_batch(
        &self,
        files: &[(String, String)],
    ) -> Result<Vec<(String, Vec<Prediction>)>> {
        match self.settings.predictor_type {
            PredictorType::TfIdf | PredictorType::MlEmbedding => {
                // For TfIdf and MlEmbedding, just use single predictions
                let mut results = Vec::new();
                for (file_path, content) in files {
                    let predictions = self.predict(content)?;
                    results.push((file_path.clone(), predictions));
                }
                Ok(results)
            }
            PredictorType::EmbeddingKnn => {
                let knn_predictor = self
                    .knn_predictor
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("KNN predictor not trained"))?;

                // Use cached embeddings for batch prediction
                let predictions = knn_predictor.predict_batch(
                    files,
                    &self.settings.model_path,
                    5, // K neighbors
                    self.settings.confidence_threshold,
                    self.settings.max_suggestions,
                )?;

                // Convert KnnPrediction to Prediction and filter excluded tags
                let results = predictions
                    .into_iter()
                    .map(|(file_path, knn_predictions)| {
                        let filtered_predictions: Vec<Prediction> = knn_predictions
                            .into_iter()
                            .filter(|p| !self.settings.excluded_tags.contains(&p.tag))
                            .map(|p| Prediction {
                                tag: p.tag,
                                confidence: p.confidence,
                            })
                            .collect();
                        (file_path, filtered_predictions)
                    })
                    .collect();

                Ok(results)
            }
        }
    }

    /// Load trained models from disk
    pub fn load_models(&mut self) -> Result<()> {
        match self.settings.predictor_type {
            PredictorType::TfIdf => {
                let model_path = self.settings.model_path.join("tfidf_model.json");

                if model_path.exists() {
                    let tfidf_predictor =
                        TfIdfPredictor::load(&model_path).context("Failed to load TF-IDF model")?;

                    self.tfidf_predictor = Some(tfidf_predictor);
                    println!("Loaded TF-IDF model from: {}", model_path.display());
                } else {
                    return Err(anyhow::anyhow!(
                        "TF-IDF model not found at: {}. Run 'zrt tag train' first.",
                        model_path.display()
                    ));
                }
                Ok(())
            }
            PredictorType::MlEmbedding => {
                let mut ml_predictor = MlPredictor::new(self.settings.clone())?;
                ml_predictor
                    .load_classifiers()
                    .context("Failed to load ML classifiers")?;

                self.ml_predictor = Some(ml_predictor);
                Ok(())
            }
            PredictorType::EmbeddingKnn => {
                let model_path = self.settings.model_path.join("knn_model.json");

                if model_path.exists() {
                    let knn_predictor = EmbeddingKnnPredictor::load(&model_path)
                        .context("Failed to load KNN model")?;

                    self.knn_predictor = Some(knn_predictor);
                    println!("Loaded KNN model from: {}", model_path.display());
                } else {
                    return Err(anyhow::anyhow!(
                        "KNN model not found at: {}. Run 'zrt tag train' first.",
                        model_path.display()
                    ));
                }
                Ok(())
            }
        }
    }

    /// Get the current predictor type
    pub fn predictor_type(&self) -> &PredictorType {
        &self.settings.predictor_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extraction::NoteData;

    #[test]
    fn test_unified_predictor_tfidf() -> Result<()> {
        let mut settings = Settings::default();
        settings.predictor_type = PredictorType::TfIdf;
        settings.confidence_threshold = 0.1;

        let mut predictor = UnifiedPredictor::new(settings)?;

        // Create test training data
        let training_data = TrainingData {
            notes: vec![
                NoteData {
                    path: "test1.md".to_owned(),
                    content: "machine learning algorithms".to_owned(),
                    tags: ["ai", "tech"].iter().map(|s| s.to_string()).collect(),
                },
                NoteData {
                    path: "test2.md".to_owned(),
                    content: "cooking pasta recipes".to_owned(),
                    tags: ["cooking", "food"].iter().map(|s| s.to_string()).collect(),
                },
            ],
            all_tags: ["ai", "tech", "cooking", "food"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };

        predictor.train(&training_data)?;

        let predictions = predictor.predict("artificial intelligence and algorithms")?;
        assert!(!predictions.is_empty());

        Ok(())
    }
}
