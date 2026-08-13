//! Trait bounds required by the generated formatting code.
//!
//! The standard `Debug` derive bounds every type parameter, which over-
//! constrains: a parameter appearing only in a masked field needs nothing,
//! since the mask is printed without reading the value. Bounding it would
//! refuse types that this crate exists to accept.
//!
//! So bounds are collected per field, from what each field's rendering
//! actually requires, and applied to the field's own type rather than to the
//! parameters inside it.

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::quote;
use syn::{GenericParam, Generics, Type, WherePredicate};

/// What the generated code needs of a field type.
#[derive(Clone, Copy)]
pub(crate) enum Requirement {
    /// The value is printed through its own `Debug`.
    Debug,
    /// The value is read as text before being masked.
    Display,
}

impl Requirement {
    fn path(self) -> TokenStream {
        match self {
            Requirement::Debug => quote!(::core::fmt::Debug),
            Requirement::Display => quote!(::core::fmt::Display),
        }
    }
}

/// Predicates gathered while planning, ready to extend a where-clause.
pub(crate) struct Bounds {
    /// Generic parameters of the type being derived, of every kind. A field
    /// type mentioning none of them needs no predicate: whether it satisfies
    /// the bound is already settled, and stating it would only add noise.
    ///
    /// Const and lifetime parameters count alongside type parameters, because
    /// an implementation can be written for one const value or one lifetime
    /// and not others. A field depending on such a parameter is as generic as
    /// one depending on a type parameter, and needs its predicate just the
    /// same.
    parameters: Vec<Ident>,
    predicates: Vec<WherePredicate>,
    /// Rendered predicates already added, so a type used by several fields is
    /// bounded once.
    seen: Vec<String>,
}

impl Bounds {
    pub(crate) fn new(generics: &Generics) -> Self {
        Self {
            parameters: generics
                .params
                .iter()
                .map(|parameter| match parameter {
                    GenericParam::Type(parameter) => parameter.ident.clone(),
                    GenericParam::Const(parameter) => parameter.ident.clone(),
                    GenericParam::Lifetime(parameter) => parameter.lifetime.ident.clone(),
                })
                .collect(),
            predicates: Vec::new(),
            seen: Vec::new(),
        }
    }

    /// Records that `ty` must satisfy `requirement`.
    pub(crate) fn require(&mut self, ty: &Type, requirement: Requirement) {
        if !self.mentions_parameter(ty) {
            return;
        }

        let path = requirement.path();
        let predicate: WherePredicate = syn::parse_quote!(#ty: #path);
        let rendered = quote!(#predicate).to_string();

        if self.seen.contains(&rendered) {
            return;
        }

        self.seen.push(rendered);
        self.predicates.push(predicate);
    }

    /// Returns `generics` extended with everything collected.
    pub(crate) fn apply(self, generics: &Generics) -> Generics {
        let mut generics = generics.clone();

        if self.predicates.is_empty() {
            return generics;
        }

        let where_clause = generics.make_where_clause();

        for predicate in self.predicates {
            where_clause.predicates.push(predicate);
        }

        generics
    }

    /// Whether `ty` is written in terms of any generic parameter.
    ///
    /// Walks the tokens rather than the type structure, which catches
    /// parameters wherever they appear — inside a nested path, a slice, a
    /// tuple, or an associated type. A name shadowed by something unrelated
    /// would match too, but the resulting predicate is one the code already
    /// needs to hold, so the cost of guessing wide is nothing.
    ///
    /// A lifetime reaches here as a `'` punct followed by its bare ident, so
    /// matching idents alone finds it. That conflates `'a` with a type named
    /// `a`, which guesses wide in the same harmless direction: this runs only
    /// for fields the generated code actually formats, so any predicate it
    /// produces is one that had to hold regardless.
    fn mentions_parameter(&self, ty: &Type) -> bool {
        fn walk(tokens: TokenStream, parameters: &[Ident]) -> bool {
            tokens.into_iter().any(|token| match token {
                TokenTree::Ident(ident) => parameters.contains(&ident),
                TokenTree::Group(group) => walk(group.stream(), parameters),
                _ => false,
            })
        }

        walk(quote!(#ty), &self.parameters)
    }
}
