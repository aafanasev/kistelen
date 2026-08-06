//! `#[secret]` on a container masks every field it covers, and
//! `#[secret(skip)]` exempts one.

// Masked fields are read by nothing — the derive prints a mask instead of
// touching them — so the dead-code lint fires on the test structs here.
#![allow(dead_code)]

use kistelen::Secret;

#[test]
fn a_struct_level_attribute_masks_every_field() {
    #[derive(Secret)]
    #[secret]
    struct AuthToken {
        value: String,
        expires_at: u64,
    }

    let token = AuthToken {
        value: "eyJhbGci".to_string(),
        expires_at: 1_700_000_000,
    };

    assert_eq!(
        format!("{token:?}"),
        "AuthToken { value: ■■■, expires_at: ■■■ }",
    );
}

#[test]
fn field_names_survive_a_struct_level_attribute() {
    #[derive(Secret)]
    #[secret]
    struct Session {
        token: String,
    }

    let session = Session {
        token: "abc".to_string(),
    };

    assert!(format!("{session:?}").contains("token"));
}

#[test]
fn skip_exempts_a_field_from_the_struct_level_rule() {
    #[derive(Secret)]
    #[secret]
    struct Account {
        #[secret(skip)]
        id: u32,
        password: String,
        api_key: String,
    }

    let account = Account {
        id: 7,
        password: "hunter2".to_string(),
        api_key: "key".to_string(),
    };

    assert_eq!(
        format!("{account:?}"),
        "Account { id: 7, password: ■■■, api_key: ■■■ }",
    );
}

#[test]
fn a_redundant_field_attribute_under_a_blanket_rule_is_accepted() {
    #[derive(Secret)]
    #[secret]
    struct Account {
        #[secret]
        password: String,
    }

    let account = Account {
        password: "hunter2".to_string(),
    };

    assert_eq!(format!("{account:?}"), "Account { password: ■■■ }");
}

#[test]
fn a_struct_level_attribute_applies_to_tuple_fields() {
    #[derive(Secret)]
    #[secret]
    struct Pair(String, u8);

    let pair = Pair("secret".to_string(), 3);

    assert_eq!(format!("{pair:?}"), "Pair(■■■, ■■■)");
}
