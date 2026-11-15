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
fn an_escaped_dollar_before_zero_is_a_literal_not_the_whole_match() {
    // `$$` is a literal `$`, so `$$0` renders the text `$0` rather than
    // restoring the matched value.
    #[derive(Secret)]
    struct Card {
        #[secret(search = r"(\d+)", replacement = "$$0")]
        number: String,
    }

    let card = Card {
        number: "4111222233334444".to_string(),
    };
    let output = format!("{card:?}");

    assert_eq!(output, "Card { number: $0 }");
    assert!(!output.contains("4111"));
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

#[test]
#[deny(non_snake_case)]
fn enum_field_names_cannot_shadow_the_pattern_cache() {
    #[derive(Secret)]
    #[allow(non_snake_case)]
    enum Contact {
        Email {
            __PATTERN0: u8,
            #[secret(search = r"[^@]+@(.+)", replacement = "***@$1")]
            formatter: String,
        },
        Pager(
            #[secret(search = r"[^@]+@(.+)", replacement = "***@$1")] String,
            u8,
        ),
    }

    let email = Contact::Email {
        __PATTERN0: 1,
        formatter: "alice@example.com".to_string(),
    };
    let pager = Contact::Pager("bob@example.org".to_string(), 2);

    assert_eq!(
        format!("{email:?}"),
        "Email { __PATTERN0: 1, formatter: ***@example.com }",
    );
    assert_eq!(format!("{pager:?}"), "Pager(***@example.org, 2)");
}

#[test]
fn a_lazy_quantifier_still_matches_the_whole_value() {
    // Unanchored, the lazy group would stop after one digit, and the value
    // would be masked despite the pattern describing all of it.
    #[derive(Secret)]
    struct Code {
        #[secret(search = r"(\d+?)", replacement = "<$1>")]
        value: String,
    }

    let code = Code {
        value: "123".to_string(),
    };

    assert_eq!(format!("{code:?}"), "Code { value: <123> }");
}

#[test]
fn a_later_alternative_can_match_the_whole_value() {
    // Unanchored, the first alternative would win with a match covering only
    // the first two digits.
    #[derive(Secret)]
    struct Code {
        #[secret(search = r"\d{2}|\d{2}-\d{2}", replacement = "**-**")]
        value: String,
    }

    let code = Code {
        value: "12-34".to_string(),
    };

    assert_eq!(format!("{code:?}"), "Code { value: **-** }");
}

#[test]
fn a_pattern_that_only_describes_part_of_the_value_still_masks() {
    #[derive(Secret)]
    struct Code {
        #[secret(search = r"\d{2}|\d{2}-\d{2}", replacement = "**-**")]
        value: String,
    }

    let code = Code {
        value: "12-34-56".to_string(),
    };

    assert_eq!(format!("{code:?}"), "Code { value: ■■■ }");
}

#[test]
fn a_trailing_comment_under_verbose_mode_cannot_escape_the_anchors() {
    // A `#` comment runs to the end of the line, so it would swallow the
    // closing anchor were it not ended first.
    #[derive(Secret)]
    struct Code {
        #[secret(
            search = r"(?x) (\d{2}) \d{2} # keep the first two",
            replacement = "$1**"
        )]
        value: String,
    }

    let whole = Code {
        value: "1234".to_string(),
    };
    let longer = Code {
        value: "123456".to_string(),
    };
    let surrounded = Code {
        value: "x1234".to_string(),
    };

    assert_eq!(format!("{whole:?}"), "Code { value: 12** }");
    assert_eq!(format!("{longer:?}"), "Code { value: ■■■ }");
    assert_eq!(format!("{surrounded:?}"), "Code { value: ■■■ }");
}

#[test]
fn multi_line_mode_does_not_loosen_the_anchors() {
    // Under `(?m)`, `^` and `$` would match at line breaks, letting one line
    // of a multi-line value stand for all of it.
    #[derive(Secret)]
    struct Code {
        #[secret(search = r"(?m)\d+", replacement = "***")]
        value: String,
    }

    let code = Code {
        value: "123\nsecret".to_string(),
    };

    assert_eq!(format!("{code:?}"), "Code { value: ■■■ }");
}

#[test]
fn a_braced_group_can_be_followed_by_letters() {
    #[derive(Secret)]
    struct Code {
        #[secret(search = r"(\d{2})(\d{2})", replacement = "${1}x")]
        value: String,
    }

    let code = Code {
        value: "1234".to_string(),
    };

    assert_eq!(format!("{code:?}"), "Code { value: 12x }");
}

#[test]
fn a_named_group_can_be_referenced() {
    #[derive(Secret)]
    struct Card {
        #[secret(search = r"\d{12}(?P<last>\d{4})", replacement = "****$last")]
        number: String,
    }

    let card = Card {
        number: "1234567890123456".to_string(),
    };

    assert_eq!(format!("{card:?}"), "Card { number: ****3456 }");
}

#[test]
fn a_doubled_dollar_is_a_literal_dollar() {
    #[derive(Secret)]
    struct Price {
        #[secret(search = r"\d+\.(\d{2})", replacement = "$$**.$1")]
        amount: String,
    }

    let price = Price {
        amount: "1999.95".to_string(),
    };

    assert_eq!(format!("{price:?}"), "Price { amount: $**.95 }");
}
