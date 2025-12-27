mod cli;
mod core;
mod count;
mod init;
mod models;
mod search;
mod utils;
mod wordcount;

use anyhow::Result;
use clap::Parser as _;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    cli::run(args)
}
