// tests/integration_tests/scanning_test.rs
use super::common::{create_ignore_file, setup_test_directory};
use anyhow::Result;
use zrt::{scan_directory_single, scan_directory_two};

#[test]
fn test_scanning_with_ignore() -> Result<()> {
    let temp_dir = setup_test_directory()?;

    create_ignore_file(
        temp_dir.path(),
        &["*.tmp", "draft/", "cache/", "node_modules/"],
    )?;

    // Test single pattern scan
    let single_stats = scan_directory_single(&temp_dir.path().to_path_buf(), "to_refactor")?;

    assert_eq!(
        single_stats.total_files, 4,
        "Should count only non-ignored files"
    );
    assert_eq!(
        single_stats.files_with_pattern, 2,
        "Should find correct number of tagged files"
    );
    assert!((single_stats.calculate_percentage() - 50.0).abs() < f64::EPSILON);

    // Test two pattern scan
    let dual_stats =
        scan_directory_two(&temp_dir.path().to_path_buf(), "refactored", "to_refactor")?;

    assert_eq!(dual_stats.total, 4, "Should count only non-ignored files");
    assert_eq!(
        dual_stats.done, 2,
        "Should find correct number of done files"
    );
    assert_eq!(
        dual_stats.todo, 2,
        "Should find correct number of todo files"
    );
    assert!((dual_stats.calculate_percentage() - 50.0).abs() < f64::EPSILON);

    Ok(())
}
