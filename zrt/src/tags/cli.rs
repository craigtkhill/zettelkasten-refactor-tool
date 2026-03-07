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
        tags: TagsArgs,
    }

    #[test]
    fn test_should_accept_exclude_tag_flag() {
        // REQ-TAGS-004

        // Given / When
        let args = TestArgs::parse_from(["program", "--exclude-tag", "refactored", "game"]);

        // Then
        assert_eq!(args.tags.exclude_tag, vec!["refactored", "game"]);
    }

    #[test]
    fn test_should_accept_limit_flag() {
        // REQ-TAGS-005

        // Given / When
        let args = TestArgs::parse_from(["program", "--limit", "5"]);

        // Then
        assert_eq!(args.tags.limit, Some(5));
    }

    #[test]
    fn test_should_default_to_current_directory() {
        // REQ-TAGS-006

        // Given / When
        let args = TestArgs::parse_from(["program"]);

        // Then
        assert_eq!(args.tags.directories, vec![PathBuf::from(".")]);
    }
}

// ============================================
// TYPE DEFINITIONS
// ============================================

#[derive(Args, Debug)]
pub struct TagsArgs {
    /// Directories to scan (space-separated, defaults to current directory)
    #[arg(short = 'd', long = "dir", num_args = 0.., default_values = &["."])]
    pub directories: Vec<PathBuf>,

    /// Directories to exclude (space-separated)
    #[arg(short, long, num_args = 0..)]
    pub exclude: Vec<String>,

    /// Tag names to exclude from results (space-separated)
    #[arg(long = "exclude-tag", num_args = 0..)]
    pub exclude_tag: Vec<String>,

    /// Show only the top N tags
    #[arg(long)]
    pub limit: Option<usize>,
}

// ============================================
// IMPLEMENTATIONS
// ============================================

pub fn run(args: TagsArgs) -> Result<()> {
    let exclude_dirs: Vec<&str> = args.exclude.iter().map(String::as_str).collect();
    let exclude_tags: Vec<&str> = args.exclude_tag.iter().map(String::as_str).collect();

    let results = crate::tags::count_tags(&args.directories, &exclude_tags, &exclude_dirs)?;

    let output = match args.limit {
        Some(n) => &results[..n.min(results.len())],
        None => &results[..],
    };

    for (tag, _) in output {
        println!("{tag}");
    }

    Ok(())
}
