//! Parsing of the `#[secret]` attribute.
//!
//! The attribute appears in two places with different meanings: on a container
//! (struct, enum, or enum variant) it masks every field within, and on a field
//! it masks that field alone. `#[secret(skip)]` exempts a field from a
//! container-level rule.

use syn::spanned::Spanned;
use syn::{Attribute, Error, Result};

/// A parsed `#[secret]` attribute.
pub(crate) struct Secret {
    /// Whether `skip` was given, exempting the field from a blanket rule.
    pub(crate) skip: bool,
    /// Span of the attribute, for reporting errors against the right tokens.
    pub(crate) span: proc_macro2::Span,
}

/// Finds and parses the `#[secret]` attribute in `attrs`, if present.
///
/// Repeating the attribute is rejected rather than silently resolved, since
/// either of two conflicting intents could have been meant.
pub(crate) fn find(attrs: &[Attribute]) -> Result<Option<Secret>> {
    let mut found: Option<Secret> = None;

    for attr in attrs {
        if !attr.path().is_ident("secret") {
            continue;
        }

        if found.is_some() {
            return Err(Error::new(attr.span(), "duplicate `#[secret]` attribute"));
        }

        found = Some(parse(attr)?);
    }

    Ok(found)
}

fn parse(attr: &Attribute) -> Result<Secret> {
    let mut secret = Secret {
        skip: false,
        span: attr.span(),
    };

    match &attr.meta {
        syn::Meta::Path(_) => Ok(secret),
        syn::Meta::List(_) => {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    secret.skip = true;
                    return Ok(());
                }

                Err(meta.error("unrecognised option, expected `skip`"))
            })?;

            Ok(secret)
        }
        syn::Meta::NameValue(_) => Err(Error::new(
            attr.span(),
            "expected `#[secret]` or `#[secret(skip)]`",
        )),
    }
}

/// Reads a container-level `#[secret]`, which masks every field it covers.
///
/// `skip` is meaningless here — there is no wider rule for a container to
/// exempt itself from — so it is rejected rather than ignored.
pub(crate) fn blanket(attrs: &[Attribute]) -> Result<bool> {
    let Some(secret) = find(attrs)? else {
        return Ok(false);
    };

    if secret.skip {
        return Err(Error::new(
            secret.span,
            "`skip` applies to fields, not to a struct, enum or variant",
        ));
    }

    Ok(true)
}
