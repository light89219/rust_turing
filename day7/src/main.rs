use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;
use itertools::Itertools;

#[derive(Debug, Clone)]
struct Record {
    name: String,
    department: String,
    salary: u32,
}

fn parse_record(line: &str) -> Option<Record> {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
    if parts.len() != 3 {
        return None;
    }

    let name = parts[0];
    let department = parts[1];
    let salary = parts[2].parse::<u32>().ok()?;

    if name.is_empty() || department.is_empty() {
        return None;
    }

    Some(Record {
        name: name.to_string(),
        department: department.to_string(),
        salary,
    })
}

fn is_header_row(line: &str) -> bool {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
    if parts.len() != 3 {
        return false;
    }
    parts[0].eq_ignore_ascii_case("name")
        && parts[1].eq_ignore_ascii_case("department")
        && parts[2].eq_ignore_ascii_case("salary")
}

fn parse_records(text: &str) -> Vec<Record> {
    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .skip_while(|line| is_header_row(line))
        .filter_map(parse_record)
        .collect()
}

fn avg_salary(records: &[Record]) -> Option<f64> {
    (!records.is_empty()).then(|| {
        let total: u64 = records.iter().map(|r| r.salary as u64).sum();
        total as f64 / records.len() as f64
    })
}

fn highest_paid(records: &[Record]) -> Option<&Record> {
    records.iter().max_by_key(|r| r.salary)
}

fn median_salary(records: &[Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sorted = records.iter().map(|r| r.salary).sorted().collect_vec();
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        Some(sorted[mid] as f64)
    } else {
        Some((sorted[mid - 1] as f64 + sorted[mid] as f64) / 2.0)
    }
}

fn salary_by_department(records: &[Record]) -> HashMap<String, u32> {
    records.iter().fold(HashMap::new(), |mut map, r| {
        *map.entry(r.department.clone()).or_insert(0) += r.salary;
        map
    })
}

fn read_all_stdin() -> Result<String, String> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("failed to read stdin: {e}"))?;
    Ok(buf)
}

fn usage(exe: &str) -> String {
    format!(
        "Usage:\n  {exe} [path]\n\nIf [path] is omitted, reads from stdin.\nInput format: name,department,salary (one row per line)\n"
    )
}

fn main() -> ExitCode {
    let mut args = env::args();
    let exe = args.next().unwrap_or_else(|| "day7".to_string());
    let maybe_path = args.next();

    if matches!(maybe_path.as_deref(), Some("-h" | "--help")) {
        print!("{}", usage(&exe));
        return ExitCode::SUCCESS;
    }

    let text = match maybe_path {
        Some(path) => match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error: failed to read file `{path}`: {e}");
                return ExitCode::from(2);
            }
        },
        None => match read_all_stdin() {
            Ok(s) => s,
            Err(msg) => {
                eprintln!("Error: {msg}");
                eprintln!();
                eprintln!("{}", usage(&exe));
                return ExitCode::from(2);
            }
        },
    };

    let records = parse_records(&text);
    println!("valid rows: {}", records.len());

    match avg_salary(&records) {
        Some(avg) => println!("average salary: {:.2}", avg),
        None => println!("average salary: (n/a)"),
    }
    match median_salary(&records) {
        Some(median) => println!("median salary: {:.2}", median),
        None => println!("median salary: (n/a)"),
    }

    match highest_paid(&records) {
        Some(r) => println!(
            "highest salary: {}, {}, {}",
            r.name, r.department, r.salary
        ),
        None => println!("highest salary: (n/a)"),
    }

    let totals = salary_by_department(&records)
        .into_iter()
        .sorted_by(|(dept_a, total_a), (dept_b, total_b)| {
            total_b.cmp(total_a).then_with(|| dept_a.cmp(dept_b))
        });
    println!("salary totals by department:");
    for (dept, total) in totals {
        println!("  {dept}: {total}");
    }

    ExitCode::SUCCESS
}
