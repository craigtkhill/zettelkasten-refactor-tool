use anyhow::{Context as _, Result};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use hf_hub::api::sync::Api;
use tokenizers::Tokenizer;

pub struct Model {
    bert: BertModel,
    device: Device,
    tokenizer: Tokenizer,
}

impl Model {
    /// Creates a new Snowflake Arctic Embed XS model
    ///
    /// # Errors
    /// Returns an error if model initialization fails
    #[inline]
    pub fn new() -> Result<Self> {
        let device = Device::Cpu;

        // Download model files from Hugging Face
        let api = Api::new().context("Failed to create HF API client")?;
        let repo = api.model("Snowflake/snowflake-arctic-embed-xs".to_owned());

        let config_path = repo
            .get("config.json")
            .context("Failed to download config.json")?;
        let tokenizer_path = repo
            .get("tokenizer.json")
            .context("Failed to download tokenizer.json")?;
        let weights_path = repo
            .get("model.safetensors")
            .context("Failed to download model weights")?;

        // Load configuration
        let config_content =
            std::fs::read_to_string(config_path).context("Failed to read config.json")?;
        let config: Config =
            serde_json::from_str(&config_content).context("Failed to parse model config")?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {e}"))?;

        // Load model weights
        let weights = candle_core::safetensors::load(&weights_path, &device)
            .context("Failed to load model weights")?;
        let var_builder = VarBuilder::from_tensors(weights, candle_core::DType::F32, &device);

        // Initialize model
        let bert =
            BertModel::load(var_builder, &config).context("Failed to initialize BERT model")?;

        Ok(Self {
            bert,
            device,
            tokenizer,
        })
    }

    /// Embeds the given text into a 384-dimensional vector
    ///
    /// # Errors
    /// Returns an error if embedding fails
    #[inline]
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Tokenize input text
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("Failed to tokenize text: {e}"))?;

        let tokens = encoding.get_ids();
        let token_ids = Tensor::new(tokens, &self.device)
            .context("Failed to create token tensor")?
            .unsqueeze(0)
            .context("Failed to add batch dimension")?;

        let token_type_ids = token_ids
            .zeros_like()
            .context("Failed to create token type ids")?;

        // Forward pass through model
        let embeddings = self
            .bert
            .forward(&token_ids, &token_type_ids, None)
            .context("Failed to run forward pass")?;

        // Mean pooling across sequence dimension (excluding special tokens)
        let attention_mask = Tensor::ones(token_ids.shape(), candle_core::DType::F32, &self.device)
            .context("Failed to create attention mask")?;

        let masked_embeddings = embeddings
            .broadcast_mul(&attention_mask.unsqueeze(2)?)
            .context("Failed to apply attention mask")?;

        let sum_embeddings = masked_embeddings
            .sum(1)
            .context("Failed to sum embeddings")?;
        let sum_mask = attention_mask
            .sum(1)
            .context("Failed to sum attention mask")?
            .unsqueeze(1)?;

        let pooled = sum_embeddings
            .broadcast_div(&sum_mask)
            .context("Failed to normalize embeddings")?;

        // Convert to Vec<f32>
        let embedding_vec = pooled
            .squeeze(0)
            .context("Failed to remove batch dimension")?
            .to_vec1::<f32>()
            .context("Failed to convert tensor to vector")?;

        Ok(embedding_vec)
    }

    /// Returns the embedding dimension (384 for Arctic Embed XS)
    #[must_use]
    #[inline]
    pub const fn embedding_dim() -> usize {
        384
    }
}

impl Default for Model {
    #[inline]
    fn default() -> Self {
        Self::new().expect("Failed to initialize default embedding model")
    }
}
