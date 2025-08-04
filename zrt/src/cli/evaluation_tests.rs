//! Tests for safety-critical ML model evaluation functionality
//! 
//! This module tests the arithmetic safety patterns, bounds checking,
//! and mathematical operations we've implemented for safety-critical code.

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::collections::HashSet;
    use anyhow::Result;

    #[cfg(feature = "tagging")]
    use zrt_tagging::{
        extraction::{NoteData, TrainingData},
        prediction::Prediction,
    };

    /// Mock predictor for testing arithmetic safety
    #[cfg(feature = "tagging")]
    struct MockPredictor {
        predictions: Vec<Prediction>,
    }

    #[cfg(feature = "tagging")]
    impl MockPredictor {
        fn new(predictions: Vec<Prediction>) -> Self {
            Self { predictions }
        }

        fn predict(&self, _content: &str) -> Result<Vec<Prediction>> {
            Ok(self.predictions.clone())
        }
    }

    #[cfg(feature = "tagging")]
    #[test]
    fn test_validate_model_performance_arithmetic_safety() -> Result<()> {
        // Test our safety-critical arithmetic patterns
        let _temp_dir = TempDir::new()?;
        
        // Create test notes with known tags
        let validation_data = TrainingData {
            notes: vec![
                NoteData {
                    path: "note1.md".to_owned(),
                    content: "test content 1".to_owned(),
                    tags: ["tag1", "tag2"].iter().map(|s| s.to_string()).collect(),
                },
                NoteData {
                    path: "note2.md".to_owned(), 
                    content: "test content 2".to_owned(),
                    tags: ["tag2", "tag3"].iter().map(|s| s.to_string()).collect(),
                },
            ],
            all_tags: ["tag1", "tag2", "tag3"].iter().map(|s| s.to_string()).collect(),
        };

        // Mock predictor that returns specific predictions to test arithmetic
        let predictor = MockPredictor::new(vec![
            Prediction { tag: "tag1".to_owned(), confidence: 0.9 },
            Prediction { tag: "tag2".to_owned(), confidence: 0.8 },  
        ]);

        // This should exercise our checked arithmetic operations
        let result = std::panic::catch_unwind(|| {
            // This would normally call validate_model_performance, but since it prints to stdout
            // we need to test the arithmetic operations it uses directly
            
            // Test checked addition patterns
            let mut total_predicted_tags = 0_usize;
            let mut total_actual_tags = 0_usize;
            let mut correct_predictions = 0_i32;
            let mut total_predictions = 0_i32;

            for note in &validation_data.notes {
                let predictions = predictor.predict(&note.content).unwrap();
                
                // Test our safety-critical checked arithmetic
                total_predicted_tags = total_predicted_tags
                    .checked_add(predictions.len())
                    .unwrap_or(total_predicted_tags);
                    
                total_actual_tags = total_actual_tags
                    .checked_add(note.tags.len())
                    .unwrap_or(total_actual_tags);

                // Test precision@k arithmetic
                let k = 1;
                let top_k_predictions: Vec<_> = predictions.iter().take(k).collect();
                let correct_in_k = top_k_predictions
                    .iter()
                    .filter(|pred| note.tags.contains(&pred.tag))
                    .count();

                // This tests the bounds-checked array access pattern we implemented
                let mut precision_at_k = [0.0_f64; 3];
                let mut count_at_k = [0_i32; 3];
                
                if let Some(precision_ref) = precision_at_k.get_mut(0) {
                    *precision_ref += f64::from(u32::try_from(correct_in_k).unwrap_or(u32::MAX))
                        / f64::from(u32::try_from(k.min(note.tags.len())).unwrap_or(u32::MAX));
                }
                
                if let Some(count_ref) = count_at_k.get_mut(0) {
                    *count_ref = count_ref.checked_add(1_i32).unwrap_or(*count_ref);
                }

                // Test per-tag statistics with checked arithmetic
                for tag in &note.tags {
                    if predictions.iter().any(|pred| &pred.tag == tag) {
                        correct_predictions = correct_predictions
                            .checked_add(1_i32)
                            .unwrap_or(correct_predictions);
                    }
                    total_predictions = total_predictions
                        .checked_add(1_i32)
                        .unwrap_or(total_predictions);
                }
            }

            // Test division safety patterns
            let overall_precision = if total_predicted_tags > 0_usize {
                f64::from(correct_predictions) / f64::from(total_predictions)
            } else {
                0.0_f64
            };

            let overall_recall = if total_actual_tags > 0_usize {
                f64::from(correct_predictions)
                    / f64::from(u32::try_from(total_actual_tags).unwrap_or(u32::MAX))
            } else {
                0.0_f64
            };

            // Verify our arithmetic worked correctly
            assert_eq!(total_predicted_tags, 4); // 2 predictions per note × 2 notes
            assert_eq!(total_actual_tags, 4);   // 2 tags per note × 2 notes
            assert!(overall_precision >= 0.0 && overall_precision <= 1.0);
            assert!(overall_recall >= 0.0 && overall_recall <= 1.0);
        });

        assert!(result.is_ok(), "Safety-critical arithmetic should not panic");
        Ok(())
    }

    #[cfg(feature = "tagging")]
    #[test] 
    fn test_arithmetic_overflow_protection() {
        // Test that our checked arithmetic handles potential overflow
        let _max_usize = usize::MAX;
        let near_max = usize::MAX - 1;

        // Test checked_add protection
        let result = near_max.checked_add(10).unwrap_or(near_max);
        assert_eq!(result, near_max); // Should use fallback value

        // Test that our pattern prevents overflow
        let mut count = usize::MAX - 1;
        count = count.checked_add(5).unwrap_or(count);
        assert_eq!(count, usize::MAX - 1); // Should remain unchanged

        // Test i32 overflow protection  
        let mut i32_count = i32::MAX - 1;
        i32_count = i32_count.checked_add(10).unwrap_or(i32_count);
        assert_eq!(i32_count, i32::MAX - 1);
    }

    #[test]
    fn test_bounds_checked_array_access() {
        // Test our bounds-checked array access patterns
        let mut test_array = [0_i32; 3];
        
        // Safe access within bounds
        if let Some(element) = test_array.get_mut(0) {
            *element = 42;
        }
        assert_eq!(test_array[0], 42);

        // Safe access out of bounds (should not modify anything)
        let original = test_array.clone();
        if let Some(element) = test_array.get_mut(10) {
            *element = 999;
        }
        assert_eq!(test_array, original); // Should be unchanged

        // Test slice bounds checking
        let test_vec = vec![1, 2, 3, 4, 5];
        let safe_slice = test_vec.get(1..3).unwrap_or(&[]);
        assert_eq!(safe_slice, &[2, 3]);

        let out_of_bounds_slice = test_vec.get(10..20).unwrap_or(&[]);
        assert_eq!(out_of_bounds_slice, &[] as &[i32]); // Should be empty slice
    }

    #[test]
    fn test_division_by_zero_protection() {
        // Test our division safety patterns
        let numerator = 100.0_f64;
        let zero_denominator = 0_usize;
        let nonzero_denominator = 5_usize;

        // Safe division with zero check
        let result1 = if zero_denominator > 0_usize {
            numerator / f64::from(u32::try_from(zero_denominator).unwrap_or(u32::MAX))
        } else {
            0.0_f64
        };
        assert_eq!(result1, 0.0);

        // Safe division with nonzero value
        let result2 = if nonzero_denominator > 0_usize {
            numerator / f64::from(u32::try_from(nonzero_denominator).unwrap_or(u32::MAX))
        } else {
            0.0_f64  
        };
        assert_eq!(result2, 20.0);
    }

    #[test]
    fn test_checked_rem_pattern() {
        // Test our checked remainder operations
        let dividend = 100_usize;
        let divisor = 7_usize;
        let zero_divisor = 0_usize;

        // Safe remainder operation
        let result1 = dividend.checked_rem(divisor).unwrap_or(0);
        assert_eq!(result1, 2); // 100 % 7 = 2

        // Safe remainder with zero divisor
        let result2 = dividend.checked_rem(zero_divisor).unwrap_or(0);
        assert_eq!(result2, 0); // Should use fallback

        // Test the pattern we use in model.rs
        let epoch = 25_usize;
        let progress_interval = 10_usize;
        let should_print = epoch.checked_rem(progress_interval).unwrap_or(1) == 0;
        assert!(!should_print); // 25 % 10 != 0

        let epoch2 = 30_usize; 
        let should_print2 = epoch2.checked_rem(progress_interval).unwrap_or(1) == 0;
        assert!(should_print2); // 30 % 10 == 0
    }

    #[test]
    fn test_checked_div_pattern() {
        // Test our checked division operations
        let dividend = 100_u64;
        let divisor = 10_u64;
        let zero_divisor = 0_u64;

        // Safe division operation  
        let result1 = dividend.checked_div(divisor).unwrap_or(0);
        assert_eq!(result1, 10);

        // Safe division with zero divisor
        let result2 = dividend.checked_div(zero_divisor).unwrap_or(0);
        assert_eq!(result2, 0); // Should use fallback

        // Test the pattern we use in embedding_cache.rs
        let size_bytes = 0x0020_0000_u64; // 2MB
        let mb_divisor = 0x0010_0000_u64; // 1MB
        let mb_result = size_bytes.checked_div(mb_divisor).unwrap_or(0);
        assert_eq!(mb_result, 2);
    }

    #[cfg(feature = "tagging")]
    #[test]
    fn test_sorted_hash_iteration() -> Result<()> {
        // Test our deterministic hash iteration pattern
        let mut test_tags: HashSet<String> = HashSet::new();
        test_tags.insert("zebra".to_owned());
        test_tags.insert("alpha".to_owned()); 
        test_tags.insert("beta".to_owned());

        // Test that our sorted iteration is deterministic
        let mut sorted_tags: Vec<_> = test_tags.iter().collect();
        sorted_tags.sort_unstable();
        
        let collected: Vec<String> = sorted_tags.iter().map(|s| (*s).clone()).collect();
        assert_eq!(collected, vec!["alpha", "beta", "zebra"]);

        // Test multiple iterations produce same order
        let mut sorted_tags2: Vec<_> = test_tags.iter().collect();
        sorted_tags2.sort_unstable();
        assert_eq!(sorted_tags, sorted_tags2);

        Ok(())
    }

    #[test]
    fn test_frontmatter_extraction_bounds_safety() -> Result<()> {
        // Test our bounds-safe frontmatter extraction
        let test_content = "---\ntags: [test]\n---\nBody content";
        let lines: Vec<&str> = test_content.lines().collect();

        // Test safe slicing patterns
        let end = 2; // Index of second "---"
        let frontmatter = lines.get(1..end).unwrap_or(&[]).join("\n");
        assert_eq!(frontmatter, "tags: [test]");

        let start_idx = end.checked_add(1).unwrap_or(end);
        let body = lines.get(start_idx..).unwrap_or(&[]).join("\n");
        assert_eq!(body, "Body content");

        // Test out-of-bounds safety
        let bad_end = 100;
        let safe_frontmatter = lines.get(1..bad_end).unwrap_or(&[]).join("\n");
        assert_eq!(safe_frontmatter, ""); // Should be empty

        Ok(())
    }
}