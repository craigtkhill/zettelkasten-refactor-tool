// src/models/comparison_stats.rs

#[derive(Debug, Default)]
pub struct ComparisonStats {
    pub done: u64,
    pub todo: u64,
    pub total: u64,
}

impl ComparisonStats {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            done: 0,
            todo: 0,
            total: 0,
        }
    }

    #[inline]
    #[must_use]
    #[expect(clippy::as_conversions, reason = "Precision not critical")]
    #[expect(clippy::cast_precision_loss, reason = "Precision not critical")]
    pub fn calculate_percentage(&self) -> f64 {
        let total_tagged = self.done.saturating_add(self.todo);
        if total_tagged == 0 {
            return 0.0;
        }
        (self.done as f64 / total_tagged as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_stats_zero_files() {
        let stats = ComparisonStats::new();
        assert_eq!(stats.calculate_percentage(), 0.0);
    }

    #[test]
    fn test_comparison_stats_fifty_percent() {
        let stats = ComparisonStats {
            done: 5,
            total: 20,
            todo: 5,
        };
        assert_eq!(stats.calculate_percentage(), 50.0);
    }
}
