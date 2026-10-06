# Changelog

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- The derive macro is built on `syn` 3 instead of `syn` 2. Nothing about the
  macro's behaviour or its minimum supported Rust version (1.71) changes; a
  dependent whose tree has already moved to `syn` 3 no longer compiles both
  majors.

## [0.1.2] — 2026-08-13

### Fixed

- Bound inference recognises const and lifetime parameters, not only type
  parameters. A field whose type depended solely on one of those was treated
  as non-generic, so its `Debug` or `Display` predicate was left out and the
  error surfaced inside the generated code
  ([#7](https://github.com/aafanasev/kistelen/issues/7)).

  An implementation can be written for one const value or one lifetime and not
  others, which makes such a field exactly as generic as one naming a type
  parameter. Constant-masked fields stay unbounded as before, since nothing
  reads them.

## [0.1.1] — 2026-08-10

### Fixed

- Trait bounds are derived from what each field renders, so a generic type no
  longer needs them written by hand. The README claimed generic types were
  supported while an unbounded parameter in a printed field failed to compile
  ([#1](https://github.com/aafanasev/kistelen/issues/1)).

  Bounds stay narrower than the standard derive: a parameter appearing only in
  masked fields keeps no bound at all, so a type implementing neither `Debug`
  nor `Display` can still be held and masked.

- A repeated option within one `#[secret]` attribute is rejected rather than
  resolved by taking the last one. `with = "*", with = "REDACTED"` silently
  used the second, which made a typo look like a working configuration
  ([#3](https://github.com/aafanasev/kistelen/issues/3)).

  This refuses input that previously compiled. Only ambiguous input is
  affected — every unambiguous combination behaves as before — so it ships as
  a patch rather than a breaking release.

### Notes

- Renaming the dependency in `Cargo.toml` is still unsupported: the generated
  code refers to `::kistelen` by name. The one-line workaround is
  `extern crate my_alias as kistelen;` in the crate root
  ([#2](https://github.com/aafanasev/kistelen/issues/2)).

## [0.1.0] — 2026-08-07

Initial release.

- `#[derive(Secret)]` generating `Debug` with `#[secret]` fields masked
- Structs, tuple structs, unit structs and enums
- `#[secret]` on a field, a type, or a single enum variant, with
  `#[secret(skip)]` to exempt a field from a wider rule
- `Option` masked through its `Some`, keeping the `Some`/`None` distinction
- `with`, `fixed` and `partial` masking
- Regex masking behind the optional `regex` feature
- Both `{:?}` and `{:#?}`

[Unreleased]: https://github.com/aafanasev/kistelen/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/aafanasev/kistelen/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/aafanasev/kistelen/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/aafanasev/kistelen/releases/tag/v0.1.0
