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
        let metrics = FileMetrics::new(path.clone(), 100, 20);

        assert_eq!(metrics.path, path);
        assert_eq!(metrics.words, 100);
        assert_eq!(metrics.lines, 20);
    }

    #[test]
    fn test_exceeds_thresholds() {
        let metrics = FileMetrics::new(PathBuf::from("test.md"), 100, 20);

        assert!(metrics.exceeds_thresholds(50, 10)); // Both exceeded
        assert!(metrics.exceeds_thresholds(50, 30)); // Words exceeded
        assert!(metrics.exceeds_thresholds(150, 10)); // Lines exceeded
        assert!(!metrics.exceeds_thresholds(150, 30)); // Neither exceeded
        assert!(metrics.exceeds_thresholds(100, 20)); // Equal counts (>= comparison)
    }

    #[test]
    fn test_conversion_to_file_word_count() {
        let metrics = FileMetrics::new(PathBuf::from("test.md"), 150, 25);

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
}

#[derive(Debug)]
pub struct FileWordCount {
    pub path: PathBuf,
    pub words: usize,
}

// ============================================
// IMPLEMENTATIONS
// ============================================

impl FileMetrics {
    #[inline]
    #[must_use]
    pub fn new(path: PathBuf, words: usize, lines: usize) -> Self {
        Self {
            path,
            words,
            lines,
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
