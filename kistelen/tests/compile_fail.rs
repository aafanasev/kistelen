//! Misuse should produce a readable message rather than a macro panic.
//!
//! Cases are split by feature: the same source produces a different message
//! depending on whether `regex` is enabled, so each set only runs where its
//! expected output applies.

#[test]
fn misuse_is_rejected_with_a_clear_message() {
    let harness = trybuild::TestCases::new();
    harness.compile_fail("tests/ui/*.rs");
}

#[test]
#[cfg(feature = "regex")]
fn unusable_patterns_are_rejected_at_expansion_time() {
    let harness = trybuild::TestCases::new();
    harness.compile_fail("tests/ui/regex/*.rs");
}

#[test]
#[cfg(not(feature = "regex"))]
fn pattern_options_report_the_missing_feature() {
    let harness = trybuild::TestCases::new();
    harness.compile_fail("tests/ui/no_regex/*.rs");
}
