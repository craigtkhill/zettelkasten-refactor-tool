// src/main.rs

mod cli;
mod core;
mod models;
mod utils;

use anyhow::Result;
use clap::Parser as _;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    cli::run(args)
}
