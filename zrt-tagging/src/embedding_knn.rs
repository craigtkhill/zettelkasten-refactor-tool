use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

use crate::embedding::Model as EmbeddingModel;
use crate::embedding_cache::EmbeddingCache;

/// A note with its embedding and tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedNote {
    pub path: String,
    pub embedding: Vec<f32>,
    pub tags: HashSet<String>,
}

/// KNN-based tag predictor using embeddings
#[derive(Serialize, Deserialize)]
pub struct EmbeddingKnnPredictor {
    /// All notes with their embeddings and tags
    embedded_notes: Vec<EmbeddedNote>,
    /// Embedding model for generating new embeddings
    #[serde(skip)]
    embedding_model: Option<EmbeddingModel>,
}

/// Tag prediction result
#[derive(Debug, Clone)]
pub struct KnnPrediction {
    pub tag: String,
    pub confidence: f32,
}

impl EmbeddingKnnPredictor {
    /// Create a new KNN predictor
    pub fn new() -> Result<Self> {
        let embedding_model = EmbeddingModel::new()
            .context("Failed to initialize embedding model")?;
            
        Ok(Self {
            embedded_notes: Vec::new(),
            embedding_model: Some(embedding_model),
        })
    }

    /// Train the predictor by storing embeddings for all notes
    pub fn train(&mut self, notes: &[(String, String, HashSet<String>)]) -> Result<()> {
        println!("Generating embeddings for {} notes...", notes.len());
        
        let embedding_model = self.embedding_model.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Embedding model not initialized"))?;

        self.embedded_notes.clear();
        
        for (path, content, tags) in notes {
            let embedding = embedding_model.embed(content)
                .with_context(|| format!("Failed to embed note: {}", path))?;
                
            self.embedded_notes.push(EmbeddedNote {
                path: path.clone(),
                embedding,
                tags: tags.clone(),
            });
        }

        println!("Generated embeddings for {} notes", self.embedded_notes.len());
        Ok(())
    }

    /// Train the predictor using cached embeddings (much faster!)
    pub fn train_with_cache(&mut self, notes: &[(String, String, HashSet<String>)], cache_dir: &Path) -> Result<()> {
        println!("Training with embedding cache for {} notes...", notes.len());
        
        let mut cache = EmbeddingCache::new(cache_dir)
            .context("Failed to initialize embedding cache")?;

        // Get embeddings (cached or newly computed)
        let cached_embeddings = cache.get_embeddings(notes)
            .context("Failed to get cached embeddings")?;

        // Convert cached embeddings to our format
        self.embedded_notes.clear();
        for cached in cached_embeddings {
            self.embedded_notes.push(EmbeddedNote {
                path: cached.file_path,
                embedding: cached.embedding,
                tags: cached.tags.into_iter().collect(),
            });
        }

        // Show cache stats
        let stats = cache.get_stats()?;
        println!(
            "Training completed! Used {} cached embeddings ({:.1} MB cache)",
            stats.total_embeddings, stats.cache_size_mb
        );

        Ok(())
    }

    /// Predict tags using K-nearest neighbors
    pub fn predict(
        &self,
        content: &str,
        k: usize,
        threshold: f32,
        max_suggestions: usize,
    ) -> Result<Vec<KnnPrediction>> {
        if self.embedded_notes.is_empty() {
            return Ok(Vec::new());
        }

        let embedding_model = self.embedding_model.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Embedding model not initialized"))?;

        // Generate embedding for the input content
        let query_embedding = embedding_model.embed(content)
            .context("Failed to generate embedding for query")?;

        // Calculate similarities to all notes
        let mut similarities: Vec<(f32, &EmbeddedNote)> = self.embedded_notes
            .iter()
            .map(|note| {
                let similarity = cosine_similarity(&query_embedding, &note.embedding);
                (similarity, note)
            })
            .collect();

        // Sort by similarity (descending) and take top K
        similarities.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(k);

        // Aggregate tags from K nearest neighbors
        let mut tag_scores: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
        let mut total_weight = 0.0;

        for (similarity, note) in &similarities {
            total_weight += similarity;
            for tag in &note.tags {
                *tag_scores.entry(tag.clone()).or_insert(0.0) += similarity;
            }
        }

        // Convert to predictions and normalize by total weight
        let mut predictions: Vec<KnnPrediction> = tag_scores
            .into_iter()
            .map(|(tag, score)| {
                let confidence = if total_weight > 0.0 { score / total_weight } else { 0.0 };
                KnnPrediction { tag, confidence }
            })
            .filter(|p| p.confidence >= threshold)
            .collect();

        // Sort by confidence (descending) and limit results
        predictions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        predictions.truncate(max_suggestions);

        Ok(predictions)
    }

    /// Predict tags for multiple files using cached embeddings (much faster!)
    pub fn predict_batch(
        &self,
        files: &[(String, String)],
        cache_dir: &std::path::Path,
        k: usize,
        threshold: f32,
        max_suggestions: usize,
    ) -> Result<Vec<(String, Vec<KnnPrediction>)>> {
        if self.embedded_notes.is_empty() {
            return Ok(Vec::new());
        }

        // Create cache to get embeddings efficiently
        let mut cache = EmbeddingCache::new(cache_dir)?;
        
        // Convert files to the format expected by cache
        let notes: Vec<(String, String, HashSet<String>)> = files
            .iter()
            .map(|(file_path, content)| (file_path.clone(), content.clone(), HashSet::new()))
            .collect();

        // Get embeddings (cached or newly computed)
        let cached_embeddings = cache.get_embeddings(&notes)?;

        // Predict for each file using cached embeddings
        let mut results = Vec::new();
        
        for cached in cached_embeddings {
            let query_embedding = &cached.embedding;
            
            // Calculate similarities to all notes
            let mut similarities: Vec<(f32, &EmbeddedNote)> = self.embedded_notes
                .iter()
                .map(|note| {
                    let similarity = cosine_similarity(query_embedding, &note.embedding);
                    (similarity, note)
                })
                .collect();

            // Sort by similarity (descending) and take top K
            similarities.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            similarities.truncate(k);

            // Aggregate tags from K nearest neighbors
            let mut tag_scores: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
            let mut total_weight = 0.0;

            for (similarity, note) in &similarities {
                total_weight += similarity;
                for tag in &note.tags {
                    *tag_scores.entry(tag.clone()).or_insert(0.0) += similarity;
                }
            }

            // Convert to predictions and normalize by total weight
            let mut predictions: Vec<KnnPrediction> = tag_scores
                .into_iter()
                .map(|(tag, score)| {
                    let confidence = if total_weight > 0.0 { score / total_weight } else { 0.0 };
                    KnnPrediction { tag, confidence }
                })
                .filter(|p| p.confidence >= threshold)
                .collect();

            // Sort by confidence (descending) and limit results
            predictions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
            predictions.truncate(max_suggestions);

            results.push((cached.file_path, predictions));
        }

        Ok(results)
    }

    /// Save the trained model to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize KNN model")?;

        std::fs::write(path, json)
            .with_context(|| format!("Failed to write KNN model to: {}", path.display()))?;

        Ok(())
    }

    /// Load a trained model from a file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read KNN model from: {}", path.display()))?;

        let mut predictor: EmbeddingKnnPredictor = serde_json::from_str(&content)
            .with_context(|| format!("Failed to deserialize KNN model from: {}", path.display()))?;

        // Reinitialize the embedding model (since it's skipped in serialization)
        predictor.embedding_model = Some(EmbeddingModel::new()
            .context("Failed to reinitialize embedding model")?);

        Ok(predictor)
    }

    /// Check if the model has been trained
    pub fn is_trained(&self) -> bool {
        !self.embedded_notes.is_empty()
    }

    /// Get the number of notes in the model
    pub fn note_count(&self) -> usize {
        self.embedded_notes.len()
    }
}

impl Default for EmbeddingKnnPredictor {
    fn default() -> Self {
        Self {
            embedded_notes: Vec::new(),
            embedding_model: None,
        }
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (val_a, val_b) in a.iter().zip(b.iter()) {
        dot_product += val_a * val_b;
        norm_a += val_a * val_a;
        norm_b += val_b * val_b;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a.sqrt() * norm_b.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_knn_prediction() -> Result<()> {
        let mut predictor = EmbeddingKnnPredictor::new()?;
        
        let notes = vec![
            (
                "note1.md".to_string(),
                "machine learning algorithms are powerful".to_string(),
                ["ai", "tech"].iter().map(|s| s.to_string()).collect(),
            ),
            (
                "note2.md".to_string(),
                "cooking pasta with olive oil".to_string(),
                ["cooking", "food"].iter().map(|s| s.to_string()).collect(),
            ),
            (
                "note3.md".to_string(),
                "artificial intelligence and neural networks".to_string(),
                ["ai", "tech"].iter().map(|s| s.to_string()).collect(),
            ),
        ];

        predictor.train(&notes)?;

        let predictions = predictor.predict("deep learning neural networks", 2, 0.0, 5)?;
        
        // Should predict AI-related tags with higher confidence
        assert!(!predictions.is_empty());
        let ai_prediction = predictions.iter().find(|p| p.tag == "ai");
        assert!(ai_prediction.is_some());

        Ok(())
    }
}