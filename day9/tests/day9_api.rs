use day9::analyze_text;

#[test]
fn public_api_returns_expected_counts_for_case_and_punctuation() {
    let result = analyze_text("Rust; rust: RUST");
    assert_eq!(result.total_words, 3);
    assert_eq!(result.unique_words, 1);
}
