// src/cli.rs
use clap::Parser;
use std::path::PathBuf;

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
