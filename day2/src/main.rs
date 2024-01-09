use std::io::{self, BufRead};

fn main() {
    let mut text = read_line("Enter initial text: ");

    loop {
        println!("\nChoose:");
        println!("1) Show length");
        println!("2) Word count");
        println!("3) Append suffix");
        println!("4) Uppercase preview");
        println!("5) Exit");

        let choice = read_line("Choice: ");

        match choice.trim() {
            "1" => println!("Length: {}", show_length(&text)),
            "2" => println!("Words: {}", word_count(&text)),
            "3" => {
                let suffix = read_line("Suffix to append: ");
                append_suffix(&mut text, suffix.trim());
                println!("Updated text: {}", text);
            }
            "4" => println!("Uppercase: {}", uppercase_preview(&text)),
            "5" => {
                println!("Goodbye");
                break;
            }
            _ => println!("Invalid option"),
        }
    }
}

fn read_line(prompt: &str) -> String {
    println!("{}", prompt);
    let mut stdin = io::stdin().lock();
    read_line_from(&mut stdin)
}

fn read_line_from<R: BufRead>(reader: &mut R) -> String {
    let mut input = String::new();
    reader
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim_end().to_string()
}

fn show_length(text: &str) -> usize {
    text.len()
}

fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

fn append_suffix(text: &mut String, suffix: &str) {
    if !suffix.is_empty() {
        text.push(' ');
        text.push_str(suffix);
    }
}

fn uppercase_preview(text: &str) -> String {
    text.to_uppercase()
}

/// Replaces every occurrence of `from` that forms a whole word (not a substring of a longer word).
#[allow(dead_code)]
fn replace_word(text: &mut String, from: &str, to: &str) {
    fn is_word_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    if from.is_empty() {
        return;
    }
    let s = text.as_str();
    let mut out = String::with_capacity(s.len().saturating_add(to.len()));
    let mut i = 0;
    while i < s.len() {
        match s[i..].find(from) {
            None => {
                out.push_str(&s[i..]);
                break;
            }
            Some(rel) => {
                let start = i + rel;
                let end = start + from.len();
                let before_ok = start == 0
                    || s[..start]
                        .chars()
                        .next_back()
                        .is_none_or(|c| !is_word_char(c));
                let after_ok = end >= s.len()
                    || s[end..]
                        .chars()
                        .next()
                        .is_none_or(|c| !is_word_char(c));
                if before_ok && after_ok {
                    out.push_str(&s[i..start]);
                    out.push_str(to);
                    i = end;
                } else {
                    let ch = s[i..].chars().next().expect("i < len");
                    out.push(ch);
                    i += ch.len_utf8();
                }
            }
        }
    }
    *text = out;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_line_from_reads_line_and_trims_end() {
        let mut c = Cursor::new(b"hello\n");
        assert_eq!(read_line_from(&mut c), "hello");
    }

    #[test]
    fn read_line_from_empty_line() {
        let mut c = Cursor::new(b"\n");
        assert_eq!(read_line_from(&mut c), "");
    }

    #[test]
    fn show_length_ascii_bytes() {
        assert_eq!(show_length(""), 0);
        assert_eq!(show_length("hi"), 2);
    }

    #[test]
    fn word_count_splits_on_whitespace() {
        assert_eq!(word_count(""), 0);
        assert_eq!(word_count("one"), 1);
        assert_eq!(word_count("a few words here"), 4);
        assert_eq!(word_count("  spaced  out  "), 2);
    }

    #[test]
    fn append_suffix_skips_empty_suffix() {
        let mut s = String::from("base");
        append_suffix(&mut s, "");
        assert_eq!(s, "base");
    }

    #[test]
    fn append_suffix_adds_space_and_text() {
        let mut s = String::from("Jane");
        append_suffix(&mut s, "Doe");
        assert_eq!(s, "Jane Doe");
    }

    #[test]
    fn uppercase_preview_basic() {
        assert_eq!(uppercase_preview(""), "");
        assert_eq!(uppercase_preview("Rust"), "RUST");
    }

    #[test]
    fn replace_word_replaces_whole_words_only() {
        let mut s = String::from("foo bar foo");
        replace_word(&mut s, "foo", "baz");
        assert_eq!(s, "baz bar baz");
    }

    #[test]
    fn replace_word_skips_substring_inside_longer_token() {
        let mut s = String::from("foobar foo");
        replace_word(&mut s, "foo", "x");
        assert_eq!(s, "foobar x");
    }

    #[test]
    fn replace_word_empty_from_is_noop() {
        let mut s = String::from("a b");
        replace_word(&mut s, "", "z");
        assert_eq!(s, "a b");
    }

    #[test]
    fn replace_word_respects_punctuation_as_boundary() {
        let mut s = String::from("foo, foo.");
        replace_word(&mut s, "foo", "bar");
        assert_eq!(s, "bar, bar.");
    }
}