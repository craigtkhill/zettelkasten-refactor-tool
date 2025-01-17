#![warn(clippy::all)]
// #![warn(clippy::pedantic)]
// #![warn(clippy::nursery)]
// #![warn(clippy::cargo)]
// #![warn(clippy::restriction)]

use anyhow::Result;
use clap::Parser;
use zrt::Args;

fn main() -> Result<()> {
    let args = Args::parse();
    zrt::run(args)
}
