use std::io;

use day9_2::{analyze_text, format_result};

fn main() {
    println!("Enter text to analyze:");

    let mut input = String::new();

    if let Err(err) = io::stdin().read_line(&mut input) {
        eprintln!("Failed to read input: {}", err);
        return;
    }

    let result = analyze_text(input.trim_end());
    println!("{}", format_result(&result));
}
