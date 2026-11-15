// Masked fields are read by nothing — the derive prints a mask instead of
// touching them — so the dead-code lint fires on every test struct here.
#![allow(dead_code)]

use kistelen::{Secret, MASK};

#[test]
fn annotated_field_is_replaced_by_the_mask() {
    #[derive(Secret)]
    struct Credentials {
        login: String,
        #[secret]
        password: String,
    }

    let credentials = Credentials {
        login: "alice".to_string(),
        password: "hunter2".to_string(),
    };

    assert_eq!(
        format!("{credentials:?}"),
        r#"Credentials { login: "alice", password: ■■■ }"#,
    );
}

#[test]
fn unannotated_fields_keep_their_own_debug_output() {
    #[derive(Debug)]
    struct Nested {
        value: u8,
    }

    #[derive(Secret)]
    struct Wrapper {
        nested: Nested,
        numbers: Vec<i32>,
        #[secret]
        token: String,
    }

    let wrapper = Wrapper {
        nested: Nested { value: 7 },
        numbers: vec![1, 2],
        token: "t0ken".to_string(),
    };

    assert_eq!(
        format!("{wrapper:?}"),
        "Wrapper { nested: Nested { value: 7 }, numbers: [1, 2], token: ■■■ }",
    );
}

#[test]
fn the_secret_value_never_appears_in_the_output() {
    #[derive(Secret)]
    struct Account {
        #[secret]
        api_key: String,
    }

    let account = Account {
        api_key: "super-secret-key".to_string(),
    };

    assert!(!format!("{account:?}").contains("super-secret-key"));
}

#[test]
fn every_field_can_be_masked() {
    #[derive(Secret)]
    struct Token {
        #[secret]
        value: String,
        #[secret]
        refresh: String,
    }

    let token = Token {
        value: "a".to_string(),
        refresh: "b".to_string(),
    };

    assert_eq!(format!("{token:?}"), "Token { value: ■■■, refresh: ■■■ }");
}

#[test]
fn masking_is_independent_of_the_field_type() {
    #[derive(Secret)]
    struct Mixed {
        #[secret]
        pin: u32,
        #[secret]
        recovery_codes: Vec<String>,
    }

    let mixed = Mixed {
        pin: 1234,
        recovery_codes: vec!["one".to_string()],
    };

    assert_eq!(
        format!("{mixed:?}"),
        "Mixed { pin: ■■■, recovery_codes: ■■■ }",
    );
}

#[test]
fn generic_structs_are_supported() {
    // No bound written here: the derive works one out per field. The wider
    // behaviour lives in `generics.rs`.
    #[derive(Secret)]
    struct Envelope<T> {
        payload: T,
        #[secret]
        signature: String,
    }

    let envelope = Envelope {
        payload: 42,
        signature: "sig".to_string(),
    };

    assert_eq!(
        format!("{envelope:?}"),
        "Envelope { payload: 42, signature: ■■■ }",
    );
}

#[test]
fn a_struct_without_secrets_behaves_like_the_standard_derive() {
    #[derive(Secret)]
    struct Public {
        id: i32,
        name: String,
    }

    let public = Public {
        id: 1,
        name: "kistelen".to_string(),
    };

    assert_eq!(
        format!("{public:?}"),
        r#"Public { id: 1, name: "kistelen" }"#
    );
}

#[test]
fn the_mask_constant_matches_what_is_printed() {
    #[derive(Secret)]
    struct Item {
        #[secret]
        value: String,
    }

    let item = Item {
        value: "x".to_string(),
    };

    assert_eq!(format!("{item:?}"), format!("Item {{ value: {MASK} }}"));
}

// Raw identifiers must print without their `r#` prefix, exactly like the
// standard derive. Each type has a `#[derive(Debug)]` twin with the same name
// in a sibling module so the outputs can be compared verbatim.
#[allow(non_camel_case_types)]
mod raw {
    pub mod ours {
        use kistelen::Secret;

        #[derive(Secret)]
        pub struct r#match {
            pub r#type: u8,
            pub r#fn: &'static str,
        }

        #[derive(Secret)]
        pub enum Token {
            r#match { r#type: u8 },
            r#loop(u8),
            r#struct,
        }

        #[derive(Secret)]
        pub struct Masked {
            pub r#type: u8,
            #[secret]
            pub r#ref: &'static str,
        }
    }

    pub mod standard {
        #[derive(Debug)]
        pub struct r#match {
            pub r#type: u8,
            pub r#fn: &'static str,
        }

        #[derive(Debug)]
        pub enum Token {
            r#match { r#type: u8 },
            r#loop(u8),
            r#struct,
        }

        #[derive(Debug)]
        pub struct Masked {
            pub r#type: u8,
            pub r#ref: &'static str,
        }
    }
}

#[test]
fn raw_identifier_struct_and_field_names_match_the_standard_derive() {
    let ours = raw::ours::r#match {
        r#type: 1,
        r#fn: "f",
    };
    let standard = raw::standard::r#match {
        r#type: 1,
        r#fn: "f",
    };

    assert_eq!(format!("{ours:?}"), format!("{standard:?}"));
    assert_eq!(format!("{ours:#?}"), format!("{standard:#?}"));
    assert_eq!(format!("{ours:?}"), r#"match { type: 1, fn: "f" }"#);
}

#[test]
fn raw_identifier_variant_names_match_the_standard_derive() {
    let pairs = [
        (
            raw::ours::Token::r#match { r#type: 2 },
            raw::standard::Token::r#match { r#type: 2 },
        ),
        (raw::ours::Token::r#loop(3), raw::standard::Token::r#loop(3)),
        (raw::ours::Token::r#struct, raw::standard::Token::r#struct),
    ];

    for (ours, standard) in &pairs {
        assert_eq!(format!("{ours:?}"), format!("{standard:?}"));
        assert_eq!(format!("{ours:#?}"), format!("{standard:#?}"));
    }
    assert_eq!(format!("{:?}", pairs[0].0), "match { type: 2 }");
}

#[test]
fn masked_raw_identifier_field_prints_its_unprefixed_name() {
    let ours = raw::ours::Masked {
        r#type: 4,
        r#ref: "hunter2",
    };
    // The standard twin prints the real value; swapping it for the mask gives
    // the expected output.
    let standard = raw::standard::Masked {
        r#type: 4,
        r#ref: "hunter2",
    };

    assert_eq!(
        format!("{ours:?}"),
        format!("{standard:?}").replace(r#""hunter2""#, MASK),
    );
    assert_eq!(
        format!("{ours:#?}"),
        format!("{standard:#?}").replace(r#""hunter2""#, MASK),
    );
    assert_eq!(
        format!("{ours:?}"),
        format!("Masked {{ type: 4, ref: {MASK} }}")
    );
}
