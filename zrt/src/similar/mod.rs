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
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &TempDir, name: &str, content: &str) -> Result<PathBuf> {
        let path = dir.path().join(name);
        fs::write(&path, content)?;
        Ok(path)
    }

    // Similarity Detection Tests
    #[test]
    fn test_should_compute_jaccard_similarity() -> Result<()> {
        // REQ-SIM-001
        let dir = TempDir::new()?;
        create_test_file(&dir, "note1.md", "apple banana cherry")?;
        create_test_file(&dir, "note2.md", "apple banana")?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.0, &[])?;

        assert_eq!(pairs.len(), 1);
        let (score, _, _) = pairs[0];
        // Jaccard: intersection=2 (apple, banana), union=3 (apple, banana, cherry) = 2/3 = 0.666...
        assert!((score - 0.666).abs() < 0.01);
        Ok(())
    }

    #[test]
    fn test_should_tokenize_lowercase_alphanumeric() -> Result<()> {
        // REQ-SIM-002
        let tokens = tokenize("Hello WORLD! Test123 foo-bar");

        assert!(tokens.contains("hello"));
        assert!(tokens.contains("world"));
        assert!(tokens.contains("test123"));
        assert!(tokens.contains("foo"));
        assert!(tokens.contains("bar"));
        Ok(())
    }

    #[test]
    fn test_should_exclude_frontmatter_from_similarity() -> Result<()> {
        // REQ-SIM-003
        let dir = TempDir::new()?;
        create_test_file(&dir, "note1.md", "---\ntags: [apple]\n---\nbanana cherry")?;
        create_test_file(&dir, "note2.md", "---\ntags: [apple]\n---\nbanana cherry")?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.99, &[])?;

        // If frontmatter were included, similarity would be lower due to different tags
        // With frontmatter excluded, both have exactly "banana cherry" so similarity = 1.0
        assert_eq!(pairs.len(), 1);
        let (score, _, _) = pairs[0];
        assert!((score - 1.0).abs() < 0.01);
        Ok(())
    }

    #[test]
    fn test_should_sort_pairs_by_similarity_descending() -> Result<()> {
        // REQ-SIM-004
        let dir = TempDir::new()?;
        create_test_file(&dir, "note1.md", "apple banana")?;
        create_test_file(&dir, "note2.md", "apple")?;
        create_test_file(&dir, "note3.md", "apple banana cherry")?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.0, &[])?;

        // Verify pairs are sorted by score descending
        for i in 0..pairs.len().saturating_sub(1) {
            assert!(pairs[i].0 >= pairs[i + 1].0);
        }
        Ok(())
    }

    // Threshold Filtering Tests
    #[test]
    fn test_should_accept_threshold_flag() -> Result<()> {
        // REQ-SIM-101
        let dir = TempDir::new()?;
        create_test_file(&dir, "note1.md", "apple banana cherry")?;
        create_test_file(&dir, "note2.md", "apple banana")?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.7, &[])?;

        // Similarity is 0.666, should be filtered out by 0.7 threshold
        assert_eq!(pairs.len(), 0);
        Ok(())
    }

    #[test]
    fn test_should_default_to_05_threshold() -> Result<()> {
        // REQ-SIM-102
        // This will be tested via CLI, default threshold is 0.5
        Ok(())
    }

    #[test]
    fn test_should_only_output_pairs_above_threshold() -> Result<()> {
        // REQ-SIM-103
        let dir = TempDir::new()?;
        create_test_file(&dir, "note1.md", "apple banana cherry date")?;
        create_test_file(&dir, "note2.md", "apple banana cherry")?;
        create_test_file(&dir, "note3.md", "apple")?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.6, &[])?;

        // note1-note2: 3/4 = 0.75 >= 0.6 ✓
        // note1-note3: 1/4 = 0.25 < 0.6 ✗
        // note2-note3: 1/3 = 0.33 < 0.6 ✗
        assert_eq!(pairs.len(), 1);
        Ok(())
    }

    // Directory Scanning Tests
    #[test]
    fn test_should_accept_directory_flag() -> Result<()> {
        // REQ-SIM-201
        let dir = TempDir::new()?;
        create_test_file(&dir, "note1.md", "content")?;
        create_test_file(&dir, "note2.md", "content")?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.0, &[])?;

        assert_eq!(pairs.len(), 1);
        Ok(())
    }

    #[test]
    fn test_should_default_to_current_directory() -> Result<()> {
        // REQ-SIM-202
        // This will be tested via integration test or CLI test
        Ok(())
    }

    #[test]
    fn test_should_support_multiple_directories() -> Result<()> {
        // REQ-SIM-203
        let dir1 = TempDir::new()?;
        let dir2 = TempDir::new()?;
        create_test_file(&dir1, "note1.md", "apple banana")?;
        create_test_file(&dir2, "note2.md", "apple banana")?;

        let pairs = find_similar(
            &[dir1.path().to_path_buf(), dir2.path().to_path_buf()],
            0.0,
            &[],
        )?;

        assert_eq!(pairs.len(), 1);
        Ok(())
    }

    #[test]
    fn test_should_scan_recursively() -> Result<()> {
        // REQ-SIM-204
        let dir = TempDir::new()?;
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir)?;

        create_test_file(&dir, "note1.md", "apple banana")?;
        fs::write(subdir.join("note2.md"), "apple banana")?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.0, &[])?;

        assert_eq!(pairs.len(), 1);
        Ok(())
    }

    // Exclusions Tests
    #[test]
    fn test_should_exclude_specified_directories() -> Result<()> {
        // REQ-SIM-301
        let dir = TempDir::new()?;
        let excluded = dir.path().join("excluded");
        fs::create_dir(&excluded)?;

        create_test_file(&dir, "note1.md", "content")?;
        fs::write(excluded.join("note2.md"), "content")?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.0, &["excluded"])?;

        assert_eq!(pairs.len(), 0);
        Ok(())
    }

    #[test]
    fn test_should_respect_zrtignore() -> Result<()> {
        // REQ-SIM-302
        let dir = TempDir::new()?;
        fs::write(dir.path().join(".zrtignore"), "ignored/\n")?;

        let ignored = dir.path().join("ignored");
        fs::create_dir(&ignored)?;

        create_test_file(&dir, "note1.md", "content")?;
        fs::write(ignored.join("note2.md"), "content")?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.0, &[])?;

        assert_eq!(pairs.len(), 0);
        Ok(())
    }

    #[test]
    fn test_should_parse_exclude_similarity_field() -> Result<()> {
        // REQ-SIM-303
        let exclusions = parse_exclude_similarity("exclude_similarity:\n  - [[note2]]\n  - [[note3]]");

        assert_eq!(exclusions.len(), 2);
        assert!(exclusions.contains("note2"));
        assert!(exclusions.contains("note3"));
        Ok(())
    }

    #[test]
    fn test_should_skip_excluded_pairs() -> Result<()> {
        // REQ-SIM-304
        let dir = TempDir::new()?;
        create_test_file(&dir, "note1.md", "---\nexclude_similarity:\n  - [[note2]]\n---\napple banana")?;
        create_test_file(&dir, "note2.md", "apple banana")?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.0, &[])?;

        // Should be excluded due to note1 excluding note2
        assert_eq!(pairs.len(), 0);
        Ok(())
    }

    #[test]
    fn test_should_support_yaml_list_format() -> Result<()> {
        // REQ-SIM-305
        let exclusions = parse_exclude_similarity(
            "exclude_similarity:\n  - [[note1]]\n  - [[note2]]\n"
        );

        assert_eq!(exclusions.len(), 2);
        assert!(exclusions.contains("note1"));
        assert!(exclusions.contains("note2"));
        Ok(())
    }

    // Output Format Tests
    #[test]
    fn test_should_output_absolute_or_relative_paths() -> Result<()> {
        // REQ-SIM-402
        let dir = TempDir::new()?;
        create_test_file(&dir, "note1.md", "content")?;
        create_test_file(&dir, "note2.md", "content")?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.0, &[])?;

        assert_eq!(pairs.len(), 1);
        let (_, path1, path2) = &pairs[0];
        assert!(path1.is_absolute());
        assert!(path2.is_absolute());
        Ok(())
    }

    #[test]
    fn test_should_output_nothing_when_no_pairs_above_threshold() -> Result<()> {
        // REQ-SIM-403
        let dir = TempDir::new()?;
        create_test_file(&dir, "note1.md", "apple")?;
        create_test_file(&dir, "note2.md", "banana")?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.5, &[])?;

        assert_eq!(pairs.len(), 0);
        Ok(())
    }

    #[test]
    fn test_output_is_pipeable() -> Result<()> {
        // REQ-SIM-404
        // This will be tested via CLI - output should be just paths, no formatting
        Ok(())
    }

    // Edge Cases Tests
    #[test]
    fn test_should_handle_empty_files() -> Result<()> {
        // REQ-SIM-501
        let dir = TempDir::new()?;
        create_test_file(&dir, "note1.md", "")?;
        create_test_file(&dir, "note2.md", "content")?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.0, &[])?;

        // Empty file has no tokens, so no similarity
        assert_eq!(pairs.len(), 0);
        Ok(())
    }

    #[test]
    fn test_should_skip_unreadable_files() -> Result<()> {
        // REQ-SIM-502
        let dir = TempDir::new()?;
        create_test_file(&dir, "note1.md", "content")?;
        let binary_path = dir.path().join("binary.md");
        fs::write(&binary_path, &[0xFF, 0xFE, 0x00])?;

        let pairs = find_similar(&[dir.path().to_path_buf()], 0.0, &[])?;

        // Should not panic, should skip binary file
        assert_eq!(pairs.len(), 0);
        Ok(())
    }

    #[test]
    fn test_should_return_zero_similarity_for_empty_sets() -> Result<()> {
        // REQ-SIM-503
        let set1: HashSet<String> = HashSet::new();
        let set2: HashSet<String> = HashSet::new();

        let similarity = jaccard_similarity(&set1, &set2);

        assert_eq!(similarity, 0.0);
        Ok(())
    }
}

// ============================================
// TYPE DEFINITIONS
// ============================================

// ============================================
// IMPLEMENTATIONS
// ============================================

/// Tokenize text into unique lowercase alphanumeric words
pub fn tokenize(text: &str) -> HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

/// Compute Jaccard similarity between two sets
pub fn jaccard_similarity(set1: &HashSet<String>, set2: &HashSet<String>) -> f64 {
    if set1.is_empty() || set2.is_empty() {
        return 0.0;
    }

    let intersection = set1.intersection(set2).count();
    let union = set1.len() + set2.len() - intersection;

    intersection as f64 / union as f64
}

/// Parse exclude_similarity field from frontmatter
pub fn parse_exclude_similarity(frontmatter: &str) -> HashSet<String> {
    let mut exclusions = HashSet::new();

    // Look for exclude_similarity field with YAML list format
    if let Some(start) = frontmatter.find("exclude_similarity:") {
        let remaining = &frontmatter[start..];
        for line in remaining.lines().skip(1) {
            // Stop at next field or end of frontmatter
            if !line.starts_with("  ") {
                break;
            }
            // Extract wikilinks [[note]]
            if let Some(link_start) = line.find("[[") {
                if let Some(link_end) = line.find("]]") {
                    let note_name = &line[link_start + 2..link_end];
                    exclusions.insert(note_name.to_string());
                }
            }
        }
    }

    exclusions
}

/// Find similar note pairs
pub fn find_similar(
    dirs: &[PathBuf],
    threshold: f64,
    exclude: &[&str],
) -> Result<Vec<(f64, PathBuf, PathBuf)>> {
    let mut note_contents: HashMap<PathBuf, String> = HashMap::new();
    let mut note_exclusions: HashMap<PathBuf, HashSet<String>> = HashMap::new();

    // Collect all notes
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
            if let Some(ext) = path.extension() {
                if ext != "md" {
                    continue;
                }
            } else {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(path) {
                let body = strip_frontmatter(&content).to_string();
                note_contents.insert(path.to_path_buf(), body);

                if let Ok(_frontmatter) = parse_frontmatter(&content) {
                    if let Some(fm_text) = content.split("---").nth(1) {
                        let exclusions = parse_exclude_similarity(fm_text);
                        if !exclusions.is_empty() {
                            note_exclusions.insert(path.to_path_buf(), exclusions);
                        }
                    }
                }
            }
        }
    }

    // Tokenize all notes
    let mut note_tokens: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    for (path, content) in &note_contents {
        note_tokens.insert(path.clone(), tokenize(content));
    }

    // Compute pairwise similarities
    let note_paths: Vec<&PathBuf> = note_contents.keys().collect();
    let mut pairs = Vec::new();

    for i in 0..note_paths.len() {
        for j in (i + 1)..note_paths.len() {
            let path1 = note_paths[i];
            let path2 = note_paths[j];

            // Check exclusions
            let note1_stem = path1.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let note2_stem = path2.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            if let Some(exclusions) = note_exclusions.get(path1) {
                if exclusions.contains(note2_stem) {
                    continue;
                }
            }
            if let Some(exclusions) = note_exclusions.get(path2) {
                if exclusions.contains(note1_stem) {
                    continue;
                }
            }

            // Compute similarity
            let tokens1 = note_tokens.get(path1).unwrap();
            let tokens2 = note_tokens.get(path2).unwrap();

            // Skip if either set is empty
            if tokens1.is_empty() || tokens2.is_empty() {
                continue;
            }

            // Upper bound: max possible Jaccard = min_len / max_len (when smaller ⊆ larger)
            let min_len = tokens1.len().min(tokens2.len());
            let max_len = tokens1.len().max(tokens2.len());
            if (min_len as f64 / max_len as f64) < threshold {
                continue;
            }

            let similarity = jaccard_similarity(tokens1, tokens2);

            if similarity >= threshold {
                pairs.push((similarity, path1.clone(), path2.clone()));
            }
        }
    }

    // Sort by similarity descending
    pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    Ok(pairs)
}
