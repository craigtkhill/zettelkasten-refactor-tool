use crate::init::SortBy;
use crate::wordcount::models::{FileMetrics, FileWordCount};

// ============================================
// TESTS
// ============================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_print_top_files() {
        let files = vec![
            FileWordCount {
                path: PathBuf::from("test.txt"),
                words: 100,
            },
            FileWordCount {
                path: PathBuf::from("test2.txt"),
                words: 50,
            },
        ];

        // Here we could capture stdout to verify the output format
        print_top_files(&files, 1);
    }
}

// ============================================
// IMPLEMENTATIONS
// ============================================

#[inline]
pub fn print_top_files(files: &[FileWordCount], top: usize) {
    for file in files.iter().take(top) {
        println!("{}", file.path.display());
    }
}

#[inline]
pub fn print_file_metrics(files: &[FileMetrics], top: usize, sort_by: SortBy) {
    let mut sorted_files = files.to_vec();

    // Sort by the specified criteria
    match sort_by {
        SortBy::Words => {
            sorted_files.sort_by(|a, b| b.words.cmp(&a.words));
        }
        SortBy::Lines => {
            sorted_files.sort_by(|a, b| b.lines.cmp(&a.lines));
        }
    }

    // Print files (just paths)
    for file in sorted_files.iter().take(top) {
        println!("{}", file.path.display());
    }
}
