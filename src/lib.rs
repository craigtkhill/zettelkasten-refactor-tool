#![warn(clippy::all)]
// #![warn(clippy::pedantic)]
// #![warn(clippy::nursery)]
// #![warn(clippy::cargo)]
// #![warn(clippy::restriction)]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use glob::Pattern;
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Deserialize, Debug, Default)]
pub struct Frontmatter {
    pub tags: Option<Vec<String>>,
}

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

    /// Single pattern to search for (e.g., "`to_refactor`")
    #[arg(short = 'p', long)]
    pub pattern: Option<String>,

    /// "Done" tag to search for (e.g., "refactored")
    #[arg(short = 'r', long)]
    pub done_tag: Option<String>,

    /// "Todo" tag to search for (e.g., "`to_refactor`")
    #[arg(short = 'o', long)]
    pub todo_tag: Option<String>,
}

#[derive(Debug, Default)]
pub struct IgnorePatterns {
    patterns: Vec<(Pattern, bool)>, // (pattern, is_negation)
    root_dir: PathBuf,
}

impl IgnorePatterns {
    #[must_use]
    pub const fn new(root_dir: PathBuf) -> Self {
        Self {
            patterns: Vec::new(),
            root_dir,
        }
    }

    pub fn add_pattern(&mut self, pattern: &str) -> Result<()> {
        // Skip empty lines and comments
        let pattern = pattern.trim();
        if pattern.is_empty() || pattern.starts_with('#') {
            return Ok(());
        }

        // Handle negation patterns
        let (pattern, is_negation) = if let Some(stripped) = pattern.strip_prefix('!') {
            (stripped, true)
        } else {
            (pattern, false)
        };

        // Flag to track if this is an absolute path pattern
        let is_absolute = pattern.starts_with('/');

        // Handle absolute paths
        let pattern = if is_absolute {
            pattern[1..].to_string()
        } else {
            pattern.to_string()
        };

        // Convert the pattern to a glob pattern
        let mut glob_pattern =
            if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
                // Replace ** with a special marker that won't match normal paths
                if pattern.contains("**") {
                    pattern.replace("**", "[GLOBSTAR]")
                } else {
                    pattern
                }
            } else if pattern.ends_with('/') {
                if is_negation {
                    format!("{pattern}**/*") // For negation, match all files in directory
                } else {
                    format!("{pattern}**")
                }
            } else if is_negation || pattern.contains('.') {
                pattern // For negation or files with extension, match exactly
            } else {
                format!("{pattern}/**") // Otherwise, match directory
            };

        // Handle case where pattern is just a filename without path
        // Only add **/ prefix for non-absolute patterns
        if !is_absolute && !glob_pattern.contains('/') && !glob_pattern.contains('\\') {
            glob_pattern = format!("**/{glob_pattern}");
        }

        // Handle file extension groups like *.{js,ts}
        if glob_pattern.contains('{') {
            // Split the pattern into multiple patterns
            let (prefix, suffix) = glob_pattern
                .split_once('{')
                .expect("Invalid pattern: missing opening brace");
            let (extensions, rest) = suffix
                .split_once('}')
                .expect("Invalid pattern: missing closing brace");
            let extensions: Vec<&str> = extensions.split(',').map(str::trim).collect();

            for ext in extensions {
                let full_pattern = format!("{prefix}{ext}{rest}").replace("[GLOBSTAR]", "**");
                let compiled = Pattern::new(&full_pattern)
                    .with_context(|| format!("Invalid pattern: {full_pattern}"))?;
                // Store whether this was an absolute path pattern
                self.patterns.push((compiled, is_negation));
            }
            return Ok(());
        }

        let glob_pattern = glob_pattern.replace("[GLOBSTAR]", "**");
        let compiled = Pattern::new(&glob_pattern)
            .with_context(|| format!("Invalid pattern: {glob_pattern}"))?;
        // Store whether this was an absolute path pattern
        self.patterns.push((compiled, is_negation));
        Ok(())
    }

    pub fn matches(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();

        // Always use relative paths for matching
        let relative_path = if path.is_absolute() {
            if let Ok(rel) = path.strip_prefix(&self.root_dir) {
                rel.to_path_buf()
            } else {
                // If we can't strip the prefix, try to get the components after root
                path.components()
                    .skip_while(|c| {
                        matches!(
                            c,
                            std::path::Component::RootDir | std::path::Component::Prefix(_)
                        )
                    })
                    .collect()
            }
        } else {
            path.to_path_buf()
        };

        let path_str = relative_path.to_string_lossy();

        // First check negation patterns
        for (pattern, _) in self.patterns.iter().filter(|(_, is_neg)| *is_neg) {
            if pattern.matches(&path_str) {
                return false; // If any negation pattern matches, don't ignore the file
            }
        }

        // Then check normal patterns
        for (pattern, _) in self.patterns.iter().filter(|(_, is_neg)| !*is_neg) {
            if pattern.matches(&path_str) {
                return true; // If any normal pattern matches, ignore the file
            }
        }

        false // If no patterns match, don't ignore the file
    }
}

pub fn load_ignore_patterns(dir: &Path) -> Result<IgnorePatterns> {
    let mut patterns = IgnorePatterns::new(dir.to_path_buf());
    let ignore_file = dir.join(".zrtignore");

    if ignore_file.exists() {
        let content = fs::read_to_string(&ignore_file)
            .with_context(|| format!("Failed to read .zrtignore file: {ignore_file:?}"))?;

        for line in content.lines() {
            patterns.add_pattern(line)?;
        }
    }

    Ok(patterns)
}

pub fn parse_frontmatter(content: &str) -> Result<Frontmatter> {
    let mut content_iter = content.lines();

    // Check for frontmatter delimiter
    if content_iter.next() != Some("---") {
        return Ok(Frontmatter::default());
    }

    // Collect frontmatter content
    let mut frontmatter_str = String::new();
    for line in content_iter {
        if line == "---" {
            break;
        }
        frontmatter_str.push_str(line);
        frontmatter_str.push('\n');
    }

    // Parse YAML
    serde_yaml_ng::from_str(&frontmatter_str)
        .map_err(|e| anyhow!("Failed to parse frontmatter: {}", e))
}

pub fn contains_tag(path: &std::path::Path, tag: &str) -> io::Result<bool> {
    let content = fs::read_to_string(path)?;

    // Parse frontmatter
    match parse_frontmatter(&content) {
        Ok(frontmatter) => {
            // Check if the tag exists in the frontmatter tags
            Ok(frontmatter
                .tags
                .is_some_and(|tags| tags.iter().any(|t| t == tag)))
        }
        Err(_) => Ok(false), // If parsing fails, assume no tags
    }
}

#[derive(Debug)]
pub struct FileWordCount {
    pub path: PathBuf,
    pub words: usize,
}

pub fn count_words(
    dir: &PathBuf,
    exclude_dirs: &[&str],
    filter_out: Option<&str>,
) -> Result<Vec<FileWordCount>> {
    let ignore_patterns = load_ignore_patterns(dir)?;
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, exclude_dirs, Some(&ignore_patterns)))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if let Ok(content) = fs::read_to_string(path) {
            // Skip file if it contains the filter_out tag
            if let Some(tag) = filter_out {
                if let Ok(frontmatter) = parse_frontmatter(&content) {
                    if let Some(tags) = frontmatter.tags {
                        if tags.iter().any(|t| t == tag) {
                            continue;
                        }
                    }
                }
            }

            let word_count = content.split_whitespace().count();
            files.push(FileWordCount {
                path: path.to_path_buf(),
                words: word_count,
            });
        }
    }

    files.sort_by(|a, b| b.words.cmp(&a.words));
    Ok(files)
}

pub fn print_top_files(files: Vec<FileWordCount>, top: usize) {
    for file in files.iter().take(top) {
        println!("{:8} words  {}", file.words, file.path.display());
    }
}

pub fn count_files(dir: &PathBuf, exclude_dirs: &[&str]) -> Result<u64> {
    let ignore_patterns = load_ignore_patterns(dir)?;
    let mut count = 0;

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, exclude_dirs, Some(&ignore_patterns)))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            count += 1;
        }
    }

    println!("Total files found: {count}");
    Ok(count)
}

#[derive(Debug, Default)]
pub struct SinglePatternStats {
    pub total_files: u64,
    pub files_with_pattern: u64,
}

impl SinglePatternStats {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_files: 0,
            files_with_pattern: 0,
        }
    }

    #[must_use]
    pub fn calculate_percentage(&self) -> f64 {
        if self.total_files == 0 {
            return 0.0;
        }
        (self.files_with_pattern as f64 / self.total_files as f64) * 100.0
    }
}

#[derive(Debug, Default)]
pub struct ComparisonStats {
    pub total_files: u64,
    pub done_files: u64,
    pub todo_files: u64,
}

impl ComparisonStats {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_files: 0,
            done_files: 0,
            todo_files: 0,
        }
    }

    #[must_use]
    pub fn calculate_percentage(&self) -> f64 {
        let total_tagged = self.done_files + self.todo_files;
        if total_tagged == 0 {
            return 0.0;
        }
        (self.done_files as f64 / total_tagged as f64) * 100.0
    }
}

pub fn scan_directory_single_pattern(dir: &PathBuf, pattern: &str) -> Result<SinglePatternStats> {
    let ignore_patterns = load_ignore_patterns(dir)?;
    let mut stats = SinglePatternStats::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, &[], Some(&ignore_patterns)))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        stats.total_files += 1;

        let path = entry.path();
        if contains_tag(path, pattern)? {
            stats.files_with_pattern += 1;
        }
    }

    Ok(stats)
}

pub fn scan_directory_two_patterns(
    dir: &PathBuf,
    done_tag: &str,
    todo_tag: &str,
) -> Result<ComparisonStats> {
    let ignore_patterns = load_ignore_patterns(dir)?;
    let mut stats = ComparisonStats::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, &[], Some(&ignore_patterns)))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        stats.total_files += 1;

        let path = entry.path();
        if contains_tag(path, done_tag)? {
            stats.done_files += 1;
        }
        if contains_tag(path, todo_tag)? {
            stats.todo_files += 1;
        }
    }

    Ok(stats)
}

#[must_use]
pub fn should_exclude(
    entry: &walkdir::DirEntry,
    exclude_dirs: &[&str],
    ignore_patterns: Option<&IgnorePatterns>,
) -> bool {
    if is_hidden(entry) {
        return true;
    }

    // Check manual exclude dirs
    if let Some(path_str) = entry.path().to_str() {
        for dir in exclude_dirs {
            if path_str.contains(&format!("/{dir}/")) {
                return true;
            }
        }
    }

    // Check ignore patterns
    if let Some(patterns) = ignore_patterns {
        if patterns.matches(entry.path()) {
            return true;
        }
    }

    false
}

#[must_use]
pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name().to_str().is_some_and(|s| {
        // Don't consider temp directories as hidden
        if s.starts_with(".tmp") {
            return false;
        }
        s.starts_with('.')
    })
}

pub fn run(args: Args) -> Result<()> {
    if args.count {
        let exclude_dirs: Vec<&str> = args.exclude.split(',').collect();
        let count = count_files(&args.directory, &exclude_dirs).with_context(|| {
            format!(
                "Failed to count files in directory: {}",
                args.directory.display()
            )
        })?;
        println!("{count}");
    } else if args.words {
        let exclude_dirs: Vec<&str> = args.exclude.split(',').collect();
        let files = count_words(&args.directory, &exclude_dirs, args.filter_out.as_deref())
            .with_context(|| {
                format!(
                    "Failed to count words in directory: {}",
                    args.directory.display()
                )
            })?;
        print_top_files(files, args.top);
    } else if let Some(pattern) = args.pattern {
        // Single pattern mode
        let stats = scan_directory_single_pattern(&args.directory, &pattern)
            .with_context(|| format!("Failed to scan directory: {}", args.directory.display()))?;
        println!("Total files: {}", stats.total_files);
        println!(
            "Files with pattern '{}': {}",
            pattern, stats.files_with_pattern
        );
        println!("Percentage: {:.2}%", stats.calculate_percentage());
    } else if let (Some(done), Some(todo)) = (args.done_tag, args.todo_tag) {
        // Compare two tags mode
        let stats = scan_directory_two_patterns(&args.directory, &done, &todo)
            .with_context(|| format!("Failed to scan directory: {}", args.directory.display()))?;
        println!("{} files: {}", done, stats.done_files);
        println!("{} files: {}", todo, stats.todo_files);
        println!("Done percentage: {:.2}%", stats.calculate_percentage());
    } else {
        // Default behavior - scan for to_refactor
        let default_pattern = String::from("to_refactor");
        let stats = scan_directory_single_pattern(&args.directory, &default_pattern)
            .with_context(|| format!("Failed to scan directory: {}", args.directory.display()))?;
        println!("Total files: {}", stats.total_files);
        println!(
            "Files with pattern '{}': {}",
            default_pattern, stats.files_with_pattern
        );
        println!("Percentage: {:.2}%", stats.calculate_percentage());
    }

    Ok(())
}
