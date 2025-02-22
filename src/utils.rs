// src/utils.rs
use crate::models::FileWordCount;
use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Deserialize, Debug, Default)]
pub struct Frontmatter {
    pub tags: Option<Vec<String>>,
}

pub fn parse_frontmatter(content: &str) -> Result<Frontmatter> {
    let mut content_iter = content.lines();

    // Check for frontmatter delimiter
    if content_iter.next() != Some("---") {
        return Ok(Frontmatter::default());
    }

    // Collect frontmatter content
    let mut frontmatter_str = String::new();
    for line in content_iter {
        if line == "---" {
            break;
        }
        frontmatter_str.push_str(line);
        frontmatter_str.push('\n');
    }

    // Parse YAML
    serde_yaml_ng::from_str(&frontmatter_str)
        .map_err(|e| anyhow!("Failed to parse frontmatter: {}", e))
}

pub fn contains_tag(path: &Path, tag: &str) -> io::Result<bool> {
    let content = fs::read_to_string(path)?;

    // Parse frontmatter
    match parse_frontmatter(&content) {
        Ok(frontmatter) => {
            // Check if the tag exists in the frontmatter tags
            Ok(frontmatter
                .tags
                .is_some_and(|tags| tags.iter().any(|t| t == tag)))
        }
        Err(_) => Ok(false), // If parsing fails, assume no tags
    }
}

pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name().to_str().is_some_and(|s| {
        // Don't consider temp directories as hidden
        if s.starts_with(".tmp") {
            return false;
        }
        s.starts_with('.')
    })
}

pub fn print_top_files(files: Vec<FileWordCount>, top: usize) {
    for file in files.iter().take(top) {
        println!("{:8} words  {}", file.words, file.path.display());
    }
}
