pub(crate) fn normalize_text(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|ch| if ".,!?;:".contains(ch) { ' ' } else { ch })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_basic() {
        assert_eq!(normalize_text("A,B"), "a b");
        assert_eq!(normalize_text("Hello!"), "hello ");
    }
}
