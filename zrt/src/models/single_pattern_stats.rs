// src/models/single_pattern_stats.rs

#[derive(Debug, Default)]
pub struct SinglePatternStats {
    pub files_with_pattern: u32,
    pub total_files: u32,
}

impl SinglePatternStats {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_files: 0,
            files_with_pattern: 0,
        }
    }
    #[inline]
    #[must_use]
    pub fn calculate_percentage(&self) -> f64 {
        if self.total_files == 0 {
            return 0.0;
        }
        (f64::from(self.files_with_pattern) / f64::from(self.total_files)) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_pattern_stats_zero_files() {
        let stats = SinglePatternStats::new();
        assert_eq!(stats.calculate_percentage(), 0.0);
    }

    #[test]
    fn test_single_pattern_stats_fifty_percent() {
        let stats = SinglePatternStats {
            files_with_pattern: 5,
            total_files: 10,
        };
        assert_eq!(stats.calculate_percentage(), 50.0);
    }
}
