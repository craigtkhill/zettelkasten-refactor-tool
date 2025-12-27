#[derive(Debug, Default)]
pub struct WordCountStats {
    pub tagged_files: u32,
    pub tagged_words: u32,
    pub total_files: u32,
    pub total_words: u32,
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
    pub fn calculate_percentage(&self) -> f64 {
        if self.total_words == 0 {
            return 0.0;
        }
        (f64::from(self.tagged_words) / f64::from(self.total_words)) * 100.0
    }
}
