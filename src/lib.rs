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
pub use core::ignore::{Patterns, load_ignore_patterns};
pub use core::scanner::{count_files, count_words, scan_directory_single, scan_directory_two};
pub use models::{ComparisonStats, FileWordCount, Frontmatter, SinglePatternStats, WordCountStats};
pub use utils::{contains_tag, is_hidden, parse_frontmatter, print_top_files};
