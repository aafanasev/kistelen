//! `search` and `replacement`: masking shapes the other modes cannot express.
//!
//! The safety property under test is that anything the pattern does not fully
//! describe falls back to the mask. A partial match would leave the rest of
//! the value untouched, and the rule would look like it was working.

#![cfg(feature = "regex")]
#![allow(dead_code)]

use kistelen::Secret;

#[derive(Secret)]
struct Card {
    #[secret(
        search = r"([0-9]{4})([0-9]{8})([0-9]{4})",
        replacement = "****-****-****-$3"
    )]
    number: String,
}

fn rendered(number: &str) -> String {
    format!(
        "{:?}",
        Card {
            number: number.to_string(),
        }
    )
    .trim_start_matches("Card { number: ")
    .trim_end_matches(" }")
    .to_string()
}

#[test]
fn a_matching_value_is_rewritten() {
    assert_eq!(rendered("1234567890123456"), "****-****-****-3456");
}

#[test]
fn a_value_that_does_not_match_falls_back_to_the_mask() {
    assert_eq!(rendered("not-a-card-number"), "■■■");
}

#[test]
fn a_value_only_partly_matching_falls_back_to_the_mask() {
    // The digits alone would match, but the surrounding text would survive
    // into the output untouched.
    let output = rendered("card:1234567890123456:end");

    assert_eq!(output, "■■■");
    assert!(!output.contains("card"));
    assert!(!output.contains("3456"));
}

#[test]
fn an_empty_value_falls_back_to_the_mask() {
    assert_eq!(rendered(""), "■■■");
}

#[test]
fn a_pattern_can_expose_a_chosen_group() {
    #[derive(Secret)]
    struct Email {
        #[secret(search = r"[^@]+@(.+)", replacement = "***@$1")]
        address: String,
    }

    let email = Email {
        address: "alice@example.com".to_string(),
    };

    assert_eq!(format!("{email:?}"), "Email { address: ***@example.com }",);
}

#[test]
fn replacement_applies_inside_an_option() {
    #[derive(Secret)]
    struct Account {
        #[secret(search = r"(\d{4})\d+", replacement = "$1…")]
        card: Option<String>,
    }

    let present = Account {
        card: Some("12345678".to_string()),
    };
    let absent = Account { card: None };

    assert_eq!(format!("{present:?}"), "Account { card: Some(1234…) }");
    assert_eq!(format!("{absent:?}"), "Account { card: None }");
}

#[test]
fn a_pattern_is_compiled_once_and_reused() {
    // Repeated formatting must stay correct; the cache is shared across calls
    // to `fmt` for the same field.
    for _ in 0..3 {
        assert_eq!(rendered("1234567890123456"), "****-****-****-3456");
    }
}

#[test]
fn replacement_applies_to_a_displayable_non_string_type() {
    #[derive(Secret)]
    struct Account {
        #[secret(search = r"\d{4}(\d{4})", replacement = "****$1")]
        number: u64,
    }

    let account = Account { number: 12345678 };

    assert_eq!(format!("{account:?}"), "Account { number: ****5678 }");
}
