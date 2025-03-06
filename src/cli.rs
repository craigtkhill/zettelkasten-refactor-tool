// src/cli.rs
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use crate::core::scanner::{
    count_files, count_word_stats, count_words, scan_directory_single, scan_directory_two,
};
use crate::utils::print_top_files;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Show total file count only
    #[arg(short = 'c', long = "count")]
    pub count: bool,

    /// Directory to scan (defaults to current directory)
    #[arg(short = 'd', long = "dir", default_value = ".")]
    pub directory: PathBuf,

    /// "Done" tag to search for (e.g., "refactored")
    #[arg(short = 'r', long = "done")]
    pub done_tag: Option<String>,

    /// Directories to exclude in word count mode (comma-separated)
    #[arg(short, long, default_value = ".git")]
    pub exclude: String,

    /// Filter out files containing this tag (e.g., "refactored")
    #[arg(short = 'f', long = "filter")]
    pub filter_out: Option<String>,

    /// Single pattern to search for (e.g., `to_refactor`)
    #[arg(short = 't', long = "tag")]
    pub pattern: Option<String>,

    /// Show word count statistics for files with a specific tag
    #[arg(short = 's', long = "stats")]
    pub stats: Option<String>,

    /// "Todo" tag to search for (e.g., `to_refactor`)
    #[arg(short = 'u', long = "todo")]
    pub todo_tag: Option<String>,

    /// Number of files to show in word count mode
    #[arg(short = 'n', long = "num", default_value = "10")]
    pub top: usize,

    /// Show word counts instead of refactor percentage
    #[arg(short = 'w', long = "wordcount")]
    pub words: bool,
}
/// Runs the tool with the provided arguments.
///
/// # Arguments
///
/// * `args` - Command-line arguments parsed into an `Args` struct
///
/// # Returns
///
/// * `Ok(())` if the command completes successfully
///
/// # Errors
///
/// This function may return an error if:
/// * The specified directory cannot be read
/// * File operations fail during counting or scanning
/// * Pattern matching operations encounter an error
#[inline]
pub fn run(args: Args) -> Result<()> {
    if args.count {
        let exclude_dirs: Vec<&str> = args.exclude.split(',').collect();
        let count = count_files(&args.directory, &exclude_dirs)?;
        println!("{count}");
    } else if let Some(tag) = args.stats.as_ref() {
        // New word count statistics mode
        let exclude_dirs: Vec<&str> = args.exclude.split(',').collect();
        let stats = count_word_stats(&args.directory, &exclude_dirs, tag)?;

        println!("Files with tag '{}': {}", tag, stats.tagged_files);
        println!("Words in tagged files: {}", stats.tagged_words);
        println!("Total files: {}", stats.total_files);
        println!("Total words in all files: {}", stats.total_words);
        println!(
            "Percentage of words tagged: {:.2}%",
            stats.calculate_percentage()
        );
    } else if args.words {
        let exclude_dirs: Vec<&str> = args.exclude.split(',').collect();
        let files = count_words(&args.directory, &exclude_dirs, args.filter_out.as_deref())?;
        print_top_files(&files, args.top);
    } else if let Some(pattern) = args.pattern {
        // Single pattern mode
        let stats = scan_directory_single(&args.directory, &pattern)?;
        println!("Total files: {}", stats.total_files);
        println!(
            "Files with pattern '{}': {}",
            pattern, stats.files_with_pattern
        );
        println!("Percentage: {:.2}%", stats.calculate_percentage());
    } else if let (Some(done), Some(todo)) = (args.done_tag, args.todo_tag) {
        // Compare two tags mode
        let stats = scan_directory_two(&args.directory, &done, &todo)?;
        println!("{} files: {}", done, stats.done);
        println!("{} files: {}", todo, stats.todo);
        println!("Done percentage: {:.2}%", stats.calculate_percentage());
    } else {
        // Default behavior - scan for to_refactor
        let default_pattern = String::from("to_refactor");
        let stats = scan_directory_single(&args.directory, &default_pattern)?;
        println!("Total files: {}", stats.total_files);
        println!(
            "Files with pattern '{}': {}",
            default_pattern, stats.files_with_pattern
        );
        println!("Percentage: {:.2}%", stats.calculate_percentage());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_default_directory_is_current() {
        let args = Args::parse_from(["program"]);
        assert_eq!(args.directory, PathBuf::from("."));
    }

    #[test]
    fn test_custom_directory_is_parsed() {
        let args = Args::parse_from(["program", "-d", "/some/path"]);
        assert_eq!(args.directory, PathBuf::from("/some/path"));
    }

    #[test]
    fn test_count_flag_is_parsed() {
        let args = Args::parse_from(["program", "--count"]);
        assert!(args.count);
    }

    #[test]
    fn test_run_with_count_flag() -> Result<()> {
        let args = Args {
            directory: PathBuf::from("."),
            count: true,
            words: false,
            stats: None,
            top: 10,
            exclude: ".git".to_owned(),
            filter_out: None,
            pattern: None,
            done_tag: None,
            todo_tag: None,
        };

        run(args)?;
        Ok(())
    }
    #[test]
    fn test_stats_option_is_parsed() {
        let args = Args::parse_from(["program", "--stats", "refactored"]);
        assert_eq!(args.stats, Some("refactored".to_owned()));
    }

    #[test]
    fn test_run_with_stats_option() -> Result<()> {
        let args = Args {
            directory: PathBuf::from("."),
            count: false,
            words: false,
            stats: Some("refactored".to_owned()),
            top: 10,
            exclude: ".git".to_owned(),
            filter_out: None,
            pattern: None,
            done_tag: None,
            todo_tag: None,
        };

        // This test just ensures the function doesn't panic
        // We can't easily test the output
        run(args)?;
        Ok(())
    }
}
