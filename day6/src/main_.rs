use std::collections::HashMap;

fn sum(nums: &[i32]) -> i32 {
    nums.iter().sum()
}

fn max(nums: &[i32]) -> Option<i32> {
    nums.iter().copied().max()
}

fn third_item(v: &[String]) -> Option<&String> {
    v.get(2)
}

fn word_counts(words: &[&str]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    for word in words {
        *counts.entry(word.to_string()).or_insert(0) += 1;
    }

    counts
}

fn main() {
    let numbers = vec![7, 2, 10, 4];
    println!("numbers: {:?}", numbers);
    println!("sum: {}", sum(&numbers));

    match max(&numbers) {
        Some(value) => println!("max: {}", value),
        None => println!("max: no value (empty list)"),
    }

    let words = vec![
        String::from("rust"),
        String::from("is"),
        String::from("fun"),
    ];
    match third_item(&words) {
        Some(item) => println!("third item: {}", item),
        None => println!("No third item yet - add more values."),
    }

    let short_words = vec![String::from("only"), String::from("two")];
    match third_item(&short_words) {
        Some(item) => println!("third item: {}", item),
        None => println!("No third item yet - add more values."),
    }

    let input_words = vec!["rust", "is", "fun", "rust", "rust", "fun"];
    let counts = word_counts(&input_words);
    println!("word counts: {:?}", counts);

    let mut labels = vec![
        String::from("borrow"),
        String::from("checker"),
        String::from("practice"),
    ];

    println!("before mutation:");
    for label in &labels {
        println!("{}", label);
    }

    for label in &mut labels {
        label.push('!');
    }

    println!("after mutation:");
    for label in &labels {
        println!("{}", label);
    }
}
