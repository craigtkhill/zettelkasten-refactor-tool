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
#![allow(
    clippy::arbitrary_source_item_ordering,
    reason = "Development: logical grouping over alphabetical"
)]
use anyhow::{Context as _, Result};
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
            // Load configuration
            let config_path = std::path::Path::new(".zrt/config.toml");
            let mut settings = if config_path.exists() {
                zrt_tagging::Settings::load_from_file(config_path)?
            } else {
                println!("No config found at .zrt/config.toml, using defaults");
                zrt_tagging::Settings::default()
            };

            // Override settings with command line arguments
            if let Some(t) = threshold {
                settings.confidence_threshold = t;
            }
            settings.max_suggestions = top;

            // Create predictor and load trained models
            let mut predictor = zrt_tagging::Predictor::new(settings)?;
            predictor.load_classifiers()?;

            if let Some(file_path) = file {
                // Suggest tags for single file
                println!("Suggesting tags for: {}", file_path.display());
                suggest_tags_for_file(&predictor, &file_path)?;
            } else {
                // Suggest tags for all files in directory
                println!("Suggesting tags for files in: {}", directory.display());
                suggest_tags_for_directory(&predictor, &directory)?;
            }

            Ok(())
        }
        PredictCommands::Validate { directory } => {
            println!("Validating model with data from: {}", directory.display());

            // Load configuration
            let config_path = std::path::Path::new(".zrt/config.toml");
            let settings = if config_path.exists() {
                zrt_tagging::Settings::load_from_file(config_path)?
            } else {
                println!("No config found at .zrt/config.toml, using defaults");
                zrt_tagging::Settings::default()
            };

            // Extract validation data from notes
            let validation_data = zrt_tagging::extraction::extract_training_data(&directory)?;

            if validation_data.notes.is_empty() {
                println!("No notes found in directory: {}", directory.display());
                return Ok(());
            }

            // Create predictor and load trained models
            let mut predictor = zrt_tagging::Predictor::new(settings)?;
            predictor.load_classifiers()?;

            // Run validation
            validate_model_performance(&predictor, &validation_data)?;

            Ok(())
        }
    }
}

#[cfg(feature = "tagging")]
#[inline]
fn suggest_tags_for_file(
    predictor: &zrt_tagging::Predictor,
    file_path: &std::path::Path,
) -> Result<()> {
    // Read file content
    let content = std::fs::read_to_string(file_path)
        .context(format!("Failed to read file: {}", file_path.display()))?;

    // Extract content (remove frontmatter if present)
    let (_, body) = extract_frontmatter_content(&content)?;

    // Get predictions
    let predictions = predictor.predict(&body)?;

    if predictions.is_empty() {
        println!("No tag suggestions above threshold");
    } else {
        println!("Suggested tags:");
        for prediction in predictions {
            println!(
                "  {} (confidence: {:.3})",
                prediction.tag, prediction.confidence
            );
        }
    }

    Ok(())
}

#[cfg(feature = "tagging")]
#[expect(
    clippy::default_numeric_fallback,
    reason = "Development: simple counter"
)]
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Development: controlled counting"
)]
#[inline]
fn suggest_tags_for_directory(
    predictor: &zrt_tagging::Predictor,
    directory: &std::path::Path,
) -> Result<()> {
    use walkdir::WalkDir;

    let mut processed_count = 0;

    for entry in WalkDir::new(directory)
        .follow_links(false)
        .into_iter()
        .filter_map(core::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Only process markdown files
        let Some(ext) = path.extension() else {
            continue;
        };
        if ext != "md" && ext != "markdown" {
            continue;
        }

        // Skip hidden files
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with('.'))
        {
            continue;
        }

        match suggest_tags_for_file(predictor, path) {
            Ok(()) => {
                processed_count += 1;
                println!(); // Add spacing between files
            }
            Err(e) => {
                println!("Warning: Failed to process {}: {}", path.display(), e);
            }
        }
    }

    println!("Processed {processed_count} files");
    Ok(())
}

#[cfg(feature = "tagging")]
#[expect(
    clippy::option_if_let_else,
    reason = "Development: clearer control flow"
)]
#[expect(
    clippy::indexing_slicing,
    reason = "Development: bounds already checked"
)]
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Development: controlled arithmetic"
)]
#[inline]
fn extract_frontmatter_content(content: &str) -> Result<(Option<String>, String)> {
    if !content.starts_with("---") {
        return Ok((None, content.to_owned()));
    }

    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 3 {
        return Ok((None, content.to_owned()));
    }

    let mut end_index = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_index = Some(i);
            break;
        }
    }

    if let Some(end) = end_index {
        let frontmatter = lines[1..end].join("\n");
        let body = lines[end + 1..].join("\n");
        Ok((Some(frontmatter), body))
    } else {
        Ok((None, content.to_owned()))
    }
}

#[cfg(feature = "tagging")]
#[derive(Debug, Default)]
struct TagMetrics {
    actual_count: usize,
    false_positives: usize,
    true_positives: usize,
}

#[cfg(feature = "tagging")]
impl TagMetrics {
    fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "tagging")]
#[expect(
    clippy::too_many_lines,
    reason = "Development: comprehensive validation function"
)]
#[expect(
    clippy::default_numeric_fallback,
    reason = "Development: simple metrics"
)]
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Development: controlled arithmetic"
)]
#[expect(
    clippy::cast_precision_loss,
    reason = "Development: metrics calculation"
)]
#[expect(
    clippy::as_conversions,
    reason = "Development: safe numeric conversions"
)]
#[expect(
    clippy::indexing_slicing,
    reason = "Development: bounds controlled by iteration"
)]
#[expect(
    clippy::iter_over_hash_type,
    reason = "Development: HashMap iteration acceptable"
)]
#[expect(
    clippy::unwrap_or_default,
    reason = "Development: explicit construction clearer"
)]
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Development: destructuring preference"
)]
#[expect(
    clippy::uninlined_format_args,
    reason = "Development: explicit args clearer"
)]
#[expect(
    clippy::cast_lossless,
    reason = "Development: explicit casting for clarity"
)]
fn validate_model_performance(
    predictor: &zrt_tagging::Predictor,
    validation_data: &zrt_tagging::extraction::TrainingData,
) -> Result<()> {
    println!(
        "Running validation on {} notes",
        validation_data.notes.len()
    );

    let mut total_predictions = 0;
    let mut correct_predictions = 0;
    let mut total_actual_tags = 0;
    let mut total_predicted_tags = 0;

    // Track precision@k metrics
    let k_values = [1, 3, 5];
    let mut precision_at_k = [0.0; 3];
    let mut count_at_k = [0; 3];

    // Per-tag metrics
    let mut tag_stats: std::collections::HashMap<String, TagMetrics> =
        std::collections::HashMap::new();

    for note in &validation_data.notes {
        // Get predictions for this note
        let predictions = predictor.predict(&note.content)?;

        total_predicted_tags += predictions.len();
        total_actual_tags += note.tags.len();

        // Calculate precision@k
        for (i, &k) in k_values.iter().enumerate() {
            if !note.tags.is_empty() {
                let top_k_predictions: Vec<_> = predictions.iter().take(k).collect();
                let correct_in_k = top_k_predictions
                    .iter()
                    .filter(|pred| note.tags.contains(&pred.tag))
                    .count();

                precision_at_k[i] += correct_in_k as f64 / k.min(note.tags.len()) as f64;
                count_at_k[i] += 1;
            }
        }

        // Per-tag statistics
        for tag in &note.tags {
            let metrics = tag_stats.entry(tag.clone()).or_insert_with(TagMetrics::new);
            metrics.actual_count += 1;

            // Check if this tag was predicted
            if predictions.iter().any(|pred| &pred.tag == tag) {
                metrics.true_positives += 1;
                correct_predictions += 1;
            }
            total_predictions += 1;
        }

        // Count false positives
        for prediction in &predictions {
            if !note.tags.contains(&prediction.tag) {
                let metrics = tag_stats
                    .entry(prediction.tag.clone())
                    .or_insert_with(TagMetrics::new);
                metrics.false_positives += 1;
            }
        }
    }

    // Calculate overall metrics
    let overall_precision = if total_predicted_tags > 0 {
        correct_predictions as f64 / total_predictions as f64
    } else {
        0.0
    };

    let overall_recall = if total_actual_tags > 0 {
        correct_predictions as f64 / total_actual_tags as f64
    } else {
        0.0
    };

    let f1_score = if overall_precision + overall_recall > 0.0 {
        2.0 * (overall_precision * overall_recall) / (overall_precision + overall_recall)
    } else {
        0.0
    };

    // Print results
    println!("\n=== Overall Performance ===");
    println!("Precision: {:.3}", overall_precision);
    println!("Recall: {:.3}", overall_recall);
    println!("F1 Score: {:.3}", f1_score);

    println!("\n=== Precision@K ===");
    for (i, &k) in k_values.iter().enumerate() {
        if count_at_k[i] > 0 {
            println!(
                "Precision@{}: {:.3}",
                k,
                precision_at_k[i] / count_at_k[i] as f64
            );
        }
    }

    // Show top/bottom performing tags
    let mut tag_performance: Vec<_> = tag_stats
        .iter()
        .filter(|(_, metrics)| metrics.actual_count >= 3) // Only tags with enough examples
        .map(|(tag, metrics)| {
            let precision = if metrics.true_positives + metrics.false_positives > 0 {
                metrics.true_positives as f64
                    / (metrics.true_positives + metrics.false_positives) as f64
            } else {
                0.0
            };
            let recall = if metrics.actual_count > 0 {
                metrics.true_positives as f64 / metrics.actual_count as f64
            } else {
                0.0
            };
            let f1 = if precision + recall > 0.0 {
                2.0 * (precision * recall) / (precision + recall)
            } else {
                0.0
            };
            (tag, f1, precision, recall, metrics.actual_count)
        })
        .collect();

    tag_performance.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));

    println!("\n=== Top Performing Tags ===");
    for (tag, f1, precision, recall, count) in tag_performance.iter().take(5) {
        println!(
            "{}: F1={:.3}, P={:.3}, R={:.3} (n={})",
            tag, f1, precision, recall, count
        );
    }

    println!("\n=== Bottom Performing Tags ===");
    for (tag, f1, precision, recall, count) in tag_performance.iter().rev().take(5) {
        println!(
            "{}: F1={:.3}, P={:.3}, R={:.3} (n={})",
            tag, f1, precision, recall, count
        );
    }

    Ok(())
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
