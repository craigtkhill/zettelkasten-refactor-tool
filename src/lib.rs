//! Zettelkasten Refactor Tool
//!
//! A tool for analyzing and managing refactoring tasks in a Zettelkasten note system.
//! Provides functionality for scanning directories, counting files and words,
//! and tracking refactoring progress through front matter tags.

// Declare all modules
pub mod cli;
pub mod core;
pub mod models;
pub mod utils;

// Re-export main types and functions for convenience
pub use cli::{Args, run};
pub use core::ignore::{IgnorePatterns, load_ignore_patterns};
pub use core::scanner::{
    count_files, count_words, scan_directory_single_pattern, scan_directory_two_patterns,
};
pub use models::{ComparisonStats, FileWordCount, Frontmatter, SinglePatternStats};
pub use utils::{contains_tag, is_hidden, parse_frontmatter, print_top_files};
