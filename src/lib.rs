use std::path::PathBuf;
use std::fs;
use std::io;
use walkdir::WalkDir;
use anyhow::{Result, Context, anyhow};
use clap::Parser;
use serde::Deserialize;

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
    serde_yaml::from_str(&frontmatter_str)
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
                .map(|tags| tags.iter().any(|t| t == tag))
                .unwrap_or(false))
        },
        Err(_) => Ok(false), // If parsing fails, assume no tags
    }
}

#[derive(Debug)]
pub struct FileWordCount {
    pub path: PathBuf,
    pub words: usize,
}

pub fn count_words(dir: &PathBuf, exclude_dirs: &[&str], filter_out: Option<&str>) -> Result<Vec<FileWordCount>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, exclude_dirs))
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
    let mut count = 0;

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            let should_exclude = should_exclude(e, exclude_dirs);
            !should_exclude
        })
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            count += 1;
        }
    }

    println!("Total files found: {}", count);
    Ok(count)
}

pub struct SinglePatternStats {
    pub total_files: u64,
    pub files_with_pattern: u64,
}

impl SinglePatternStats {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            files_with_pattern: 0,
        }
    }

    pub fn calculate_percentage(&self) -> f64 {
        if self.total_files == 0 {
            return 0.0;
        }
        (self.files_with_pattern as f64 / self.total_files as f64) * 100.0
    }
}

pub struct ComparisonStats {
    pub total_files: u64,
    pub done_files: u64,
    pub todo_files: u64,
}

impl ComparisonStats {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            done_files: 0,
            todo_files: 0,
        }
    }

    pub fn calculate_percentage(&self) -> f64 {
        let total_tagged = self.done_files + self.todo_files;
        if total_tagged == 0 {
            return 0.0;
        }
        (self.done_files as f64 / total_tagged as f64) * 100.0
    }
}

pub fn scan_directory_single_pattern(dir: &PathBuf, pattern: &str) -> Result<SinglePatternStats> {
    let mut stats = SinglePatternStats::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
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

pub fn scan_directory_two_patterns(dir: &PathBuf, done_tag: &str, todo_tag: &str) -> Result<ComparisonStats> {
    let mut stats = ComparisonStats::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
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

pub fn should_exclude(entry: &walkdir::DirEntry, exclude_dirs: &[&str]) -> bool {
    if is_hidden(entry) {
        return true;
    }

    // Convert the path to a string for comparison
    if let Some(path_str) = entry.path().to_str() {
        for dir in exclude_dirs {
            // Check if path contains the excluded directory
            if path_str.contains(&format!("/{}/", dir)) {
                return true;
            }
        }
    }
    false
}

pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| {
            // Don't consider temp directories as hidden
            if s.starts_with(".tmp") {
                return false;
            }
            s.starts_with(".")
        })
        .unwrap_or(false)
}

pub fn run(args: Args) -> Result<()> {
    if args.count {
        let exclude_dirs: Vec<&str> = args.exclude.split(',').collect();
        let count = count_files(&args.directory, &exclude_dirs)
            .with_context(|| format!("Failed to count files in directory: {}", args.directory.display()))?;
        println!("{}", count);
    } else if args.words {
        let exclude_dirs: Vec<&str> = args.exclude.split(',').collect();
        let files = count_words(&args.directory, &exclude_dirs, args.filter_out.as_deref())
            .with_context(|| format!("Failed to count words in directory: {}", args.directory.display()))?;
        print_top_files(files, args.top);
    } else if let Some(pattern) = args.pattern {
        // Single pattern mode
        let stats = scan_directory_single_pattern(&args.directory, &pattern)
            .with_context(|| format!("Failed to scan directory: {}", args.directory.display()))?;
        println!("Total files: {}", stats.total_files);
        println!("Files with pattern '{}': {}", pattern, stats.files_with_pattern);
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
        println!("Files with pattern '{}': {}", default_pattern, stats.files_with_pattern);
        println!("Percentage: {:.2}%", stats.calculate_percentage());
    }

    Ok(())
}