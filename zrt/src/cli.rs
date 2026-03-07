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
    #[command(alias = "i")]
    Init(crate::init::cli::InitArgs),

    /// Show files ordered by word count
    #[command(alias = "wc")]
    Wordcount(crate::wordcount::cli::WordcountArgs),

    /// Search for files by tag criteria
    #[command(alias = "s")]
    Search(crate::search::cli::SearchArgs),

    /// Count files, words, or calculate percentage by tags
    #[command(alias = "c")]
    Count(crate::count::cli::CountArgs),

    /// Find similar notes for refactoring
    #[command(alias = "sim")]
    Similar(crate::similar::cli::SimilarArgs),

    /// List tags by frequency across notes
    #[command(alias = "t")]
    Tags(crate::tags::cli::TagsArgs),

    /// Find the most connected notes for a given tag
    #[command(alias = "con")]
    Connected(crate::connected::cli::ConnectedArgs),
}

#[inline]
pub fn run(args: Args) -> Result<()> {
    match args.command {
        Commands::Init(args) => crate::init::cli::run(args),
        Commands::Wordcount(args) => crate::wordcount::cli::run(args),
        Commands::Search(args) => crate::search::cli::run(args),
        Commands::Count(args) => crate::count::cli::run(args),
        Commands::Similar(args) => crate::similar::cli::run(args),
        Commands::Tags(args) => crate::tags::cli::run(args),
        Commands::Connected(args) => crate::connected::cli::run(args),
    }
}

