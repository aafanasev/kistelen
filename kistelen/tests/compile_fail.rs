//! Misuse should produce a readable message rather than a macro panic.

#[test]
fn unsupported_shapes_are_rejected_with_a_clear_message() {
    let harness = trybuild::TestCases::new();
    harness.compile_fail("tests/ui/*.rs");
}
