// src/cli.rs
#![allow(clippy::absolute_paths, reason = "Development: std:: paths are clear")]
#![allow(
    clippy::semicolon_outside_block,
    reason = "Development: formatting preference"
)]
#![allow(
    clippy::unnecessary_wraps,
    reason = "Development: consistency with future error handling"
)]
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::core::scanner::{
    count_files, count_word_stats, count_words, scan_directory_only_tag, scan_directory_single,
    scan_directory_two,
};
use crate::utils::print_top_files;

#[cfg(feature = "tagging")]
use zrt_tagging::Settings;

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

    /// Count files in directory
    Count {
        /// Directory to scan (defaults to current directory)
        #[arg(short = 'd', long = "dir", default_value = ".")]
        directory: PathBuf,

        /// Directories to exclude (comma-separated)
        #[arg(short, long, default_value = ".git")]
        exclude: String,
    },

    /// Show word count statistics for files with a specific tag
    Stats {
        /// Directory to scan (defaults to current directory)
        #[arg(short = 'd', long = "dir", default_value = ".")]
        directory: PathBuf,

        /// Tag to analyze
        tag: String,

        /// Directories to exclude (comma-separated)
        #[arg(short, long, default_value = ".git")]
        exclude: String,
    },

    /// Show files ordered by word count
    Words {
        /// Directory to scan (defaults to current directory)
        #[arg(short = 'd', long = "dir", default_value = ".")]
        directory: PathBuf,

        /// Filter out files containing this tag
        #[arg(short = 'f', long = "filter")]
        filter_out: Option<String>,

        /// Number of files to show
        #[arg(short = 'n', long = "num", default_value = "10")]
        top: usize,

        /// Directories to exclude (comma-separated)
        #[arg(short, long, default_value = ".git")]
        exclude: String,

        /// Show suggested tags for each file
        #[cfg(feature = "tagging")]
        #[arg(long = "suggest-tags")]
        suggest_tags: bool,
    },

    /// Search for files with a specific pattern/tag
    Search {
        /// Directory to scan (defaults to current directory)
        #[arg(short = 'd', long = "dir", default_value = ".")]
        directory: PathBuf,

        /// Pattern to search for
        pattern: String,
    },

    /// Compare two tags
    Compare {
        /// Directory to scan (defaults to current directory)
        #[arg(short = 'd', long = "dir", default_value = ".")]
        directory: PathBuf,

        /// First tag (done)
        done_tag: String,

        /// Second tag (todo)
        todo_tag: String,
    },

    /// Show files that have only a specific tag
    Only {
        /// Directory to scan (defaults to current directory)
        #[arg(short = 'd', long = "dir", default_value = ".")]
        directory: PathBuf,

        /// Tag to filter by
        tag: String,
    },

    #[cfg(feature = "tagging")]
    /// Tag prediction commands
    Predict {
        #[command(subcommand)]
        command: PredictCommands,
    },
}

#[cfg(feature = "tagging")]
#[derive(Subcommand, Debug)]
pub enum PredictCommands {
    /// Train the tag prediction model
    Train {
        /// Directory to scan for training data
        #[arg(short = 'd', long = "dir", default_value = ".")]
        directory: PathBuf,
    },

    /// Suggest tags for files
    Suggest {
        /// Directory to scan (defaults to current directory)  
        #[arg(short = 'd', long = "dir", default_value = ".")]
        directory: PathBuf,

        /// Specific file to suggest tags for
        #[arg(short = 'f', long = "file")]
        file: Option<PathBuf>,

        /// Confidence threshold for suggestions
        #[arg(short = 't', long = "threshold")]
        threshold: Option<f32>,

        /// Number of top results to show
        #[arg(short = 'n', long = "num", default_value = "10")]
        top: usize,
    },

    /// Validate model performance
    Validate {
        /// Directory to scan for validation data
        #[arg(short = 'd', long = "dir", default_value = ".")]
        directory: PathBuf,
    },
}

#[inline]
pub fn run(args: Args) -> Result<()> {
    match args.command {
        Commands::Init => run_init(),
        Commands::Count { directory, exclude } => {
            let exclude_dirs: Vec<&str> = exclude.split(',').collect();
            let count = count_files(&directory, &exclude_dirs)?;
            println!("{count}");
            Ok(())
        }
        Commands::Stats {
            directory,
            tag,
            exclude,
        } => {
            let exclude_dirs: Vec<&str> = exclude.split(',').collect();
            let stats = count_word_stats(&directory, &exclude_dirs, &tag)?;

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
        Commands::Words {
            directory,
            filter_out,
            top,
            exclude,
            #[cfg(feature = "tagging")]
            suggest_tags,
        } => {
            let exclude_dirs: Vec<&str> = exclude.split(',').collect();
            let files = count_words(&directory, &exclude_dirs, filter_out.as_deref())?;
            print_top_files(&files, top);

            #[cfg(feature = "tagging")]
            if suggest_tags {
                println!("\n--- Tag Suggestions ---");
                // TODO: Implement tag suggestions for word count results
                println!("Tag suggestions not yet implemented");
            }

            Ok(())
        }
        Commands::Search { directory, pattern } => {
            let stats = scan_directory_single(&directory, &pattern)?;
            println!("Total files: {}", stats.total_files);
            println!(
                "Files with pattern '{}': {}",
                pattern, stats.files_with_pattern
            );
            println!("Percentage: {:.2}%", stats.calculate_percentage());
            Ok(())
        }
        Commands::Compare {
            directory,
            done_tag,
            todo_tag,
        } => {
            let stats = scan_directory_two(&directory, &done_tag, &todo_tag)?;
            println!("{} files: {}", done_tag, stats.done);
            println!("{} files: {}", todo_tag, stats.todo);
            println!("Done percentage: {:.2}%", stats.calculate_percentage());
            Ok(())
        }
        Commands::Only { directory, tag } => {
            let stats = scan_directory_only_tag(&directory, &tag)?;
            println!("Total files: {}", stats.total_files);
            println!(
                "Files with only tag '{}': {}",
                tag, stats.files_with_pattern
            );
            println!("Percentage: {:.2}%", stats.calculate_percentage());
            Ok(())
        }
        #[cfg(feature = "tagging")]
        Commands::Predict { command } => run_predict_command(command),
    }
}

#[inline]
fn run_init() -> Result<()> {
    let zrt_dir = std::path::Path::new(".zrt");

    if zrt_dir.exists() {
        println!("ZRT directory already exists at .zrt/");
        return Ok(());
    }

    std::fs::create_dir_all(zrt_dir)?;
    std::fs::create_dir_all(zrt_dir.join("models"))?;

    #[cfg(feature = "tagging")]
    {
        let config = Settings::default();
        config.save_to_file(&zrt_dir.join("config.toml"))?;
    }

    println!("Initialized ZRT directory at .zrt/");
    #[cfg(feature = "tagging")]
    println!("Created default tagging configuration at .zrt/config.toml");

    Ok(())
}

#[cfg(feature = "tagging")]
#[inline]
fn run_predict_command(command: PredictCommands) -> Result<()> {
    match command {
        PredictCommands::Train { directory } => {
            println!("Training model with data from: {}", directory.display());

            // Load configuration
            let config_path = std::path::Path::new(".zrt/config.toml");
            let settings = if config_path.exists() {
                zrt_tagging::Settings::load_from_file(config_path)?
            } else {
                println!("No config found at .zrt/config.toml, using defaults");
                zrt_tagging::Settings::default()
            };

            // Extract training data from notes
            let training_data = zrt_tagging::extraction::extract_training_data(&directory)?;

            if training_data.notes.is_empty() {
                println!("No notes found in directory: {}", directory.display());
                return Ok(());
            }

            // Create and train predictor
            let mut predictor = zrt_tagging::Predictor::new(settings)?;
            predictor.train(&training_data)?;

            println!("Training completed successfully!");
            Ok(())
        }
        PredictCommands::Suggest {
            directory,
            file,
            threshold,
            top,
        } => {
            if let Some(file) = file {
                println!("Suggesting tags for: {}", file.display());
            } else {
                println!("Suggesting tags for files in: {}", directory.display());
            }
            if let Some(t) = threshold {
                println!("Using threshold: {t}");
            }
            println!("Showing top {top} results");
            // TODO: Implement suggestion
            Ok(())
        }
        PredictCommands::Validate { directory } => {
            println!("Validating model with data from: {}", directory.display());
            // TODO: Implement validation
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_command_parsing() {
        let args = Args::parse_from(["program", "init"]);
        matches!(args.command, Commands::Init);
    }

    #[test]
    fn test_count_command_parsing() {
        let args = Args::parse_from(["program", "count", "-d", "test"]);
        if let Commands::Count { directory, .. } = args.command {
            assert_eq!(directory, PathBuf::from("test"));
        } else {
            panic!("Expected Count command");
        }
    }

    #[test]
    fn test_words_command_parsing() {
        let args = Args::parse_from(["program", "words", "-n", "5"]);
        if let Commands::Words { top, .. } = args.command {
            assert_eq!(top, 5);
        } else {
            panic!("Expected Words command");
        }
    }

    #[cfg(feature = "tagging")]
    #[test]
    fn test_predict_suggest_parsing() {
        let args = Args::parse_from(["program", "predict", "suggest", "-t", "0.8"]);
        if let Commands::Predict { command } = args.command {
            if let PredictCommands::Suggest { threshold, .. } = command {
                assert_eq!(threshold, Some(0.8));
            } else {
                panic!("Expected Suggest subcommand");
            }
        } else {
            panic!("Expected Predict command");
        }
    }
}
