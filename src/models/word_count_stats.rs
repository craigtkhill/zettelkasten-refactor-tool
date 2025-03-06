// src/models/word_count_stats.rs

#[derive(Debug, Default)]
pub struct WordCountStats {
    pub tagged_files: u64,
    pub tagged_words: u64,
    pub total_files: u64,
    pub total_words: u64,
}

impl WordCountStats {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tagged_files: 0,
            tagged_words: 0,
            total_files: 0,
            total_words: 0,
        }
    }
    #[inline]
    #[must_use]
    #[expect(clippy::as_conversions, reason = "Precision not critical")]
    #[expect(clippy::cast_precision_loss, reason = "Precision not critical")]
    pub fn calculate_percentage(&self) -> f64 {
        if self.total_words == 0 {
            return 0.0;
        }
        (self.tagged_words as f64 / self.total_words as f64) * 100.0
    }
}
