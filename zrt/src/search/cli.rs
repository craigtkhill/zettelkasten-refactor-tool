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
    fn test_should_accept_tags_flag_with_single_tag() {
        // REQ-SEARCH-011

        // Given / When
        let args = TestArgs::parse_from(["program", "--tags", "refactor"]);

        // Then
        assert_eq!(args.search.tags.unwrap(), vec!["refactor"]);
    }

    #[test]
    fn test_should_accept_tags_flag_with_multiple_tags() {
        // REQ-SEARCH-011

        // Given / When
        let args = TestArgs::parse_from(["program", "--tags", "refactor", "draft"]);

        // Then
        assert_eq!(args.search.tags.unwrap(), vec!["refactor", "draft"]);
    }

    #[test]
    fn test_should_require_at_least_one_tag_with_tags_flag() {
        // REQ-SEARCH-012

        // Given / When
        let result = TestArgs::try_parse_from(["program", "--tags"]);

        // Then
        assert!(result.is_err());
    }

    #[test]
    fn test_should_accept_no_tags_flag() {
        // REQ-SEARCH-013

        // Given / When
        let args = TestArgs::parse_from(["program", "--no-tags"]);

        // Then
        assert!(args.search.no_tags);
    }

    #[test]
    fn test_should_reject_tags_and_no_tags_together() {
        // REQ-SEARCH-016

        // Given / When
        let result = TestArgs::try_parse_from(["program", "--tags", "foo", "--no-tags"]);

        // Then
        assert!(result.is_err());
    }

    #[test]
    fn test_search_with_directories() {
        // REQ-SEARCH-005

        // Given / When
        let args = TestArgs::parse_from(["program", "--tags", "refactor", "-d", "dir1", "dir2"]);

        // Then
        assert_eq!(args.search.directories.len(), 2);
    }

    #[test]
    fn test_search_default_directory() {
        // REQ-SEARCH-006

        // Given / When
        let args = TestArgs::parse_from(["program", "--tags", "refactor"]);

        // Then
        assert_eq!(args.search.directories.len(), 1);
        assert_eq!(args.search.directories[0], PathBuf::from("."));
    }

    #[test]
    fn test_search_with_exclude() {
        // REQ-SEARCH-007

        // Given / When
        let args = TestArgs::parse_from(["program", "--tags", "refactor", "-e", "node_modules", "target"]);

        // Then
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
    #[arg(long, num_args = 1.., conflicts_with = "no_tags")]
    pub tags: Option<Vec<String>>,

    /// Find files that have no tags
    #[arg(long, conflicts_with = "tags")]
    pub no_tags: bool,
}

// ============================================
// IMPLEMENTATIONS
// ============================================

pub fn run(args: SearchArgs) -> Result<()> {
    if args.tags.is_none() && !args.no_tags {
        anyhow::bail!("At least one filter flag (--tags or --no-tags) must be specified");
    }

    let exclude_dirs: Vec<&str> = args.exclude.iter().map(String::as_str).collect();

    if let Some(tags) = args.tags {
        let tag_refs: Vec<&str> = tags.iter().map(String::as_str).collect();
        let files = crate::search::search_exactly(&args.directories, &tag_refs, &exclude_dirs)?;
        for file in &files {
            println!("{}", file);
        }
    } else if args.no_tags {
        let files = crate::search::search_missing_tags(&args.directories, &exclude_dirs)?;
        for file in &files {
            println!("{}", file);
        }
    }

    Ok(())
}
