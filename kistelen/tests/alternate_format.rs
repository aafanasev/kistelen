//! The pretty-printing form, `{:#?}`, must indent like the standard derive.

#![allow(dead_code)]

use kistelen::Secret;

#[test]
fn a_struct_is_pretty_printed_across_lines() {
    #[derive(Secret)]
    struct User {
        id: i32,
        #[secret]
        password: String,
    }

    let user = User {
        id: 1,
        password: "hunter2".to_string(),
    };

    assert_eq!(
        format!("{user:#?}"),
        "User {\n    id: 1,\n    password: ■■■,\n}",
    );
}

#[test]
fn a_tuple_struct_is_pretty_printed_across_lines() {
    #[derive(Secret)]
    struct Pair(u8, #[secret] String);

    let pair = Pair(1, "secret".to_string());

    assert_eq!(format!("{pair:#?}"), "Pair(\n    1,\n    ■■■,\n)");
}

#[test]
fn a_masked_option_is_pretty_printed_across_lines() {
    #[derive(Secret)]
    struct Account {
        #[secret]
        password: Option<String>,
    }

    let account = Account {
        password: Some("hunter2".to_string()),
    };

    assert_eq!(
        format!("{account:#?}"),
        "Account {\n    password: Some(\n        ■■■,\n    ),\n}",
    );
}

#[test]
fn nesting_indents_consistently() {
    #[derive(Secret)]
    struct Inner {
        #[secret]
        value: String,
    }

    #[derive(Secret)]
    struct Outer {
        inner: Inner,
    }

    let outer = Outer {
        inner: Inner {
            value: "x".to_string(),
        },
    };

    assert_eq!(
        format!("{outer:#?}"),
        "Outer {\n    inner: Inner {\n        value: ■■■,\n    },\n}",
    );
}
