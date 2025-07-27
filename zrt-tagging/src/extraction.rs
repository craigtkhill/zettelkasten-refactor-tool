use anyhow::{Context as _, Result};
use serde_yaml_ng::Value;
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct NoteData {
    pub content: String,
    pub path: String,
    pub tags: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct TrainingData {
    pub all_tags: HashSet<String>,
    pub notes: Vec<NoteData>,
}

impl NoteData {
    /// Creates a NoteData instance from a file
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed
    #[inline]
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read note file: {}", path.display()))?;

        let (frontmatter, body) = extract_frontmatter(&content)?;
        let tags = extract_tags_from_frontmatter(&frontmatter)?;

        Ok(Self {
            content: body,
            path: path.to_string_lossy().to_string(),
            tags,
        })
    }
}

impl TrainingData {
    /// Creates a new empty TrainingData instance
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            all_tags: HashSet::new(),
            notes: Vec::new(),
        }
    }

    #[inline]
    pub fn add_note(&mut self, note: NoteData) {
        for tag in &note.tags {
            self.all_tags.insert(tag.clone());
        }
        self.notes.push(note);
    }

    #[inline]
    pub fn filter_by_min_examples(&mut self, min_examples: usize) {
        let mut tag_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for note in &self.notes {
            for tag in &note.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        let valid_tags: HashSet<String> = tag_counts
            .into_iter()
            .filter_map(|(tag, count)| {
                if count >= min_examples {
                    Some(tag)
                } else {
                    None
                }
            })
            .collect();

        for note in &mut self.notes {
            note.tags.retain(|tag| valid_tags.contains(tag));
        }

        self.notes.retain(|note| !note.tags.is_empty());
        self.all_tags = valid_tags;
    }

    #[inline]
    pub fn exclude_tags(&mut self, excluded_tags: &HashSet<String>) {
        for note in &mut self.notes {
            note.tags.retain(|tag| !excluded_tags.contains(tag));
        }

        self.notes.retain(|note| !note.tags.is_empty());
        self.all_tags.retain(|tag| !excluded_tags.contains(tag));
    }

    /// Returns all unique tags in the training data
    #[must_use]
    #[inline]
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.all_tags.iter().cloned().collect();
        tags.sort();
        tags
    }
}

#[inline]
fn extract_frontmatter(content: &str) -> Result<(Option<String>, String)> {
    if !content.starts_with("---") {
        return Ok((None, content.to_owned()));
    }

    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 3 {
        return Ok((None, content.to_owned()));
    }

    let mut end_index = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_index = Some(i);
            break;
        }
    }

    if let Some(end) = end_index {
        let frontmatter = lines[1..end].join("\n");
        let body = lines[end + 1..].join("\n");
        Ok((Some(frontmatter), body))
    } else {
        Ok((None, content.to_owned()))
    }
}

#[inline]
fn extract_tags_from_frontmatter(frontmatter: &Option<String>) -> Result<HashSet<String>> {
    let Some(fm) = frontmatter else {
        return Ok(HashSet::new());
    };

    let yaml: Value =
        serde_yaml_ng::from_str(fm).with_context(|| "Failed to parse YAML frontmatter")?;

    let mut tags = HashSet::new();

    if let Some(tags_value) = yaml.get("tags") {
        match tags_value {
            Value::Sequence(seq) => {
                for item in seq {
                    if let Some(tag_str) = item.as_str() {
                        tags.insert(clean_tag(tag_str));
                    }
                }
            }
            Value::String(tag_str) => {
                tags.insert(clean_tag(tag_str));
            }
            _ => {}
        }
    }

    Ok(tags)
}

#[inline]
fn clean_tag(tag: &str) -> String {
    tag.trim()
        .trim_start_matches('#')
        .trim()
        .to_lowercase()
        .replace(' ', "_")
}

/// Extracts training data from a directory of notes
///
/// # Errors
/// Returns an error if directory traversal or file reading fails
#[expect(
    clippy::missing_inline_in_public_items,
    reason = "Development: file I/O function"
)]
pub fn extract_training_data(directory: &Path) -> Result<TrainingData> {
    let mut training_data = TrainingData::new();

    println!("Scanning directory: {}", directory.display());

    for entry in WalkDir::new(directory)
        .follow_links(false)
        .into_iter()
        .filter_map(core::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Only process markdown files
        let Some(ext) = path.extension() else {
            continue;
        };
        if ext != "md" && ext != "markdown" {
            continue;
        }

        // Skip hidden files and directories
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with('.'))
        {
            continue;
        }

        match NoteData::from_file(path) {
            Ok(note) => {
                if !note.tags.is_empty() {
                    training_data.add_note(note);
                }
            }
            Err(e) => {
                println!("Warning: Failed to process {}: {}", path.display(), e);
            }
        }
    }

    println!("Found {} notes with tags", training_data.notes.len());
    Ok(training_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tempfile::TempDir;

    fn create_test_note(dir: &Path, filename: &str, content: &str) -> Result<()> {
        let file_path = dir.join(filename);
        std::fs::write(file_path, content)?;
        Ok(())
    }

    #[test]
    fn test_note_data_from_file_with_tags() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let note_content = r#"---
tags: ["ml", "ai", "research"]
title: "Test Note"
---

# Machine Learning

This is a test note about machine learning and AI research.
"#;

        create_test_note(temp_dir.path(), "test.md", note_content)?;
        let note_path = temp_dir.path().join("test.md");

        let note_data = NoteData::from_file(&note_path)?;

        assert_eq!(
            note_data.content,
            "\n# Machine Learning\n\nThis is a test note about machine learning and AI research."
        );
        assert!(note_data.tags.contains("ml"));
        assert!(note_data.tags.contains("ai"));
        assert!(note_data.tags.contains("research"));
        assert_eq!(note_data.tags.len(), 3);

        Ok(())
    }

    #[test]
    fn test_note_data_from_file_no_frontmatter() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let note_content = "# Simple Note\n\nThis note has no frontmatter.";

        create_test_note(temp_dir.path(), "simple.md", note_content)?;
        let note_path = temp_dir.path().join("simple.md");

        let note_data = NoteData::from_file(&note_path)?;

        assert_eq!(note_data.content, note_content);
        assert!(note_data.tags.is_empty());

        Ok(())
    }

    #[test]
    fn test_note_data_from_file_string_tag() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let note_content = r#"---
tags: "single-tag"
---

Content here.
"#;

        create_test_note(temp_dir.path(), "single_tag.md", note_content)?;
        let note_path = temp_dir.path().join("single_tag.md");

        let note_data = NoteData::from_file(&note_path)?;

        assert!(note_data.tags.contains("single-tag"));
        assert_eq!(note_data.tags.len(), 1);

        Ok(())
    }

    #[test]
    fn test_training_data_add_note() {
        let mut training_data = TrainingData::new();
        let mut tags = HashSet::new();
        tags.insert("test".to_owned());
        tags.insert("example".to_owned());

        let note = NoteData {
            content: "Test content".to_owned(),
            path: "/test/path".to_owned(),
            tags,
        };

        training_data.add_note(note);

        assert_eq!(training_data.notes.len(), 1);
        assert_eq!(training_data.all_tags.len(), 2);
        assert!(training_data.all_tags.contains("test"));
        assert!(training_data.all_tags.contains("example"));
    }

    #[test]
    fn test_training_data_filter_by_min_examples() {
        let mut training_data = TrainingData::new();

        // Add notes with different tag frequencies
        for i in 0..10 {
            let mut tags = HashSet::new();
            tags.insert("frequent".to_owned()); // Will appear in all 10 notes
            if i < 3 {
                tags.insert("rare".to_owned()); // Will appear in only 3 notes
            }
            if i < 7 {
                tags.insert("common".to_owned()); // Will appear in 7 notes
            }

            let note = NoteData {
                content: format!("Content {i}"),
                path: format!("/test/{i}.md"),
                tags,
            };
            training_data.add_note(note);
        }

        // Filter with min_examples = 5
        training_data.filter_by_min_examples(5);

        // Should keep "frequent" (10 examples) and "common" (7 examples)
        // Should remove "rare" (3 examples)
        assert!(training_data.all_tags.contains("frequent"));
        assert!(training_data.all_tags.contains("common"));
        assert!(!training_data.all_tags.contains("rare"));

        // All notes should still be present since they all have at least one valid tag
        assert_eq!(training_data.notes.len(), 10);
    }

    #[test]
    fn test_training_data_exclude_tags() {
        let mut training_data = TrainingData::new();

        let mut tags = HashSet::new();
        tags.insert("keep".to_owned());
        tags.insert("exclude_me".to_owned());
        tags.insert("also_exclude".to_owned());

        let note = NoteData {
            content: "Test content".to_owned(),
            path: "/test/path".to_owned(),
            tags,
        };
        training_data.add_note(note);

        let mut excluded = HashSet::new();
        excluded.insert("exclude_me".to_owned());
        excluded.insert("also_exclude".to_owned());

        training_data.exclude_tags(&excluded);

        assert!(training_data.all_tags.contains("keep"));
        assert!(!training_data.all_tags.contains("exclude_me"));
        assert!(!training_data.all_tags.contains("also_exclude"));
        assert_eq!(training_data.notes.len(), 1);
        assert_eq!(training_data.notes[0].tags.len(), 1);
    }

    #[test]
    fn test_extract_training_data_from_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create test notes
        create_test_note(
            temp_dir.path(),
            "note1.md",
            r#"---
tags: ["ml", "ai"]
---
Machine learning content."#,
        )?;

        create_test_note(
            temp_dir.path(),
            "note2.md",
            r#"---
tags: ["ai", "research"]
---
AI research content."#,
        )?;

        create_test_note(temp_dir.path(), "note3.md", "No frontmatter note.")?;

        // Create a non-markdown file (should be ignored)
        create_test_note(temp_dir.path(), "readme.txt", "Not a markdown file.")?;

        // Create hidden file (should be ignored)
        create_test_note(
            temp_dir.path(),
            ".hidden.md",
            r#"---
tags: ["hidden"]
---
Hidden note."#,
        )?;

        let training_data = extract_training_data(temp_dir.path())?;

        // Should find 2 notes with tags (note1.md and note2.md)
        assert_eq!(training_data.notes.len(), 2);

        // Should have 3 unique tags: ml, ai, research
        assert_eq!(training_data.all_tags.len(), 3);
        assert!(training_data.all_tags.contains("ml"));
        assert!(training_data.all_tags.contains("ai"));
        assert!(training_data.all_tags.contains("research"));

        Ok(())
    }

    #[test]
    fn test_clean_tag() {
        assert_eq!(clean_tag("#tag"), "tag");
        assert_eq!(clean_tag("  spaced tag  "), "spaced_tag");
        assert_eq!(clean_tag("UPPERCASE"), "uppercase");
        assert_eq!(clean_tag("#Multi Word Tag"), "multi_word_tag");
    }

    #[test]
    fn test_extract_frontmatter() -> Result<()> {
        let content_with_frontmatter = r#"---
title: "Test"
tags: ["test"]
---

Body content here."#;

        let (frontmatter, body) = extract_frontmatter(content_with_frontmatter)?;
        assert!(frontmatter.is_some());
        assert!(frontmatter.unwrap().contains("title: \"Test\""));
        assert_eq!(body, "\nBody content here.");

        let content_without_frontmatter = "Just body content.";
        let (frontmatter2, body2) = extract_frontmatter(content_without_frontmatter)?;
        assert!(frontmatter2.is_none());
        assert_eq!(body2, "Just body content.");

        Ok(())
    }

    #[test]
    fn test_extract_tags_from_frontmatter() -> Result<()> {
        // Test array tags
        let frontmatter_array = Some(
            r#"title: "Test"
tags: ["tag1", "tag2", "tag3"]"#
                .to_owned(),
        );
        let tags = extract_tags_from_frontmatter(&frontmatter_array)?;
        assert_eq!(tags.len(), 3);
        assert!(tags.contains("tag1"));
        assert!(tags.contains("tag2"));
        assert!(tags.contains("tag3"));

        // Test string tag
        let frontmatter_string = Some(
            r#"title: "Test"
tags: "single-tag""#
                .to_owned(),
        );
        let tags2 = extract_tags_from_frontmatter(&frontmatter_string)?;
        assert_eq!(tags2.len(), 1);
        assert!(tags2.contains("single-tag"));

        // Test no tags
        let frontmatter_no_tags = Some("title: \"Test\"".to_owned());
        let tags3 = extract_tags_from_frontmatter(&frontmatter_no_tags)?;
        assert!(tags3.is_empty());

        // Test None frontmatter
        let tags4 = extract_tags_from_frontmatter(&None)?;
        assert!(tags4.is_empty());

        Ok(())
    }
}
