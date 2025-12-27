// src/search/cli.rs
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
        search: SearchArgs,
    }

    #[test]
    fn test_search_exactly_flag() {
        // REQ-SEARCH-011
        let args = TestArgs::parse_from(["program", "--exactly", "refactor"]);
        assert!(args.search.exactly.is_some());
        assert_eq!(args.search.exactly.unwrap(), vec!["refactor"]);
    }

    #[test]
    fn test_search_exactly_multiple_tags() {
        // REQ-SEARCH-011
        let args = TestArgs::parse_from(["program", "--exactly", "refactor", "draft"]);
        assert_eq!(
            args.search.exactly.unwrap(),
            vec!["refactor", "draft"]
        );
    }

    #[test]
    fn test_search_with_directories() {
        // REQ-SEARCH-005
        let args = TestArgs::parse_from(["program", "--exactly", "refactor", "-d", "dir1", "dir2"]);
        assert_eq!(args.search.directories.len(), 2);
    }

    #[test]
    fn test_search_default_directory() {
        // REQ-SEARCH-006
        let args = TestArgs::parse_from(["program", "--exactly", "refactor"]);
        assert_eq!(args.search.directories.len(), 1);
        assert_eq!(args.search.directories[0], PathBuf::from("."));
    }

    #[test]
    fn test_search_with_exclude() {
        // REQ-SEARCH-007
        let args = TestArgs::parse_from(["program", "--exactly", "refactor", "-e", "node_modules", "target"]);
        assert_eq!(args.search.exclude.len(), 2);
    }
}

// ============================================
// TYPE DEFINITIONS
// ============================================

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Directories to scan (space-separated, defaults to current directory)
    #[arg(short = 'd', long = "dir", num_args = 0.., default_values = &["."])]
    pub directories: Vec<PathBuf>,

    /// Directories to exclude (space-separated)
    #[arg(short, long, num_args = 0..)]
    pub exclude: Vec<String>,

    /// Find files with exactly these tags (space-separated)
    #[arg(long, num_args = 1..)]
    pub exactly: Option<Vec<String>>,
}

// ============================================
// IMPLEMENTATIONS
// ============================================

pub fn run(args: SearchArgs) -> Result<()> {
    // Require at least one filter flag
    if args.exactly.is_none() {
        anyhow::bail!("At least one filter flag (--exactly) must be specified");
    }

    let exclude_dirs: Vec<&str> = args.exclude.iter().map(String::as_str).collect();

    if let Some(tags) = args.exactly {
        let tag_refs: Vec<&str> = tags.iter().map(String::as_str).collect();
        let files = crate::search::search_exactly(&args.directories, &tag_refs, &exclude_dirs)?;

        for file in &files {
            println!("{}", file);
        }
    }

    Ok(())
}
