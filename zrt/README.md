# ZRT (Zettelkasten Refactor Tool)
ZRT is a powerful command-line tool for refactoring zettelkasten notes.

## Installation

### From Source
```bash
git clone https://github.com/yourusername/zettelkasten-refactor-tool
cd zettelkasten-refactor-tool
cargo install --path .
```

### Usage
List the top 100 files without the refactored tag ordered by wordcount
```bash
zrt -d PROJECTS/ -f refactored -w -n 100
```

List the files with only the tag "refactored"
```bash
zrt -d PROJECTS/ -o refactored
```