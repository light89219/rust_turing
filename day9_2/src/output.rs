use crate::AnalysisResult;

pub fn format_result(result: &AnalysisResult) -> String {
    format!(
        "Total words: {}\nUnique words: {}",
        result.total_words, result.unique_words
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_result_basic() {
        let result = AnalysisResult {
            total_words: 5,
            unique_words: 2,
        };
        assert_eq!(format_result(&result), "Total words: 5\nUnique words: 2");
    }
}
