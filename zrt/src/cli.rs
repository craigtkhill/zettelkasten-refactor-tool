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
    count_file_metrics, count_files, count_word_stats, count_words, scan_directory_only_tag,
    scan_directory_single, scan_directory_two,
};
use crate::settings::{SortBy, ZrtConfig};
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

    /// Show files ordered by word count (alias: wc)
    #[command(alias = "wc")]
    Wordcount {
        /// Directory to scan (defaults to current directory)
        #[arg(short = 'd', long = "dir", default_value = ".")]
        directory: PathBuf,

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
    Tag {
        #[command(subcommand)]
        command: TagCommands,
    },
}

#[cfg(feature = "tagging")]
#[derive(Subcommand, Debug)]
pub enum TagCommands {
    /// Train the tag prediction model
    Train {
        /// Directory to scan for training data
        #[arg(short = 'd', long = "dir", default_value = ".")]
        directory: PathBuf,

        /// Tags to exclude from training (space-separated)
        #[arg(short = 'e', long = "exclude-tags", num_args = 0..)]
        exclude_tags: Vec<String>,
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

        /// Tags to exclude from suggestions (space-separated)
        #[arg(short = 'e', long = "exclude-tags", num_args = 0..)]
        exclude_tags: Vec<String>,
    },

    /// Validate model performance
    Validate {
        /// Directory to scan for validation data
        #[arg(short = 'd', long = "dir", default_value = ".")]
        directory: PathBuf,

        /// Tags to exclude from validation (space-separated)
        #[arg(short = 'e', long = "exclude-tags", num_args = 0..)]
        exclude_tags: Vec<String>,
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
        Commands::Wordcount {
            directory,
            filter_out,
            top,
            exclude,
            exceeds,
            sort_by,
            #[cfg(feature = "tagging")]
            suggest_tags,
        } => {
            let exclude_dirs: Vec<&str> = exclude.iter().map(String::as_str).collect();
            let filter_tags: Vec<&str> = filter_out.iter().map(String::as_str).collect();

            if exceeds {
                let config = ZrtConfig::load_or_default();
                let sort_preference = sort_by.unwrap_or(config.refactor.sort_by);

                let metrics = count_file_metrics(
                    &directory,
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
                    &directory,
                    &exclude_dirs,
                    if filter_tags.is_empty() {
                        None
                    } else {
                        Some(filter_tags[0])
                    },
                )?;
                print_top_files(&files, top);
            }

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
        Commands::Tag { command } => run_tag_command(command),
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

    // Create config with both refactor and tagging settings
    let config = ZrtConfig::default();
    config.save_to_file(&zrt_dir.join("config.toml"))?;

    println!("Initialized ZRT directory at .zrt/");
    println!("Created default configuration at .zrt/config.toml");
    println!("  - Refactor thresholds: 250+ words, 30+ lines");

    #[cfg(feature = "tagging")]
    println!("  - Tagging configuration included");

    Ok(())
}

#[cfg(feature = "tagging")]
#[inline]
fn run_tag_command(command: TagCommands) -> Result<()> {
    match command {
        TagCommands::Train {
            directory,
            exclude_tags,
        } => {
            println!("Training model with data from: {}", directory.display());

            // Load configuration
            let config_path = std::path::Path::new(".zrt/config.toml");
            let mut settings = if config_path.exists() {
                zrt_tagging::Settings::load_from_file(config_path)?
            } else {
                println!("No config found at .zrt/config.toml, using defaults");
                zrt_tagging::Settings::default()
            };

            // Override excluded tags from command line
            if !exclude_tags.is_empty() {
                let additional_excluded: std::collections::HashSet<String> =
                    exclude_tags.into_iter().collect();
                settings.excluded_tags.extend(additional_excluded);
                if !settings.excluded_tags.is_empty() {
                    let tags_list: Vec<String> = settings.excluded_tags.iter().cloned().collect();
                    println!("Excluding tags: {}", tags_list.join(", "));
                }
            }

            // Extract training data from notes
            let mut training_data = zrt_tagging::extraction::extract_training_data(&directory)?;

            // Apply tag exclusions to training data
            if !settings.excluded_tags.is_empty() {
                training_data.exclude_tags(&settings.excluded_tags);
                println!(
                    "After exclusions: {} notes with tags",
                    training_data.notes.len()
                );
            }

            if training_data.notes.is_empty() {
                println!("No notes found in directory: {}", directory.display());
                return Ok(());
            }

            // Create and train predictor
            let mut predictor = zrt_tagging::UnifiedPredictor::new(settings)?;
            predictor.train(&training_data)?;

            println!("Training completed successfully!");
            Ok(())
        }
        TagCommands::Suggest {
            directory,
            file,
            threshold,
            top,
            exclude_tags,
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

            // Override excluded tags from command line
            if !exclude_tags.is_empty() {
                let additional_excluded: std::collections::HashSet<String> =
                    exclude_tags.into_iter().collect();
                settings.excluded_tags.extend(additional_excluded);
                if !settings.excluded_tags.is_empty() {
                    let tags_list: Vec<String> = settings.excluded_tags.iter().cloned().collect();
                    println!("Excluding tags: {}", tags_list.join(", "));
                }
            }

            // Create predictor and load trained models
            let mut predictor = zrt_tagging::UnifiedPredictor::new(settings)?;
            predictor.load_models()?;

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
        TagCommands::Validate {
            directory,
            exclude_tags,
        } => {
            println!("Validating model with data from: {}", directory.display());

            // Load configuration
            let config_path = std::path::Path::new(".zrt/config.toml");
            let mut settings = if config_path.exists() {
                zrt_tagging::Settings::load_from_file(config_path)?
            } else {
                println!("No config found at .zrt/config.toml, using defaults");
                zrt_tagging::Settings::default()
            };

            // Override excluded tags from command line
            if !exclude_tags.is_empty() {
                let additional_excluded: std::collections::HashSet<String> =
                    exclude_tags.into_iter().collect();
                settings.excluded_tags.extend(additional_excluded);
                if !settings.excluded_tags.is_empty() {
                    let tags_list: Vec<String> = settings.excluded_tags.iter().cloned().collect();
                    println!("Excluding tags: {}", tags_list.join(", "));
                }
            }

            // Extract validation data from notes
            let mut validation_data = zrt_tagging::extraction::extract_training_data(&directory)?;

            // Apply tag exclusions to validation data
            if !settings.excluded_tags.is_empty() {
                validation_data.exclude_tags(&settings.excluded_tags);
                println!(
                    "After exclusions: {} notes with tags",
                    validation_data.notes.len()
                );
            }

            if validation_data.notes.is_empty() {
                println!("No notes found in directory: {}", directory.display());
                return Ok(());
            }

            // Create predictor and load trained models
            let mut predictor = zrt_tagging::UnifiedPredictor::new(settings)?;
            predictor.load_models()?;

            // Run validation
            validate_model_performance(&predictor, &validation_data)?;

            Ok(())
        }
    }
}

#[cfg(feature = "tagging")]
#[inline]
fn suggest_tags_for_file(
    predictor: &zrt_tagging::UnifiedPredictor,
    file_path: &std::path::Path,
) -> Result<()> {
    // Read file content
    let content = std::fs::read_to_string(file_path)
        .context(format!("Failed to read file: {}", file_path.display()))?;

    // Extract content and frontmatter
    let (frontmatter, body) = extract_frontmatter_content(&content)?;

    // Parse existing tags from frontmatter
    let existing_tags = frontmatter.map_or_else(std::collections::HashSet::new, |fm| {
        parse_tags_from_frontmatter(&fm).unwrap_or_default()
    });

    // Get predictions
    let predictions = predictor.predict(&body)?;

    // Filter out tags that already exist in the file
    let filtered_predictions: Vec<_> = predictions
        .into_iter()
        .filter(|p| !existing_tags.contains(&p.tag))
        .collect();

    if !existing_tags.is_empty() {
        let tags_list: Vec<String> = existing_tags.iter().cloned().collect();
        println!("Existing tags: {}", tags_list.join(", "));
    }

    if !filtered_predictions.is_empty() {
        println!("Suggested new tags:");
        for prediction in filtered_predictions {
            println!(
                "  {} (confidence: {:.3})",
                prediction.tag, prediction.confidence
            );
        }
    } else if !existing_tags.is_empty() {
        println!("No new tag suggestions (all predicted tags already exist)");
    } else {
        println!("No tag suggestions found");
    }

    Ok(())
}

#[cfg(feature = "tagging")]
#[inline]
fn suggest_tags_for_directory(
    predictor: &zrt_tagging::UnifiedPredictor,
    directory: &std::path::Path,
) -> Result<()> {
    use walkdir::WalkDir;

    // Collect all markdown files, their content, and existing tags for batch processing
    let mut files_to_process: Vec<(String, String)> = Vec::new();
    let mut file_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut existing_tags_map: std::collections::HashMap<
        String,
        std::collections::HashSet<String>,
    > = std::collections::HashMap::new();

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

        // Read file content
        match std::fs::read_to_string(path) {
            Ok(content) => {
                // Extract content and frontmatter
                match extract_frontmatter_content(&content) {
                    Ok((frontmatter, body)) => {
                        let file_path_str = path.to_string_lossy().to_string();

                        // Parse existing tags from frontmatter
                        let existing_tags = frontmatter
                            .map_or_else(std::collections::HashSet::new, |fm| {
                                parse_tags_from_frontmatter(&fm).unwrap_or_default()
                            });

                        files_to_process.push((file_path_str.clone(), body));
                        file_paths.push(path.to_path_buf());
                        existing_tags_map.insert(file_path_str, existing_tags);
                    }
                    Err(e) => {
                        println!("Warning: Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
            Err(e) => {
                println!("Warning: Failed to read {}: {}", path.display(), e);
            }
        }
    }

    // Use batch prediction for efficiency (especially for EmbeddingKnn)
    let batch_results = predictor.predict_batch(&files_to_process)?;

    // Convert batch results to the expected format with confidence scores, filtering existing tags
    let mut file_predictions: Vec<(std::path::PathBuf, f32, Vec<zrt_tagging::Prediction>)> =
        Vec::new();

    for (file_path_str, predictions) in batch_results {
        // Find the corresponding PathBuf and existing tags
        if let Some(path_buf) = file_paths
            .iter()
            .find(|p| p.to_string_lossy() == file_path_str)
        {
            let existing_tags = existing_tags_map
                .get(&file_path_str)
                .cloned()
                .unwrap_or_default();

            // Filter out tags that already exist in the file
            let filtered_predictions: Vec<zrt_tagging::Prediction> = predictions
                .into_iter()
                .filter(|p| !existing_tags.contains(&p.tag))
                .collect();

            if !filtered_predictions.is_empty() {
                let max_confidence = filtered_predictions
                    .iter()
                    .map(|p| p.confidence)
                    .fold(0.0_f32, f32::max);

                file_predictions.push((path_buf.clone(), max_confidence, filtered_predictions));
            }
        }
    }

    // Sort files by highest confidence (descending)
    file_predictions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));

    // Display files in confidence order (only those with predictions)
    let files_with_predictions: Vec<_> = file_predictions
        .iter()
        .filter(|(_, _, predictions)| !predictions.is_empty())
        .collect();

    for (file_path, _, predictions) in &files_with_predictions {
        let tags_string: Vec<String> = predictions
            .iter()
            .map(|p| format!("{} ({:.3})", p.tag, p.confidence))
            .collect();
        println!("{} {}", file_path.display(), tags_string.join(" "));
    }

    if files_with_predictions.is_empty() {
        println!("No files found with tag suggestions above threshold");
    } else {
        println!(
            "\nShowing {} files with tag suggestions (processed {} total)",
            files_with_predictions.len(),
            file_predictions.len()
        );
    }
    Ok(())
}

#[cfg(feature = "tagging")]
#[inline]
fn parse_tags_from_frontmatter(frontmatter: &str) -> Result<std::collections::HashSet<String>> {
    use serde_yaml_ng as serde_yaml;

    let yaml: serde_yaml::Value =
        serde_yaml::from_str(frontmatter).context("Failed to parse YAML frontmatter")?;

    let mut tags = std::collections::HashSet::new();

    if let Some(tags_value) = yaml.get("tags") {
        match tags_value {
            serde_yaml::Value::Sequence(tag_list) => {
                for tag in tag_list {
                    if let Some(tag_str) = tag.as_str() {
                        tags.insert(tag_str.to_owned());
                    }
                }
            }
            serde_yaml::Value::String(single_tag) => {
                tags.insert(single_tag.clone());
            }
            _ => {}
        }
    }

    Ok(tags)
}

#[cfg(feature = "tagging")]
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

    end_index.map_or_else(
        || Ok((None, content.to_owned())),
        |end| {
            let frontmatter = lines.get(1..end).unwrap_or(&[]).join("\n");
            let start_idx = end.checked_add(1).unwrap_or(end);
            let body = lines.get(start_idx..).unwrap_or(&[]).join("\n");
            Ok((Some(frontmatter), body))
        },
    )
}

#[cfg(feature = "tagging")]
#[derive(Debug, Default)]
struct TagMetrics {
    actual_count: usize,
    false_positives: usize,
    true_positives: usize,
}

#[cfg(feature = "tagging")]
fn validate_model_performance(
    predictor: &zrt_tagging::UnifiedPredictor,
    validation_data: &zrt_tagging::extraction::TrainingData,
) -> Result<()> {
    println!(
        "Running validation on {} notes",
        validation_data.notes.len()
    );

    let mut total_predictions = 0_i32;
    let mut correct_predictions = 0_i32;
    let mut total_actual_tags = 0_usize;
    let mut total_predicted_tags = 0_usize;

    // Track precision@k metrics
    let k_values = [1, 3, 5];
    let mut precision_at_k = [0.0_f64; 3];
    let mut count_at_k = [0_i32; 3];

    // Per-tag metrics
    let mut tag_stats: std::collections::HashMap<String, TagMetrics> =
        std::collections::HashMap::new();

    for note in &validation_data.notes {
        // Get predictions for this note
        let predictions = predictor.predict(&note.content)?;

        total_predicted_tags = total_predicted_tags
            .checked_add(predictions.len())
            .unwrap_or(total_predicted_tags);
        total_actual_tags = total_actual_tags
            .checked_add(note.tags.len())
            .unwrap_or(total_actual_tags);

        // Calculate precision@k
        for (i, &k) in k_values.iter().enumerate() {
            if !note.tags.is_empty() {
                let top_k_predictions: Vec<_> = predictions.iter().take(k).collect();
                let correct_in_k = top_k_predictions
                    .iter()
                    .filter(|pred| note.tags.contains(&pred.tag))
                    .count();

                if let Some(precision_ref) = precision_at_k.get_mut(i) {
                    *precision_ref += f64::from(u32::try_from(correct_in_k).unwrap_or(u32::MAX))
                        / f64::from(u32::try_from(k.min(note.tags.len())).unwrap_or(u32::MAX));
                }
                if let Some(count_ref) = count_at_k.get_mut(i) {
                    *count_ref = count_ref.checked_add(1_i32).unwrap_or(*count_ref);
                }
            }
        }

        // Per-tag statistics (sorted for deterministic iteration)
        let mut sorted_tags: Vec<_> = note.tags.iter().collect();
        sorted_tags.sort_unstable();
        for tag in sorted_tags {
            let metrics = tag_stats.entry(tag.clone()).or_default();
            metrics.actual_count = metrics
                .actual_count
                .checked_add(1)
                .unwrap_or(metrics.actual_count);

            // Check if this tag was predicted
            if predictions.iter().any(|pred| &pred.tag == tag) {
                metrics.true_positives = metrics
                    .true_positives
                    .checked_add(1)
                    .unwrap_or(metrics.true_positives);
                correct_predictions = correct_predictions
                    .checked_add(1_i32)
                    .unwrap_or(correct_predictions);
            }
            total_predictions = total_predictions
                .checked_add(1_i32)
                .unwrap_or(total_predictions);
        }

        // Count false positives
        for prediction in &predictions {
            if !note.tags.contains(&prediction.tag) {
                let metrics = tag_stats.entry(prediction.tag.clone()).or_default();
                metrics.false_positives = metrics
                    .false_positives
                    .checked_add(1)
                    .unwrap_or(metrics.false_positives);
            }
        }
    }

    // Calculate overall metrics
    let overall_precision = if total_predicted_tags > 0_usize {
        f64::from(correct_predictions) / f64::from(total_predictions)
    } else {
        0.0_f64
    };

    let overall_recall = if total_actual_tags > 0_usize {
        f64::from(correct_predictions)
            / f64::from(u32::try_from(total_actual_tags).unwrap_or(u32::MAX))
    } else {
        0.0_f64
    };

    let f1_score = if overall_precision + overall_recall > 0.0_f64 {
        2.0_f64 * (overall_precision * overall_recall) / (overall_precision + overall_recall)
    } else {
        0.0_f64
    };

    // Print results
    println!("\n=== Overall Performance ===");
    println!("Precision: {overall_precision:.3}");
    println!("Recall: {overall_recall:.3}");
    println!("F1 Score: {f1_score:.3}");

    println!("\n=== Precision@K ===");
    for (i, &k) in k_values.iter().enumerate() {
        if let (Some(&count), Some(&precision)) = (count_at_k.get(i), precision_at_k.get(i)) {
            if count > 0_i32 {
                println!("Precision@{}: {:.3}", k, precision / f64::from(count));
            }
        }
    }

    // Show top/bottom performing tags
    let mut tag_performance: Vec<_> = tag_stats
        .iter()
        .filter(|(_, metrics)| metrics.actual_count >= 3) // Only tags with enough examples
        .map(|(tag, metrics)| {
            let precision = if metrics
                .true_positives
                .checked_add(metrics.false_positives)
                .unwrap_or(0_usize)
                > 0_usize
            {
                f64::from(u32::try_from(metrics.true_positives).unwrap_or(u32::MAX))
                    / f64::from(
                        u32::try_from(
                            metrics
                                .true_positives
                                .checked_add(metrics.false_positives)
                                .unwrap_or(0_usize),
                        )
                        .unwrap_or(u32::MAX),
                    )
            } else {
                0.0_f64
            };
            let recall = if metrics.actual_count > 0_usize {
                f64::from(u32::try_from(metrics.true_positives).unwrap_or(u32::MAX))
                    / f64::from(u32::try_from(metrics.actual_count).unwrap_or(u32::MAX))
            } else {
                0.0_f64
            };
            let f1 = if precision + recall > 0.0_f64 {
                2.0_f64 * (precision * recall) / (precision + recall)
            } else {
                0.0_f64
            };
            (tag, f1, precision, recall, metrics.actual_count)
        })
        .collect();

    tag_performance.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));

    println!("\n=== Top Performing Tags ===");
    for (tag, f1, precision, recall, count) in tag_performance.iter().take(5) {
        println!("{tag}: F1={f1:.3}, P={precision:.3}, R={recall:.3} (n={count})");
    }

    println!("\n=== Bottom Performing Tags ===");
    for (tag, f1, precision, recall, count) in tag_performance.iter().rev().take(5) {
        println!("{tag}: F1={f1:.3}, P={precision:.3}, R={recall:.3} (n={count})");
    }

    Ok(())
}

#[cfg(test)]
mod evaluation_tests;

#[cfg(test)]
mod integration_tests;

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

    #[cfg(feature = "tagging")]
    #[test]
    fn test_tag_suggest_parsing() {
        let args = Args::parse_from(["program", "tag", "suggest", "-t", "0.8"]);
        if let Commands::Tag { command } = args.command {
            if let TagCommands::Suggest { threshold, .. } = command {
                assert_eq!(threshold, Some(0.8));
            } else {
                panic!("Expected Suggest subcommand");
            }
        } else {
            panic!("Expected Tag command");
        }
    }

    #[cfg(feature = "tagging")]
    #[test]
    fn test_tag_suggest_exclude_tags_parsing() {
        let args = Args::parse_from([
            "program", "tag", "suggest", "-e", "draft", "private", "temp",
        ]);
        if let Commands::Tag { command } = args.command {
            if let TagCommands::Suggest { exclude_tags, .. } = command {
                assert_eq!(
                    exclude_tags,
                    vec!["draft".to_owned(), "private".to_owned(), "temp".to_owned()]
                );
            } else {
                panic!("Expected Suggest subcommand");
            }
        } else {
            panic!("Expected Tag command");
        }
    }

    #[cfg(feature = "tagging")]
    #[test]
    fn test_tag_train_exclude_tags_parsing() {
        let args = Args::parse_from(["program", "tag", "train", "-e", "archived", "deleted"]);
        if let Commands::Tag { command } = args.command {
            if let TagCommands::Train { exclude_tags, .. } = command {
                assert_eq!(
                    exclude_tags,
                    vec!["archived".to_owned(), "deleted".to_owned()]
                );
            } else {
                panic!("Expected Train subcommand");
            }
        } else {
            panic!("Expected Tag command");
        }
    }
}
