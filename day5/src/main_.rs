fn parse_port(s: &str) -> Result<u16, String> {
    let t = s.trim();
    if t.is_empty() {
        return Err("port must not be empty".to_string());
    }
    let n = t
        .parse::<u16>()
        .map_err(|e| format!("invalid port {:?}: {}", t, e))?;
    if !(1024..=65535).contains(&n) {
        return Err(format!(
            "port must be between 1024 and 65535 (inclusive), got {}",
            n
        ));
    }
    Ok(n)
}

fn read_and_parse_port(path: &str) -> Result<u16, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("failed to read {:?}: {}", path, e))?;
    parse_port(&content)
}

#[derive(Debug, PartialEq)]
struct User {
    name: String,
    age: i32,
}

fn build_user(name: &str, age: i32) -> Result<User, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("name must not be empty".to_string());
    }
    if !(0..=130).contains(&age) {
        return Err(format!(
            "age must be between 0 and 130 (inclusive), got {}",
            age
        ));
    }
    Ok(User {
        name: name.to_string(),
        age,
    })
}

/// Exercise D: `unwrap_or`, `unwrap_or_else`, and `or_else` on `Result`.
fn exercise_d_recovery_demonstration() {
    // `unwrap_or`: if `Err`, substitute a fixed default value (same type as `Ok`).
    let port = parse_port("").unwrap_or(3000);
    println!("unwrap_or on empty input -> use default: {port}");

    // `unwrap_or_else`: if `Err`, compute default from the error (lazy / can inspect `Err`).
    let port = parse_port("oops").unwrap_or_else(|err| {
        println!("  unwrap_or_else: err was {err:?}");
        4000
    });
    println!("unwrap_or_else -> {port}");

    // `or_else`: if `Err`, try another `Result` (fallback strategy; stays `Result` until you unwrap).
    let outcome = parse_port("1023").or_else(|err| {
        println!("  or_else: primary failed ({err}), trying fallback");
        parse_port("9000")
    });
    println!("or_else chain -> {outcome:?}");
}

fn main() {
    for s in ["8080", " 443 ", "", "1023", "65536", "not-a-port"] {
        println!("{s:?} -> {:?}", parse_port(s));
    }

    println!("\n--- Exercise D: recover with defaults ---");
    exercise_d_recovery_demonstration();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_bounds_and_trims() {
        assert_eq!(parse_port(" 8080 ").unwrap(), 8080);
        assert_eq!(parse_port("1024").unwrap(), 1024);
        assert_eq!(parse_port("65535").unwrap(), 65535);
    }

    #[test]
    fn rejects_empty_and_range() {
        assert!(parse_port("").unwrap_err().contains("empty"));
        assert!(parse_port("   ").unwrap_err().contains("empty"));
        assert!(parse_port("1022").unwrap_err().contains("1024"));
        assert!(parse_port("1023").unwrap_err().contains("1024"));
    }

    #[test]
    fn read_and_parse_port_reads_file_and_parses() {
        let dir = std::env::temp_dir();
        let path = dir.join("day5_read_port_test.txt");
        std::fs::write(&path, " 9090 \n").unwrap();
        let p = path.to_str().unwrap();
        assert_eq!(read_and_parse_port(p).unwrap(), 9090);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn read_and_parse_port_maps_io_error() {
        let err = read_and_parse_port("/nonexistent/path/port.txt").unwrap_err();
        assert!(err.contains("failed to read"));
    }

    #[test]
    fn build_user_ok() {
        assert_eq!(
            build_user("  Ada ", 42).unwrap(),
            User {
                name: "Ada".to_string(),
                age: 42,
            }
        );
        assert_eq!(build_user("x", 0).unwrap().age, 0);
        assert_eq!(build_user("x", 130).unwrap().age, 130);
    }

    #[test]
    fn build_user_rejects_name_and_age() {
        assert!(build_user("", 20).unwrap_err().contains("name"));
        assert!(build_user("   ", 20).unwrap_err().contains("name"));
        assert!(build_user("a", -1).unwrap_err().contains("age"));
        assert!(build_user("a", 131).unwrap_err().contains("age"));
    }

    #[test]
    fn exercise_d_unwrap_or() {
        assert_eq!(parse_port("").unwrap_or(4444), 4444);
        assert_eq!(parse_port("8080").unwrap_or(4444), 8080);
    }

    #[test]
    fn exercise_d_unwrap_or_else() {
        let n = parse_port("not-a-number").unwrap_or_else(|_| 5555);
        assert_eq!(n, 5555);
    }

    #[test]
    fn exercise_d_or_else_fallback() {
        let r = parse_port("1023").or_else(|_| parse_port("9000"));
        assert_eq!(r.unwrap(), 9000);
    }
}
