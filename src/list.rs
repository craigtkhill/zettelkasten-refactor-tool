mod main;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <markdown-file>", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];

    match count_word_frequencies(file_path) {
        Ok(frequencies) => display_rankings(frequencies),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn count_word_frequencies(file_path: &str) -> Result<HashMap<String, usize>, std::io::Error> {
    // Read the file contents
    let contents = fs::read_to_string(file_path)?;

    // Create a HashMap to store word frequencies
    let mut word_counts: HashMap<String, usize> = HashMap::new();

    // Process each word
    for word in contents.split_whitespace() {
        // Convert to lowercase and remove common punctuation
        let cleaned_word = word
            .trim_matches(|c: char| !c.is_alphabetic())
            .to_lowercase();

        if !cleaned_word.is_empty() {
            *word_counts.entry(cleaned_word).or_insert(0) += 1;
        }
    }

    Ok(word_counts)
}

fn display_rankings(frequencies: HashMap<String, usize>) {
    // Convert HashMap to vector for sorting
    let mut word_counts: Vec<_> = frequencies.into_iter().collect();

    // Sort by count (descending) and then alphabetically for ties
    word_counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    // Display results
    println!("Word frequency ranking:");
    println!("{:<20} {:<10}", "WORD", "COUNT");
    println!("{:-<30}", "");

    for (word, count) in word_counts {
        println!("{:<20} {:<10}", word, count);
    }
}
