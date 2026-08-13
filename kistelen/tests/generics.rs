//! Bounds are derived from what each field's rendering actually requires.
//!
//! The property worth protecting is that a parameter used only by masked
//! fields stays unbounded. Bounding every parameter, as the standard derive
//! does, would refuse the very types this crate exists to accept.

#![allow(dead_code)]

use kistelen::Secret;

/// Implements neither `Debug` nor `Display`, so it can only appear where no
/// bound is generated.
struct Opaque;

/// Formats only at one const value, so a field holding it is generic in `N`
/// even though no type parameter appears in the field's type.
struct OnlyOne<const N: usize>;

impl core::fmt::Debug for OnlyOne<1> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("one")
    }
}

impl core::fmt::Display for OnlyOne<1> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("1234567890123456")
    }
}

/// The lifetime counterpart: formats only at `'static`.
struct OnlyStatic<'a>(&'a str);

impl core::fmt::Debug for OnlyStatic<'static> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("static")
    }
}

#[test]
fn an_unbounded_parameter_works_when_the_field_is_printed() {
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
fn a_masked_parameter_needs_no_bound_at_all() {
    #[derive(Secret)]
    struct Envelope<T> {
        #[secret]
        payload: T,
    }

    let envelope = Envelope { payload: Opaque };

    assert_eq!(format!("{envelope:?}"), "Envelope { payload: ■■■ }");
}

#[test]
fn a_whole_masked_type_needs_no_bound() {
    #[derive(Secret)]
    #[secret]
    struct Envelope<T, U> {
        first: T,
        second: U,
    }

    let envelope = Envelope {
        first: Opaque,
        second: Opaque,
    };

    assert_eq!(
        format!("{envelope:?}"),
        "Envelope { first: ■■■, second: ■■■ }",
    );
}

#[test]
fn one_parameter_can_be_bounded_while_another_is_not() {
    #[derive(Secret)]
    struct Mixed<T, U> {
        shown: T,
        #[secret]
        hidden: U,
    }

    let mixed = Mixed {
        shown: "visible",
        hidden: Opaque,
    };

    assert_eq!(
        format!("{mixed:?}"),
        r#"Mixed { shown: "visible", hidden: ■■■ }"#,
    );
}

#[test]
fn partial_masking_bounds_the_parameter_by_display() {
    #[derive(Secret)]
    struct Card<T> {
        #[secret(partial)]
        number: T,
    }

    let card = Card {
        number: 1234567890123456u64,
    };

    assert_eq!(format!("{card:?}"), "Card { number: 123■■■■■■■■■■456 }");
}

#[test]
fn a_partially_masked_option_bounds_the_contained_type() {
    // The bound belongs to `T`, not `Option<T>`: it is the contained value
    // that gets read.
    #[derive(Secret)]
    struct Card<T> {
        #[secret(partial)]
        number: Option<T>,
    }

    let present = Card {
        number: Some(1234567890123456u64),
    };
    let absent: Card<u64> = Card { number: None };

    assert_eq!(
        format!("{present:?}"),
        "Card { number: Some(123■■■■■■■■■■456) }",
    );
    assert_eq!(format!("{absent:?}"), "Card { number: None }");
}

#[test]
fn a_constant_masked_option_leaves_the_contained_type_unbounded() {
    #[derive(Secret)]
    struct Holder<T> {
        #[secret]
        value: Option<T>,
    }

    let holder = Holder {
        value: Some(Opaque),
    };

    assert_eq!(format!("{holder:?}"), "Holder { value: Some(■■■) }");
}

#[test]
fn a_nested_generic_type_is_bounded_as_written() {
    #[derive(Secret)]
    struct Batch<T> {
        items: Vec<T>,
        #[secret]
        key: String,
    }

    let batch = Batch {
        items: vec![1, 2],
        key: "k".to_string(),
    };

    assert_eq!(format!("{batch:?}"), "Batch { items: [1, 2], key: ■■■ }",);
}

#[test]
fn an_existing_where_clause_is_preserved() {
    #[derive(Secret)]
    struct Bounded<T>
    where
        T: Clone,
    {
        value: T,
        #[secret]
        secret: String,
    }

    let bounded = Bounded {
        value: 1,
        secret: "s".to_string(),
    };

    assert_eq!(format!("{bounded:?}"), "Bounded { value: 1, secret: ■■■ }");
}

#[test]
fn an_inline_bound_is_preserved() {
    #[derive(Secret)]
    struct Inline<T: Clone> {
        value: T,
        #[secret]
        secret: String,
    }

    let inline = Inline {
        value: 1,
        secret: "s".to_string(),
    };

    assert_eq!(format!("{inline:?}"), "Inline { value: 1, secret: ■■■ }");
}

#[test]
fn a_lifetime_parameter_is_carried_through() {
    #[derive(Secret)]
    struct Borrowed<'a, T> {
        value: &'a T,
        #[secret]
        secret: &'a str,
    }

    let value = 7;
    let borrowed = Borrowed {
        value: &value,
        secret: "s",
    };

    assert_eq!(
        format!("{borrowed:?}"),
        "Borrowed { value: 7, secret: ■■■ }"
    );
}

#[test]
fn repeating_a_type_across_fields_bounds_it_once() {
    // Two fields of the same type must not produce a duplicate predicate.
    #[derive(Secret)]
    struct Pair<T> {
        first: T,
        second: T,
        #[secret]
        secret: String,
    }

    let pair = Pair {
        first: 1,
        second: 2,
        secret: "s".to_string(),
    };

    assert_eq!(
        format!("{pair:?}"),
        "Pair { first: 1, second: 2, secret: ■■■ }",
    );
}

#[test]
fn generic_enums_are_bounded_per_variant() {
    #[derive(Secret)]
    enum Message<T, U> {
        Shown(T),
        #[secret]
        Hidden(U),
    }

    let shown: Message<i32, Opaque> = Message::Shown(1);
    let hidden: Message<i32, Opaque> = Message::Hidden(Opaque);

    assert_eq!(format!("{shown:?}"), "Shown(1)");
    assert_eq!(format!("{hidden:?}"), "Hidden(■■■)");
}

#[test]
#[cfg(feature = "regex")]
fn pattern_replacement_bounds_the_parameter_by_display() {
    #[derive(Secret)]
    struct Account<T> {
        #[secret(search = r"\d{4}(\d{4})", replacement = "****$1")]
        number: T,
    }

    let account = Account {
        number: 12345678u64,
    };

    assert_eq!(format!("{account:?}"), "Account { number: ****5678 }");
}

#[test]
fn a_field_depending_only_on_a_const_parameter_is_bounded() {
    #[derive(Secret)]
    struct Wrapper<const N: usize> {
        value: OnlyOne<N>,
        #[secret]
        secret: String,
    }

    let wrapper = Wrapper::<1> {
        value: OnlyOne::<1>,
        secret: "s".to_string(),
    };

    assert_eq!(
        format!("{wrapper:?}"),
        "Wrapper { value: one, secret: ■■■ }"
    );
}

#[test]
fn a_partially_masked_const_dependent_field_is_bounded_by_display() {
    #[derive(Secret)]
    struct Wrapper<const N: usize> {
        #[secret(partial)]
        value: OnlyOne<N>,
    }

    let wrapper = Wrapper::<1> {
        value: OnlyOne::<1>,
    };

    assert_eq!(
        format!("{wrapper:?}"),
        "Wrapper { value: 123■■■■■■■■■■456 }",
    );
}

#[test]
fn a_constant_masked_const_dependent_field_stays_unbounded() {
    // `OnlyOne<2>` formats no way at all, so this compiles only if the mask
    // is printed without reaching for a bound.
    #[derive(Secret)]
    struct Wrapper<const N: usize> {
        #[secret]
        value: OnlyOne<N>,
    }

    let wrapper = Wrapper::<2> {
        value: OnlyOne::<2>,
    };

    assert_eq!(format!("{wrapper:?}"), "Wrapper { value: ■■■ }");
}

#[test]
fn a_field_depending_only_on_a_lifetime_is_bounded() {
    #[derive(Secret)]
    struct Wrapper<'a> {
        value: OnlyStatic<'a>,
        #[secret]
        secret: &'a str,
    }

    let wrapper = Wrapper {
        value: OnlyStatic("x"),
        secret: "s",
    };

    assert_eq!(
        format!("{wrapper:?}"),
        "Wrapper { value: static, secret: ■■■ }",
    );
}

#[test]
fn a_generic_tuple_struct_is_bounded() {
    #[derive(Secret)]
    struct Pair<T>(T, #[secret] T);

    let pair = Pair(1, 2);

    assert_eq!(format!("{pair:?}"), "Pair(1, ■■■)");
}
