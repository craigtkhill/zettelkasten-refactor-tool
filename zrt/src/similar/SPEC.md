# Similar Notes Detection Specification

## Feature: Find Similar Notes for Refactoring
As a zettelkasten user
I want to find notes with similar content so that I can combine notes that should be together
Possible Solutions:
- Use Jaccard similarity to compare word overlap between notes, with configurable threshold filtering

## Requirements
Format: `[IS-TEST-IMPLEMENTED][IS-CODE-IMPLEMENTED] IDENTIFIER: example case`
- U = implemented via unit test
- A = implemented via acceptance test
- X = implemented
- O = not yet implemented

### Similarity Detection
- [U][X] REQ-SIM-001: Computes Jaccard similarity between all note pairs
- [U][X] REQ-SIM-002: Tokenizes note content into unique words (lowercase, alphanumeric only)
- [U][X] REQ-SIM-003: Excludes frontmatter from similarity computation
- [U][X] REQ-SIM-004: Returns pairs sorted by similarity score descending

### Threshold Filtering
- [U][X] REQ-SIM-101: Accepts --threshold flag with float value
- [U][X] REQ-SIM-102: Defaults to 0.5 threshold when not specified
- [U][X] REQ-SIM-103: Only outputs pairs with similarity >= threshold

### Directory Scanning
- [U][X] REQ-SIM-201: Accepts -d/--dir flag for directory paths
- [U][X] REQ-SIM-202: Defaults to current directory when no -d flag provided
- [U][X] REQ-SIM-203: Supports multiple directories via repeated -d flags
- [U][X] REQ-SIM-204: Scans directories recursively for markdown files

### Exclusions
- [U][X] REQ-SIM-301: Accepts -e/--exclude flag for directory exclusions
- [U][X] REQ-SIM-302: Respects .zrtignore patterns
- [U][X] REQ-SIM-303: Parses exclude_similarity field from frontmatter
- [U][X] REQ-SIM-304: Skips pairs when one note excludes the other via wikilink
- [U][X] REQ-SIM-305: Supports YAML list format for exclude_similarity field

### Output Format
- [U][X] REQ-SIM-402: Outputs absolute paths or paths relative to scan directory
- [U][X] REQ-SIM-403: Produces no output when no pairs exceed threshold
- [U][X] REQ-SIM-404: Output is pipeable (no formatting, headers, or labels)

### Edge Cases
- [U][X] REQ-SIM-501: Handles empty files gracefully
- [U][X] REQ-SIM-502: Skips files without readable content
- [U][X] REQ-SIM-503: Returns 0.0 similarity for empty token sets
