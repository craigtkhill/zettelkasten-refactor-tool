# Multi-Directory Support Specification

## Feature: Scan Multiple Directories for Stats
As a zettelkasten user
I want to run stats command across multiple directories
So that I can see combined statistics from different note locations like thoughts/ and blog/

## Requirements
Format: `[IS-TEST-IMPLEMENTED][IS-CODE-IMPLEMENTED] IDENTIFIER: example case`
- U = implemented via unit test
- A = implemented via acceptance test
- X = implemented
- O = not yet implemented

### CLI Arguments
- [U][X] REQ-STATS-MULTI-001: User can specify multiple directories with `-d` flag
- [U][X] REQ-STATS-MULTI-002: Multiple directories are space-separated
- [U][X] REQ-STATS-MULTI-003: When no `-d` flag provided, defaults to current directory

### Statistics Output
- [U][X] REQ-STATS-MULTI-101: Total file count includes files from all specified directories
- [U][X] REQ-STATS-MULTI-102: Total word count includes words from all specified directories
- [U][X] REQ-STATS-MULTI-103: Tagged file count includes tagged files from all specified directories
- [U][X] REQ-STATS-MULTI-104: Tagged word count includes tagged words from all specified directories
- [U][X] REQ-STATS-MULTI-105: Percentage calculation uses aggregated totals

### Directory Processing
- [U][X] REQ-STATS-MULTI-201: Each directory is scanned for markdown files
- [U][X] REQ-STATS-MULTI-202: Results from all directories are combined before display
- [U][X] REQ-STATS-MULTI-203: Exclude patterns apply to all specified directories

## Feature: Scan Multiple Directories for Wordcount
As a zettelkasten user
I want to run wordcount command across multiple directories
So that I can find the largest files across different note locations like thoughts/ and blog/

### CLI Arguments
- [U][X] REQ-WC-MULTI-001: User can specify multiple directories with `-d` flag
- [U][X] REQ-WC-MULTI-002: Multiple directories are space-separated
- [U][X] REQ-WC-MULTI-003: When no `-d` flag provided, defaults to current directory

### Wordcount Output
- [U][X] REQ-WC-MULTI-101: Results include files from all specified directories
- [U][X] REQ-WC-MULTI-102: Files are sorted by word count across all directories
- [U][X] REQ-WC-MULTI-103: Top N files are selected from combined results

### Directory Processing
- [U][X] REQ-WC-MULTI-201: Each directory is scanned for markdown files
- [U][X] REQ-WC-MULTI-202: Filter tags apply to all specified directories
- [U][X] REQ-WC-MULTI-203: Exclude patterns apply to all specified directories
