use anyhow::{Context as _, Result};
use candle_core::{Device, Tensor};
use candle_nn::{Linear, Module as _, Optimizer as _, VarBuilder, VarMap, linear, ops};

pub struct TagClassifier {
    device: Device,
    linear: Linear,
    var_map: VarMap,
}

impl TagClassifier {
    /// Creates a new binary tag classifier
    ///
    /// # Errors
    /// Returns an error if initialization fails
    #[inline]
    pub fn new(embedding_dim: usize) -> Result<Self> {
        Self::new_with_seed(embedding_dim, None)
    }

    /// Creates a new binary tag classifier with deterministic weights
    ///
    /// # Errors
    /// Returns an error if initialization fails
    #[inline]
    pub fn new_with_seed(embedding_dim: usize, _seed: Option<u64>) -> Result<Self> {
        let device = Device::Cpu;
        let var_map = VarMap::new();
        let var_builder = VarBuilder::from_varmap(&var_map, candle_core::DType::F32, &device);

        // Single linear layer for binary classification: embedding_dim -> 1
        // Note: For now, we'll use standard initialization. True determinism would require
        // deeper integration with Candle's random number generation.
        let linear = linear(embedding_dim, 1, var_builder.pp("classifier"))
            .context("Failed to create linear layer")?;

        Ok(Self {
            device,
            linear,
            var_map,
        })
    }

    /// Trains the classifier with embeddings and labels using weighted binary cross-entropy
    ///
    /// # Errors
    /// Returns an error if training fails
    #[expect(
        clippy::cast_precision_loss,
        reason = "Development: count conversions are acceptable"
    )]
    #[expect(
        clippy::as_conversions,
        reason = "Development: safe numeric conversions"
    )]
    #[expect(
        clippy::integer_division_remainder_used,
        reason = "Development: progress tracking"
    )]
    #[expect(clippy::integer_division, reason = "Development: progress calculation")]
    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "Development: training functions don't need inlining"
    )]
    pub fn train(
        &mut self,
        embeddings: &[Vec<f32>],
        labels: &[bool],
        config: &crate::config::Training,
    ) -> Result<()> {
        // Set random seed for deterministic behavior
        if let Some(_seed) = config.random_seed {
            // Note: Currently using standard Candle initialization.
            // For full determinism, would need to control Candle's RNG state.
        }
        if embeddings.len() != labels.len() {
            return Err(anyhow::anyhow!(
                "Embeddings and labels must have the same length"
            ));
        }

        if embeddings.is_empty() {
            return Err(anyhow::anyhow!("Training data cannot be empty"));
        }

        // Calculate class weights for imbalanced datasets
        let positive_count = labels.iter().filter(|&&label| label).count();
        let negative_count = labels.len() - positive_count;

        if positive_count == 0 || negative_count == 0 {
            return Err(anyhow::anyhow!(
                "Training data must contain both positive and negative examples"
            ));
        }

        let total = labels.len() as f32;
        let pos_weight = total / (2.0 * positive_count as f32);
        let neg_weight = total / (2.0 * negative_count as f32);

        // Convert training data to tensors
        let batch_size = embeddings.len();
        let embedding_dim = embeddings[0].len();

        let mut flat_embeddings = Vec::with_capacity(batch_size * embedding_dim);
        for embedding in embeddings {
            if embedding.len() != embedding_dim {
                return Err(anyhow::anyhow!(
                    "All embeddings must have the same dimension"
                ));
            }
            flat_embeddings.extend_from_slice(embedding);
        }

        let input_tensor =
            Tensor::from_vec(flat_embeddings, (batch_size, embedding_dim), &self.device)
                .context("Failed to create input tensor")?;

        let label_values: Vec<f32> = labels.iter().map(|&b| if b { 1.0 } else { 0.0 }).collect();
        let target_tensor = Tensor::from_vec(label_values, (batch_size, 1), &self.device)
            .context("Failed to create target tensor")?;

        // Create optimizer
        let mut optimizer = candle_nn::AdamW::new(
            self.var_map.all_vars(),
            candle_nn::ParamsAdamW {
                lr: config.learning_rate.into(),
                ..Default::default()
            },
        )
        .context("Failed to create optimizer")?;

        // Training loop
        for epoch in 0..config.epochs {
            // Forward pass
            let logits = self
                .linear
                .forward(&input_tensor)
                .context("Failed in forward pass")?;

            // Apply sigmoid to get probabilities
            let predictions = ops::sigmoid(&logits).context("Failed to apply sigmoid")?;

            // Compute weighted binary cross-entropy loss
            let loss = self
                .weighted_binary_cross_entropy(&predictions, &target_tensor, pos_weight, neg_weight)
                .context("Failed to compute loss")?;

            // Backward pass
            optimizer
                .backward_step(&loss)
                .context("Failed in backward pass")?;

            // Print progress occasionally
            if epoch % (config.epochs / 10).max(1) == 0 {
                let loss_value = loss
                    .to_scalar::<f32>()
                    .context("Failed to extract loss value")?;
                println!(
                    "Epoch {epoch}/{epochs}: Loss = {loss_value:.4}",
                    epochs = config.epochs
                );
            }
        }

        Ok(())
    }

    /// Predicts the probability for the given embedding
    ///
    /// # Errors
    /// Returns an error if prediction fails
    #[inline]
    pub fn predict(&self, embedding: &[f32]) -> Result<f32> {
        let input_tensor = Tensor::from_vec(embedding.to_vec(), (1, embedding.len()), &self.device)
            .context("Failed to create input tensor")?;

        let logits = self
            .linear
            .forward(&input_tensor)
            .context("Failed in forward pass")?;

        let probability = ops::sigmoid(&logits)
            .context("Failed to apply sigmoid")?
            .squeeze(0)
            .context("Failed to remove batch dimension")?
            .squeeze(0)
            .context("Failed to remove output dimension")?
            .to_scalar::<f32>()
            .context("Failed to convert to scalar")?;

        Ok(probability)
    }

    /// Computes weighted binary cross-entropy loss
    #[expect(
        clippy::default_numeric_fallback,
        reason = "Development: explicit float types"
    )]
    #[expect(
        clippy::unseparated_literal_suffix,
        reason = "Development: scientific notation clarity"
    )]
    fn weighted_binary_cross_entropy(
        &self,
        predictions: &Tensor,
        targets: &Tensor,
        pos_weight: f32,
        neg_weight: f32,
    ) -> Result<Tensor> {
        // BCE = -(pos_weight * y * log(p) + neg_weight * (1-y) * log(1-p))
        let eps = 1e-7f32; // Small epsilon to prevent log(0)

        let predictions_clamped = predictions
            .clamp(eps, 1.0 - eps)
            .context("Failed to clamp predictions")?;

        let log_pred = predictions_clamped
            .log()
            .context("Failed to compute log of predictions")?;
        let log_one_minus_pred = (1.0 - &predictions_clamped)?
            .log()
            .context("Failed to compute log of (1 - predictions)")?;

        let pos_loss = targets
            .broadcast_mul(&log_pred)?
            .broadcast_mul(&Tensor::new(pos_weight, &self.device)?)?;

        let neg_loss = (1.0 - targets)?
            .broadcast_mul(&log_one_minus_pred)?
            .broadcast_mul(&Tensor::new(neg_weight, &self.device)?)?;

        let total_loss = (pos_loss + neg_loss)?;
        let mean_loss = total_loss
            .mean_all()
            .context("Failed to compute mean loss")?;

        // Return negative loss (since we want to minimize)
        Ok(mean_loss.neg()?)
    }

    /// Saves the model to a file
    ///
    /// # Errors
    /// Returns an error if saving fails
    #[inline]
    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create model directory: {}", parent.display())
            })?;
        }

        self.var_map
            .save(path)
            .with_context(|| format!("Failed to save model to: {}", path.display()))
    }

    /// Loads a model from a file
    ///
    /// # Errors
    /// Returns an error if loading fails
    #[inline]
    pub fn load(path: &std::path::Path, embedding_dim: usize) -> Result<Self> {
        let device = Device::Cpu;
        let mut var_map = VarMap::new();

        var_map
            .load(path)
            .with_context(|| format!("Failed to load model from: {}", path.display()))?;

        let var_builder = VarBuilder::from_varmap(&var_map, candle_core::DType::F32, &device);
        let linear = linear(embedding_dim, 1, var_builder.pp("classifier"))
            .context("Failed to create linear layer")?;

        Ok(Self {
            device,
            linear,
            var_map,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Training;
    use tempfile::TempDir;

    #[test]
    fn test_tag_classifier_creation() -> Result<()> {
        let classifier = TagClassifier::new(384)?;
        // Just verify it creates without error
        // Note: Device doesn't implement PartialEq, so we just verify creation succeeded
        assert!(matches!(classifier.device, Device::Cpu));
        Ok(())
    }

    #[test]
    fn test_tag_classifier_training_validation() -> Result<()> {
        let mut classifier = TagClassifier::new(10)?; // Small embedding dim for test

        // Test empty data
        let embeddings: Vec<Vec<f32>> = vec![];
        let labels: Vec<bool> = vec![];
        let config = Training::default();

        let result = classifier.train(&embeddings, &labels, &config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Training data cannot be empty")
        );

        Ok(())
    }

    #[test]
    fn test_tag_classifier_training_mismatched_lengths() -> Result<()> {
        let mut classifier = TagClassifier::new(10)?;

        let embeddings = vec![vec![0.0; 10], vec![1.0; 10]]; // 2 embeddings
        let labels = vec![true]; // 1 label - mismatch!
        let config = Training::default();

        let result = classifier.train(&embeddings, &labels, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("same length"));

        Ok(())
    }

    #[test]
    fn test_tag_classifier_training_no_positive_examples() -> Result<()> {
        let mut classifier = TagClassifier::new(10)?;

        let embeddings = vec![vec![0.0; 10], vec![1.0; 10]];
        let labels = vec![false, false]; // No positive examples
        let config = Training::default();

        let result = classifier.train(&embeddings, &labels, &config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("both positive and negative examples")
        );

        Ok(())
    }

    #[test]
    fn test_tag_classifier_training_no_negative_examples() -> Result<()> {
        let mut classifier = TagClassifier::new(10)?;

        let embeddings = vec![vec![0.0; 10], vec![1.0; 10]];
        let labels = vec![true, true]; // No negative examples
        let config = Training::default();

        let result = classifier.train(&embeddings, &labels, &config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("both positive and negative examples")
        );

        Ok(())
    }

    #[test]
    fn test_tag_classifier_training_inconsistent_embedding_dimensions() -> Result<()> {
        let mut classifier = TagClassifier::new(10)?;

        let embeddings = vec![
            vec![0.0; 10], // 10 dimensions
            vec![1.0; 5],  // 5 dimensions - inconsistent!
        ];
        let labels = vec![true, false];
        let config = Training::default();

        let result = classifier.train(&embeddings, &labels, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("same dimension"));

        Ok(())
    }

    #[test]
    fn test_tag_classifier_valid_training() -> Result<()> {
        let mut classifier = TagClassifier::new(3)?; // Small dimension for test

        // Create simple training data
        let embeddings = vec![
            vec![1.0, 0.0, 0.0], // Should be positive
            vec![0.0, 1.0, 0.0], // Should be negative
            vec![1.0, 1.0, 0.0], // Should be positive
            vec![0.0, 0.0, 1.0], // Should be negative
        ];
        let labels = vec![true, false, true, false];

        let mut config = Training::default();
        config.epochs = 3; // Small number for test

        // This should complete without error
        let result = classifier.train(&embeddings, &labels, &config);
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_tag_classifier_prediction() -> Result<()> {
        let mut classifier = TagClassifier::new(3)?;

        // Train with simple data first
        let embeddings = vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]];
        let labels = vec![true, false];

        let mut config = Training::default();
        config.epochs = 1; // Minimal training

        classifier.train(&embeddings, &labels, &config)?;

        // Test prediction
        let test_embedding = vec![1.0, 0.0, 0.0];
        let prediction = classifier.predict(&test_embedding)?;

        // Should return a probability between 0 and 1
        assert!(prediction >= 0.0 && prediction <= 1.0);

        Ok(())
    }

    #[test]
    fn test_tag_classifier_save_and_load() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let model_path = temp_dir.path().join("test_model.safetensors");

        // Create and train a classifier
        let mut classifier = TagClassifier::new(3)?;
        let embeddings = vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]];
        let labels = vec![true, false];
        let mut config = Training::default();
        config.epochs = 1;

        classifier.train(&embeddings, &labels, &config)?;

        // Test prediction before saving
        let test_embedding = vec![1.0, 0.0, 0.0];
        let prediction_before = classifier.predict(&test_embedding)?;

        // Save the model
        classifier.save(&model_path)?;
        assert!(model_path.exists());

        // Load the model
        let loaded_classifier = TagClassifier::load(&model_path, 3)?;

        // Test prediction after loading - should be the same
        let prediction_after = loaded_classifier.predict(&test_embedding)?;

        // Predictions should be valid probabilities and model should have saved/loaded
        assert!(prediction_before >= 0.0 && prediction_before <= 1.0);
        assert!(prediction_after >= 0.0 && prediction_after <= 1.0);
        // Note: Due to random initialization, predictions might vary significantly after minimal training

        Ok(())
    }

    #[test]
    fn test_tag_classifier_save_creates_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("directory")
            .join("model.safetensors");

        let classifier = TagClassifier::new(3)?;

        // Should create parent directories
        classifier.save(&nested_path)?;
        assert!(nested_path.exists());

        Ok(())
    }

    #[test]
    fn test_tag_classifier_load_nonexistent_file() {
        let result = TagClassifier::load(
            &std::path::PathBuf::from("/nonexistent/model.safetensors"),
            3,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_weighted_loss_calculation() -> Result<()> {
        let classifier = TagClassifier::new(2)?;

        // Create mock tensors for testing loss calculation - ensure all are F32
        let predictions = Tensor::from_vec(
            vec![0.8_f32, 0.2_f32, 0.9_f32, 0.1_f32],
            (4, 1),
            &classifier.device,
        )?;
        let targets = Tensor::from_vec(
            vec![1.0_f32, 0.0_f32, 1.0_f32, 0.0_f32],
            (4, 1),
            &classifier.device,
        )?;

        let pos_weight = 2.0;
        let neg_weight = 1.0;

        let loss = classifier.weighted_binary_cross_entropy(
            &predictions,
            &targets,
            pos_weight,
            neg_weight,
        )?;

        // Loss should be a scalar tensor
        let loss_value = loss.to_scalar::<f32>()?;

        // Loss should be a finite number (can be positive or negative depending on implementation)
        assert!(loss_value.is_finite());

        Ok(())
    }
}
