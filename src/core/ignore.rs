// src/core/ignore.rs
mod loader;
mod patterns;

pub use loader::load_ignore_patterns;
pub use patterns::Patterns;
