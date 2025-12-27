# Frontmatter Module Specification

## Feature: YAML Frontmatter Handling
As a developer
I want to parse and manipulate YAML frontmatter in markdown files
Possible Solutions:
- Provide utilities to parse, strip, and work with frontmatter

## Requirements
Format: `[IS-TEST-IMPLEMENTED][IS-CODE-IMPLEMENTED] IDENTIFIER: example case`
- U = implemented via unit test
- A = implemented via acceptance test
- X = implemented
- O = not yet implemented

### Parse Frontmatter
- [U][X] REQ-PARSE-001: Returns default frontmatter when content is empty
- [U][X] REQ-PARSE-002: Returns default frontmatter when no delimiter present
- [U][X] REQ-PARSE-003: Parses and returns tags when valid frontmatter present

### Strip Frontmatter
- [U][X] REQ-STRIP-001: Returns body content when frontmatter is present
- [U][X] REQ-STRIP-002: Returns original content when no frontmatter present
- [U][X] REQ-STRIP-003: Returns original content when frontmatter incomplete (no closing ---)
- [U][X] REQ-STRIP-004: Returns empty string when only frontmatter present

### Frontmatter Model
- [U][X] REQ-MODEL-001: Deserializes YAML with tags array
- [U][X] REQ-MODEL-002: Handles empty frontmatter (no tags)
