// src/core/scanner.rs
use crate::core::ignore::{load_ignore_patterns, IgnorePatterns};
use crate::models::{ComparisonStats, FileWordCount, SinglePatternStats};
use crate::utils::{contains_tag, is_hidden};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn count_files(dir: &PathBuf, exclude_dirs: &[&str]) -> Result<u64> {
    let ignore_patterns = load_ignore_patterns(dir)?;
    let mut count = 0;

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, exclude_dirs, Some(&ignore_patterns)))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            count += 1;
        }
    }

    println!("Total files found: {count}");
    Ok(count)
}

pub fn count_words(
    dir: &PathBuf,
    exclude_dirs: &[&str],
    filter_out: Option<&str>,
) -> Result<Vec<FileWordCount>> {
    let ignore_patterns = load_ignore_patterns(dir)?;
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, exclude_dirs, Some(&ignore_patterns)))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if let Ok(content) = fs::read_to_string(path) {
            // Skip file if it contains the filter_out tag
            if let Some(tag) = filter_out {
                if let Ok(frontmatter) = crate::utils::parse_frontmatter(&content) {
                    if let Some(tags) = frontmatter.tags {
                        if tags.iter().any(|t| t == tag) {
                            continue;
                        }
                    }
                }
            }

            let word_count = content.split_whitespace().count();
            files.push(FileWordCount {
                path: path.to_path_buf(),
                words: word_count,
            });
        }
    }

    files.sort_by(|a, b| b.words.cmp(&a.words));
    Ok(files)
}

pub fn scan_directory_single_pattern(dir: &PathBuf, pattern: &str) -> Result<SinglePatternStats> {
    let ignore_patterns = load_ignore_patterns(dir)?;
    let mut stats = SinglePatternStats::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, &[], Some(&ignore_patterns)))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        stats.total_files += 1;

        let path = entry.path();
        if contains_tag(path, pattern)? {
            stats.files_with_pattern += 1;
        }
    }

    Ok(stats)
}

pub fn scan_directory_two_patterns(
    dir: &PathBuf,
    done_tag: &str,
    todo_tag: &str,
) -> Result<ComparisonStats> {
    let ignore_patterns = load_ignore_patterns(dir)?;
    let mut stats = ComparisonStats::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e, &[], Some(&ignore_patterns)))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        stats.total_files += 1;

        let path = entry.path();
        if contains_tag(path, done_tag)? {
            stats.done_files += 1;
        }
        if contains_tag(path, todo_tag)? {
            stats.todo_files += 1;
        }
    }

    Ok(stats)
}

fn should_exclude(
    entry: &walkdir::DirEntry,
    exclude_dirs: &[&str],
    ignore_patterns: Option<&IgnorePatterns>,
) -> bool {
    if is_hidden(entry) {
        return true;
    }

    // Check manual exclude dirs
    if let Some(path_str) = entry.path().to_str() {
        for dir in exclude_dirs {
            if path_str.contains(&format!("/{dir}/")) {
                return true;
            }
        }
    }

    // Check ignore patterns
    if let Some(patterns) = ignore_patterns {
        if patterns.matches(entry.path()) {
            return true;
        }
    }

    false
}