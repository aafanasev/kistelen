//! Rewriting a value with a pattern. Needs the `regex` feature.
//!
//! Run with: `cargo run --example patterns --features regex`

use kistelen::Secret;

#[derive(Secret)]
struct Payment {
    #[secret(
        search = r"([0-9]{4})([0-9]{8})([0-9]{4})",
        replacement = "****-****-****-$3"
    )]
    card: String,

    #[secret(search = r"[^@]+@(.+)", replacement = "***@$1")]
    email: String,
}

fn main() {
    let payment = Payment {
        card: "1234567890123456".to_string(),
        email: "alice@example.com".to_string(),
    };

    println!("{payment:?}");

    // A pattern must describe the whole value. This card number carries
    // surrounding text the pattern does not account for, so rather than
    // rewriting the digits and printing the rest verbatim, the value is
    // masked entirely.
    let unexpected = Payment {
        card: "card:1234567890123456".to_string(),
        email: "not-an-address".to_string(),
    };

    println!("{unexpected:?}");
}
