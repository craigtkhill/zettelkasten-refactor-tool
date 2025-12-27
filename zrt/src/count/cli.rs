// src/count/cli.rs
use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

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
        count: CountArgs,
    }

    #[test]
    fn test_count_files_flag() {
        // REQ-COUNT-015
        let args = TestArgs::parse_from(["program", "--files", "refactor"]);
        assert!(args.count.files);
        assert_eq!(args.count.tags, vec!["refactor"]);
    }

    #[test]
    fn test_count_words_flag() {
        // REQ-COUNT-016
        let args = TestArgs::parse_from(["program", "--words", "refactor"]);
        assert!(args.count.words);
        assert_eq!(args.count.tags, vec!["refactor"]);
    }

    #[test]
    fn test_count_percentage_flag() {
        // REQ-COUNT-017
        let args = TestArgs::parse_from(["program", "--percentage", "refactor"]);
        assert!(args.count.percentage);
        assert_eq!(args.count.tags, vec!["refactor"]);
    }

    #[test]
    fn test_count_multiple_tags() {
        let args = TestArgs::parse_from(["program", "--files", "refactor", "draft"]);
        assert_eq!(args.count.tags, vec!["refactor", "draft"]);
    }

    #[test]
    fn test_count_no_tags() {
        let args = TestArgs::parse_from(["program", "--files"]);
        assert!(args.count.tags.is_empty());
    }

    #[test]
    fn test_count_multiple_directories() {
        let args = TestArgs::parse_from(["program", "--files", "-d", "dir1", "dir2"]);
        assert_eq!(args.count.directories.len(), 2);
    }

    #[test]
    fn test_count_no_exclude_defaults_to_empty() {
        let args = TestArgs::parse_from(["program", "--files"]);
        assert!(args.count.exclude.is_empty());
    }
}

// ============================================
// TYPE DEFINITIONS
// ============================================

#[derive(Args, Debug)]
pub struct CountArgs {
    /// Directories to scan (space-separated, defaults to current directory)
    #[arg(short = 'd', long = "dir", num_args = 0.., default_values = &["."])]
    pub directories: Vec<PathBuf>,

    /// Tags to filter by (space-separated, omit to count all)
    #[arg(num_args = 0..)]
    pub tags: Vec<String>,

    /// Directories to exclude (space-separated)
    #[arg(short, long, num_args = 0..)]
    pub exclude: Vec<String>,

    /// Count files
    #[arg(long, group = "count_type")]
    pub files: bool,

    /// Count words
    #[arg(long, group = "count_type")]
    pub words: bool,

    /// Calculate percentage
    #[arg(long, group = "count_type")]
    pub percentage: bool,
}

// ============================================
// IMPLEMENTATIONS
// ============================================

pub fn run(args: CountArgs) -> Result<()> {
    // Ensure exactly one flag is provided
    let flags_set = [args.files, args.words, args.percentage]
        .iter()
        .filter(|&&f| f)
        .count();
    if flags_set != 1 {
        anyhow::bail!("Exactly one of --files, --words, or --percentage must be specified");
    }

    let exclude_dirs: Vec<&str> = args.exclude.iter().map(String::as_str).collect();
    let tag_refs: Vec<&str> = args.tags.iter().map(String::as_str).collect();

    if args.files {
        let count = crate::count::count_files(&args.directories, &tag_refs, &exclude_dirs)?;
        println!("{}", count);
    } else if args.words {
        let count = crate::count::count_words(&args.directories, &tag_refs, &exclude_dirs)?;
        println!("{}", count);
    } else if args.percentage {
        let pct =
            crate::count::calculate_percentage(&args.directories, &tag_refs, &exclude_dirs)?;
        println!("{:.2}", pct);
    }

    Ok(())
}
