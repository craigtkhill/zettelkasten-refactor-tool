use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// TF-IDF based tag predictor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TfIdfPredictor {
    /// Document frequency for each term (how many documents contain this term)
    document_frequency: HashMap<String, usize>,
    /// Total number of documents
    total_documents: usize,
    /// For each tag, store TF-IDF vectors of documents that have this tag
    tag_profiles: HashMap<String, Vec<HashMap<String, f32>>>,
    /// Vocabulary (all unique terms)
    vocabulary: HashSet<String>,
}

/// A document with its content and tags
#[derive(Debug, Clone)]
pub struct Document {
    pub content: String,
    pub tags: HashSet<String>,
}

/// Tag prediction result
#[derive(Debug, Clone)]
pub struct TfIdfPrediction {
    pub tag: String,
    pub confidence: f32,
}

impl TfIdfPredictor {
    /// Create a new TF-IDF predictor
    pub fn new() -> Self {
        Self {
            document_frequency: HashMap::new(),
            total_documents: 0,
            tag_profiles: HashMap::new(),
            vocabulary: HashSet::new(),
        }
    }

    /// Train the predictor with a collection of documents
    pub fn train(&mut self, documents: &[Document]) -> Result<()> {
        self.total_documents = documents.len();

        // Step 1: Build vocabulary and document frequency
        for doc in documents {
            let terms = Self::tokenize(&doc.content);
            let unique_terms: HashSet<_> = terms.into_iter().collect();

            for term in &unique_terms {
                self.vocabulary.insert(term.clone());
                *self.document_frequency.entry(term.clone()).or_insert(0) += 1;
            }
        }

        // Step 2: Build TF-IDF profiles for each tag
        for doc in documents {
            let tf_idf_vector = self.compute_tfidf_vector(&doc.content)?;

            for tag in &doc.tags {
                self.tag_profiles
                    .entry(tag.clone())
                    .or_default()
                    .push(tf_idf_vector.clone());
            }
        }

        Ok(())
    }

    /// Predict tags for new content
    pub fn predict(
        &self,
        content: &str,
        threshold: f32,
        max_suggestions: usize,
    ) -> Result<Vec<TfIdfPrediction>> {
        let content_vector = self.compute_tfidf_vector(content)?;
        let mut predictions = Vec::new();

        for (tag, tag_documents) in &self.tag_profiles {
            // Compute average similarity to all documents with this tag
            let mut total_similarity = 0.0;
            let mut count = 0_i32;

            for doc_vector in tag_documents {
                let similarity = Self::cosine_similarity(&content_vector, doc_vector);
                total_similarity += similarity;
                count += 1_i32;
            }

            let avg_similarity = if count > 0_i32 {
                total_similarity / count as f32
            } else {
                0.0
            };

            if avg_similarity >= threshold {
                predictions.push(TfIdfPrediction {
                    tag: tag.clone(),
                    confidence: avg_similarity,
                });
            }
        }

        // Sort by confidence (descending) and limit results
        predictions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        predictions.truncate(max_suggestions);

        Ok(predictions)
    }

    /// Save the trained TF-IDF model to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let json =
            serde_json::to_string_pretty(self).context("Failed to serialize TF-IDF model")?;

        std::fs::write(path, json)
            .with_context(|| format!("Failed to write TF-IDF model to: {}", path.display()))?;

        Ok(())
    }

    /// Load a trained TF-IDF model from a file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read TF-IDF model from: {}", path.display()))?;

        let predictor: Self = serde_json::from_str(&content).with_context(|| {
            format!(
                "Failed to deserialize TF-IDF model from: {}",
                path.display()
            )
        })?;

        Ok(predictor)
    }

    /// Check if the model has been trained (has vocabulary)
    pub fn is_trained(&self) -> bool {
        !self.vocabulary.is_empty()
    }

    /// Tokenize text into terms
    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .filter_map(|word| {
                // Remove punctuation and keep only alphabetic words
                let cleaned: String = word.chars().filter(|c| c.is_alphabetic()).collect();
                if cleaned.len() >= 2 {
                    // Filter out very short words
                    Some(cleaned)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Compute TF-IDF vector for a document
    fn compute_tfidf_vector(&self, content: &str) -> Result<HashMap<String, f32>> {
        let terms = Self::tokenize(content);
        let mut tf_map = HashMap::new();

        // Compute term frequency (TF)
        for term in &terms {
            *tf_map.entry(term.clone()).or_insert(0_i32) += 1_i32;
        }

        let total_terms = terms.len() as f32;
        let mut tfidf_vector = HashMap::new();

        // Compute TF-IDF for each term
        for (term, count) in tf_map {
            if self.vocabulary.contains(&term) {
                let tf = count as f32 / total_terms;
                let df = self.document_frequency.get(&term).copied().unwrap_or(1) as f32;
                let idf = (self.total_documents as f32 / df).ln();
                let tfidf = tf * idf;

                if tfidf > 0.0 {
                    tfidf_vector.insert(term, tfidf);
                }
            }
        }

        Ok(tfidf_vector)
    }

    /// Compute cosine similarity between two TF-IDF vectors
    fn cosine_similarity(vec1: &HashMap<String, f32>, vec2: &HashMap<String, f32>) -> f32 {
        let mut dot_product = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;

        // Get all terms from both vectors
        let all_terms: HashSet<_> = vec1.keys().chain(vec2.keys()).collect();

        for term in all_terms {
            let val1 = vec1.get(term).copied().unwrap_or(0.0);
            let val2 = vec2.get(term).copied().unwrap_or(0.0);

            dot_product += val1 * val2;
            norm1 += val1 * val1;
            norm2 += val2 * val2;
        }

        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1.sqrt() * norm2.sqrt())
        }
    }
}

impl Default for TfIdfPredictor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = TfIdfPredictor::tokenize("Hello, world! This is a test.");
        assert_eq!(tokens, vec!["hello", "world", "this", "is", "test"]);
    }

    #[test]
    fn test_tfidf_prediction() -> Result<()> {
        let mut predictor = TfIdfPredictor::new();

        let documents = vec![
            Document {
                content: "machine learning algorithms are powerful".to_owned(),
                tags: ["ai", "tech"].iter().map(|s| s.to_string()).collect(),
            },
            Document {
                content: "cooking pasta with olive oil".to_owned(),
                tags: ["cooking", "food"].iter().map(|s| s.to_string()).collect(),
            },
            Document {
                content: "artificial intelligence and machine learning".to_owned(),
                tags: ["ai", "tech"].iter().map(|s| s.to_string()).collect(),
            },
        ];

        predictor.train(&documents)?;

        let predictions =
            predictor.predict("neural networks and artificial intelligence", 0.0, 5)?;

        // Should predict "ai" and "tech" with higher confidence than "cooking" and "food"
        assert!(!predictions.is_empty());
        let ai_prediction = predictions.iter().find(|p| p.tag == "ai");
        assert!(ai_prediction.is_some());

        Ok(())
    }

    #[test]
    fn test_cosine_similarity() {
        let mut vec1 = HashMap::new();
        vec1.insert("hello".to_string(), 1.0);
        vec1.insert("world".to_string(), 1.0);

        let mut vec2 = HashMap::new();
        vec2.insert("hello".to_string(), 1.0);
        vec2.insert("world".to_string(), 1.0);

        let similarity = TfIdfPredictor::cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 0.001); // Should be 1.0 (identical vectors)
    }
}
