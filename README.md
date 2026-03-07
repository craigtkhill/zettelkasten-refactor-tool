# Zettelkasten Refactor Tool

zrt is a command-line tool for analyzing and managing refactoring tasks in a Zettelkasten note system. It helps you identify files that need attention and track progress through tags.

## Features

- **File Analysis**: Count files and words with tag-based filtering
- **Tag-based Search**: Find files with exact tag matches
- **Similarity Detection**: Find similar notes for consolidation
- **Word/Line Metrics**: Identify files exceeding thresholds
- **Flexible Configuration**: Customize thresholds and sorting

## Installation

### From Source

```bash
git clone https://github.com/yourusername/zettelkasten-refactor-tool
cd zettelkasten-refactor-tool
cargo install --path zrt
```

## Quick Start

1. **Initialize zrt in your notes directory:**
   ```bash
   zrt init
   ```

2. **List files by word count:**
   ```bash
   zrt wc -n 20
   ```

3. **Find similar notes:**
   ```bash
   zrt similar --threshold 0.6
   ```

## Commands

### `zrt init` (alias: `i`)

Initialize zrt configuration in the current directory.

```bash
zrt init
```

Creates `.zrt/config.toml` with default refactor thresholds (300 words, 60 lines).

### `zrt count` (alias: `c`)

Count files, words, or calculate percentages by tags.

```bash
zrt count [OPTIONS] [TAGS...]
```

**Flags (exactly one required):**
- `--files` - Count files matching tags
- `--words` - Count words in files matching tags
- `--percentage` - Calculate percentage of words in tagged files

**Options:**
- `-d, --dir <DIRECTORY>` - Directories to scan (space-separated, default: current)
- `-e, --exclude <DIRS>` - Directories to exclude (space-separated)
- `[TAGS...]` - Tags to filter by (omit to count all)

**Examples:**
```bash
# Count all files
zrt count --files

# Count files with "refactored" tag
zrt count --files refactored

# Count words in files with "draft" or "wip" tags
zrt count --words draft wip

# Calculate percentage of words in refactored files
zrt count --percentage refactored

# Scan multiple directories
zrt count --files -d ~/notes ~/work refactored
```

**Output:** Single number (pipeable)

### `zrt wordcount` (alias: `wc`)

Show files ordered by word or line count.

```bash
zrt wordcount [OPTIONS]
```

**Options:**
- `-d, --dir <DIRECTORY>` - Directories to scan (space-separated, default: current)
- `-f, --filter <TAGS>` - Filter out files with these tags (space-separated)
- `-n, --num <TOP>` - Number of files to show (default: 10)
- `-e, --exclude <DIRS>` - Directories to exclude (space-separated, default: `.git`)
- `--exceeds` - Only show files exceeding configured thresholds
- `--sort-by <SORT>` - Sort by `words` or `lines` (overrides config)

**Examples:**
```bash
# Top 10 files by word count
zrt wc

# Files without "refactored" tag, top 20
zrt wc -f refactored -n 20

# Only files exceeding thresholds
zrt wc --exceeds

# Sort by line count
zrt wc --sort-by lines
```

**Output:** File paths, one per line (pipeable)

### `zrt search` (alias: `s`)

Search for files with exact tag matches.

```bash
zrt search [OPTIONS] (--tags <TAGS...> | --no-tags)
```

**Options:**
- `-d, --dir <DIRECTORY>` - Directories to scan (space-separated, default: current)
- `-e, --exclude <DIRS>` - Directories to exclude (space-separated)
- `--tags <TAGS>` - Find files with exactly these tags (no more, no less)
- `--no-tags` - Find files that have no tags at all

**Examples:**
```bash
# Files with only "refactored" tag
zrt search --tags refactored

# Files with exactly "draft" and "review" tags
zrt search --tags draft review

# Search in specific directory
zrt search -d ~/notes --tags refactored

# Files missing tags entirely
zrt search --no-tags -d thoughts/ blog/
```

**Output:** File paths, one per line (pipeable)

### `zrt similar` (alias: `sim`)

Find similar notes for refactoring and consolidation.

```bash
zrt similar [OPTIONS]
```

**Options:**
- `-d, --dir <DIRECTORY>` - Directories to scan (space-separated, default: current)
- `-e, --exclude <DIRS>` - Directories to exclude (space-separated)
- `--threshold <THRESHOLD>` - Similarity threshold 0.0-1.0 (default: 0.5)

**Examples:**
```bash
# Find similar notes with default threshold
zrt similar

# Higher threshold for stricter matching
zrt similar --threshold 0.7

# Scan multiple directories
zrt similar -d ~/notes ~/work --threshold 0.6
```

**Output:** Pairs of file paths (space-separated), one pair per line, sorted by similarity (pipeable)

**Frontmatter Exclusions:**

Exclude specific pairs from results using `exclude_similarity` field:

```markdown
---
tags: [research]
exclude_similarity:
  - [[duplicate-note]]
  - [[old-version]]
---
```

## Configuration

zrt uses `.zrt/config.toml` to customize behavior.

### Default Configuration

```toml
[refactor]
word_threshold = 300      # Files with 300+ words are large
line_threshold = 60       # Files with 60+ lines are large
sort_by = "words"        # Sort by "words" or "lines"
```

### Configuration Options

- **word_threshold**: Minimum word count for `--exceeds` filtering
- **line_threshold**: Minimum line count for `--exceeds` filtering
- **sort_by**: Default sorting method ("words" or "lines")

## Ignore Patterns

Create a `.zrtignore` file to exclude directories (gitignore-style):

```
.git/
node_modules/
archive/
```

## File Format

zrt works with markdown files containing YAML frontmatter:

```markdown
---
tags:
  - research
  - methodology
  - draft
exclude_similarity:
  - [[old-version]]
---

# Content here

Your note content...
```

Both inline `tags: [tag1, tag2]` and list format are supported.
