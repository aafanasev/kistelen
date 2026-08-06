//! Derive macro implementation for [`kistelen`](https://docs.rs/kistelen).
//!
//! This crate is an implementation detail. Depend on `kistelen` instead, which
//! re-exports everything here.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};

/// Derives [`Debug`] with `#[secret]` fields replaced by a mask.
#[proc_macro_derive(Secret, attributes(secret))]
pub fn derive_secret(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = named_fields(input)?;

    let entries = fields.iter().map(|field| {
        // Every named field has an identifier, so the unwrap cannot fire.
        let ident = field.ident.as_ref().unwrap();
        let name = ident.to_string();

        if is_secret(field) {
            quote! { .field(#name, &::kistelen::__private::Mask) }
        } else {
            quote! { .field(#name, &self.#ident) }
        }
    });

    let ident = &input.ident;
    let name = ident.to_string();
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::core::fmt::Debug for #ident #type_generics #where_clause {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                formatter.debug_struct(#name)
                    #(#entries)*
                    .finish()
            }
        }
    })
}

/// Structs with named fields are the only supported shape for now; tuple
/// structs and enums arrive later.
fn named_fields(
    input: &DeriveInput,
) -> syn::Result<&syn::punctuated::Punctuated<syn::Field, syn::token::Comma>> {
    let data = match &input.data {
        Data::Struct(data) => data,
        Data::Enum(_) => {
            return Err(syn::Error::new(
                input.span(),
                "Secret cannot be derived for enums yet",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new(
                input.span(),
                "Secret cannot be derived for unions",
            ));
        }
    };

    match &data.fields {
        Fields::Named(fields) => Ok(&fields.named),
        Fields::Unnamed(_) => Err(syn::Error::new(
            input.span(),
            "Secret cannot be derived for tuple structs yet",
        )),
        Fields::Unit => Err(syn::Error::new(
            input.span(),
            "Secret cannot be derived for unit structs, which have no fields to mask",
        )),
    }
}

fn is_secret(field: &syn::Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("secret"))
}
