// tests/integration_tests.rs
#[path = "integration_tests/common.rs"]
mod common;

#[path = "integration_tests/edge_cases_test.rs"]
mod edge_cases_test;

#[path = "integration_tests/file_operations_test.rs"]
mod file_operations_test;

#[path = "integration_tests/frontmatter_test.rs"]
mod frontmatter_test;

#[path = "integration_tests/ignore_patterns_test.rs"]
mod ignore_patterns_test;

#[path = "integration_tests/scanning_test.rs"]
mod scanning_test;

#[path = "integration_tests/word_counting_test.rs"]
mod word_counting_test;
