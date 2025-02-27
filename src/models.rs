// src/models.rs
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Default)]
pub struct Frontmatter {
    pub tags: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct FileWordCount {
    pub path: PathBuf,
    pub words: usize,
}

#[derive(Debug, Default)]
pub struct SinglePatternStats {
    pub total_files: u64,
    pub files_with_pattern: u64,
}

impl SinglePatternStats {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_files: 0,
            files_with_pattern: 0,
        }
    }

    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "Precision not critical")]
    pub fn calculate_percentage(&self) -> f64 {
        if self.total_files == 0 {
            return 0.0;
        }
        (self.files_with_pattern as f64 / self.total_files as f64) * 100.0
    }
}

#[derive(Debug, Default)]
pub struct ComparisonStats {
    pub total_files: u64,
    pub done_files: u64,
    pub todo_files: u64,
}

impl ComparisonStats {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_files: 0,
            done_files: 0,
            todo_files: 0,
        }
    }

    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "Precision not critical")]
    pub fn calculate_percentage(&self) -> f64 {
        let total_tagged = self.done_files + self.todo_files;
        if total_tagged == 0 {
            return 0.0;
        }
        (self.done_files as f64 / total_tagged as f64) * 100.0
    }
}

#[derive(Debug, Default)]
pub struct WordCountStats {
    pub total_files: u64,
    pub tagged_files: u64,
    pub total_words: u64,
    pub tagged_words: u64,
}

impl WordCountStats {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_files: 0,
            tagged_files: 0,
            total_words: 0,
            tagged_words: 0,
        }
    }

    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "Precision not critical")]
    pub fn calculate_percentage(&self) -> f64 {
        if self.total_words == 0 {
            return 0.0;
        }
        (self.tagged_words as f64 / self.total_words as f64) * 100.0
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
            total_files: 10,
            files_with_pattern: 5,
        };
        assert_eq!(stats.calculate_percentage(), 50.0);
    }

    #[test]
    fn test_comparison_stats_zero_files() {
        let stats = ComparisonStats::new();
        assert_eq!(stats.calculate_percentage(), 0.0);
    }

    #[test]
    fn test_comparison_stats_fifty_percent() {
        let stats = ComparisonStats {
            total_files: 20,
            done_files: 5,
            todo_files: 5,
        };
        assert_eq!(stats.calculate_percentage(), 50.0);
    }

    #[test]
    fn test_frontmatter_deserialize() {
        let yaml = "
            tags:
              - tag1
              - tag2
        ";
        let frontmatter: Frontmatter = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(frontmatter.tags.unwrap(), vec!["tag1", "tag2"]);
    }

    #[test]
    fn test_frontmatter_no_tags() {
        let yaml = "{}";
        let frontmatter: Frontmatter = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(frontmatter.tags.is_none());
    }
}
