// src/core/ignore.rs
use anyhow::{Context as _, Result};
use glob::Pattern;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct Patterns {
    /// Collection of ignore patterns with metadata.
    /// Each tuple contains:
    /// - The pattern to match against file paths
    /// - Whether the pattern is a negation (to explicitly include files that would otherwise be ignored)
    /// - Whether the pattern is anchored to the root directory
    patterns: Vec<(Pattern, bool, bool)>, // (pattern, is_negation, is_anchored_to_root)
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

        // Flag to track if this is an absolute path pattern (anchored to root)
        let is_anchored = pattern.starts_with('/');

        // Handle absolute paths
        let pattern_str = if is_anchored {
            pattern.chars().skip(1).collect::<String>()
        } else {
            pattern.to_owned()
        };

        // Create a clone early to preserve the original value for later use
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

        // Only add **/ prefix for non-absolute patterns that don't have path separators
        if !is_anchored && !glob_pattern.contains('/') && !glob_pattern.contains('\\') {
            glob_pattern = format!("**/{glob_pattern}");
        }

        // Handle file extension groups like *.{js,ts}
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

        // Create both a path pattern and a filename pattern for bare filenames
        if is_bare_filename && !is_anchored {
            // Create the path pattern (with **/ prefix) using the saved copy
            let path_pattern = format!("**/{pattern_str_for_later}");
            let compiled = Pattern::new(&path_pattern)
                .with_context(|| format!("Invalid path pattern: {path_pattern}"))?;
            self.patterns.push((compiled, is_negation, false));

            // Also create a direct filename pattern (without the path)
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

        // Get the path string and filename
        let path_str = path.to_string_lossy();
        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy())
            .unwrap_or_default();

        // First check negation patterns
        for tuple in &self.patterns {
            let (pattern, is_neg, is_anchored) = (&tuple.0, tuple.1, tuple.2);

            // For an anchored pattern with no subdirectories in the pattern itself,
            // it should only match files at the root level
            let is_simple_anchored = is_anchored && !pattern.as_str().contains('/');

            if is_simple_anchored && path_str.contains('/') {
                // Skip this pattern for paths with subdirectories
                continue;
            }

            if is_neg && (pattern.matches(&path_str) || pattern.matches(&filename)) {
                return false;
            }
        }

        // Handle normal patterns
        for tuple in &self.patterns {
            let (pattern, is_neg, is_anchored) = (&tuple.0, tuple.1, tuple.2);

            // For an anchored pattern with no subdirectories in the pattern itself,
            // it should only match files at the root level
            let is_simple_anchored = is_anchored && !pattern.as_str().contains('/');

            if is_simple_anchored && path_str.contains('/') {
                // Skip this pattern for paths with subdirectories
                continue;
            }

            if !is_neg && (pattern.matches(&path_str) || pattern.matches(&filename)) {
                return true;
            }
        }

        false
    }
}

/// Loads ignore patterns from .zrtignore files starting from the given directory
/// and recursively checking parent directories until a file is found.
///
/// # Arguments
///
/// * `dir` - The starting directory to search for .zrtignore files
///
/// # Returns
///
/// * `Ok(Patterns)` containing the loaded patterns
///
/// # Errors
///
/// This function may return an error if:
/// * The .zrtignore file exists but cannot be read
/// * The file contains invalid pattern syntax
/// * File system operations fail during the search
#[inline]
pub fn load_ignore_patterns(dir: &Path) -> Result<Patterns> {
    let mut patterns = Patterns::new(PathBuf::new());

    let mut current_dir = dir.to_path_buf();

    let mut visited = HashSet::new();

    while !visited.contains(&current_dir) {
        visited.insert(current_dir.clone());

        let ignore_file = current_dir.join(".zrtignore");

        if ignore_file.exists() {
            let content = fs::read_to_string(&ignore_file)
                .with_context(|| format!("Failed to read .zrtignore file: {ignore_file:?}"))?;

            for line in content.lines() {
                patterns.add_pattern(line)?;
            }

            break;
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    Ok(patterns)
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

        // Should match at root level
        assert!(
            patterns.matches("absolute_path.md"),
            "Should match anchored path at root"
        );

        // Should not match in subdirectory
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

    #[test]
    fn test_relative_path_matching() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;

        // Create a .zrtignore file with a specific pattern
        let ignore_file = temp_dir.path().join(".zrtignore");
        std::fs::write(&ignore_file, "ignore_me.tmp\n")?;

        // Load patterns
        let patterns = load_ignore_patterns(temp_dir.path())?;

        // Test with relative path
        let relative_path = PathBuf::from("ignore_me.tmp");

        assert!(
            patterns.matches(&relative_path),
            "Should match relative path"
        );

        Ok(())
    }

    #[test]
    fn test_load_ignore_patterns() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let ignore_file = temp_dir.path().join(".zrtignore");
        std::fs::write(
            &ignore_file,
            "*.txt\n!important.txt\n# comment\n\n/src/generated/*.rs",
        )?;

        let patterns = load_ignore_patterns(temp_dir.path())?;
        assert!(patterns.matches("file.txt"));
        assert!(!patterns.matches("important.txt"));
        assert!(patterns.matches("src/generated/test.rs"));
        assert!(!patterns.matches("src/main.rs"));
        Ok(())
    }

    #[test]
    fn test_todo_chores_ignore() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;

        let ignore_file = temp_dir.path().join(".zrtignore");
        std::fs::write(
            &ignore_file,
            "ARCHIVE/\nCALENDAR/\nDRAWINGS/\nIMAGES/\n.git/\nTODO-CHORES.md\n",
        )?;

        let todo_file = temp_dir.path().join("TODO-CHORES.md");
        std::fs::write(&todo_file, "Test content")?;

        let other_file = temp_dir.path().join("OTHER-FILE.md");
        std::fs::write(&other_file, "Other content")?;

        let patterns = load_ignore_patterns(temp_dir.path())?;

        assert!(
            patterns.matches(&todo_file),
            "TODO-CHORES.md should match the ignore pattern"
        );

        assert!(
            !patterns.matches(&other_file),
            "OTHER-FILE.md should not match any ignore pattern"
        );

        Ok(())
    }
}
