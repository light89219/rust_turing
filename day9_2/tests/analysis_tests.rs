use day9_2::{analyze_text, format_result};

#[test]
fn analyze_text_public_api() {
    let result = analyze_text("Hello, hello world");
    assert_eq!(result.total_words, 3);
    assert_eq!(result.unique_words, 2);
}

#[test]
fn format_result_public_api() {
    let result = analyze_text("a b a");
    let output = format_result(&result);
    assert_eq!(output, "Total words: 3\nUnique words: 2");
}
