use anyhow::{Context as _, Result};
use serde_yaml_ng::Value;
use std::collections::HashSet;
use std::path::Path;

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
