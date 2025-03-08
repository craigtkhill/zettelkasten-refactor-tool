// src/core/ignore/patterns.rs
use anyhow::{Context as _, Result};
use glob::Pattern;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct Patterns {
    /// Collection of ignore patterns with metadata.
    /// Each tuple contains:
    /// - The pattern to match against file paths
    /// - Whether the pattern is a negation (to explicitly include files that would otherwise be ignored)
    /// - Whether the pattern is anchored to the root directory
    patterns: Vec<(Pattern, bool, bool)>,
}

impl Patterns {
    /// Adds a new pattern to the ignore list.
    ///
    /// Parses the pattern string, handling various pattern formats:
    /// - Negation with `!` prefix
    /// - Directory-specific patterns ending with `/`
    /// - File extension groups like `*.{js,ts}`
    /// - Absolute path patterns starting with `/`
    /// - Bare filenames
    ///
    /// # Arguments
    ///
    /// * `pattern` - The pattern string to add
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the pattern was successfully added
    ///
    /// # Errors
    ///
    /// This function may return an error if:
    /// * The pattern contains invalid glob syntax
    /// * The pattern has mismatched braces in extension groups
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// * A pattern contains an opening brace `{` without a matching closing brace `}`
    /// * A pattern contains a closing brace `}` without a matching opening brace `{`
    #[inline]
    pub fn add_pattern(&mut self, pattern: &str) -> Result<()> {
        let pattern = pattern.trim();
        if pattern.is_empty() || pattern.starts_with('#') {
            return Ok(());
        }

        let (pattern, is_negation) = pattern
            .strip_prefix('!')
            .map_or((pattern, false), |stripped| (stripped, true));
        let is_anchored = pattern.starts_with('/');
        let pattern_str = if is_anchored {
            pattern.chars().skip(1).collect::<String>()
        } else {
            pattern.to_owned()
        };
        let pattern_str_for_later = pattern_str.clone();

        let is_bare_filename = !pattern_str.contains('/')
            && !pattern_str.contains('\\')
            && !pattern_str.contains('*')
            && !pattern_str.contains('?')
            && !pattern_str.contains('[');

        let mut glob_pattern = if pattern_str.contains('*')
            || pattern_str.contains('?')
            || pattern_str.contains('[')
        {
            if pattern_str.contains("**") {
                pattern_str.replace("**", "[GLOBSTAR]")
            } else {
                pattern_str
            }
        } else if pattern_str.ends_with('/') {
            if is_negation {
                format!("**/{pattern_str}**/*")
            } else {
                format!("**/{pattern_str}**")
            }
        } else if is_negation || pattern_str.contains('.') || is_bare_filename {
            pattern_str
        } else {
            format!("{pattern_str}/**")
        };
        if !is_anchored && !glob_pattern.contains('/') && !glob_pattern.contains('\\') {
            glob_pattern = format!("**/{glob_pattern}");
        }

        if glob_pattern.contains('{') {
            let (prefix, suffix) = glob_pattern
                .split_once('{')
                .ok_or_else(|| anyhow::anyhow!("Invalid pattern: missing opening brace"))?;
            let (extensions, rest) = suffix
                .split_once('}')
                .ok_or_else(|| anyhow::anyhow!("Invalid pattern: missing closing brace"))?;
            let extensions: Vec<&str> = extensions.split(',').map(str::trim).collect();

            for ext in extensions {
                let full_pattern = format!("{prefix}{ext}{rest}").replace("[GLOBSTAR]", "**");
                let pattern_compiled = Pattern::new(&full_pattern)
                    .with_context(|| format!("Invalid pattern: {full_pattern}"))?;
                self.patterns
                    .push((pattern_compiled, is_negation, is_anchored));
            }
            return Ok(());
        }
        if is_bare_filename && !is_anchored {
            let path_pattern = format!("**/{pattern_str_for_later}");
            let compiled = Pattern::new(&path_pattern)
                .with_context(|| format!("Invalid path pattern: {path_pattern}"))?;
            self.patterns.push((compiled, is_negation, false));
            let pattern_compiled = Pattern::new(&pattern_str_for_later)
                .with_context(|| format!("Invalid filename pattern: {pattern_str_for_later}"))?;
            self.patterns.push((pattern_compiled, is_negation, false));

            return Ok(());
        }

        let glob_pattern = glob_pattern.replace("[GLOBSTAR]", "**");
        let compiled = Pattern::new(&glob_pattern)
            .with_context(|| format!("Invalid pattern: {glob_pattern}"))?;
        self.patterns.push((compiled, is_negation, is_anchored));
        Ok(())
    }

    #[inline]
    #[must_use]
    pub fn new(_root_dir: PathBuf) -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    #[inline]
    pub fn matches<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        let path_str = path.to_string_lossy();
        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy())
            .unwrap_or_default();
        for tuple in &self.patterns {
            let (pattern, is_neg, is_anchored) = (&tuple.0, tuple.1, tuple.2);
            let is_simple_anchored = is_anchored && !pattern.as_str().contains('/');

            if is_simple_anchored && path_str.contains('/') {
                continue;
            }

            if is_neg && (pattern.matches(&path_str) || pattern.matches(&filename)) {
                return false;
            }
        }
        for tuple in &self.patterns {
            let (pattern, is_neg, is_anchored) = (&tuple.0, tuple.1, tuple.2);
            let is_simple_anchored = is_anchored && !pattern.as_str().contains('/');

            if is_simple_anchored && path_str.contains('/') {
                continue;
            }

            if !is_neg && (pattern.matches(&path_str) || pattern.matches(&filename)) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_empty_patterns_match_nothing() {
        let patterns = Patterns::new(PathBuf::from("/test"));
        assert!(!patterns.matches("file.txt"));
    }

    #[test]
    fn test_simple_file_pattern() -> Result<()> {
        let mut patterns = Patterns::new(PathBuf::from("/test"));
        patterns.add_pattern("*.txt")?;
        assert!(patterns.matches("file.txt"));
        assert!(!patterns.matches("file.rs"));
        Ok(())
    }

    #[test]
    fn test_directory_pattern() -> Result<()> {
        let mut patterns = Patterns::new(PathBuf::from("/test"));
        patterns.add_pattern("node_modules/")?;

        assert!(
            patterns.matches("node_modules/package.json"),
            "Should match file directly in node_modules"
        );

        assert!(
            patterns.matches("src/node_modules/package.json"),
            "Should match node_modules in subdirectory"
        );

        assert!(
            !patterns.matches("nodemodules/file.txt"),
            "Should not match directory with similar name"
        );
        Ok(())
    }

    #[test]
    fn test_negation_pattern() -> Result<()> {
        let mut patterns = Patterns::new(PathBuf::from("/test"));
        patterns.add_pattern("*.txt")?;
        patterns.add_pattern("!important.txt")?;
        assert!(patterns.matches("file.txt"));
        assert!(!patterns.matches("important.txt"));
        Ok(())
    }

    #[test]
    fn test_absolute_path_pattern() -> Result<()> {
        let mut patterns = Patterns::new(PathBuf::from("/test"));
        patterns.add_pattern("/src/generated/*.rs")?;
        assert!(patterns.matches("src/generated/file.rs"));
        assert!(!patterns.matches("other/generated/file.rs"));
        Ok(())
    }

    #[test]
    fn test_anchored_path_pattern() -> Result<()> {
        let mut patterns = Patterns::new(PathBuf::from("/test"));
        patterns.add_pattern("/absolute_path.md")?;
        assert!(
            patterns.matches("absolute_path.md"),
            "Should match anchored path at root"
        );
        assert!(
            !patterns.matches("subdirectory/absolute_path.md"),
            "Should not match anchored path in subdirectory"
        );

        Ok(())
    }

    #[test]
    fn test_extension_group_pattern() -> Result<()> {
        let mut patterns = Patterns::new(PathBuf::from("/test"));
        patterns.add_pattern("*.{js,ts}")?;
        assert!(patterns.matches("file.js"));
        assert!(patterns.matches("file.ts"));
        assert!(!patterns.matches("file.rs"));
        Ok(())
    }

    #[test]
    fn test_double_star_pattern() -> Result<()> {
        let mut patterns = Patterns::new(PathBuf::from("/test"));
        patterns.add_pattern("**/temp/**")?;
        assert!(patterns.matches("temp/file.txt"));
        assert!(patterns.matches("src/temp/file.txt"));
        assert!(patterns.matches("src/temp/subfolder/file.txt"));
        assert!(!patterns.matches("src/temporary/file.txt"));
        Ok(())
    }

    #[test]
    fn test_comment_and_empty_lines() -> Result<()> {
        let mut patterns = Patterns::new(PathBuf::from("/test"));
        patterns.add_pattern("")?;
        patterns.add_pattern("# This is a comment")?;
        patterns.add_pattern("*.txt")?;
        assert!(patterns.matches("file.txt"));
        Ok(())
    }

    #[test]
    fn test_bare_filename_pattern() -> Result<()> {
        let mut patterns = Patterns::new(PathBuf::from("/test"));
        patterns.add_pattern("TODO-CHORES.md")?;

        assert!(
            patterns.matches("TODO-CHORES.md"),
            "Should match exact filename at root"
        );

        assert!(
            patterns.matches("subdir/TODO-CHORES.md"),
            "Should match filename in subdirectory"
        );

        assert!(
            !patterns.matches("NOT-TODO-CHORES.md"),
            "Should not match similar filenames"
        );

        Ok(())
    }
}
