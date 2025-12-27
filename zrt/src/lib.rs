//! Zettelkasten Refactor Tool
//!
//! A tool for analyzing and managing refactoring tasks in a Zettelkasten note system.
//! Provides functionality for scanning directories, counting files and words,
//! and tracking refactoring progress through front matter tags.

#![allow(
    clippy::multiple_crate_versions,
    reason = "ML dependencies have complex version requirements"
)]
// Development phase allows - will be removed before release
#![allow(clippy::absolute_paths, reason = "Development: std:: paths are clear")]
#![allow(
    clippy::exhaustive_enums,
    reason = "Development: CLI enums are internal"
)]
#![allow(
    clippy::missing_errors_doc,
    reason = "Development: docs will be completed"
)]
#![allow(
    clippy::unnecessary_wraps,
    reason = "Development: error handling consistency"
)]
#![allow(
    clippy::semicolon_outside_block,
    reason = "Development: formatting preference"
)]

// Declare all modules
pub mod cli;
pub mod core;
pub mod models;
pub mod settings;
pub mod utils;

// Re-export main types and functions for convenience
pub use core::ignore::{Patterns, load_ignore_patterns};
pub use core::scanner::{
    count_file_metrics, count_words, scan_directory_only_tag,
};
pub use models::{
    FileMetrics, FileWordCount, Frontmatter, SinglePatternStats, WordCountStats,
};
pub use settings::{RefactorConfig, SortBy, ZrtConfig};
pub use utils::{contains_tag, is_hidden, parse_frontmatter, print_file_metrics, print_top_files};
