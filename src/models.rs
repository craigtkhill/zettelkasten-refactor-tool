// src/models.rs
use std::path::PathBuf;

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
    #[must_use] pub const fn new() -> Self {
        Self {
            total_files: 0,
            files_with_pattern: 0,
        }
    }

    #[must_use] pub fn calculate_percentage(&self) -> f64 {
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
    #[must_use] pub const fn new() -> Self {
        Self {
            total_files: 0,
            done_files: 0,
            todo_files: 0,
        }
    }

    #[must_use] pub fn calculate_percentage(&self) -> f64 {
        let total_tagged = self.done_files + self.todo_files;
        if total_tagged == 0 {
            return 0.0;
        }
        (self.done_files as f64 / total_tagged as f64) * 100.0
    }
}