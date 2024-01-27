use std::io;

use day9::analyze_text;

fn main() {
    println!("Enter text to analyze:");

    let mut input = String::new();
    if let Err(err) = io::stdin().read_line(&mut input) {
        eprintln!("Failed to read input: {}", err);
        return;
    }

    let result = analyze_text(input.trim_end());
    println!("Total words: {}", result.total_words);
    println!("Unique words: {}", result.unique_words);
}
