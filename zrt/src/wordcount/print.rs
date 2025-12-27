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
        println!("{:8} words  {}", file.words, file.path.display());
    }
}

#[inline]
pub fn print_file_metrics(
    files: &[FileMetrics],
    top: usize,
    sort_by: SortBy,
    thresholds: Option<(usize, usize)>,
) {
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

    // Print header with thresholds if provided
    if let Some((word_threshold, line_threshold)) = thresholds {
        println!(
            "Files exceeding size thresholds ({word_threshold}+ words, {line_threshold}+ lines):"
        );
    }

    // Print files with their metrics
    for file in sorted_files.iter().take(top) {
        let tags_display = if file.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", file.tags.join(", "))
        };

        println!(
            "{:8} words  {:4} lines  {}{}",
            file.words,
            file.lines,
            file.path.display(),
            tags_display
        );
    }
}
