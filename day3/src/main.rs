use std::fmt;
use std::io::{self, Write};

#[derive(Debug)]
enum OrderState {
    Draft,
    Confirmed,
    Paid,
    Shipped,
    Delivered,
    Cancelled(String),
}

#[derive(Debug)]
struct Order {
    id: u32,
    customer: String,
    total: f64,
    state: OrderState,
}

impl Order {
    fn confirm(&mut self) -> Result<(), String> {
        match self.state {
            OrderState::Draft => {
                self.state = OrderState::Confirmed;
                Ok(())
            }
            _ => Err("Only draft orders can be confirmed".to_string()),
        }
    }

    fn pay(&mut self) -> Result<(), String> {
        match self.state {
            OrderState::Confirmed => {
                self.state = OrderState::Paid;
                Ok(())
            }
            _ => Err("Only confirmed orders can be paid".to_string()),
        }
    }

    fn ship(&mut self) -> Result<(), String> {
        match self.state {
            OrderState::Paid => {
                self.state = OrderState::Shipped;
                Ok(())
            }
            _ => Err("Only paid orders can be shipped".to_string()),
        }
    }

    fn deliver(&mut self) -> Result<(), String> {
        match self.state {
            OrderState::Shipped => {
                self.state = OrderState::Delivered;
                Ok(())
            }
            _ => Err("Only shipped orders can be delivered".to_string()),
        }
    }

    fn cancel(&mut self, reason: String) -> Result<(), String> {
        match self.state {
            OrderState::Draft | OrderState::Confirmed => {
                self.state = OrderState::Cancelled(reason);
                Ok(())
            }
            _ => Err("Order cannot be cancelled in current state".to_string()),
        }
    }
}

fn main() {
    let mut order = Order {
        id: 1001,
        customer: "Alice".to_string(),
        total: 149.99,
        state: OrderState::Draft,
    };

    loop {
        println!(
            "\nOrder #{} | Customer: {} | Total: {:.2} | State: {}",
            order.id, order.customer, order.total, order.state
        );
        println!("Actions: confirm | pay | ship | deliver | cancel | quit");
        print!("> ");
        io::stdout().flush().expect("failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("failed to read action");
        let action = input.trim().to_lowercase();

        let result = match action.as_str() {
            "confirm" => order.confirm(),
            "pay" => order.pay(),
            "ship" => order.ship(),
            "deliver" => order.deliver(),
            "cancel" => {
                print!("Cancel reason: ");
                io::stdout().flush().expect("failed to flush stdout");
                let mut reason = String::new();
                io::stdin()
                    .read_line(&mut reason)
                    .expect("failed to read cancel reason");
                order.cancel(reason.trim().to_string())
            }
            "quit" => break,
            _ => {
                println!("Unknown action");
                continue;
            }
        };

        match result {
            Ok(()) => println!("Action succeeded. New state: {}", order.state),
            Err(err) => println!("Action failed: {err}"),
        }
    }
}

impl fmt::Display for OrderState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderState::Draft => write!(f, "Draft"),
            OrderState::Confirmed => write!(f, "Confirmed"),
            OrderState::Paid => write!(f, "Paid"),
            OrderState::Shipped => write!(f, "Shipped"),
            OrderState::Delivered => write!(f, "Delivered"),
            OrderState::Cancelled(reason) => write!(f, "Cancelled ({reason})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_order() -> Order {
        Order {
            id: 1,
            customer: "Test".to_string(),
            total: 10.0,
            state: OrderState::Draft,
        }
    }

    #[test]
    fn confirm_from_draft_succeeds() {
        let mut order = sample_order();
        assert!(order.confirm().is_ok());
        assert!(matches!(order.state, OrderState::Confirmed));
    }

    #[test]
    fn pay_from_confirmed_succeeds() {
        let mut order = sample_order();
        order.confirm().unwrap();
        assert!(order.pay().is_ok());
        assert!(matches!(order.state, OrderState::Paid));
    }

    #[test]
    fn ship_from_paid_succeeds() {
        let mut order = sample_order();
        order.confirm().unwrap();
        order.pay().unwrap();
        assert!(order.ship().is_ok());
        assert!(matches!(order.state, OrderState::Shipped));
    }

    #[test]
    fn deliver_from_shipped_succeeds() {
        let mut order = sample_order();
        order.confirm().unwrap();
        order.pay().unwrap();
        order.ship().unwrap();
        assert!(order.deliver().is_ok());
        assert!(matches!(order.state, OrderState::Delivered));
    }

    #[test]
    fn cancel_from_draft_succeeds() {
        let mut order = sample_order();
        assert!(order.cancel("changed mind".to_string()).is_ok());
        assert!(matches!(order.state, OrderState::Cancelled(_)));
    }

    #[test]
    fn cancel_from_confirmed_succeeds() {
        let mut order = sample_order();
        order.confirm().unwrap();
        assert!(order.cancel("stock issue".to_string()).is_ok());
        assert!(matches!(order.state, OrderState::Cancelled(_)));
    }

    #[test]
    fn cancel_from_paid_fails() {
        let mut order = sample_order();
        order.confirm().unwrap();
        order.pay().unwrap();
        assert!(order.cancel("too late".to_string()).is_err());
        assert!(matches!(order.state, OrderState::Paid));
    }

    #[test]
    fn duplicate_confirm_fails() {
        let mut order = sample_order();
        order.confirm().unwrap();
        assert!(order.confirm().is_err());
        assert!(matches!(order.state, OrderState::Confirmed));
    }
}