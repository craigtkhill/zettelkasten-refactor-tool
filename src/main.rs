use anyhow::Result;
use clap::Parser;
use zrt::Args;

fn main() -> Result<()> {
    let args = Args::parse();
    zrt::run(args)
}