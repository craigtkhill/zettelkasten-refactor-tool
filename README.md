# ZRT (Zettelkasten Refactor Tool)

ZRT is a powerful command-line tool for analyzing and managing refactoring tasks in a Zettelkasten note system. It helps you identify files that need attention, track progress through tags, and leverage machine learning to suggest appropriate tags for your notes.

## Features

- **File Analysis**: Count and analyze markdown files with YAML frontmatter
- **Tag-based Filtering**: Find files based on tag presence or absence
- **Progress Tracking**: Compare completion states between different tags
- **Word/Line Metrics**: Identify files that exceed configurable thresholds
- **ML Tag Prediction**: Train models and get intelligent tag suggestions
- **Flexible Configuration**: Customize thresholds, exclusions, and behavior

## Installation

### From Source

```bash
git clone https://github.com/yourusername/zettelkasten-refactor-tool
cd zettelkasten-refactor-tool
cargo install --path .
```

## Quick Start

1. **Initialize ZRT in your notes directory:**
   ```bash
   zrt init
   ```
   This creates a `.zrt/` directory with default configuration.

2. **Find files needing refactoring:**
   ```bash
   zrt wordcount -f refactored -n 20
   ```
   Shows the top 20 files by word count that don't have the "refactored" tag.

3. **Check your progress:**
   ```bash
   zrt stats refactored
   ```
   Shows statistics about files with the "refactored" tag.

## CLI Commands Reference

### `zrt init`

Initialize ZRT configuration in the current directory.

```bash
zrt init
```

**What it creates:**
- `.zrt/config.toml` - Main configuration file
- `.zrt/models/` - Directory for ML models (if tagging enabled)
- `.zrt/ml_config.toml` - ML-specific configuration (if tagging enabled)

**Example output:**
```
Initialized ZRT directory at .zrt/
Created default configuration at .zrt/config.toml
  - Refactor thresholds: 300+ words, 60+ lines
  - Tagging configuration included
```

### `zrt count`

Count total files in a directory with optional exclusions.

```bash
zrt count [OPTIONS]
```

**Options:**
- `-d, --dir <DIRECTORY>` - Directory to scan (default: current directory)
- `-e, --exclude <EXCLUDE>` - Directories to exclude (comma-separated, default: `.git`)

**Examples:**
```bash
# Count all markdown files in current directory
zrt count

# Count files in specific directory, excluding multiple dirs
zrt count -d ~/notes -e ".git,drafts,archive"
```

### `zrt stats`

Show detailed statistics for files containing a specific tag.

```bash
zrt stats [OPTIONS] <TAG>
```

**Options:**
- `-d, --dir <DIRECTORY>` - Directory to scan (default: current directory)
- `-e, --exclude <EXCLUDE>` - Directories to exclude (comma-separated, default: `.git`)

**Examples:**
```bash
# Show statistics for "refactored" tag
zrt stats refactored

# Analyze "draft" tag in specific directory
zrt stats -d ~/projects/notes draft
```

**Example output:**
```
Files with tag 'refactored': 45
Words in tagged files: 23850
Total files: 127
Total words in all files: 89430
Percentage of words tagged: 26.67%
```

### `zrt wordcount` (alias: `wc`)

Show files ordered by word count with powerful filtering options.

```bash
zrt wordcount [OPTIONS]
zrt wc [OPTIONS]  # Short alias
```

**Options:**
- `-d, --dir <DIRECTORY>` - Directory to scan (default: current directory)
- `-f, --filter [<TAGS>...]` - Filter out files containing these tags (space-separated)
- `-n, --num <TOP>` - Number of files to show (default: 10)
- `-e, --exclude [<DIRS>...]` - Directories to exclude (space-separated, default: `.git`)
- `--exceeds` - Only show files exceeding configured thresholds
- `--sort-by <SORT_BY>` - Sort by `words` or `lines` (overrides config)
- `--suggest-tags` - Show ML-suggested tags for each file (requires tagging feature)

**Examples:**
```bash
# Top 10 files by word count (default)
zrt wc

# Files without "refactored" tag, top 20
zrt wc -f refactored -n 20

# Files without multiple tags
zrt wc -f refactored completed reviewed -n 15

# Only files exceeding configured thresholds
zrt wc --exceeds

# Sort by line count instead of word count
zrt wc --sort-by lines -n 5

# Include ML tag suggestions
zrt wc --suggest-tags -n 5
```

**Example output:**
```
1. notes/complex-topic.md (1247 words)
2. notes/research-findings.md (892 words)
3. notes/project-analysis.md (654 words)
4. notes/meeting-notes.md (543 words)
5. notes/ideas-collection.md (421 words)
```

### `zrt search`

Search for files containing a specific pattern or tag.

```bash
zrt search [OPTIONS] <PATTERN>
```

**Options:**
- `-d, --dir <DIRECTORY>` - Directory to scan (default: current directory)

**Examples:**
```bash
# Find files with "urgent" tag
zrt search urgent

# Search in specific directory
zrt search -d ~/work-notes priority
```

**Example output:**
```
Total files: 127
Files with pattern 'urgent': 8
Percentage: 6.30%
```

### `zrt compare`

Compare the distribution between two tags (typically "done" vs "todo").

```bash
zrt compare [OPTIONS] <DONE_TAG> <TODO_TAG>
```

**Options:**
- `-d, --dir <DIRECTORY>` - Directory to scan (default: current directory)

**Examples:**
```bash
# Compare refactored vs needs-refactor
zrt compare refactored needs-refactor

# Compare completed vs pending
zrt compare completed pending
```

**Example output:**
```
refactored files: 45
needs-refactor files: 23
Done percentage: 66.18%
```

### `zrt only`

Show files that have only a specific tag (no other tags).

```bash
zrt only [OPTIONS] <TAG>
```

**Options:**
- `-d, --dir <DIRECTORY>` - Directory to scan (default: current directory)

**Examples:**
```bash
# Files with only "draft" tag
zrt only draft

# Files with only "refactored" tag
zrt only refactored
```

**Example output:**
```
Total files: 127
Files with only tag 'draft': 12
Percentage: 9.45%
```

### `zrt tag` (ML Tag Prediction)

Machine learning-powered tag prediction commands. Requires the `tagging` feature to be enabled.

#### `zrt tag train`

Train the tag prediction model using existing tagged files.

```bash
zrt tag train [OPTIONS]
```

**Options:**
- `-d, --dir <DIRECTORY>` - Directory to scan for training data (default: current directory)
- `-e, --exclude-tags [<TAGS>...]` - Tags to exclude from training (space-separated)

**Examples:**
```bash
# Train model on all tagged files
zrt tag train

# Train excluding certain tags
zrt tag train -e draft private temporary

# Train on specific directory
zrt tag train -d ~/knowledge-base
```

#### `zrt tag suggest`

Get ML-powered tag suggestions for files.

```bash
zrt tag suggest [OPTIONS]
```

**Options:**
- `-d, --dir <DIRECTORY>` - Directory to scan (default: current directory)
- `-f, --file <FILE>` - Specific file to suggest tags for
- `-t, --threshold <THRESHOLD>` - Confidence threshold for suggestions
- `-n, --num <NUM>` - Number of top results to show (default: 10)
- `-e, --exclude-tags [<TAGS>...]` - Tags to exclude from suggestions

**Examples:**
```bash
# Suggest tags for all files in directory
zrt tag suggest

# Suggest tags for specific file
zrt tag suggest -f notes/new-article.md

# High-confidence suggestions only
zrt tag suggest -t 0.8 -n 5

# Exclude certain tags from suggestions
zrt tag suggest -e draft temporary
```

**Example output:**
```
notes/new-article.md
Existing tags: draft
Suggested new tags:
  research (confidence: 0.847)
  methodology (confidence: 0.734)
  analysis (confidence: 0.692)
```

#### `zrt tag validate`

Validate the performance of the trained model.

```bash
zrt tag validate [OPTIONS]
```

**Options:**
- `-d, --dir <DIRECTORY>` - Directory to scan for validation data (default: current directory)
- `-e, --exclude-tags [<TAGS>...]` - Tags to exclude from validation

**Examples:**
```bash
# Validate model performance
zrt tag validate

# Validate excluding certain tags
zrt tag validate -e test experimental
```

## Configuration

ZRT uses a configuration file at `.zrt/config.toml` to customize behavior.

### Default Configuration

```toml
[refactor]
word_threshold = 300      # Files with 300+ words are considered large
line_threshold = 60       # Files with 60+ lines are considered large
max_suggestions = 20      # Maximum number of suggestions to show
exclude_tags = []         # Tags to exclude from analysis
sort_by = "words"        # Sort by "words" or "lines"

[tagging]
enabled = true           # Enable ML tag prediction features
```

### Configuration Options

- **word_threshold**: Minimum word count for `--exceeds` filtering
- **line_threshold**: Minimum line count for `--exceeds` filtering
- **max_suggestions**: Default number of items to show in results
- **exclude_tags**: Tags to automatically exclude from analysis
- **sort_by**: Default sorting method ("words" or "lines")

## Common Workflows

### Finding Files to Refactor

1. **Identify largest unprocessed files:**
   ```bash
   zrt wc -f refactored -n 20
   ```

2. **Find files exceeding your thresholds:**
   ```bash
   zrt wc --exceeds -f refactored
   ```

3. **Focus on specific content types:**
   ```bash
   zrt wc -f refactored completed -e drafts archive
   ```

### Progress Tracking

1. **Check overall progress:**
   ```bash
   zrt stats refactored
   ```

2. **Compare completion states:**
   ```bash
   zrt compare refactored needs-work
   ```

3. **Find files that need initial categorization:**
   ```bash
   zrt only draft
   ```

### Using ML Tag Suggestions

1. **Train the model on your existing notes:**
   ```bash
   zrt tag train
   ```

2. **Get suggestions for new or untagged files:**
   ```bash
   zrt tag suggest -t 0.7
   ```

3. **Validate model accuracy:**
   ```bash
   zrt tag validate
   ```

## File Format Requirements

ZRT works with markdown files containing YAML frontmatter:

```markdown
---
tags:
  - research
  - methodology
  - draft
title: "My Research Notes"
---

# Content here

Your note content goes here...
```