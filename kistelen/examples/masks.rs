//! Choosing how a value is masked: `with`, `fixed` and `partial`.
//!
//! Run with: `cargo run --example masks`

// A masked field is read by nothing — the derive prints a mask rather than
// touching it — so the dead-code lint fires on these example types.
#![allow(dead_code)]

use kistelen::Secret;

#[derive(Secret)]
struct Card {
    #[secret(with = "REDACTED")]
    holder: String,

    // `fixed` hides the length as well as the contents, which matters for a
    // value as short as a CVV: a mask tracking the real length would say how
    // many digits to guess.
    #[secret(fixed = 3)]
    cvv: String,

    // `partial` keeps the last digits legible, which is what makes a card
    // identifiable to its owner without being usable.
    #[secret(partial)]
    number: String,

    #[secret(partial, with = '*')]
    account: String,
}

#[derive(Secret)]
struct ShortValues {
    // Below eight characters `partial` exposes nothing and falls back to a
    // fixed-width mask, so short values cannot be narrowed and their length
    // is not disclosed either.
    #[secret(partial)]
    pin: String,
}

fn main() {
    let card = Card {
        holder: "Alice Smith".to_string(),
        cvv: "123".to_string(),
        number: "1234567890123456".to_string(),
        account: "GB29NWBK60161331926819".to_string(),
    };

    println!("{card:#?}");

    let short = ShortValues {
        pin: "4021".to_string(),
    };

    println!("{short:?}");
}
