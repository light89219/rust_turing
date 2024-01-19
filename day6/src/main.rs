use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::time::Instant;

const DEFAULT_TOP_N: usize = 10;
const STOP_WORDS: [&str; 12] = ["the", "a", "an", "is", "to", "of", "in", "on", "and", "or", "for", "with"];

#[derive(Debug, Clone)]
struct Config {
    input: Option<String>,
    top_n: usize,
    as_json: bool,
    bench: bool,
}

fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if ".,!?;:".contains(ch) {
            out.push(' ');
        } else {
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
        }
    }
    out
}

fn word_counts(text: &str, stop_words: &[&str]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    let stop_words: std::collections::HashSet<&str> = stop_words.iter().copied().collect();
    for word in normalize(text).split_whitespace() {
        if stop_words.contains(word) {
            continue;
        }
        *counts.entry(word.to_string()).or_insert(0) += 1;
    }
    counts
}

fn top_n(counts: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    let mut items: Vec<(&String, &usize)> = counts.iter().collect();
    items.sort_by(|(wa, ca), (wb, cb)| cb.cmp(ca).then_with(|| wa.cmp(wb)));

    items
        .into_iter()
        .take(n)
        .map(|(w, c)| (w.clone(), *c))
        .collect()
}

fn read_all_stdin() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

fn parse_args(args: &[String]) -> Result<Config, String> {
    let mut input: Option<String> = None;
    let mut top_n = DEFAULT_TOP_N;
    let mut as_json = false;
    let mut bench = false;

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--top" => {
                i += 1;
                if i >= args.len() {
                    return Err(String::from("missing value for --top"));
                }
                top_n = args[i]
                    .parse::<usize>()
                    .map_err(|_| String::from("--top must be a positive number"))?;
            }
            "--json" => as_json = true,
            "--bench" => bench = true,
            arg if arg.starts_with("--") => return Err(format!("unknown flag: {arg}")),
            path => {
                if input.is_some() {
                    return Err(String::from("only one input source is supported (FILE or -)"));
                }
                input = Some(path.to_string());
            }
        }
        i += 1;
    }

    Ok(Config {
        input,
        top_n,
        as_json,
        bench,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let prog = args.first().map(String::as_str).unwrap_or("day6");
    let config = match parse_args(&args) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!(
                "error: {msg}\nusage: {prog} [FILE|-] [--top N] [--json] [--bench]\n  FILE defaults to stdin when omitted\n  --top N prints top N words (default 10)\n  --json prints sorted output as JSON\n  --bench prints phase timings"
            );
            std::process::exit(1);
        }
    };

    let read_start = Instant::now();
    let text = match config.input.as_deref() {
        Some(path) => {
            let text = if path == "-" {
                read_all_stdin()?
            } else {
                fs::read_to_string(path)?
            };
            text
        }
        None => read_all_stdin()?,
    };

    let trimmed = text.trim();
    if trimmed.is_empty() {
        println!("(empty input — no words to count)");
        return Ok(());
    }

    let count_start = Instant::now();
    let counts = word_counts(trimmed, &STOP_WORDS);
    if counts.is_empty() {
        println!("(empty input — no words to count)");
        return Ok(());
    }

    let sort_start = Instant::now();
    let top = top_n(&counts, config.top_n);
    if config.as_json {
        println!("{}", serde_json::to_string_pretty(&top)?);
    } else {
        for (word, count) in top {
            println!("{}\t{}", count, word);
        }
    }

    if config.bench {
        eprintln!(
            "bench: read={}ms count={}ms sort+output={}ms",
            read_start.elapsed().as_millis(),
            count_start.elapsed().as_millis(),
            sort_start.elapsed().as_millis()
        );
    }

    Ok(())
}
