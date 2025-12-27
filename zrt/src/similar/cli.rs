use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

// ============================================
// TESTS
// ============================================
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct TestArgs {
        #[command(flatten)]
        similar: SimilarArgs,
    }

    #[test]
    fn test_similar_threshold_flag() {
        let args = TestArgs::parse_from(["program", "--threshold", "0.7"]);
        assert_eq!(args.similar.threshold, 0.7);
    }

    #[test]
    fn test_similar_default_threshold() {
        let args = TestArgs::parse_from(["program"]);
        assert_eq!(args.similar.threshold, 0.5);
    }

    #[test]
    fn test_similar_with_directories() {
        let args = TestArgs::parse_from(["program", "-d", "dir1", "dir2"]);
        assert_eq!(args.similar.directories.len(), 2);
    }

    #[test]
    fn test_similar_default_directory() {
        let args = TestArgs::parse_from(["program"]);
        assert_eq!(args.similar.directories.len(), 1);
        assert_eq!(args.similar.directories[0], PathBuf::from("."));
    }

    #[test]
    fn test_similar_with_exclude() {
        let args = TestArgs::parse_from(["program", "-e", "node_modules", "target"]);
        assert_eq!(args.similar.exclude.len(), 2);
    }
}

// ============================================
// TYPE DEFINITIONS
// ============================================

#[derive(Args, Debug)]
pub struct SimilarArgs {
    /// Directories to scan (space-separated, defaults to current directory)
    #[arg(short = 'd', long = "dir", num_args = 0.., default_values = &["."])]
    pub directories: Vec<PathBuf>,

    /// Directories to exclude (space-separated)
    #[arg(short, long, num_args = 0..)]
    pub exclude: Vec<String>,

    /// Similarity threshold (0.0-1.0)
    #[arg(long, default_value = "0.5")]
    pub threshold: f64,
}

// ============================================
// IMPLEMENTATIONS
// ============================================

pub fn run(args: SimilarArgs) -> Result<()> {
    let exclude_dirs: Vec<&str> = args.exclude.iter().map(String::as_str).collect();

    let pairs = crate::similar::find_similar(&args.directories, args.threshold, &exclude_dirs)?;

    for (_, path1, path2) in &pairs {
        println!("{} {}", path1.display(), path2.display());
    }

    Ok(())
}
