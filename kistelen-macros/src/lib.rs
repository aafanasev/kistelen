//! Derive macro implementation for [`kistelen`](https://docs.rs/kistelen).
//!
//! This crate is an implementation detail. Depend on `kistelen` instead, which
//! re-exports everything here.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod attr;
mod expand;

/// Derives [`Debug`] with `#[secret]` fields replaced by a mask.
#[proc_macro_derive(Secret, attributes(secret))]
pub fn derive_secret(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand::derive(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
