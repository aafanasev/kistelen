//! `with` and `fixed`: masks that do not depend on the value.

#![allow(dead_code)]

use kistelen::Secret;

#[test]
fn with_replaces_the_mask_string() {
    #[derive(Secret)]
    struct Account {
        #[secret(with = "REDACTED")]
        password: String,
    }

    let account = Account {
        password: "hunter2".to_string(),
    };

    assert_eq!(format!("{account:?}"), "Account { password: REDACTED }");
}

#[test]
fn with_accepts_a_character_literal() {
    #[derive(Secret)]
    struct Account {
        #[secret(with = '*')]
        password: String,
    }

    let account = Account {
        password: "hunter2".to_string(),
    };

    assert_eq!(format!("{account:?}"), "Account { password: * }");
}

#[test]
fn fixed_prints_a_set_number_of_characters() {
    #[derive(Secret)]
    struct Card {
        #[secret(fixed = 3)]
        cvv: String,
    }

    let card = Card {
        cvv: "123".to_string(),
    };

    assert_eq!(format!("{card:?}"), "Card { cvv: ■■■ }");
}

#[test]
fn fixed_hides_the_length_of_the_value() {
    #[derive(Secret)]
    struct Account {
        #[secret(fixed = 8)]
        password: String,
    }

    let short = Account {
        password: "a".to_string(),
    };
    let long = Account {
        password: "a-considerably-longer-password".to_string(),
    };

    assert_eq!(format!("{short:?}"), format!("{long:?}"));
}

#[test]
fn fixed_combines_with_a_custom_character() {
    #[derive(Secret)]
    struct Account {
        #[secret(fixed = 4, with = 'x')]
        password: String,
    }

    let account = Account {
        password: "hunter2".to_string(),
    };

    assert_eq!(format!("{account:?}"), "Account { password: xxxx }");
}

#[test]
fn a_container_rule_carries_its_options_to_every_field() {
    #[derive(Secret)]
    #[secret(with = "-")]
    struct Session {
        token: String,
        refresh: String,
    }

    let session = Session {
        token: "a".to_string(),
        refresh: "b".to_string(),
    };

    assert_eq!(format!("{session:?}"), "Session { token: -, refresh: - }");
}

#[test]
fn a_field_rule_overrides_the_container_rule() {
    #[derive(Secret)]
    #[secret(with = "-")]
    struct Session {
        token: String,
        #[secret(with = "+")]
        refresh: String,
    }

    let session = Session {
        token: "a".to_string(),
        refresh: "b".to_string(),
    };

    assert_eq!(format!("{session:?}"), "Session { token: -, refresh: + }");
}

#[test]
fn modes_apply_inside_an_option() {
    #[derive(Secret)]
    struct Account {
        #[secret(with = "REDACTED")]
        password: Option<String>,
        #[secret(fixed = 2)]
        pin: Option<u32>,
    }

    let account = Account {
        password: Some("hunter2".to_string()),
        pin: None,
    };

    assert_eq!(
        format!("{account:?}"),
        "Account { password: Some(REDACTED), pin: None }",
    );
}

#[test]
fn a_custom_mask_still_applies_to_a_non_printable_type() {
    // `with` and `fixed` never read the value, so any type can carry them.
    #[derive(Secret)]
    struct Holder {
        #[secret(with = "gone")]
        values: Vec<String>,
    }

    let holder = Holder {
        values: vec!["a".to_string()],
    };

    assert_eq!(format!("{holder:?}"), "Holder { values: gone }");
}
