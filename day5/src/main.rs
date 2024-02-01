use std::env;
use std::error::Error;
use std::fmt;
use std::fs;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[derive(Debug, PartialEq)]
enum AppError {
    Usage(String),
    Io(String),
    EmptyFile,
    WhitespaceOnly,
    NoFirstLine,
    InvalidFirstLine(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Usage(msg) => write!(f, "{}", msg),
            AppError::Io(msg) => write!(f, "{}", msg),
            AppError::EmptyFile => write!(f, "File is empty"),
            AppError::WhitespaceOnly => write!(f, "File contains only whitespace"),
            AppError::NoFirstLine => write!(f, "No first line found"),
            AppError::InvalidFirstLine(line) => {
                write!(f, "First line is not a valid i32: '{}'", line)
            }
        }
    }
}

impl Error for AppError {}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    run_with_args(&args).map_err(|e| Box::new(e) as Box<dyn Error>)
}

fn run_with_args(args: &[String]) -> Result<(), AppError> {
    let strict = args.iter().any(|a| a == "--strict-first-line-int");
    let path = args
        .iter()
        .find(|a| a.as_str() != "--strict-first-line-int")
        .ok_or_else(|| AppError::Usage("Usage: cargo run -- <file_path> [--strict-first-line-int]".to_string()))?;

    let text = read_text(path)?;

    let lines = count_lines(&text)?;
    let words = count_words(&text)?;
    println!("Lines: {}", lines);
    println!("Words: {}", words);

    match parse_first_line_i32(&text) {
        Ok(n) => println!("First line as i32: {}", n),
        Err(e) if strict => return Err(e),
        Err(e) => println!("First line parse skipped: {}", e),
    }

    Ok(())
}

fn read_text(path: &str) -> Result<String, AppError> {
    fs::read_to_string(path).map_err(|e| AppError::Io(format!("Failed to read '{}': {}", path, e)))
}

fn count_lines(text: &str) -> Result<usize, AppError> {
    if text.is_empty() {
        return Err(AppError::EmptyFile);
    }
    Ok(text.lines().count())
}

fn count_words(text: &str) -> Result<usize, AppError> {
    if text.trim().is_empty() {
        return Err(AppError::WhitespaceOnly);
    }
    Ok(text.split_whitespace().count())
}

fn parse_first_line_i32(text: &str) -> Result<i32, AppError> {
    let first = text.lines().next().ok_or(AppError::NoFirstLine)?;
    first
        .trim()
        .parse::<i32>()
        .map_err(|_| AppError::InvalidFirstLine(first.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_text_error_branch() {
        let err = read_text("definitely-missing-file-12345.txt").unwrap_err();
        assert!(matches!(err, AppError::Io(_)));
    }

    #[test]
    fn count_lines_empty_branch() {
        assert_eq!(count_lines("").unwrap_err(), AppError::EmptyFile);
    }

    #[test]
    fn count_words_whitespace_branch() {
        assert_eq!(count_words("   \n\t ").unwrap_err(), AppError::WhitespaceOnly);
    }

    #[test]
    fn parse_first_line_missing_branch() {
        assert_eq!(parse_first_line_i32("").unwrap_err(), AppError::NoFirstLine);
    }

    #[test]
    fn parse_first_line_invalid_branch() {
        assert_eq!(
            parse_first_line_i32("abc\n2").unwrap_err(),
            AppError::InvalidFirstLine("abc".to_string())
        );
    }

    #[test]
    fn run_with_args_usage_branch() {
        let args: Vec<String> = vec![];
        assert!(matches!(run_with_args(&args).unwrap_err(), AppError::Usage(_)));
    }

    #[test]
    fn strict_mode_returns_error() {
        let dir = std::env::temp_dir();
        let path = dir.join("day5_strict_mode_invalid_i32.txt");
        std::fs::write(&path, "not-an-int\nhello world").unwrap();
        let args = vec![
            path.to_string_lossy().to_string(),
            "--strict-first-line-int".to_string(),
        ];
        let result = run_with_args(&args);
        assert!(matches!(result.unwrap_err(), AppError::InvalidFirstLine(_)));
        let _ = std::fs::remove_file(&path);
    }
}