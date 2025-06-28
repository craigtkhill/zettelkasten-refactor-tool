// src/core/scanner.rs
pub mod file;
pub mod pattern;
mod utils;
pub mod word;

#[cfg(test)]
pub mod test_utils;

pub use file::count_files;
pub use pattern::{scan_directory_only_tag, scan_directory_single, scan_directory_two};
pub use word::{count_word_stats, count_words};
