// src/cli.rs
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use crate::core::scanner::{
    count_files, count_words, scan_directory_single_pattern, scan_directory_two_patterns,
};
use crate::utils::print_top_files;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Directory to scan (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    pub directory: PathBuf,

    /// Show total file count only
    #[arg(short = 'c', long)]
    pub count: bool,

    /// Show word counts instead of refactor percentage
    #[arg(short, long)]
    pub words: bool,

    /// Number of files to show in word count mode
    #[arg(short = 't', long, default_value = "10")]
    pub top: usize,

    /// Directories to exclude in word count mode (comma-separated)
    #[arg(short, long, default_value = ".git")]
    pub exclude: String,

    /// Filter out files containing this tag (e.g., "refactored")
    #[arg(short = 'f', long)]
    pub filter_out: Option<String>,

    /// Single pattern to search for (e.g., "to_refactor")
    #[arg(short = 'p', long)]
    pub pattern: Option<String>,

    /// "Done" tag to search for (e.g., "refactored")
    #[arg(short = 'r', long)]
    pub done_tag: Option<String>,

    /// "Todo" tag to search for (e.g., "to_refactor")
    #[arg(short = 'o', long)]
    pub todo_tag: Option<String>,
}

pub fn run(args: Args) -> Result<()> {
    if args.count {
        let exclude_dirs: Vec<&str> = args.exclude.split(',').collect();
        let count = count_files(&args.directory, &exclude_dirs)?;
        println!("{count}");
    } else if args.words {
        let exclude_dirs: Vec<&str> = args.exclude.split(',').collect();
        let files = count_words(&args.directory, &exclude_dirs, args.filter_out.as_deref())?;
        print_top_files(files, args.top);
    } else if let Some(pattern) = args.pattern {
        // Single pattern mode
        let stats = scan_directory_single_pattern(&args.directory, &pattern)?;
        println!("Total files: {}", stats.total_files);
        println!(
            "Files with pattern '{}': {}",
            pattern, stats.files_with_pattern
        );
        println!("Percentage: {:.2}%", stats.calculate_percentage());
    } else if let (Some(done), Some(todo)) = (args.done_tag, args.todo_tag) {
        // Compare two tags mode
        let stats = scan_directory_two_patterns(&args.directory, &done, &todo)?;
        println!("{} files: {}", done, stats.done_files);
        println!("{} files: {}", todo, stats.todo_files);
        println!("Done percentage: {:.2}%", stats.calculate_percentage());
    } else {
        // Default behavior - scan for to_refactor
        let default_pattern = String::from("to_refactor");
        let stats = scan_directory_single_pattern(&args.directory, &default_pattern)?;
        println!("Total files: {}", stats.total_files);
        println!(
            "Files with pattern '{}': {}",
            default_pattern, stats.files_with_pattern
        );
        println!("Percentage: {:.2}%", stats.calculate_percentage());
    }

    Ok(())
}
