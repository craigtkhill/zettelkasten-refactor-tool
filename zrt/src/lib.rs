//! Zettelkasten Refactor Tool
//!
//! A tool for analyzing and managing refactoring tasks in a Zettelkasten note system.
//! Provides functionality for scanning directories, counting files and words,
//! and tracking refactoring progress through front matter tags.

pub mod cli;
pub mod core;
pub mod count;
pub mod init;
pub mod search;
pub mod wordcount;

pub use core::frontmatter::{Frontmatter, parse_frontmatter};
pub use core::ignore::{Patterns, load_ignore_patterns};
pub use core::scanner::utils::is_hidden;
pub use init::{RefactorConfig, SortBy, ZrtConfig};
pub use wordcount::models::{FileMetrics, FileWordCount, WordCountStats};
pub use wordcount::{count_file_metrics, count_words, print_file_metrics, print_top_files};
