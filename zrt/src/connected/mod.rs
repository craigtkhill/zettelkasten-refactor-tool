pub mod cli;

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::core::filter::utils::should_exclude;
use crate::core::frontmatter::{parse_frontmatter, strip_frontmatter};
use crate::core::ignore::load_ignore_patterns;

// ============================================
// TESTS
// ============================================
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &TempDir, name: &str, content: &str) -> Result<PathBuf> {
        let path = dir.path().join(name);
        fs::write(&path, content)?;
        Ok(path)
    }

    #[test]
    fn test_should_only_include_notes_with_tag() -> Result<()> {
        // REQ-CONN-003

        // Given
        let dir = TempDir::new()?;
        create_test_file(&dir, "a.md", "---\ntags: [writing]\n---\nContent")?;
        create_test_file(&dir, "b.md", "---\ntags: [ideas]\n---\nContent")?;

        // When
        let results = most_connected(&[dir.path().to_path_buf()], "writing", &[])?;

        // Then
        assert_eq!(results.len(), 1);
        assert!(results[0].0.ends_with("a.md"));
        Ok(())
    }

    #[test]
    fn test_should_count_outgoing_links_to_tagged_notes() -> Result<()> {
        // REQ-CONN-005

        // Given
        let dir = TempDir::new()?;
        create_test_file(&dir, "a.md", "---\ntags: [writing]\n---\n[[b]]")?;
        create_test_file(&dir, "b.md", "---\ntags: [writing]\n---\nContent")?;

        // When
        let results = most_connected(&[dir.path().to_path_buf()], "writing", &[])?;

        // Then
        let a_score = results.iter().find(|(p, _)| p.ends_with("a.md")).map(|(_, c)| *c);
        assert_eq!(a_score, Some(1));
        Ok(())
    }

    #[test]
    fn test_should_count_incoming_links_from_tagged_notes() -> Result<()> {
        // REQ-CONN-006

        // Given
        let dir = TempDir::new()?;
        create_test_file(&dir, "a.md", "---\ntags: [writing]\n---\n[[b]]")?;
        create_test_file(&dir, "b.md", "---\ntags: [writing]\n---\nContent")?;

        // When
        let results = most_connected(&[dir.path().to_path_buf()], "writing", &[])?;

        // Then
        let b_score = results.iter().find(|(p, _)| p.ends_with("b.md")).map(|(_, c)| *c);
        assert_eq!(b_score, Some(1));
        Ok(())
    }

    #[test]
    fn test_should_not_count_links_to_untagged_notes() -> Result<()> {
        // REQ-CONN-004

        // Given
        let dir = TempDir::new()?;
        create_test_file(&dir, "a.md", "---\ntags: [writing]\n---\n[[c]]")?;
        create_test_file(&dir, "b.md", "---\ntags: [writing]\n---\nContent")?;
        create_test_file(&dir, "c.md", "---\ntags: [ideas]\n---\nContent")?;

        // When
        let results = most_connected(&[dir.path().to_path_buf()], "writing", &[])?;

        // Then
        let a_score = results.iter().find(|(p, _)| p.ends_with("a.md")).map(|(_, c)| *c);
        assert_eq!(a_score, Some(0));
        Ok(())
    }

    #[test]
    fn test_should_sort_by_total_connections_descending() -> Result<()> {
        // REQ-CONN-007

        // Given: a links to b and c (score=2), b and c each have 1 incoming (score=1)
        let dir = TempDir::new()?;
        create_test_file(&dir, "a.md", "---\ntags: [writing]\n---\n[[b]] [[c]]")?;
        create_test_file(&dir, "b.md", "---\ntags: [writing]\n---\nContent")?;
        create_test_file(&dir, "c.md", "---\ntags: [writing]\n---\nContent")?;

        // When
        let results = most_connected(&[dir.path().to_path_buf()], "writing", &[])?;

        // Then
        assert!(results[0].0.ends_with("a.md"));
        Ok(())
    }

    #[test]
    fn test_should_scan_multiple_directories() -> Result<()> {
        // REQ-CONN-010

        // Given
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;
        create_test_file(&dir1, "a.md", "---\ntags: [writing]\n---\nContent")?;
        create_test_file(&dir2, "b.md", "---\ntags: [writing]\n---\nContent")?;

        // When
        let results = most_connected(
            &[dir1.path().to_path_buf(), dir2.path().to_path_buf()],
            "writing",
            &[],
        )?;

        // Then
        assert_eq!(results.len(), 2);
        Ok(())
    }

    #[test]
    fn test_should_exclude_directories() -> Result<()> {
        // REQ-CONN-011

        // Given
        let dir = TempDir::new()?;
        let excluded = dir.path().join("excluded");
        fs::create_dir(&excluded)?;
        create_test_file(&dir, "a.md", "---\ntags: [writing]\n---\nContent")?;
        fs::write(excluded.join("b.md"), "---\ntags: [writing]\n---\nContent")?;

        // When
        let results = most_connected(&[dir.path().to_path_buf()], "writing", &["excluded"])?;

        // Then
        assert_eq!(results.len(), 1);
        Ok(())
    }
}

// ============================================
// IMPLEMENTATIONS
// ============================================

/// Extract wikilink targets from note body text.
/// Handles [[link]] and [[link|alias]] formats, stripping directory prefixes.
fn extract_wikilinks(body: &str) -> HashSet<String> {
    let mut links = HashSet::new();
    let mut remaining = body;

    while let Some(start) = remaining.find("[[") {
        remaining = &remaining[start + 2..];
        if let Some(end) = remaining.find("]]") {
            let raw = &remaining[..end];
            // Strip alias: [[link|alias]] → link
            let target = raw.split('|').next().unwrap_or(raw).trim();
            // Strip directory prefix: [[dir/note]] → note
            let stem = target.split('/').next_back().unwrap_or(target);
            if !stem.is_empty() {
                links.insert(stem.to_string());
            }
            remaining = &remaining[end + 2..];
        } else {
            break;
        }
    }

    links
}

/// Find the most connected notes for a given tag.
/// Returns (file_path, total_connection_score) sorted by score descending.
/// Only connections between notes that both have the tag are counted.
pub fn most_connected(
    dirs: &[PathBuf],
    tag: &str,
    exclude: &[&str],
) -> Result<Vec<(String, usize)>> {
    // Collect all notes: stem → (path_string, has_tag, body)
    let mut notes: Vec<(String, String, bool, String)> = Vec::new(); // (stem, path, has_tag, body)

    for dir in dirs {
        let absolute_dir = if dir.is_absolute() {
            dir.clone()
        } else {
            std::env::current_dir()?.join(dir)
        };

        let ignore_patterns = load_ignore_patterns(&absolute_dir)?;

        for entry in WalkDir::new(&absolute_dir)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !should_exclude(e, exclude, Some(&ignore_patterns)))
        {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            if let Ok(content) = std::fs::read_to_string(path) {
                let has_tag = parse_frontmatter(&content)
                    .ok()
                    .and_then(|fm| fm.tags)
                    .map_or(false, |tags| tags.iter().any(|t| t == tag));
                let body = strip_frontmatter(&content).to_string();
                notes.push((stem, path.display().to_string(), has_tag, body));
            }
        }
    }

    // Set of tagged note stems for fast lookup
    let tagged_stems: HashSet<&str> = notes
        .iter()
        .filter(|(_, _, has_tag, _)| *has_tag)
        .map(|(stem, _, _, _)| stem.as_str())
        .collect();

    // Build outgoing link map: stem → set of stems it links to
    let mut outgoing: HashMap<&str, HashSet<String>> = HashMap::new();
    for (stem, _, _, body) in &notes {
        let links = extract_wikilinks(body);
        outgoing.insert(stem.as_str(), links);
    }

    // Score each tagged note
    let mut scores: Vec<(String, usize)> = notes
        .iter()
        .filter(|(_, _, has_tag, _)| *has_tag)
        .map(|(stem, path, _, _)| {
            let out_count = outgoing
                .get(stem.as_str())
                .map_or(0, |links| links.iter().filter(|l| tagged_stems.contains(l.as_str()) && l.as_str() != stem.as_str()).count());

            let in_count = notes
                .iter()
                .filter(|(other_stem, _, _, _)| other_stem != stem && tagged_stems.contains(other_stem.as_str()))
                .filter(|(other_stem, _, _, _)| {
                    outgoing
                        .get(other_stem.as_str())
                        .map_or(false, |links| links.contains(stem.as_str()))
                })
                .count();

            (path.clone(), out_count + in_count)
        })
        .collect();

    scores.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    Ok(scores)
}
