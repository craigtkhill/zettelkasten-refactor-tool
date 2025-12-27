// src/cli.rs
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::core::scanner::{
    count_file_metrics, count_word_stats, count_words, scan_directory_only_tag,
};
use crate::init::{SortBy, ZrtConfig};
use crate::utils::{print_file_metrics, print_top_files};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize ZRT configuration
    Init,

    /// Show word count statistics for files with a specific tag
    Stats {
        /// Directories to scan (space-separated, defaults to current directory)
        #[arg(short = 'd', long = "dir", num_args = 0.., default_values = &["."])]
        directories: Vec<PathBuf>,

        /// Tag to analyze
        tag: String,

        /// Directories to exclude (comma-separated)
        #[arg(short, long, default_value = ".git")]
        exclude: String,
    },

    /// Show files ordered by word count (alias: wc)
    #[command(alias = "wc")]
    Wordcount {
        /// Directories to scan (space-separated, defaults to current directory)
        #[arg(short = 'd', long = "dir", num_args = 0.., default_values = &["."])]
        directories: Vec<PathBuf>,

        /// Filter out files containing these tags (space-separated)
        #[arg(short = 'f', long = "filter", num_args = 0..)]
        filter_out: Vec<String>,

        /// Number of files to show
        #[arg(short = 'n', long = "num", default_value = "10")]
        top: usize,

        /// Directories to exclude (space-separated)
        #[arg(short, long, num_args = 0.., default_values = &[".git"])]
        exclude: Vec<String>,

        /// Only show files exceeding configured thresholds
        #[arg(long)]
        exceeds: bool,

        /// Sort by words or lines (overrides config)
        #[arg(long, value_enum)]
        sort_by: Option<SortBy>,
    },

    /// Show files that have only a specific tag
    Only {
        /// Directories to scan (space-separated, defaults to current directory)
        #[arg(short = 'd', long = "dir", num_args = 0.., default_values = &["."])]
        directories: Vec<PathBuf>,

        /// Tag to filter by
        tag: String,
    },
}

#[inline]
pub fn run(args: Args) -> Result<()> {
    match args.command {
        Commands::Init => crate::init::run(None),
        Commands::Stats {
            directories,
            tag,
            exclude,
        } => {
            let exclude_dirs: Vec<&str> = exclude.split(',').collect();
            let stats = count_word_stats(&directories, &exclude_dirs, &tag)?;

            println!("Files with tag '{}': {}", tag, stats.tagged_files);
            println!("Words in tagged files: {}", stats.tagged_words);
            println!("Total files: {}", stats.total_files);
            println!("Total words in all files: {}", stats.total_words);
            println!(
                "Percentage of words tagged: {:.2}%",
                stats.calculate_percentage()
            );
            Ok(())
        }
        Commands::Wordcount {
            directories,
            filter_out,
            top,
            exclude,
            exceeds,
            sort_by,
        } => {
            let exclude_dirs: Vec<&str> = exclude.iter().map(String::as_str).collect();
            let filter_tags: Vec<&str> = filter_out.iter().map(String::as_str).collect();

            if exceeds {
                let config = ZrtConfig::load_or_default();
                let sort_preference = sort_by.unwrap_or(config.refactor.sort_by);

                let metrics = count_file_metrics(
                    &directories,
                    &exclude_dirs,
                    &filter_tags,
                    Some((
                        config.refactor.word_threshold,
                        config.refactor.line_threshold,
                    )),
                )?;

                print_file_metrics(
                    &metrics,
                    top,
                    sort_preference,
                    Some((
                        config.refactor.word_threshold,
                        config.refactor.line_threshold,
                    )),
                );
            } else {
                let files = count_words(
                    &directories,
                    &exclude_dirs,
                    if filter_tags.is_empty() {
                        None
                    } else {
                        Some(filter_tags[0])
                    },
                )?;
                print_top_files(&files, top);
            }

            Ok(())
        }
        Commands::Only { directories, tag } => {
            let stats = scan_directory_only_tag(&directories, &tag)?;
            println!("Total files: {}", stats.total_files);
            println!(
                "Files with only tag '{}': {}",
                tag, stats.files_with_pattern
            );
            println!("Percentage: {:.2}%", stats.calculate_percentage());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wordcount_command_parsing() {
        let args = Args::parse_from(["program", "wordcount", "-n", "5"]);
        if let Commands::Wordcount { top, .. } = args.command {
            assert_eq!(top, 5);
        } else {
            panic!("Expected Wordcount command");
        }
    }

    #[test]
    fn test_wordcount_alias_parsing() {
        let args = Args::parse_from(["program", "wc", "-n", "5", "--exceeds"]);
        if let Commands::Wordcount { top, exceeds, .. } = args.command {
            assert_eq!(top, 5);
            assert!(exceeds);
        } else {
            panic!("Expected Wordcount command");
        }
    }

}
