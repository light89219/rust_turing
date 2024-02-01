use std::fmt;

trait Reportable {
    fn id(&self) -> u32;
    fn label(&self) -> String;
    fn severity(&self) -> u8;
}

#[derive(Debug)]
struct Invoice {
    id: u32,
    customer: String,
    total: f64,
}

#[derive(Debug)]
struct Ticket {
    id: u32,
    title: String,
    priority: u8,
}

impl fmt::Display for Invoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invoice {} — {} (${:.2})",
            self.id, self.customer, self.total
        )
    }
}

impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ticket {} — {} [P{}]", self.id, self.title, self.priority)
    }
}

#[derive(Debug, Default)]
struct Repository<T> {
    items: Vec<T>,
}

impl<T> Repository<T> {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn add(&mut self, item: T) {
        self.items.push(item);
    }

    fn as_slice(&self) -> &[T] {
        &self.items
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

impl<T: Reportable> Repository<T> {
    fn highest_severity_item(&self) -> Option<&T> {
        highest_severity(self.as_slice())
    }
}

impl Reportable for Invoice {
    fn id(&self) -> u32 {
        self.id
    }

    fn label(&self) -> String {
        format!("Invoice for {} (${:.2})", self.customer, self.total)
    }

    fn severity(&self) -> u8 {
        if self.total > 10_000.0 {
            9
        } else {
            4
        }
    }
}

impl Reportable for Ticket {
    fn id(&self) -> u32 {
        self.id
    }

    fn label(&self) -> String {
        format!("Ticket: {}", self.title)
    }

    fn severity(&self) -> u8 {
        self.priority
    }
}

fn format_line<T: Reportable>(item: &T) -> String {
    format!("#{} [{}] {}", item.id(), item.severity(), item.label())
}

fn format_all<T: Reportable>(items: &[T]) -> Vec<String> {
    items.iter().map(|item| format_line(item)).collect()
}

fn highest_severity<T: Reportable>(items: &[T]) -> Option<&T> {
    items.iter().max_by_key(|x| x.severity())
}

/// `impl Trait` in **argument** position: accept any concrete type that implements `Reportable`.
fn print_reportable(item: &impl Reportable) {
    println!("(impl Trait arg) {}", item.label());
}

/// `impl Trait` in **return** position: callers see `impl Reportable`, implementation is one concrete type.
fn example_high_value_invoice() -> impl Reportable {
    Invoice {
        id: 99,
        customer: String::from("Stretch Co"),
        total: 20_000.0,
    }
}

fn main() {
    let invoices = vec![
        Invoice {
            id: 1,
            customer: String::from("Acme Corp"),
            total: 500.0,
        },
        Invoice {
            id: 2,
            customer: String::from("BigCo"),
            total: 15_000.0,
        },
    ];

    let tickets = vec![
        Ticket {
            id: 101,
            title: String::from("Login broken"),
            priority: 8,
        },
        Ticket {
            id: 102,
            title: String::from("Typo in footer"),
            priority: 2,
        },
    ];

    println!("--- Invoice lines ---");
    for line in format_all(&invoices) {
        println!("{line}");
    }

    println!("\n--- Ticket lines ---");
    for line in format_all(&tickets) {
        println!("{line}");
    }

    if let Some(top) = highest_severity(&invoices) {
        println!("\nHighest-severity invoice: {}", format_line(top));
        println!("Same value via Display: {top}");
    }
    if let Some(top) = highest_severity(&tickets) {
        println!("Highest-severity ticket: {}", format_line(top));
        println!("Same value via Display: {top}");
    }

    print_reportable(&invoices[0]);
    let stretch = example_high_value_invoice();
    println!(
        "Returned impl Reportable: id={} severity={}",
        stretch.id(),
        stretch.severity()
    );

    let mut invoice_repo = Repository::new();
    invoice_repo.add(Invoice {
        id: 10,
        customer: String::from("Repo Small"),
        total: 100.0,
    });
    invoice_repo.add(Invoice {
        id: 11,
        customer: String::from("Repo Huge"),
        total: 50_000.0,
    });
    println!(
        "\nRepository<Invoice> len={}; top by severity: {:?}",
        invoice_repo.len(),
        invoice_repo
            .highest_severity_item()
            .map(|i| format!("{} (sev {})", i, i.severity()))
    );

    let mixed: Vec<Box<dyn Reportable>> = vec![
        Box::new(Invoice {
            id: 3,
            customer: String::from("Dyn LLC"),
            total: 99.0,
        }),
        Box::new(Ticket {
            id: 103,
            title: String::from("Mixed queue item"),
            priority: 5,
        }),
    ];

    println!("\n--- Mixed (dyn) labels and severities ---");
    for item in &mixed {
        println!("label: {} | severity: {}", item.label(), item.severity());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highest_severity_invoice_picks_higher_severity_band() {
        let items = [
            Invoice {
                id: 1,
                customer: String::from("Low"),
                total: 100.0,
            },
            Invoice {
                id: 2,
                customer: String::from("High"),
                total: 15_000.0,
            },
        ];
        let top = highest_severity(&items).expect("non-empty");
        assert_eq!(top.id, 2);
        assert_eq!(top.severity(), 9);
    }

    #[test]
    fn highest_severity_ticket_picks_max_priority() {
        let items = [
            Ticket {
                id: 10,
                title: String::from("Minor"),
                priority: 1,
            },
            Ticket {
                id: 20,
                title: String::from("Urgent"),
                priority: 9,
            },
            Ticket {
                id: 30,
                title: String::from("Medium"),
                priority: 5,
            },
        ];
        let top = highest_severity(&items).expect("non-empty");
        assert_eq!(top.id, 20);
        assert_eq!(top.severity(), 9);
    }

    #[test]
    fn repository_highest_severity_delegates() {
        let mut repo = Repository::new();
        repo.add(Ticket {
            id: 1,
            title: String::from("a"),
            priority: 2,
        });
        repo.add(Ticket {
            id: 2,
            title: String::from("b"),
            priority: 7,
        });
        let top = repo.highest_severity_item().expect("non-empty");
        assert_eq!(top.id, 2);
    }
}
