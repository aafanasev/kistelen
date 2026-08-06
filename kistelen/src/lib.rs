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

#[doc(hidden)]
pub mod __private {
    /// Stands in for a secret value inside a [`Debug`] implementation.
    ///
    /// Printed without quotes so masked output is never mistaken for a string
    /// whose contents happen to be the mask characters.
    pub struct Mask;

    impl core::fmt::Debug for Mask {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            formatter.write_str(crate::MASK)
        }
    }
}
