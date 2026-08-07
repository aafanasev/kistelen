//! Masking individual fields — the smallest useful thing the macro does.
//!
//! Run with: `cargo run --example fields`

// A masked field is read by nothing — the derive prints a mask rather than
// touching it — so the dead-code lint fires on these example types.
#![allow(dead_code)]

use kistelen::Secret;

#[derive(Secret)]
struct User {
    id: i32,
    username: String,
    #[secret]
    password: String,
}

fn main() {
    let user = User {
        id: 1,
        username: "alice".to_string(),
        password: "hunter2".to_string(),
    };

    // Unmasked fields print exactly as the standard derive would.
    println!("{user:?}");

    // The pretty-printed form works too.
    println!("{user:#?}");
}
