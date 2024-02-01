use std::io;

fn main() {
    loop {
        let a = read_number("Enter first number: ");
        let op = read_operator();
        let b = read_number("Enter second number: ");

        match calculate(a, op, b) {
            Ok(result) => println!("Result: {:.2}", result),
            Err(err) => println!("Error: {}", err),
        }

        if !ask_continue() {
            println!("Goodbye!");
            break;
        }
    }
}

fn read_number(prompt: &str) -> f64 {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");

        match input.trim().parse::<f64>() {
            Ok(n) => return n,
            Err(_) => println!("Invalid number. Try again."),
        }
    }
}

fn read_operator() -> char {
    loop {
        println!("Enter operator (+, -, *, /): ");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");

        let mut chars = input.trim().chars();
        if let Some(op) = chars.next() {
            if matches!(op, '+' | '-' | '*' | '/') {
                return op;
            }
        }

        println!("Invalid operator. Try again.");
    }
}

fn ask_continue() -> bool {
    loop {
        println!("Do another calculation? (y/n): ");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please enter y or n."),
        }
    }
}

fn calculate(a: f64, op: char, b: f64) -> Result<f64, String> {
    match op {
        '+' => Ok(a + b),
        '-' => Ok(a - b),
        '*' => Ok(a * b),
        '/' => {
            if b == 0.0 {
                Err("Cannot divide by zero".to_string())
            } else {
                Ok(a / b)
            }
        }
        _ => Err("Unsupported operator".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::calculate;

    #[test]
    fn add_works() {
        assert_eq!(calculate(2.0, '+', 3.0).unwrap(), 5.0);
    }

    #[test]
    fn subtract_works() {
        assert_eq!(calculate(7.0, '-', 2.0).unwrap(), 5.0);
    }

    #[test]
    fn multiply_works() {
        assert_eq!(calculate(4.0, '*', 2.5).unwrap(), 10.0);
    }

    #[test]
    fn divide_works() {
        assert_eq!(calculate(9.0, '/', 3.0).unwrap(), 3.0);
    }

    #[test]
    fn divide_by_zero_fails() {
        assert!(calculate(5.0, '/', 0.0).is_err());
    }
}