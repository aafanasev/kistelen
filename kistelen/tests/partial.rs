//! `partial`: some of the value exposed, the rest masked.
//!
//! The safety guard is the point of these tests. A rule that exposes part of
//! a value is only defensible while the exposed part stays small relative to
//! the whole, so short values must fall back to being masked entirely.

#![allow(dead_code)]

use kistelen::{Secret, MINIMUM_PARTIAL_LENGTH};

#[derive(Secret)]
struct Card {
    #[secret(partial)]
    number: String,
}

fn masked(number: &str) -> String {
    format!(
        "{:?}",
        Card {
            number: number.to_string(),
        }
    )
}

/// Just the rendered field, so assertions cannot be satisfied by the
/// surrounding struct and field names.
fn rendered(number: &str) -> String {
    masked(number)
        .trim_start_matches("Card { number: ")
        .trim_end_matches(" }")
        .to_string()
}

#[test]
fn a_long_value_exposes_both_ends() {
    assert_eq!(
        masked("1234567890123456"),
        "Card { number: 123■■■■■■■■■■456 }"
    );
}

#[test]
fn a_short_value_is_masked_entirely() {
    assert_eq!(rendered("hunter2"), "■■■");
}

#[test]
fn nothing_of_a_short_value_survives_at_any_length_below_the_guard() {
    for length in 0..MINIMUM_PARTIAL_LENGTH {
        let value = "a".repeat(length);

        assert!(
            !rendered(&value).contains('a'),
            "a value of {length} characters exposed part of itself",
        );
    }
}

#[test]
fn a_short_value_does_not_disclose_its_length() {
    // Every value below the guard renders identically, including an empty
    // one, so the output says nothing about how long the value was.
    let outputs: Vec<String> = (0..MINIMUM_PARTIAL_LENGTH)
        .map(|length| rendered(&"a".repeat(length)))
        .collect();

    assert!(
        outputs.windows(2).all(|pair| pair[0] == pair[1]),
        "short values rendered differently from one another: {outputs:?}",
    );
}

#[test]
fn an_empty_value_still_renders_a_mask() {
    assert_eq!(rendered(""), "■■■");
}

#[test]
fn exposure_never_exceeds_the_cap() {
    // A long value exposes at most four characters at each end, so the
    // proportion revealed keeps falling as the value grows.
    let value = "x".repeat(200);
    let output = masked(&value);
    let exposed = output.chars().filter(|character| *character == 'x').count();

    assert_eq!(exposed, 8);
}

#[test]
fn the_value_length_is_preserved() {
    // Preserving length is a deliberate disclosure; `fixed` is the mode that
    // hides it. This test pins the behaviour so a change is a decision.
    assert_eq!(rendered("1234567890").chars().count(), 10);
}

#[test]
fn multi_byte_characters_are_counted_not_split() {
    #[derive(Secret)]
    struct Note {
        #[secret(partial)]
        text: String,
    }

    let note = Note {
        text: "кистэлэҥ туһунан".to_string(),
    };

    let output = format!("{note:?}");
    let rendered = output
        .trim_start_matches("Note { text: ")
        .trim_end_matches(" }");

    assert_eq!(rendered.chars().count(), "кистэлэҥ туһунан".chars().count());
    assert!(rendered.starts_with("кис"));
}

#[test]
fn partial_accepts_a_custom_character() {
    #[derive(Secret)]
    struct Card {
        #[secret(partial, with = '*')]
        number: String,
    }

    let card = Card {
        number: "1234567890123456".to_string(),
    };

    assert_eq!(format!("{card:?}"), "Card { number: 123**********456 }");
}

#[test]
fn partial_applies_to_a_non_string_type_that_can_be_displayed() {
    #[derive(Secret)]
    struct Account {
        #[secret(partial)]
        number: u64,
    }

    let account = Account {
        number: 1234567890123456,
    };

    assert_eq!(
        format!("{account:?}"),
        "Account { number: 123■■■■■■■■■■456 }"
    );
}

#[test]
fn partial_reaches_inside_an_option() {
    #[derive(Secret)]
    struct Account {
        #[secret(partial)]
        card: Option<String>,
    }

    let present = Account {
        card: Some("1234567890123456".to_string()),
    };
    let absent = Account { card: None };

    assert_eq!(
        format!("{present:?}"),
        "Account { card: Some(123■■■■■■■■■■456) }",
    );
    assert_eq!(format!("{absent:?}"), "Account { card: None }");
}
