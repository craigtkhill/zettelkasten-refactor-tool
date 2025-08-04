use anyhow::{Context as _, Result};
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::embedding::Model as EmbeddingModel;

/// Represents a cached embedding with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEmbedding {
    pub file_path: String,
    pub content_hash: String,
    pub embedding: Vec<f32>,
    pub tags: Vec<String>,
    pub last_modified: i64,
}

/// High-performance embedding cache using JSON storage
pub struct EmbeddingCache {
    cache_path: PathBuf,
    embedding_model: EmbeddingModel,
}

impl EmbeddingCache {
    /// Create a new embedding cache
    pub fn new(cache_dir: &Path) -> Result<Self> {
        let cache_path = cache_dir.join("embeddings.json");
        
        // Ensure cache directory exists
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create cache directory: {}", parent.display()))?;
        }

        let embedding_model = EmbeddingModel::new()
            .context("Failed to initialize embedding model")?;

        Ok(Self {
            cache_path,
            embedding_model,
        })
    }

    /// Load existing cache or create empty vector
    fn load_cache(&self) -> Result<Vec<CachedEmbedding>> {
        if self.cache_path.exists() {
            let content = std::fs::read_to_string(&self.cache_path)
                .with_context(|| format!("Failed to read cache file: {}", self.cache_path.display()))?;
            
            let embeddings: Vec<CachedEmbedding> = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse cache file: {}", self.cache_path.display()))?;
            
            Ok(embeddings)
        } else {
            Ok(Vec::new())
        }
    }

    /// Save embeddings to cache
    fn save_cache(&self, embeddings: &[CachedEmbedding]) -> Result<()> {
        let json = serde_json::to_string_pretty(embeddings)
            .context("Failed to serialize embeddings")?;

        std::fs::write(&self.cache_path, json)
            .with_context(|| format!("Failed to write cache file: {}", self.cache_path.display()))?;

        Ok(())
    }

    /// Compute content hash for a file
    fn compute_content_hash(content: &str) -> String {
        let mut hasher = Hasher::new();
        hasher.update(content.as_bytes());
        hasher.finalize().to_hex().to_string()
    }

    /// Get file modification time as Unix timestamp
    fn get_file_mtime(path: &Path) -> Result<i64> {
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to get metadata for: {}", path.display()))?;
        
        let mtime = metadata.modified()
            .context("Failed to get modification time")?
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Invalid modification time")?
            .as_secs() as i64;

        Ok(mtime)
    }

    /// Get or compute embeddings for a list of notes
    pub fn get_embeddings(
        &mut self, 
        notes: &[(String, String, HashSet<String>)]
    ) -> Result<Vec<CachedEmbedding>> {
        println!("Loading embedding cache...");
        let mut cached_embeddings = self.load_cache()?;
        
        // Create lookup map from existing cache
        let mut existing_embeddings: HashMap<String, CachedEmbedding> = HashMap::new();
        for cached in &cached_embeddings {
            existing_embeddings.insert(cached.content_hash.clone(), cached.clone());
        }

        println!("Found {} cached embeddings", existing_embeddings.len());

        // Determine which embeddings need to be computed
        let mut results = Vec::new();
        let mut new_embeddings = Vec::new();

        for (file_path, content, tags) in notes {
            let content_hash = Self::compute_content_hash(content);
            
            // Check if we have a cached embedding with matching hash
            if let Some(cached) = existing_embeddings.get(&content_hash) {
                // Use cached embedding
                results.push(cached.clone());
            } else {
                // Need to compute new embedding
                println!("Computing embedding for: {}", file_path);
                let embedding = self.embedding_model.embed(content)
                    .with_context(|| format!("Failed to embed file: {}", file_path))?;

                let mtime = Self::get_file_mtime(Path::new(file_path)).unwrap_or(0);
                let tags_vec: Vec<String> = tags.iter().cloned().collect();

                let cached_emb = CachedEmbedding {
                    file_path: file_path.clone(),
                    content_hash: content_hash.clone(),
                    embedding: embedding.clone(),
                    tags: tags_vec.clone(),
                    last_modified: mtime,
                };

                results.push(cached_emb.clone());
                new_embeddings.push(cached_emb);
            }
        }

        // If we have new embeddings, append them to cache
        if !new_embeddings.is_empty() {
            println!("Caching {} new embeddings...", new_embeddings.len());
            cached_embeddings.extend(new_embeddings);
            self.save_cache(&cached_embeddings)?;
        }

        println!("Total embeddings ready: {}", results.len());
        Ok(results)
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> Result<CacheStats> {
        let cached_embeddings = self.load_cache()?;
        
        Ok(CacheStats {
            total_embeddings: cached_embeddings.len(),
            cache_size_mb: if self.cache_path.exists() {
                std::fs::metadata(&self.cache_path)?.len() as f64 / 1_048_576.0
            } else {
                0.0
            },
        })
    }

    /// Clear the entire cache
    pub fn clear(&self) -> Result<()> {
        if self.cache_path.exists() {
            std::fs::remove_file(&self.cache_path)
                .with_context(|| format!("Failed to remove cache file: {}", self.cache_path.display()))?;
        }
        Ok(())
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub total_embeddings: usize,
    pub cache_size_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_embedding_cache() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut cache = EmbeddingCache::new(temp_dir.path())?;

        let notes = vec![
            (
                "test1.md".to_string(),
                "machine learning algorithms".to_string(),
                ["ai", "tech"].iter().map(|s| s.to_string()).collect(),
            ),
            (
                "test2.md".to_string(),
                "cooking pasta recipes".to_string(), 
                ["cooking", "food"].iter().map(|s| s.to_string()).collect(),
            ),
        ];

        // First call should compute embeddings
        let embeddings1 = cache.get_embeddings(&notes)?;
        assert_eq!(embeddings1.len(), 2);

        // Second call should use cached embeddings
        let embeddings2 = cache.get_embeddings(&notes)?;
        assert_eq!(embeddings2.len(), 2);

        // Embeddings should be identical
        assert_eq!(embeddings1[0].embedding, embeddings2[0].embedding);
        assert_eq!(embeddings1[1].embedding, embeddings2[1].embedding);

        Ok(())
    }

    #[test]
    fn test_cache_stats() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache = EmbeddingCache::new(temp_dir.path())?;

        let stats = cache.get_stats()?;
        assert_eq!(stats.total_embeddings, 0);
        assert_eq!(stats.cache_size_mb, 0.0);

        Ok(())
    }
}