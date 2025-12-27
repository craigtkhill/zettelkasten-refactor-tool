use std::path::PathBuf;

// ============================================
// TESTS
// ============================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_metrics_new() {
        let path = PathBuf::from("test.md");
        let tags = vec!["tag1".to_owned(), "tag2".to_owned()];
        let metrics = FileMetrics::new(path.clone(), 100, 20, tags.clone());

        assert_eq!(metrics.path, path);
        assert_eq!(metrics.words, 100);
        assert_eq!(metrics.lines, 20);
        assert_eq!(metrics.tags, tags);
    }

    #[test]
    fn test_exceeds_thresholds() {
        let metrics = FileMetrics::new(PathBuf::from("test.md"), 100, 20, vec![]);

        assert!(metrics.exceeds_thresholds(50, 10)); // Both exceeded
        assert!(metrics.exceeds_thresholds(50, 30)); // Words exceeded
        assert!(metrics.exceeds_thresholds(150, 10)); // Lines exceeded
        assert!(!metrics.exceeds_thresholds(150, 30)); // Neither exceeded
        assert!(metrics.exceeds_thresholds(100, 20)); // Equal counts (>= comparison)
    }

    #[test]
    fn test_conversion_to_file_word_count() {
        let metrics = FileMetrics::new(PathBuf::from("test.md"), 150, 25, vec!["draft".to_owned()]);

        let word_count: FileWordCount = metrics.into();
        assert_eq!(word_count.path, PathBuf::from("test.md"));
        assert_eq!(word_count.words, 150);
    }
}

// ============================================
// TYPE DEFINITIONS
// ============================================

#[derive(Debug, Clone)]
pub struct FileMetrics {
    pub path: PathBuf,
    pub words: usize,
    pub lines: usize,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub struct FileWordCount {
    pub path: PathBuf,
    pub words: usize,
}

#[derive(Debug, Default)]
pub struct WordCountStats {
    pub tagged_files: u32,
    pub tagged_words: u32,
    pub total_files: u32,
    pub total_words: u32,
}

// ============================================
// IMPLEMENTATIONS
// ============================================

impl FileMetrics {
    #[inline]
    #[must_use]
    pub fn new(path: PathBuf, words: usize, lines: usize, tags: Vec<String>) -> Self {
        Self {
            path,
            words,
            lines,
            tags,
        }
    }

    #[inline]
    #[must_use]
    pub fn exceeds_thresholds(&self, word_threshold: usize, line_threshold: usize) -> bool {
        self.words >= word_threshold || self.lines >= line_threshold
    }
}

impl From<FileMetrics> for FileWordCount {
    #[inline]
    fn from(metrics: FileMetrics) -> Self {
        Self {
            path: metrics.path,
            words: metrics.words,
        }
    }
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
