//! Parsing of the `#[secret]` attribute.
//!
//! The attribute appears in two places with different meanings: on a container
//! (struct, enum, or enum variant) it masks every field within, and on a field
//! it masks that field alone. Options refine how the mask is rendered.

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Lit, Result};

/// The replacement given by `with`.
pub(crate) enum With {
    /// A whole string standing in for the value.
    Text(String),
    /// A single character, repeatable to a length.
    Character(char),
}

/// How a masked value is rendered.
pub(crate) enum Mode {
    /// The default mask.
    Full,
    /// A caller-supplied string in place of the value.
    Literal(String),
    /// A set number of mask characters, hiding the value's length.
    Fixed { count: usize, character: char },
    /// Leading and trailing characters exposed, the middle masked.
    Partial { character: char },
    /// The value rewritten by a pattern, for shapes the other modes cannot
    /// express.
    Replace {
        pattern: String,
        replacement: String,
    },
}

impl Mode {
    /// Whether rendering needs to read the value.
    ///
    /// Constant masks do not, which is what lets a field of any type be
    /// masked without requiring it to be printable.
    pub(crate) fn reads_value(&self) -> bool {
        matches!(self, Mode::Partial { .. } | Mode::Replace { .. })
    }
}

/// A parsed `#[secret]` attribute, before its options are reconciled.
pub(crate) struct Secret {
    pub(crate) skip: bool,
    with: Option<With>,
    fixed: Option<usize>,
    partial: bool,
    search: Option<String>,
    replacement: Option<String>,
    /// Span of the attribute, for reporting errors against the right tokens.
    pub(crate) span: Span,
}

impl Secret {
    /// Resolves the options into a single rendering mode.
    ///
    /// Options that cannot both apply are rejected rather than silently
    /// ordered, since either could reasonably have been meant.
    pub(crate) fn mode(&self) -> Result<Mode> {
        if let Some(mode) = self.replace_mode()? {
            return Ok(mode);
        }

        if self.fixed.is_some() && self.partial {
            return Err(Error::new(
                self.span,
                "`fixed` and `partial` cannot be combined: one hides the length, the other exposes part of the value",
            ));
        }

        if let Some(count) = self.fixed {
            return Ok(Mode::Fixed {
                count,
                character: self.character()?,
            });
        }

        if self.partial {
            return Ok(Mode::Partial {
                character: self.character()?,
            });
        }

        match &self.with {
            Some(With::Text(text)) => Ok(Mode::Literal(text.clone())),
            Some(With::Character(character)) => Ok(Mode::Literal(character.to_string())),
            None => Ok(Mode::Full),
        }
    }

    /// Resolves `search` and `replacement` into a replacing mode, if given.
    ///
    /// Both are required together: a pattern with nothing to put in its place,
    /// or a replacement with nothing to match, is a half-written rule rather
    /// than a usable one.
    fn replace_mode(&self) -> Result<Option<Mode>> {
        let (pattern, replacement) = match (&self.search, &self.replacement) {
            (None, None) => return Ok(None),
            (Some(pattern), Some(replacement)) => (pattern, replacement),
            (Some(_), None) => {
                return Err(Error::new(self.span, "`search` requires `replacement`"));
            }
            (None, Some(_)) => {
                return Err(Error::new(self.span, "`replacement` requires `search`"));
            }
        };

        if self.fixed.is_some() || self.partial || self.with.is_some() {
            return Err(Error::new(
                self.span,
                "`search` and `replacement` decide the whole output, so they cannot be combined with `with`, `fixed` or `partial`",
            ));
        }

        validate_pattern(pattern, replacement, self.span)?;

        Ok(Some(Mode::Replace {
            pattern: pattern.clone(),
            replacement: replacement.clone(),
        }))
    }

    /// The mask character for modes that repeat one.
    ///
    /// A multi-character `with` has no meaning when a count decides the
    /// output, so it is rejected rather than truncated.
    fn character(&self) -> Result<char> {
        match &self.with {
            Some(With::Character(character)) => Ok(*character),
            Some(With::Text(text)) => {
                let mut characters = text.chars();

                match (characters.next(), characters.next()) {
                    (Some(character), None) => Ok(character),
                    _ => Err(Error::new(
                        self.span,
                        "`with` must be a single character when combined with `fixed` or `partial`, which repeat it",
                    )),
                }
            }
            None => Ok(crate::DEFAULT_MASK_CHARACTER),
        }
    }
}

/// Rejects patterns that cannot work, at expansion time rather than on the
/// first line of output that needs them.
#[cfg(feature = "regex")]
fn validate_pattern(pattern: &str, replacement: &str, span: Span) -> Result<()> {
    if let Err(error) = regex_syntax::Parser::new().parse(pattern) {
        return Err(Error::new(
            span,
            format!("invalid `search` pattern: {error}"),
        ));
    }

    // `$0` and `${0}` stand for the whole match, so a replacement containing
    // one would print back the value the pattern just matched.
    if replacement.contains("$0") || replacement.contains("${0}") {
        return Err(Error::new(
            span,
            "`replacement` must not contain `$0`, which would restore the whole matched value",
        ));
    }

    Ok(())
}

#[cfg(not(feature = "regex"))]
fn validate_pattern(_pattern: &str, _replacement: &str, span: Span) -> Result<()> {
    Err(Error::new(
        span,
        "`search` and `replacement` need the `regex` feature: kistelen = { version = \"0.1\", features = [\"regex\"] }",
    ))
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
        with: None,
        fixed: None,
        partial: false,
        search: None,
        replacement: None,
        span: attr.span(),
    };

    match &attr.meta {
        syn::Meta::Path(_) => return Ok(secret),
        syn::Meta::NameValue(_) => {
            return Err(Error::new(
                attr.span(),
                "expected `#[secret]` or `#[secret(...)]` with options",
            ));
        }
        syn::Meta::List(_) => {}
    }

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("skip") {
            secret.skip = true;
            return Ok(());
        }

        if meta.path.is_ident("partial") {
            secret.partial = true;
            return Ok(());
        }

        if meta.path.is_ident("with") {
            secret.with = Some(match meta.value()?.parse::<Lit>()? {
                Lit::Str(text) => With::Text(text.value()),
                Lit::Char(character) => With::Character(character.value()),
                other => {
                    return Err(Error::new(
                        other.span(),
                        "`with` expects a string or character literal",
                    ));
                }
            });

            return Ok(());
        }

        if meta.path.is_ident("fixed") {
            let count: usize = match meta.value()?.parse::<Lit>()? {
                Lit::Int(integer) => integer.base10_parse()?,
                other => {
                    return Err(Error::new(other.span(), "`fixed` expects an integer"));
                }
            };

            if count == 0 {
                return Err(meta.error("`fixed` must be at least 1, or nothing is printed"));
            }

            secret.fixed = Some(count);
            return Ok(());
        }

        for (name, target) in [
            ("search", &mut secret.search),
            ("replacement", &mut secret.replacement),
        ] {
            if meta.path.is_ident(name) {
                *target = Some(match meta.value()?.parse::<Lit>()? {
                    Lit::Str(text) => text.value(),
                    other => {
                        return Err(Error::new(
                            other.span(),
                            format!("`{name}` expects a string literal"),
                        ));
                    }
                });

                return Ok(());
            }
        }

        Err(meta.error(
            "unrecognised option, expected `skip`, `with`, `fixed`, `partial`, `search` or `replacement`",
        ))
    })?;

    if secret.skip
        && (secret.with.is_some()
            || secret.fixed.is_some()
            || secret.partial
            || secret.search.is_some()
            || secret.replacement.is_some())
    {
        return Err(Error::new(
            secret.span,
            "`skip` cannot be combined with other options, as the value is not masked at all",
        ));
    }

    Ok(secret)
}

/// Reads a container-level `#[secret]`, which masks every field it covers.
///
/// `skip` is meaningless here — there is no wider rule for a container to
/// exempt itself from — so it is rejected rather than ignored.
pub(crate) fn blanket(attrs: &[Attribute]) -> Result<Option<Secret>> {
    let Some(secret) = find(attrs)? else {
        return Ok(None);
    };

    if secret.skip {
        return Err(Error::new(
            secret.span,
            "`skip` applies to fields, not to a struct, enum or variant",
        ));
    }

    Ok(Some(secret))
}
