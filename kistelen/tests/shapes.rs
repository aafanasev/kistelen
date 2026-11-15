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

#[test]
#[deny(non_snake_case)]
fn enum_field_names_cannot_shadow_the_generated_code() {
    // Fields named after identifiers the generated `fmt` uses, read and
    // discarded alike. Bound under their own names, `formatter` would shadow
    // the formatter parameter and the derive would fail to compile.
    //
    // The `deny` covers the generated impl: renaming a binding must not
    // raise a lint the caller has allowed on their own fields.
    #[derive(Secret)]
    #[allow(non_snake_case)]
    enum Shadowing {
        Plain {
            formatter: u8,
            __field0: u8,
            __PATTERN0: u8,
            __self_formatter: u8,
        },
        Masked {
            #[secret]
            formatter: String,
            #[secret]
            __field0: String,
        },
        Read {
            #[secret]
            formatter: Option<String>,
            #[secret(partial, with = '*')]
            __self_formatter: String,
            __field1: u8,
        },
    }

    let plain = Shadowing::Plain {
        formatter: 1,
        __field0: 2,
        __PATTERN0: 3,
        __self_formatter: 4,
    };
    let masked = Shadowing::Masked {
        formatter: "hunter2".to_string(),
        __field0: "t0ken".to_string(),
    };
    let read = Shadowing::Read {
        formatter: Some("hunter2".to_string()),
        __self_formatter: "1234567890123456".to_string(),
        __field1: 5,
    };

    assert_eq!(
        format!("{plain:?}"),
        "Plain { formatter: 1, __field0: 2, __PATTERN0: 3, __self_formatter: 4 }",
    );
    assert_eq!(
        format!("{masked:?}"),
        "Masked { formatter: ■■■, __field0: ■■■ }",
    );
    assert_eq!(
        format!("{read:?}"),
        "Read { formatter: Some(■■■), __self_formatter: 123**********456, __field1: 5 }",
    );
}

#[test]
fn raw_identifier_enum_fields_are_bound() {
    // How a raw name is printed is a separate concern; this only checks that
    // the binding built from it is a valid identifier and reads the value.
    #[derive(Secret)]
    enum Item {
        Typed {
            r#type: u8,
            #[secret]
            r#match: String,
        },
    }

    let item = Item::Typed {
        r#type: 7,
        r#match: "hunter2".to_string(),
    };
    let output = format!("{item:?}");

    assert!(output.starts_with("Typed { "), "{output}");
    assert!(output.contains("type: 7, "), "{output}");
    assert!(output.ends_with("match: ■■■ }"), "{output}");
}
