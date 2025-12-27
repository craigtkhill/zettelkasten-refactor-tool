# Strip Frontmatter Utility Specification

## Feature: Strip YAML Frontmatter from Content
As a developer
I want to remove YAML frontmatter from file content
Possible Solutions:
- Provide utility function that strips frontmatter for accurate word/line counting

## Requirements
Format: `[IS-TEST-IMPLEMENTED][IS-CODE-IMPLEMENTED] IDENTIFIER: example case`
- U = implemented via unit test
- A = implemented via acceptance test
- X = implemented
- O = not yet implemented

### Frontmatter Detection
- [U][X] REQ-STRIP-001: Returns body content when frontmatter is present
- [U][X] REQ-STRIP-002: Returns original content when no frontmatter present
- [U][X] REQ-STRIP-003: Returns original content when frontmatter incomplete (no closing ---)
- [U][X] REQ-STRIP-004: Returns empty string when only frontmatter present
