mod cli;
mod core;
mod count;
mod init;
mod search;
mod similar;
mod wordcount;

use anyhow::Result;
use clap::Parser as _;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    cli::run(args)
}
