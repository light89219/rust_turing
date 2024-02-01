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
    use super::*;

    #[test]
    fn analyze_counts_words() {
        let result = analyze_text("one two three");
        assert_eq!(result.total_words, 3);
        assert_eq!(result.unique_words, 3);
    }

    #[test]
    fn analyze_treats_same_word_after_normalize_as_one() {
        let result = analyze_text("Hello, hello");
        assert_eq!(result.total_words, 2);
        assert_eq!(result.unique_words, 1);
    }
}
