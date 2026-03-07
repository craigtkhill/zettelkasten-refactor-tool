use anyhow::Result;
use clap::Args;
use std::io::{self, Read};
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
        connected: ConnectedArgs,
    }

    #[test]
    fn test_should_accept_tag_as_positional_argument() {
        // REQ-CONN-001

        // Given / When
        let args = TestArgs::parse_from(["program", "writing"]);

        // Then
        assert_eq!(args.connected.tag, Some("writing".to_string()));
    }

    #[test]
    fn test_should_accept_limit_flag() {
        // REQ-CONN-008

        // Given / When
        let args = TestArgs::parse_from(["program", "writing", "--limit", "3"]);

        // Then
        assert_eq!(args.connected.limit, 3);
    }

    #[test]
    fn test_should_default_limit_to_twenty() {
        // REQ-CONN-008

        // Given / When
        let args = TestArgs::parse_from(["program", "writing"]);

        // Then
        assert_eq!(args.connected.limit, 20);
    }
}

// ============================================
// TYPE DEFINITIONS
// ============================================

#[derive(Args, Debug)]
pub struct ConnectedArgs {
    /// Tag to filter by (reads from stdin if not provided)
    pub tag: Option<String>,

    /// Directories to scan (space-separated, defaults to current directory)
    #[arg(short = 'd', long = "dir", num_args = 0.., default_values = &["."])]
    pub directories: Vec<PathBuf>,

    /// Directories to exclude (space-separated)
    #[arg(short, long, num_args = 0..)]
    pub exclude: Vec<String>,

    /// Number of results to show (default: 20)
    #[arg(long, default_value = "20")]
    pub limit: usize,
}

// ============================================
// IMPLEMENTATIONS
// ============================================

pub fn run(args: ConnectedArgs) -> Result<()> {
    let tag = match args.tag {
        Some(t) => t,
        None => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                anyhow::bail!("No tag provided via argument or stdin");
            }
            trimmed
        }
    };

    let exclude_dirs: Vec<&str> = args.exclude.iter().map(String::as_str).collect();
    let results = crate::connected::most_connected(&args.directories, &tag, &exclude_dirs)?;

    for (path, _) in results.iter().take(args.limit) {
        println!("{tag} {path}");
    }

    Ok(())
}
