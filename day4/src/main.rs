use std::io::{self, Write};

fn main() {
    println!("Safe parser. Commands: u32 <s>, i64 <s>, f64 <s>, bool <s>, opti32 <s>, hex <s>, quit");
    let cfg = Config::from_options(None, Some("true"), None);
    println!("config defaults -> {:?}", cfg);
    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "quit" {
            break;
        }

        let mut parts = line.splitn(2, char::is_whitespace);
        let cmd = parts.next().unwrap_or("");
        let rest = parts.next().unwrap_or("");

        match cmd {
            "u32" => match parse_u32(rest) {
                Some(n) => println!("{} -> doubled {}", n, n.saturating_mul(2)),
                None => println!("invalid u32"),
            },
            "i64" => match parse_i64(rest) {
                Some(n) => println!("{} -> abs {}", n, n.saturating_abs()),
                None => println!("invalid i64"),
            },
            "f64" => match parse_f64(rest) {
                Some(n) => println!("{} -> half {}", n, n / 2.0),
                None => println!("invalid f64"),
            },
            "bool" => match parse_bool(rest) {
                Some(b) => println!("{}", b),
                None => println!("invalid bool"),
            },
            "opti32" => match parse_optional_i32(rest) {
                Some(Some(n)) => println!("optional i32: {}", n),
                Some(None) => println!("optional i32: <missing field>"),
                None => println!("invalid optional i32"),
            },
            "hex" => match parse_hex_u32(rest) {
                Some(n) => println!("hex {} -> {}", rest.trim(), n),
                None => println!("invalid hex u32 (expected 0x...)"),
            },
            _ => println!("unknown command"),
        }
    }
}

fn parse_u32(s: &str) -> Option<u32> {
    s.trim().parse().ok()
}

fn parse_i64(s: &str) -> Option<i64> {
    s.trim().parse().ok()
}

fn parse_f64(s: &str) -> Option<f64> {
    s.trim().parse().ok()
}

/// Empty input means field is present but missing value: `Some(None)`.
/// Valid integer means present value: `Some(Some(i32))`.
/// Invalid text means parse failure: `None`.
fn parse_optional_i32(s: &str) -> Option<Option<i32>> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        Some(None)
    } else {
        trimmed.parse::<i32>().ok().map(Some)
    }
}

/// Parse `0x`-prefixed hexadecimal text into `u32`.
fn parse_hex_u32(s: &str) -> Option<u32> {
    let trimmed = s.trim();
    let rest = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X"))?;
    u32::from_str_radix(rest, 16).ok()
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

#[derive(Debug)]
struct Config {
    retries: u32,
    debug: bool,
    timeout_ms: u64,
}

impl Config {
    /// Build from optional text fields and use lazy defaults.
    fn from_options(retries: Option<&str>, debug: Option<&str>, timeout_ms: Option<&str>) -> Self {
        let retries = retries
            .and_then(parse_u32)
            .unwrap_or_else(default_retries);
        let debug = debug
            .and_then(parse_bool)
            .unwrap_or_else(default_debug);
        let timeout_ms = timeout_ms
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or_else(default_timeout_ms);

        Self {
            retries,
            debug,
            timeout_ms,
        }
    }
}

fn default_retries() -> u32 {
    3
}

fn default_debug() -> bool {
    false
}

fn default_timeout_ms() -> u64 {
    5_000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_u32_works() {
        assert_eq!(parse_u32("42"), Some(42));
        assert_eq!(parse_u32(" -1 "), None);
    }

    #[test]
    fn parse_i64_works() {
        assert_eq!(parse_i64("-42"), Some(-42));
        assert_eq!(parse_i64("abc"), None);
    }

    #[test]
    fn parse_f64_works() {
        assert_eq!(parse_f64(" 3.5 "), Some(3.5));
        assert_eq!(parse_f64("nan?"), None);
    }

    #[test]
    fn parse_bool_works() {
        assert_eq!(parse_bool("yes"), Some(true));
        assert_eq!(parse_bool("FALSE"), Some(false));
        assert_eq!(parse_bool("maybe"), None);
    }

    #[test]
    fn parse_optional_i32_works() {
        assert_eq!(parse_optional_i32(""), Some(None));
        assert_eq!(parse_optional_i32(" 12 "), Some(Some(12)));
        assert_eq!(parse_optional_i32("1.2"), None);
    }

    #[test]
    fn parse_hex_u32_works() {
        assert_eq!(parse_hex_u32("0x2A"), Some(42));
        assert_eq!(parse_hex_u32("0Xff"), Some(255));
        assert_eq!(parse_hex_u32("2A"), None);
    }

    #[test]
    fn config_uses_defaults_and_parsed_values() {
        let cfg = Config::from_options(Some("10"), Some("true"), None);
        assert_eq!(cfg.retries, 10);
        assert!(cfg.debug);
        assert_eq!(cfg.timeout_ms, 5_000);
    }
}