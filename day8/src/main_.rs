trait Describe {
    fn describe(&self) -> String;

    fn short(&self) -> String {
        let full = self.describe();
        let mut it = full.chars();
        let head: String = it.by_ref().take(32).collect();
        if it.next().is_some() {
            format!("{head}...")
        } else {
            head
        }
    }
}

struct Book {
    title: String,
    pages: u32,
}

struct Movie {
    title: String,
    duration_min: u32,
}

impl Describe for Book {
    fn describe(&self) -> String {
        format!("Book '{}' has {} pages.", self.title, self.pages)
    }
}

impl Describe for Movie {
    fn describe(&self) -> String {
        format!(
            "Movie '{}' runs for {} minutes.",
            self.title, self.duration_min
        )
    }

    fn short(&self) -> String {
        format!("{} ({} min)", self.title, self.duration_min)
    }
}

fn last<T>(items: &[T]) -> Option<&T> {
    items.last()
}

fn swap_pair<T>(a: T, b: T) -> (T, T) {
    (b, a)
}

fn are_equal<T: PartialEq>(a: &T, b: &T) -> bool {
    a == b
}

fn main() {
    let book = Book {
        title: String::from("Rust in Action"),
        pages: 400,
    };

    let movie = Movie {
        title: String::from("Rustacean Story"),
        duration_min: 120,
    };

    let numbers = vec![10, 20, 30];
    let names = vec!["Ana", "Bruno", "Carla"];
    let swapped = swap_pair("left", "right");
    let same_number = are_equal(&42, &42);
    let different_number = are_equal(&10, &20);
    let same_text = are_equal(&"rust", &"rust");

    println!("{}", book.describe());
    println!("{}", movie.describe());
    println!("book short (default): {}", book.short());
    println!("movie short (override): {}", movie.short());
    println!("Last number: {:?}", last(&numbers));
    println!("Last name: {:?}", last(&names));
    println!("Swapped pair: {:?}", swapped);
    println!("42 == 42 ? {}", same_number);
    println!("10 == 20 ? {}", different_number);
    println!("\"rust\" == \"rust\" ? {}", same_text);
}
