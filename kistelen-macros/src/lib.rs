//! Derive macro implementation for [`kistelen`](https://docs.rs/kistelen).
//!
//! This crate is an implementation detail. Depend on `kistelen` instead, which
//! re-exports everything here.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod attr;
mod bounds;
mod expand;

/// The character repeated by modes that build a mask to a length.
///
/// Kept in step with `kistelen::MASK`, which is this character three times.
const DEFAULT_MASK_CHARACTER: char = '■';

/// Derives [`Debug`] with `#[secret]` fields replaced by a mask.
///
/// Works on structs, tuple structs, unit structs and enums, generic or not,
/// and formats identically to the standard derive for everything not masked,
/// including the pretty-printed `{:#?}` form.
///
/// ```ignore
/// #[derive(Secret)]
/// struct User {
///     id: i32,
///     #[secret]
///     password: String,
/// }
/// ```
///
/// # Where the attribute goes
///
/// On a **field**, that field is masked. On a **struct, enum or variant**,
/// every field it covers is masked, and a field may override the rule with
/// options of its own.
///
/// ```ignore
/// #[derive(Secret)]
/// #[secret(with = "-")]
/// struct Session {
///     token: String,                 // masked with `-`
///     #[secret(fixed = 4)]
///     refresh: String,               // masked with four characters
///     #[secret(skip)]
///     id: u32,                       // not masked
/// }
/// ```
///
/// # Options
///
/// ## skip
///
/// Exempts a field from the rule its container sets. Using it where no such
/// rule applies is a compile error: it would read as protection that is not
/// there.
///
/// ## with
///
/// `with = "REDACTED"` or `with = '*'` replaces the mask. Combined with
/// `fixed` or `partial`, which repeat a single character, it must be one
/// character.
///
/// ## fixed
///
/// `fixed = 3` prints exactly three mask characters whatever the value is.
/// The point is hiding the length: a mask that tracks the real length
/// discloses it.
///
/// ## partial
///
/// Exposes a fifth of the value at each end, never more than four characters,
/// so the proportion revealed falls as the value grows. Below eight
/// characters nothing is exposed and a fixed-width mask is printed instead.
///
/// Reads the value, so the field must implement [`Display`](core::fmt::Display).
///
/// ## Patterns
///
/// With the `regex` feature, `search` and `replacement` rewrite the value:
///
/// ```ignore
/// #[secret(search = r"(\d{4})\d{8}(\d{4})", replacement = "$1-****-$2")]
/// number: String,
/// ```
///
/// The pattern must match the value from end to end; anything not fully
/// matched is masked instead. A partial match would leave everything outside
/// it untouched, so a pattern written for one shape of value would print a
/// different shape verbatim. `$0` is rejected, since it would restore the
/// whole matched value.
///
/// Patterns are parsed as the macro expands, so an unusable one is a compile
/// error. Each is compiled at most once and reused.
///
/// # Types
///
/// A masked [`Option`] keeps its `Some`/`None` shape and masks the value
/// inside. This is recognised by how the type is written, so an alias for
/// `Option<T>` is masked whole — the safe reading of a type that cannot be
/// inspected.
///
/// Every mode except `partial` and pattern replacement ignores the value
/// entirely, so any type can carry them.
///
/// # Generics
///
/// Bounds are worked out per field, from what that field's rendering needs:
/// [`Debug`] for a field printed normally, [`Display`](core::fmt::Display) for
/// one read as text before masking, and nothing at all for a constant mask.
///
/// This is deliberately narrower than the standard derive, which bounds every
/// type parameter by [`Debug`]. A parameter appearing only in masked fields
/// stays unbounded, so a type implementing neither trait can still be held and
/// masked:
///
/// ```ignore
/// struct Opaque;                 // implements nothing
///
/// #[derive(Secret)]
/// struct Envelope<T> {
///     #[secret]
///     payload: T,
/// }
///
/// // Envelope { payload: ■■■ }
/// println!("{:?}", Envelope { payload: Opaque });
/// ```
///
/// Bounds land on the field's own type rather than the parameters within it,
/// and a masked [`Option`] read by `partial` or a pattern bounds the contained
/// type rather than the option. Any bound already written on the type is kept.
#[proc_macro_derive(Secret, attributes(secret))]
pub fn derive_secret(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand::derive(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
