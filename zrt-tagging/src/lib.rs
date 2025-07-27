#![allow(
    clippy::multiple_crate_versions,
    reason = "ML dependencies have complex version requirements"
)]
#![allow(
    clippy::todo,
    reason = "Development phase - todos will be removed before release"
)]
// Development phase allows - will be removed before release
#![allow(clippy::absolute_paths, reason = "Development: std:: paths are clear")]
#![allow(
    clippy::arithmetic_side_effects,
    reason = "Development: basic arithmetic is safe"
)]
#![allow(clippy::indexing_slicing, reason = "Development: controlled slicing")]
#![allow(
    clippy::iter_over_hash_type,
    reason = "Development: HashSet iteration acceptable"
)]
#![allow(
    clippy::wildcard_enum_match_arm,
    reason = "Development: exhaustive matching not needed"
)]
#![allow(
    clippy::missing_errors_doc,
    reason = "Development: docs will be completed"
)]
#![allow(
    clippy::missing_panics_doc,
    reason = "Development: docs will be completed"
)]
#![allow(
    clippy::must_use_candidate,
    reason = "Development: will add must_use appropriately"
)]
#![allow(
    clippy::new_without_default,
    reason = "Development: will add Default impls"
)]
#![allow(
    clippy::unnecessary_wraps,
    reason = "Development: error handling may be needed"
)]
#![allow(
    clippy::ref_option,
    reason = "Development: some APIs clearer with &Option"
)]
#![allow(
    clippy::pattern_type_mismatch,
    reason = "Development: pattern style preference"
)]
#![allow(
    clippy::option_if_let_else,
    reason = "Development: if let can be clearer"
)]
#![allow(
    clippy::if_then_some_else_none,
    reason = "Development: explicit conditions clearer"
)]
#![allow(
    clippy::doc_markdown,
    reason = "Development: documentation formatting will be polished"
)]

pub mod config;
pub mod embedding;
pub mod embedding_cache;
pub mod embedding_knn;
pub mod extraction;
pub mod model;
pub mod prediction;
pub mod tfidf;
pub mod unified_predictor;

pub use config::{PredictorType, Settings};
pub use prediction::{Prediction, Predictor};
pub use unified_predictor::UnifiedPredictor;
