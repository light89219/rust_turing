/// Exercise A: keep only strictly positive even integers, then multiply each by 3.
fn positive_even_tripled(nums: Vec<i32>) -> Vec<i32> {
    nums.into_iter()
        .filter(|&n| n > 0 && n % 2 == 0)
        .map(|n| n * 3)
        .collect()
}

/// Exercise B: total of all elements (empty slice → 0).
fn sum(nums: &[i32]) -> i32 {
    nums.iter().fold(0, |acc, &n| acc + n)
}

/// Exercise B: product of all elements (empty slice → 1).
fn product(nums: &[i32]) -> i32 {
    nums.iter().fold(1, |acc, &n| acc * n)
}

/// Exercise C: parse only valid integers.
fn parse_ints(items: Vec<&str>) -> Vec<i32> {
    items
        .into_iter()
        .filter_map(|s| s.parse::<i32>().ok())
        .collect()
}

/// Exercise D: count values strictly greater than `threshold`; the closure captures `threshold`.
fn count_exceeding(values: &[i32], threshold: i32) -> usize {
    values.iter().copied().filter(|&n| n > threshold).count()
}

fn main() {
    let nums = vec![-4, 0, 2, 3, 4, 5, 6, 7, 8];
    let out = positive_even_tripled(nums);
    println!("{out:?}");

    let sample = [1, 2, 3, 4];
    println!("sum: {}", sum(&sample));
    println!("product: {}", product(&sample));

    let raw = vec!["10", "x", "-7", "42", "3.14"];
    println!("parsed ints: {:?}", parse_ints(raw));

    let threshold = 5;
    let readings = [1, 6, 3, 8, 5, 12];
    println!(
        "count > {threshold}: {}",
        count_exceeding(&readings, threshold)
    );
}
