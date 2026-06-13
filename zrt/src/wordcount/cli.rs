use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::init::{SortBy, ZrtConfig};
use crate::wordcount::{count_file_metrics, count_words, print_file_metrics, print_top_files};

// ============================================
// TESTS
// ============================================
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct TestArgs {
        #[command(flatten)]
        wc: WordcountArgs,
    }

    #[test]
    fn test_wordcount_args_parsing() {
        let args = TestArgs::parse_from(["program", "-n", "5"]);
        assert_eq!(args.wc.top, 5);
    }

    #[test]
    fn test_wordcount_alias() {
        // Alias would be tested at the command level
        let args = TestArgs::parse_from(["program", "-n", "5", "--exceeds"]);
        assert_eq!(args.wc.top, 5);
        assert!(args.wc.exceeds);
    }

    #[test]
    fn test_wordcount_with_filter() {
        let args = TestArgs::parse_from(["program", "-f", "draft", "wip"]);
        assert_eq!(args.wc.filter_out, vec!["draft", "wip"]);
    }

    #[test]
    fn test_wordcount_with_directories() {
        let args = TestArgs::parse_from(["program", "-d", "dir1", "dir2"]);
        assert_eq!(args.wc.directories.len(), 2);
    }

    #[test]
    fn test_wordcount_with_exclude() {
        let args = TestArgs::parse_from(["program", "-e", "node_modules", "target"]);
        assert_eq!(args.wc.exclude, vec!["node_modules", "target"]);
    }

    #[test]
    fn test_wordcount_sort_by() {
        let args = TestArgs::parse_from(["program", "--sort-by", "lines"]);
        assert!(args.wc.sort_by.is_some());
    }

    #[test]
    fn test_should_parse_with_tags_long_flag() {
        // REQ-WC-004b
        let args = TestArgs::parse_from(["program", "--with-tags", "long_file", "draft"]);
        assert_eq!(args.wc.with_tags, vec!["long_file", "draft"]);
    }

    #[test]
    fn test_should_parse_with_tags_short_flag() {
        // REQ-WC-004b
        let args = TestArgs::parse_from(["program", "-t", "long_file"]);
        assert_eq!(args.wc.with_tags, vec!["long_file"]);
    }

    #[test]
    fn test_should_parse_below_flag() {
        // REQ-WC-011
        let args = TestArgs::parse_from(["program", "--below"]);
        assert!(args.wc.below);
    }
}

// ============================================
// TYPE DEFINITIONS
// ============================================

#[derive(Args, Debug)]
pub struct WordcountArgs {
    /// Directories to scan (space-separated, defaults to current directory)
    #[arg(short = 'd', long = "dir", num_args = 0.., default_values = &["."])]
    pub directories: Vec<PathBuf>,

    /// Filter out files containing these tags (space-separated)
    #[arg(short = 'f', long = "filter", num_args = 0..)]
    pub filter_out: Vec<String>,

    /// Number of files to show
    #[arg(short = 'n', long = "num", default_value = "10")]
    pub top: usize,

    /// Directories to exclude (space-separated)
    #[arg(short, long, num_args = 0.., default_values = &[".git"])]
    pub exclude: Vec<String>,

    /// Only show files containing these tags (space-separated)
    #[arg(short = 't', long = "with-tags", num_args = 0..)]
    pub with_tags: Vec<String>,

    /// Only show files exceeding configured thresholds
    #[arg(long)]
    pub exceeds: bool,

    /// Only show files below configured thresholds
    #[arg(long)]
    pub below: bool,

    /// Sort by words or lines (overrides config)
    #[arg(long, value_enum)]
    pub sort_by: Option<SortBy>,
}

// ============================================
// IMPLEMENTATIONS
// ============================================

pub fn run(args: WordcountArgs) -> Result<()> {
    let exclude_dirs: Vec<&str> = args.exclude.iter().map(String::as_str).collect();
    let filter_tags: Vec<&str> = args.filter_out.iter().map(String::as_str).collect();
    let with_tags: Vec<&str> = args.with_tags.iter().map(String::as_str).collect();

    if args.exceeds || args.below || !with_tags.is_empty() {
        let config = ZrtConfig::load_or_default();
        let sort_preference = args.sort_by.unwrap_or(config.refactor.sort_by);

        let thresholds = if args.exceeds {
            Some((
                config.refactor.word_threshold,
                config.refactor.line_threshold,
                false,
            ))
        } else if args.below {
            Some((
                config.refactor.word_threshold,
                config.refactor.line_threshold,
                true,
            ))
        } else {
            None
        };

        let metrics = count_file_metrics(
            &args.directories,
            &exclude_dirs,
            &filter_tags,
            thresholds,
            &with_tags,
        )?;

        print_file_metrics(&metrics, args.top, sort_preference);
    } else {
        let files = count_words(
            &args.directories,
            &exclude_dirs,
            if filter_tags.is_empty() {
                None
            } else {
                Some(filter_tags[0])
            },
        )?;
        print_top_files(&files, args.top);
    }

    Ok(())
}
