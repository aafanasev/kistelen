//! Generation of the `Debug` implementation.
//!
//! Each container is planned before any tokens are produced: masking decides
//! whether a field's value is read at all, and for enums that in turn decides
//! whether the match pattern binds it or discards it. Binding a value the body
//! never reads would raise an `unused_variables` warning in the caller's crate.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Error, Field, Fields, Result, Variant};

use crate::attr::{self, Mode, Secret};

pub(crate) fn derive(input: &DeriveInput) -> Result<TokenStream> {
    let body = match &input.data {
        Data::Struct(data) => struct_body(input, data)?,
        Data::Enum(data) => enum_body(input, data)?,
        Data::Union(_) => {
            return Err(Error::new_spanned(
                &input.ident,
                "Secret cannot be derived for unions, whose fields cannot be read safely",
            ));
        }
    };

    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::core::fmt::Debug for #ident #type_generics #where_clause {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #body
            }
        }
    })
}

/// How one field will be rendered. `None` leaves the value untouched.
struct Plan<'a> {
    field: &'a Field,
    mode: Option<Mode>,
    /// Masked, and written as an `Option`, so the mask goes inside the `Some`.
    masked_option: bool,
}

impl Plan<'_> {
    /// Whether the body reads this field's value.
    ///
    /// A constant mask never reads it. A masked option does, to tell `Some`
    /// from `None`, and partial masking does, since the output depends on the
    /// value itself.
    fn reads_value(&self) -> bool {
        match &self.mode {
            None => true,
            Some(mode) => self.masked_option || mode.reads_value(),
        }
    }
}

fn plan<'a>(fields: &'a Fields, blanket: Option<&Secret>) -> Result<Vec<Plan<'a>>> {
    fields
        .iter()
        .map(|field| {
            let mode = mode_for(field, blanket)?;

            Ok(Plan {
                field,
                masked_option: mode.is_some() && is_option(&field.ty),
                mode,
            })
        })
        .collect()
}

/// Resolves the rendering mode for one field, falling back to the rule its
/// container sets.
fn mode_for(field: &Field, blanket: Option<&Secret>) -> Result<Option<Mode>> {
    let Some(secret) = attr::find(&field.attrs)? else {
        return blanket.map(Secret::mode).transpose();
    };

    if !secret.skip {
        return Ok(Some(secret.mode()?));
    }

    if blanket.is_none() {
        return Err(Error::new(
            secret.span,
            "`skip` has no effect here, as no `#[secret]` on the struct, enum or variant covers this field",
        ));
    }

    Ok(None)
}

fn struct_body(input: &DeriveInput, data: &syn::DataStruct) -> Result<TokenStream> {
    let blanket = attr::blanket(&input.attrs)?;
    let plans = plan(&data.fields, blanket.as_ref())?;
    let name = input.ident.to_string();

    let accessors = data
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| match &field.ident {
            Some(ident) => quote!(&self.#ident),
            None => {
                let index = syn::Index::from(index);
                quote!(&self.#index)
            }
        })
        .collect::<Vec<_>>();

    Ok(format_fields(&name, &data.fields, &plans, &accessors))
}

fn enum_body(input: &DeriveInput, data: &syn::DataEnum) -> Result<TokenStream> {
    let blanket = attr::blanket(&input.attrs)?;

    if data.variants.is_empty() {
        // An uninhabited enum cannot reach `fmt`; an empty match proves it.
        return Ok(quote! {
            let _ = formatter;
            match *self {}
        });
    }

    let arms = data
        .variants
        .iter()
        .map(|variant| variant_arm(&input.ident, variant, blanket.as_ref()))
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        match self {
            #(#arms)*
        }
    })
}

fn variant_arm(
    enum_ident: &syn::Ident,
    variant: &Variant,
    blanket: Option<&Secret>,
) -> Result<TokenStream> {
    // A variant may carry its own `#[secret]`, which takes precedence over one
    // on the enum for the fields it holds.
    let own = attr::blanket(&variant.attrs)?;
    let blanket = own.as_ref().or(blanket);
    let plans = plan(&variant.fields, blanket)?;

    let variant_ident = &variant.ident;
    let name = variant_ident.to_string();

    let accessors = plans
        .iter()
        .enumerate()
        .map(|(index, plan)| {
            let ident = binding(plan.field, index);
            quote!(#ident)
        })
        .collect::<Vec<_>>();

    let body = format_fields(&name, &variant.fields, &plans, &accessors);
    let pattern = variant_pattern(enum_ident, variant, &plans);

    Ok(quote! {
        #pattern => { #body }
    })
}

/// The identifier a field's value is bound to inside a match pattern.
///
/// Named fields reuse their own identifier so the pattern can use shorthand;
/// tuple fields get a prefixed positional name that cannot collide with one.
fn binding(field: &Field, index: usize) -> syn::Ident {
    match &field.ident {
        Some(ident) => ident.clone(),
        None => format_ident!("__field{}", index),
    }
}

/// Builds the match pattern, discarding any field the body will not read.
fn variant_pattern(enum_ident: &syn::Ident, variant: &Variant, plans: &[Plan]) -> TokenStream {
    let variant_ident = &variant.ident;

    match &variant.fields {
        Fields::Named(fields) => {
            let entries = fields.named.iter().zip(plans).map(|(field, plan)| {
                let ident = field.ident.as_ref().unwrap();

                if plan.reads_value() {
                    quote!(#ident)
                } else {
                    quote!(#ident: _)
                }
            });

            quote!(#enum_ident::#variant_ident { #(#entries),* })
        }
        Fields::Unnamed(fields) => {
            let entries =
                fields
                    .unnamed
                    .iter()
                    .zip(plans)
                    .enumerate()
                    .map(|(index, (field, plan))| {
                        if plan.reads_value() {
                            let ident = binding(field, index);
                            quote!(#ident)
                        } else {
                            quote!(_)
                        }
                    });

            quote!(#enum_ident::#variant_ident( #(#entries),* ))
        }
        Fields::Unit => quote!(#enum_ident::#variant_ident),
    }
}

/// Builds the formatting expression for one struct or variant.
///
/// `accessors` must yield a reference to each field, in declaration order.
/// Entries for fields that are not read are never emitted, so a discarded
/// accessor is harmless.
fn format_fields(
    name: &str,
    fields: &Fields,
    plans: &[Plan],
    accessors: &[TokenStream],
) -> TokenStream {
    let mut caches = Vec::new();
    let mut values = Vec::with_capacity(plans.len());

    for (index, (plan, accessor)) in plans.iter().zip(accessors).enumerate() {
        if let Some(Mode::Replace { .. }) = &plan.mode {
            // Compiling a pattern on every call to `fmt` would make formatting
            // far more expensive than the value is worth. The cache is
            // declared alongside the code that reads it, one per field.
            let cache = cache_ident(index);

            caches.push(quote! {
                static #cache: ::kistelen::__private::PatternCache =
                    ::kistelen::__private::PatternCache::new();
            });
        }

        values.push(value(plan, accessor, index));
    }

    let expression = format_expression(name, fields, &values);

    quote! {
        #(#caches)*
        #expression
    }
}

fn cache_ident(index: usize) -> syn::Ident {
    format_ident!("__PATTERN{}", index)
}

fn format_expression(name: &str, fields: &Fields, values: &[TokenStream]) -> TokenStream {
    match fields {
        Fields::Named(named) => {
            let names = named
                .named
                .iter()
                .map(|field| field.ident.as_ref().unwrap().to_string());

            quote! {
                formatter.debug_struct(#name)
                    #(.field(#names, #values))*
                    .finish()
            }
        }
        Fields::Unnamed(_) => quote! {
            formatter.debug_tuple(#name)
                #(.field(#values))*
                .finish()
        },
        // Matches what the standard derive prints for a unit shape.
        Fields::Unit => quote!(formatter.write_str(#name)),
    }
}

fn value(plan: &Plan, accessor: &TokenStream, index: usize) -> TokenStream {
    let Some(mode) = &plan.mode else {
        return quote!(#accessor);
    };

    // Masking an `Option` wholesale would hide whether a value is set at all,
    // which is rarely the intent, so the mask goes inside the `Some`.
    let option = plan.masked_option;

    match mode {
        Mode::Partial { character } if option => {
            quote!(&::kistelen::__private::PartialOption(#accessor, #character))
        }
        Mode::Partial { character } => {
            quote!(&::kistelen::__private::Partial(#accessor, #character))
        }
        Mode::Replace {
            pattern,
            replacement,
        } => {
            let cache = cache_ident(index);

            if option {
                quote!(&::kistelen::__private::ReplacedOption(
                    #accessor, &#cache, #pattern, #replacement
                ))
            } else {
                quote!(&::kistelen::__private::Replaced(
                    #accessor, &#cache, #pattern, #replacement
                ))
            }
        }
        constant if option => {
            let mask = constant_mask(constant);
            quote!(&::kistelen::__private::MaskedOption(#accessor, #mask))
        }
        constant => {
            let mask = constant_mask(constant);
            quote!(&#mask)
        }
    }
}

/// Tokens for a mask that does not depend on the value.
fn constant_mask(mode: &Mode) -> TokenStream {
    match mode {
        Mode::Full => quote!(::kistelen::__private::Mask),
        Mode::Literal(text) => quote!(::kistelen::__private::Literal(#text)),
        Mode::Fixed { count, character } => {
            quote!(::kistelen::__private::Fixed(#count, #character))
        }
        // Modes that read the value are handled before this point.
        Mode::Partial { .. } | Mode::Replace { .. } => {
            unreachable!("value-dependent masking is not a constant")
        }
    }
}

/// Recognises `Option<T>` by name.
///
/// This can only ever be syntactic: at expansion time a type alias for
/// `Option<T>` is indistinguishable from any other path, so an aliased option
/// is masked whole rather than through its `Some`.
fn is_option(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };

    if path.qself.is_some() {
        return false;
    }

    let Some(segment) = path.path.segments.last() else {
        return false;
    };

    if segment.ident != "Option" {
        return false;
    }

    matches!(&segment.arguments, syn::PathArguments::AngleBracketed(args)
        if args.args.len() == 1)
}
