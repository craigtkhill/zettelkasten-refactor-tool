// src/core/ignore.rs
use anyhow::{Context, Result};
use glob::Pattern;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct IgnorePatterns {
    patterns: Vec<(Pattern, bool)>, // (pattern, is_negation)
    root_dir: PathBuf,
}

impl IgnorePatterns {
    #[must_use]
    pub const fn new(root_dir: PathBuf) -> Self {
        Self {
            patterns: Vec::new(),
            root_dir,
        }
    }

    pub fn add_pattern(&mut self, pattern: &str) -> Result<()> {
        // Skip empty lines and comments
        let pattern = pattern.trim();
        if pattern.is_empty() || pattern.starts_with('#') {
            return Ok(());
        }

        // Handle negation patterns
        let (pattern, is_negation) = if let Some(stripped) = pattern.strip_prefix('!') {
            (stripped, true)
        } else {
            (pattern, false)
        };

        // Flag to track if this is an absolute path pattern
        let is_absolute = pattern.starts_with('/');

        // Handle absolute paths
        let pattern = if is_absolute {
            pattern[1..].to_string()
        } else {
            pattern.to_string()
        };

        // Convert the pattern to a glob pattern
        let mut glob_pattern =
            if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
                // Replace ** with a special marker that won't match normal paths
                if pattern.contains("**") {
                    pattern.replace("**", "[GLOBSTAR]")
                } else {
                    pattern
                }
            } else if pattern.ends_with('/') {
                if is_negation {
                    format!("{pattern}**/*") // For negation, match all files in directory
                } else {
                    format!("{pattern}**")
                }
            } else if is_negation || pattern.contains('.') {
                pattern // For negation or files with extension, match exactly
            } else {
                format!("{pattern}/**") // Otherwise, match directory
            };

        // Handle case where pattern is just a filename without path
        // Only add **/ prefix for non-absolute patterns
        if !is_absolute && !glob_pattern.contains('/') && !glob_pattern.contains('\\') {
            glob_pattern = format!("**/{glob_pattern}");
        }

        // Handle file extension groups like *.{js,ts}
        if glob_pattern.contains('{') {
            // Split the pattern into multiple patterns
            let (prefix, suffix) = glob_pattern
                .split_once('{')
                .expect("Invalid pattern: missing opening brace");
            let (extensions, rest) = suffix
                .split_once('}')
                .expect("Invalid pattern: missing closing brace");
            let extensions: Vec<&str> = extensions.split(',').map(str::trim).collect();

            for ext in extensions {
                let full_pattern = format!("{prefix}{ext}{rest}").replace("[GLOBSTAR]", "**");
                let compiled = Pattern::new(&full_pattern)
                    .with_context(|| format!("Invalid pattern: {full_pattern}"))?;
                self.patterns.push((compiled, is_negation));
            }
            return Ok(());
        }

        let glob_pattern = glob_pattern.replace("[GLOBSTAR]", "**");
        let compiled = Pattern::new(&glob_pattern)
            .with_context(|| format!("Invalid pattern: {glob_pattern}"))?;
        self.patterns.push((compiled, is_negation));
        Ok(())
    }

    pub fn matches(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();

        // Always use relative paths for matching
        let relative_path = if path.is_absolute() {
            if let Ok(rel) = path.strip_prefix(&self.root_dir) {
                rel.to_path_buf()
            } else {
                path.components()
                    .skip_while(|c| {
                        matches!(
                            c,
                            std::path::Component::RootDir | std::path::Component::Prefix(_)
                        )
                    })
                    .collect()
            }
        } else {
            path.to_path_buf()
        };

        let path_str = relative_path.to_string_lossy();

        // First check negation patterns
        for (pattern, _) in self.patterns.iter().filter(|(_, is_neg)| *is_neg) {
            if pattern.matches(&path_str) {
                return false;
            }
        }

        // Then check normal patterns
        for (pattern, _) in self.patterns.iter().filter(|(_, is_neg)| !*is_neg) {
            if pattern.matches(&path_str) {
                return true;
            }
        }

        false
    }
}

pub fn load_ignore_patterns(dir: &Path) -> Result<IgnorePatterns> {
    let mut patterns = IgnorePatterns::new(dir.to_path_buf());
    let ignore_file = dir.join(".zrtignore");

    if ignore_file.exists() {
        let content = fs::read_to_string(&ignore_file)
            .with_context(|| format!("Failed to read .zrtignore file: {ignore_file:?}"))?;

        for line in content.lines() {
            patterns.add_pattern(line)?;
        }
    }

    Ok(patterns)
}
