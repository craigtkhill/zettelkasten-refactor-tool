// src/wordcount/mod.rs
pub mod cli;
pub mod word;

pub use word::{count_file_metrics, count_words};
