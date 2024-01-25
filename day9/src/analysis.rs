use std::collections::HashSet;

use crate::normalize::normalize_text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisResult {
    pub total_words: usize,
    pub unique_words: usize,
}

pub fn analyze_text(input: &str) -> AnalysisResult {
    let normalized = normalize_text(input);
    let words: Vec<&str> = normalized.split_whitespace().collect();
    let unique_words: HashSet<&str> = words.iter().copied().collect();

    AnalysisResult {
        total_words: words.len(),
        unique_words: unique_words.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::analyze_text;

    #[test]
    fn analyze_table_driven_cases() {
        let cases = vec![
            ("Hello, hello!", 2, 1),
            ("Rust; rust: RUST", 3, 1),
            ("", 0, 0),
            ("a b c a", 4, 3),
        ];

        for (input, total, unique) in cases {
            let result = analyze_text(input);
            assert_eq!(result.total_words, total);
            assert_eq!(result.unique_words, unique);
        }
    }

    #[test]
    fn analyze_property_style_randomish_inputs_respect_bounds() {
        const PUNCTUATION: &str = ".,!?;:";
        let mut seed: u64 = 0xC0FFEE;

        for _ in 0..150 {
            let mut input = String::new();
            for _ in 0..120 {
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let bucket = (seed % 12) as u8;
                let ch = match bucket {
                    0..=4 => (b'A' + ((seed >> 8) % 26) as u8) as char,
                    5..=8 => (b'a' + ((seed >> 16) % 26) as u8) as char,
                    9 => ' ',
                    10 => '\n',
                    _ => {
                        let idx = ((seed >> 24) as usize) % PUNCTUATION.len();
                        PUNCTUATION.as_bytes()[idx] as char
                    }
                };
                input.push(ch);
            }

            let result = analyze_text(&input);
            assert!(result.unique_words <= result.total_words);
            assert!(result.total_words <= input.len());
        }
    }
}
