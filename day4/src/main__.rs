use std::sync::atomic::{AtomicU64, Ordering};

static EXPENSIVE_DEFAULT_CALLS: AtomicU64 = AtomicU64::new(0);

fn main() {
    println!("{}", describe(None));
    println!("{}", describe(Some(42)));

    println!("non_empty_len: {:?}", non_empty_len(None));
    println!("non_empty_len: {:?}", non_empty_len(Some(String::new())));
    println!("non_empty_len: {:?}", non_empty_len(Some("hi".to_string())));

    println!("square_chain: {:?}", square_chain(None));
    println!("square_chain: {:?}", square_chain(Some(4)));
    println!("square_chain: {:?}", square_chain(Some(12)));

    // Exercise C: cheap default — `unwrap_or` is fine (no extra closure).
    println!("unwrap_or cheap: {}", Some(7).unwrap_or(0));

    // `unwrap_or(default)` evaluates `default` even when we have `Some` (eager).
    EXPENSIVE_DEFAULT_CALLS.store(0, Ordering::Relaxed);
    let _ = Some(1i64).unwrap_or(simulate_expensive_default());
    println!(
        "unwrap_or + Some: expensive_default ran {} time(s) (eager)",
        EXPENSIVE_DEFAULT_CALLS.load(Ordering::Relaxed)
    );

    // `unwrap_or_else` runs the closure only on `None`.
    EXPENSIVE_DEFAULT_CALLS.store(0, Ordering::Relaxed);
    let _ = Some(1i64).unwrap_or_else(|| simulate_expensive_default());
    println!(
        "unwrap_or_else + Some: expensive_default ran {} time(s) (lazy)",
        EXPENSIVE_DEFAULT_CALLS.load(Ordering::Relaxed)
    );

    EXPENSIVE_DEFAULT_CALLS.store(0, Ordering::Relaxed);
    let v = None::<i64>.unwrap_or_else(|| simulate_expensive_default());
    println!(
        "unwrap_or_else + None: got {v}, expensive_default ran {} time(s)",
        EXPENSIVE_DEFAULT_CALLS.load(Ordering::Relaxed)
    );

    println!(
        "label with env-like default: {}",
        label_or_else(None::<String>)
    );

    println!("bonus_points: {}", bonus_points(None));
    println!("bonus_points: {}", bonus_points(Some("Gold")));
    println!("bonus_points: {}", bonus_points(Some("Silver")));
    println!("bonus_points: {}", bonus_points(Some("Bronze")));
}

/// Gold → 100, Silver → 50, anything else (including no tier) → 0. No `unwrap` / `expect`.
fn bonus_points(tier: Option<&str>) -> u32 {
    tier.map_or(0, |t| match t {
        "Gold" => 100,
        "Silver" => 50,
        _ => 0,
    })
}

fn describe(opt: Option<i32>) -> String {
    match opt {
        None => "none".to_string(),
        Some(n) => format!("some: {n}"),
    }
}

/// `Some(len)` only when the inner string is non-empty; `None` if missing or empty.
fn non_empty_len(opt: Option<String>) -> Option<usize> {
    opt.and_then(|s| if s.is_empty() { None } else { Some(s.len()) })
}

/// Square with `map`, then two chained `and_then` steps (filter to “small” squares, then bump by 1).
fn square_chain(opt: Option<i32>) -> Option<i32> {
    opt.map(|n| n * n)
        .and_then(|sq| if sq <= 100 { Some(sq) } else { None })
        .and_then(|sq| Some(sq + 1))
}

fn simulate_expensive_default() -> i64 {
    EXPENSIVE_DEFAULT_CALLS.fetch_add(1, Ordering::Relaxed);
    99
}

/// Prefer `unwrap_or_else` when building the default does real work (here: allocation + fake “env”).
fn label_or_else(opt: Option<String>) -> String {
    opt.unwrap_or_else(|| {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "guest".to_string())
    })
}
