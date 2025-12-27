# Search Command Specification

## Feature: Search Files by Tag Criteria
As a zettelkasten user
I want to search for files based on tag filtering criteria
Possible Solutions:
- Provide search command with --exactly flag for precise tag matching

## Requirements
Format: `[IS-TEST-IMPLEMENTED][IS-CODE-IMPLEMENTED] IDENTIFIER: example case`
- U = implemented via unit test
- A = implemented via acceptance test
- X = implemented
- O = not yet implemented

### Exact Tag Matching
- [U][X] REQ-SEARCH-001: Finds files with exactly one specified tag
- [U][X] REQ-SEARCH-002: Finds files with exactly multiple specified tags
- [U][X] REQ-SEARCH-003: Excludes files that have additional tags beyond specified ones
- [U][X] REQ-SEARCH-004: Excludes files that are missing any of the specified tags

### Directory Scanning
- [U][X] REQ-SEARCH-005: Scans multiple directories specified via -d/--dir
- [U][X] REQ-SEARCH-006: Defaults to current directory when no -d specified
- [U][X] REQ-SEARCH-007: Excludes directories specified via -e/--exclude

### Output Format
- [U][X] REQ-SEARCH-009: Displays files matching criteria

### Command Flags
- [U][X] REQ-SEARCH-011: Accepts --exactly flag with space-separated tags
- [U][X] REQ-SEARCH-012: Requires at least one tag when using --exactly
