// src/cli.rs
use anyhow::Result;
use clap::{Parser, Subcommand};


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize ZRT configuration
    Init(crate::init::cli::InitArgs),

    /// Show files ordered by word count (alias: wc)
    #[command(alias = "wc")]
    Wordcount(crate::wordcount::cli::WordcountArgs),

    /// Search for files by tag criteria
    Search(crate::search::cli::SearchArgs),

    /// Count files, words, or calculate percentage by tags
    Count(crate::count::cli::CountArgs),
}

#[inline]
pub fn run(args: Args) -> Result<()> {
    match args.command {
        Commands::Init(args) => crate::init::cli::run(args),
        Commands::Wordcount(args) => crate::wordcount::cli::run(args),
        Commands::Search(args) => crate::search::cli::run(args),
        Commands::Count(args) => crate::count::cli::run(args),
    }
}

