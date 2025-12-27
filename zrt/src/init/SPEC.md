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
