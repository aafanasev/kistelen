//! Masking a whole type, and exempting one field from the rule.
//!
//! Run with: `cargo run --example whole_types`

// A masked field is read by nothing — the derive prints a mask rather than
// touching it — so the dead-code lint fires on these example types.
#![allow(dead_code)]

use kistelen::Secret;

#[derive(Secret)]
#[secret]
struct Session {
    // Not sensitive, and useful for correlating log lines.
    #[secret(skip)]
    id: u32,
    token: String,
    refresh_token: String,
}

#[derive(Secret)]
enum Credential {
    Anonymous,
    Token(#[secret] String),
    // The whole variant is sensitive, including who it belongs to.
    #[secret]
    Password {
        login: String,
        value: String,
    },
}

fn main() {
    let session = Session {
        id: 7,
        token: "eyJhbGci".to_string(),
        refresh_token: "r-eyJhbGci".to_string(),
    };

    println!("{session:?}");

    for credential in [
        Credential::Anonymous,
        Credential::Token("t0ken".to_string()),
        Credential::Password {
            login: "alice".to_string(),
            value: "hunter2".to_string(),
        },
    ] {
        println!("{credential:?}");
    }
}
