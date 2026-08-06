//! `Option` fields keep their `Some`/`None` shape while the value is masked.

#![allow(dead_code)]

use kistelen::Secret;

#[derive(Secret)]
struct Account {
    #[secret]
    password: Option<String>,
}

#[test]
fn a_present_value_is_masked_inside_some() {
    let account = Account {
        password: Some("hunter2".to_string()),
    };

    assert_eq!(format!("{account:?}"), "Account { password: Some(■■■) }");
}

#[test]
fn an_absent_value_prints_as_none() {
    let account = Account { password: None };

    assert_eq!(format!("{account:?}"), "Account { password: None }");
}

#[test]
fn the_contained_value_never_appears() {
    let account = Account {
        password: Some("hunter2".to_string()),
    };

    assert!(!format!("{account:?}").contains("hunter2"));
}

#[test]
fn a_fully_qualified_option_is_recognised() {
    #[derive(Secret)]
    struct Qualified {
        #[secret]
        value: std::option::Option<u8>,
    }

    let qualified = Qualified { value: Some(1) };

    assert_eq!(format!("{qualified:?}"), "Qualified { value: Some(■■■) }");
}

#[test]
fn an_aliased_option_is_masked_whole() {
    // The macro matches on the written type, so an alias cannot be seen
    // through. Masking the whole value is the safe reading of an unknown type.
    type MaybeSecret = Option<String>;

    #[derive(Secret)]
    struct Aliased {
        #[secret]
        value: MaybeSecret,
    }

    let aliased = Aliased {
        value: Some("hunter2".to_string()),
    };

    assert_eq!(format!("{aliased:?}"), "Aliased { value: ■■■ }");
}

#[test]
fn an_unmasked_option_is_left_alone() {
    #[derive(Secret)]
    struct Plain {
        value: Option<u8>,
    }

    let plain = Plain { value: Some(3) };

    assert_eq!(format!("{plain:?}"), "Plain { value: Some(3) }");
}
