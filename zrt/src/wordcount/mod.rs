pub mod cli;
pub mod models;
pub mod print;
pub mod word;

pub use print::{print_file_metrics, print_top_files};
pub use word::{count_file_metrics, count_words};
