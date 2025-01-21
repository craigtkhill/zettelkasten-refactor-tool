// src/main.rs
use clap::Parser;
use anyhow::Result;

mod cli;
mod core;
mod models;
mod utils;

use cli::Args;
use crate::core::scanner::{count_files, count_words, scan_directory_single_pattern, scan_directory_two_patterns};
use utils::print_top_files;

fn main() -> Result<()> {
    let args = Args::parse();
    run(args)
}

fn run(args: Args) -> Result<()> {
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