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
        let device = Device::Cpu;
        let var_map = VarMap::new();
        let var_builder = VarBuilder::from_varmap(&var_map, candle_core::DType::F32, &device);

        // Single linear layer for binary classification: embedding_dim -> 1
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
