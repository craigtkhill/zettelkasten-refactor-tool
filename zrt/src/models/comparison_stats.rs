// src/models/comparison_stats.rs

#[derive(Debug, Default)]
pub struct ComparisonStats {
    pub done: u32,
    pub todo: u32,
    pub total: u32,
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
    pub fn calculate_percentage(&self) -> f64 {
        let total_tagged = self.done.saturating_add(self.todo);
        if total_tagged == 0 {
            return 0.0;
        }
        (f64::from(self.done) / f64::from(total_tagged)) * 100.0
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
