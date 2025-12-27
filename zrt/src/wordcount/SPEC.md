# Wordcount Command Specification

## Feature: List Files Ordered by Word Count
As a zettelkasten user
I want to see files ordered by their word count
Possible Solutions:
- Provide wordcount command that lists top N files by word/line count

## Requirements
Format: `[IS-TEST-IMPLEMENTED][IS-CODE-IMPLEMENTED] IDENTIFIER: example case`
- U = implemented via unit test
- A = implemented via acceptance test
- X = implemented
- O = not yet implemented

### File Listing
- [U][X] REQ-WC-001: Lists top N files ordered by word count
- [U][X] REQ-WC-002: Allows specifying number of files to show via -n/--num
- [U][X] REQ-WC-003: Defaults to showing 10 files

### Filtering
- [U][X] REQ-WC-004a: Filters out files containing specified tags via -f/--filter
- [U][X] REQ-WC-004b: Only show files containing specified tags via -t/--tags
- [U][X] REQ-WC-005: Shows only files exceeding configured thresholds via --exceeds

### Sorting
- [U][X] REQ-WC-006: Sorts by word count by default
- [U][X] REQ-WC-007: Allows sorting by line count via --sort-by

### Directory Scanning
- [U][X] REQ-WC-008: Scans multiple directories specified via -d/--dir
- [U][X] REQ-WC-009: Defaults to current directory when no -d specified
- [U][X] REQ-WC-010: Excludes directories specified via -e/--exclude
