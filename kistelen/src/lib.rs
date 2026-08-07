//! Keeps sensitive struct fields out of [`Debug`] output.
//!
//! Rust's derived [`Debug`] prints every field. When a struct holding a
//! password or a token reaches a log line, a panic message or a tracing span,
//! the value goes with it. `kistelen` replaces the derive so annotated fields
//! render as a mask instead:
//!
//! ```
//! use kistelen::Secret;
//!
//! #[derive(Secret)]
//! struct User {
//!     id: i32,
//!     username: String,
//!     #[secret]
//!     password: String,
//! }
//!
//! let user = User {
//!     id: 1,
//!     username: "alice".to_string(),
//!     password: "hunter2".to_string(),
//! };
//!
//! assert_eq!(
//!     format!("{user:?}"),
//!     r#"User { id: 1, username: "alice", password: ■■■ }"#,
//! );
//! ```
//!
//! Fields without `#[secret]` are formatted by their own [`Debug`]
//! implementation, exactly as the standard derive would.
//!
//! # What this does not cover
//!
//! Masking applies to [`Debug`] alone. A value can still reach the outside
//! world through [`Display`](core::fmt::Display), serialisation, or a direct
//! read of the field. Reaching for this macro protects the accidental path —
//! the one nobody wrote and nobody reviews — not every path.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub use kistelen_macros::Secret;

/// The string printed in place of a secret value.
pub const MASK: &str = "■■■";

/// The character repeated by `fixed` and `partial` masking.
pub const MASK_CHARACTER: char = '■';

/// Values shorter than this are masked entirely by `partial`.
///
/// Exposing part of a short value narrows it too far to be worth the
/// readability: four characters of an eight-character password is a
/// meaningful head start, while four of a forty-character token is not.
pub const MINIMUM_PARTIAL_LENGTH: usize = 8;

/// The most characters `partial` will expose at each end.
pub const MAXIMUM_PARTIAL_EXPOSURE: usize = 4;

#[doc(hidden)]
pub mod __private {
    use core::fmt::{Debug, Display, Formatter, Result};

    /// Stands in for a secret value inside a [`Debug`] implementation.
    ///
    /// Printed without quotes so masked output is never mistaken for a string
    /// whose contents happen to be the mask characters.
    pub struct Mask;

    impl Debug for Mask {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
            formatter.write_str(crate::MASK)
        }
    }

    /// A caller-supplied string standing in for the value, from `with`.
    pub struct Literal(pub &'static str);

    impl Debug for Literal {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
            formatter.write_str(self.0)
        }
    }

    /// A set number of mask characters, from `fixed`.
    ///
    /// The value is not consulted, so its length is hidden along with its
    /// contents.
    pub struct Fixed(pub usize, pub char);

    impl Debug for Fixed {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
            use core::fmt::Write;

            for _ in 0..self.0 {
                formatter.write_char(self.1)?;
            }

            Ok(())
        }
    }

    /// Leading and trailing characters exposed, the middle masked, from
    /// `partial`.
    pub struct Partial<'a, T>(pub &'a T, pub char);

    impl<T: Display> Debug for Partial<'_, T> {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
            formatter.write_str(&partially_mask(&self.0.to_string(), self.1))
        }
    }

    /// Stands in for a secret [`Option`], masking the contained value while
    /// leaving the `Some`/`None` distinction visible.
    ///
    /// Whether a value is set is usually structural rather than sensitive —
    /// hiding it makes output harder to read without protecting anything.
    pub struct MaskedOption<'a, T, M>(pub &'a Option<T>, pub M);

    impl<T, M: Debug> Debug for MaskedOption<'_, T, M> {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
            match self.0 {
                Some(_) => formatter.debug_tuple("Some").field(&self.1).finish(),
                None => formatter.write_str("None"),
            }
        }
    }

    /// A partially masked [`Option`], which must reach the contained value.
    pub struct PartialOption<'a, T>(pub &'a Option<T>, pub char);

    impl<T: Display> Debug for PartialOption<'_, T> {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
            match self.0 {
                Some(value) => formatter
                    .debug_tuple("Some")
                    .field(&Partial(value, self.1))
                    .finish(),
                None => formatter.write_str("None"),
            }
        }
    }

    /// Masks the middle of `value`, exposing a little of each end.
    ///
    /// Counts characters rather than bytes, so multi-byte text is neither
    /// split nor miscounted. Length is preserved, which is itself a small
    /// disclosure — `fixed` is the mode that hides it.
    ///
    /// Values too short to expose safely fall back to a mask of fixed width,
    /// which hides their length as well as their contents. Repeating the mask
    /// character once per character would disclose the length of exactly the
    /// values the guard exists to protect, and an empty value would render as
    /// nothing at all.
    fn partially_mask(value: &str, character: char) -> String {
        let length = value.chars().count();

        if length < crate::MINIMUM_PARTIAL_LENGTH {
            let width = crate::MASK.chars().count();
            return core::iter::repeat(character).take(width).collect();
        }

        // A fifth of each end, so the proportion exposed falls as the value
        // grows, with a floor of one character and the documented ceiling.
        let exposed = (length / 5).clamp(1, crate::MAXIMUM_PARTIAL_EXPOSURE);

        value
            .chars()
            .enumerate()
            .map(|(index, original)| {
                let leading = index < exposed;
                let trailing = index >= length - exposed;

                if leading || trailing {
                    original
                } else {
                    character
                }
            })
            .collect()
    }
}
