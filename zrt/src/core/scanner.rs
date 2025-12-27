// src/core/scanner.rs
pub mod pattern;
mod utils;
pub mod word;

#[cfg(test)]
pub mod test_utils;

pub use pattern::scan_directory_only_tag;
pub use word::{count_file_metrics, count_word_stats, count_words};
