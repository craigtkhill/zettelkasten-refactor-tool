//! Zettelkasten Refactor Tool
//!
//! A tool for analyzing and managing refactoring tasks in a Zettelkasten note system.
//! Provides functionality for scanning directories, counting files and words,
//! and tracking refactoring progress through front matter tags.

pub mod cli;
pub mod core;
pub mod count;
pub mod init;
pub mod models;
pub mod utils;

pub use core::ignore::{Patterns, load_ignore_patterns};
pub use core::scanner::{
    count_file_metrics, count_words, scan_directory_only_tag,
};
pub use models::{
    FileMetrics, FileWordCount, Frontmatter, SinglePatternStats, WordCountStats,
};
pub use init::{RefactorConfig, SortBy, ZrtConfig};
pub use utils::{is_hidden, parse_frontmatter, print_file_metrics, print_top_files};
