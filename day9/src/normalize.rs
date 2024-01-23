pub(crate) fn normalize_text(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|ch| if ".,!?;:".contains(ch) { ' ' } else { ch })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::normalize_text;

    #[test]
    fn normalize_table_driven_cases() {
        let cases = vec![
            ("Rust; rust: RUST", "rust  rust  rust"),
            ("Hello, hello!", "hello  hello "),
            ("", ""),
            ("MiXeD CaSe", "mixed case"),
        ];

        for (input, expected) in cases {
            assert_eq!(normalize_text(input), expected);
        }
    }

    #[test]
    fn normalize_property_style_randomish_inputs_remove_target_punctuation() {
        const PUNCTUATION: &str = ".,!?;:";
        let mut seed: u64 = 0xA11CE5EED;

        for _ in 0..200 {
            let mut input = String::new();
            for _ in 0..80 {
                // Simple deterministic LCG for repeatable pseudo-random test data.
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let bucket = (seed % 10) as u8;
                let ch = match bucket {
                    0..=2 => (b'A' + ((seed >> 8) % 26) as u8) as char,
                    3..=5 => (b'a' + ((seed >> 16) % 26) as u8) as char,
                    6 => ' ',
                    7 => '\t',
                    _ => {
                        let idx = ((seed >> 24) as usize) % PUNCTUATION.len();
                        PUNCTUATION.as_bytes()[idx] as char
                    }
                };
                input.push(ch);
            }

            let normalized = normalize_text(&input);
            assert!(!normalized.chars().any(|c| PUNCTUATION.contains(c)));
            assert_eq!(normalized, normalized.to_lowercase());
        }
    }
}
