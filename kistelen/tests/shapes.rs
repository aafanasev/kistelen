//! Tuple structs, unit structs and enums.

#![allow(dead_code)]

use kistelen::Secret;

#[test]
fn tuple_struct_fields_can_be_masked_positionally() {
    #[derive(Secret)]
    struct Credentials(String, #[secret] String);

    let credentials = Credentials("alice".to_string(), "hunter2".to_string());

    assert_eq!(format!("{credentials:?}"), r#"Credentials("alice", ■■■)"#);
}

#[test]
fn a_unit_struct_prints_its_name() {
    #[derive(Secret)]
    struct Anonymous;

    assert_eq!(format!("{:?}", Anonymous), "Anonymous");
}

#[test]
fn enum_variants_mask_their_own_fields() {
    #[derive(Secret)]
    enum Credential {
        Password {
            login: String,
            #[secret]
            value: String,
        },
        Token(#[secret] String),
        Anonymous,
    }

    let password = Credential::Password {
        login: "alice".to_string(),
        value: "hunter2".to_string(),
    };
    let token = Credential::Token("t0ken".to_string());

    assert_eq!(
        format!("{password:?}"),
        r#"Password { login: "alice", value: ■■■ }"#,
    );
    assert_eq!(format!("{token:?}"), "Token(■■■)");
    assert_eq!(format!("{:?}", Credential::Anonymous), "Anonymous");
}

#[test]
fn a_variant_level_attribute_masks_that_variant_only() {
    #[derive(Secret)]
    enum Message {
        #[secret]
        Private {
            body: String,
            recipient: String,
        },
        Public {
            body: String,
        },
    }

    let private = Message::Private {
        body: "meet at noon".to_string(),
        recipient: "bob".to_string(),
    };
    let public = Message::Public {
        body: "hello".to_string(),
    };

    assert_eq!(
        format!("{private:?}"),
        "Private { body: ■■■, recipient: ■■■ }",
    );
    assert_eq!(format!("{public:?}"), r#"Public { body: "hello" }"#);
}

#[test]
fn an_enum_level_attribute_covers_every_variant() {
    #[derive(Secret)]
    #[secret]
    enum Secrets {
        Named { value: String },
        Positional(u8),
    }

    assert_eq!(
        format!(
            "{:?}",
            Secrets::Named {
                value: "x".to_string()
            }
        ),
        "Named { value: ■■■ }",
    );
    assert_eq!(format!("{:?}", Secrets::Positional(1)), "Positional(■■■)");
}

#[test]
fn skip_works_inside_an_enum_variant() {
    #[derive(Secret)]
    #[secret]
    enum Event {
        Login {
            #[secret(skip)]
            at: u64,
            token: String,
        },
    }

    let event = Event::Login {
        at: 42,
        token: "t".to_string(),
    };

    assert_eq!(format!("{event:?}"), "Login { at: 42, token: ■■■ }");
}

#[test]
fn an_empty_enum_compiles() {
    #[derive(Secret)]
    enum Never {}

    // Nothing to assert: the value cannot be constructed. Compiling is the test.
    fn _accepts(_: &Never) {}
}
