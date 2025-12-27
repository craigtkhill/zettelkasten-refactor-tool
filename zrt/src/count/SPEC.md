# Count Command Specification

## Feature: Count Files and Words by Tag
As a zettelkasten user
I want to count files and words filtered by tags
Possible Solutions:
- Provide Unix-style pipeable count commands with tag filters

## Requirements
Format: `[IS-TEST-IMPLEMENTED][IS-CODE-IMPLEMENTED] IDENTIFIER: example case`
- U = implemented via unit test
- A = implemented via acceptance test
- X = implemented
- O = not yet implemented

### File Counting
- [U][X] REQ-COUNT-001: Counts files containing specified tag
- [U][X] REQ-COUNT-002: Counts files containing any of multiple tags
- [U][X] REQ-COUNT-003: Counts all files when no tags specified

### Word Counting
- [U][X] REQ-COUNT-004: Counts words in files containing specified tag
- [U][X] REQ-COUNT-005: Counts words in files containing any of multiple tags
- [U][X] REQ-COUNT-006: Counts all words when no tags specified

### Percentage Calculation
- [U][X] REQ-COUNT-007: Calculates percentage of words in tagged files
- [U][X] REQ-COUNT-008: Calculates percentage of words containing any of multiple tags
- [U][X] REQ-COUNT-008a: Calculates percentage of all words when no tags specified

### Directory Scanning
- [U][X] REQ-COUNT-009: Scans multiple directories specified via -d/--dir
- [U][X] REQ-COUNT-010: Defaults to current directory when no -d specified
- [U][X] REQ-COUNT-011: Excludes directories specified via -e/--exclude

### Output Format
- [U][X] REQ-COUNT-013: Outputs single numeric value for piping
- [U][X] REQ-COUNT-014: Outputs percentage with two decimal places

### Command Flags
- [U][X] REQ-COUNT-015: Accepts --files flag for file counting
- [U][X] REQ-COUNT-016: Accepts --words flag for word counting
- [U][X] REQ-COUNT-017: Accepts --percentage flag for percentage calculation
- [U][X] REQ-COUNT-018: Requires exactly one flag (--files, --words, or --percentage)
