// src/models/file_word_count.rs

use std::path::PathBuf;

#[derive(Debug)]
pub struct FileWordCount {
    pub path: PathBuf,
    pub words: usize,
}
