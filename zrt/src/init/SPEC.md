# Init Command Specification

## Feature: Initialize zrt Configuration
As a zettelkasten user
I want to be able to customize settings of zrt
Possible Solutions:
- Initialize a zrt configuration directory

## Requirements
Format: `[IS-TEST-IMPLEMENTED][IS-CODE-IMPLEMENTED] IDENTIFIER: example case`
- U = implemented via unit test
- A = implemented via acceptance test
- X = implemented
- O = not yet implemented

### Directory Creation
- [U][X] REQ-INIT-001: Creates .zrt directory when it doesn't exist
- [U][X] REQ-INIT-002: Succeeds without error when .zrt directory already exists

### Configuration File Management
- [U][X] REQ-INIT-003: Creates config.toml file with default refactor thresholds
- [U][X] REQ-INIT-004: Does not overwrite existing config.toml file

### Testability
- [U][X] REQ-INIT-005: Accepts optional base_path parameter for test isolation
- [U][X] REQ-INIT-006: Uses current directory when base_path is None

### Configuration Structure
- [U][X] REQ-INIT-007: Defines ZrtConfig with refactor settings
- [U][X] REQ-INIT-008: Defines RefactorConfig with word and line thresholds
- [U][X] REQ-INIT-009: Defines SortBy enum with Words and Lines variants
- [U][X] REQ-INIT-010: Does not include TaggingConfig

### Configuration Defaults
- [U][X] REQ-INIT-011: Sets default word_threshold to 300
- [U][X] REQ-INIT-012: Sets default line_threshold to 60
- [U][X] REQ-INIT-015: Sets default sort_by to Words

### Configuration Serialization
- [U][X] REQ-INIT-016: Saves config to TOML file via save_to_file
- [U][X] REQ-INIT-017: Loads config from TOML file via load_from_file
- [U][X] REQ-INIT-018: Loads config from default location or returns default via load_or_default
- [U][X] REQ-INIT-019: Serializes SortBy enum as lowercase strings
