// src/main.rs

use anyhow::Result;
use clap::Parser;

mod cli;
mod core;
mod models;
mod utils;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    cli::run(args)
}
