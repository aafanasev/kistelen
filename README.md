# kistelen

A derive macro that keeps sensitive struct fields out of `Debug` output.

*Kistelen* (кистэлэҥ) is the Yakut word for *secret*.

## The problem

Every production system logs, and logs travel: through application servers,
tracing systems, log aggregators, third-party monitoring, and long-term
storage. Each hop is somewhere a value can be read by someone who was never
meant to see it.

The exposure is almost never deliberate. Someone logs a request struct, an
error path prints the state it was holding, a tracing span captures a
parameter. The value goes along because nothing said it shouldn't.

`#[derive(Debug)]` makes this the default. It prints every field, and it is
applied reflexively — often to types that hold a password, a token, or a card
number. Because the implementation is generated, there is nothing in the source
for a reviewer to catch. Adding one field to a struct is enough to put a
credential into a log pipeline, and the diff that does it looks unremarkable.

## The approach

Replace the derive. `kistelen` generates the `Debug` implementation itself and
prints a mask in place of any field marked `#[secret]`:

```rust
use kistelen::Secret;

#[derive(Secret)]
struct User {
    id: i32,
    username: String,
    #[secret]
    password: String,
}
```

```text
User { id: 1, username: "alice", password: ■■■ }
```

Fields without the attribute are formatted by their own `Debug`, exactly as the
standard derive would. Marking a new field is a one-line change; a reviewer
seeing `#[secret]` disappear from a diff sees a deliberate act.

The mask is printed unquoted, so masked output can never be confused with a
string whose contents happen to look like a mask.

## Masking a whole type

When everything a type holds is sensitive, annotate the type rather than each
field. `#[secret(skip)]` exempts anything that is not:

```rust
#[derive(Secret)]
#[secret]
struct Session {
    #[secret(skip)]
    id: u32,
    token: String,
    refresh_token: String,
}
```

```text
Session { id: 7, token: ■■■, refresh_token: ■■■ }
```

Field names are kept. They are rarely the sensitive part, and they carry the
structure that made the value worth logging in the first place.

The same applies to enums, on the type or on a single variant:

```rust
#[derive(Secret)]
enum Credential {
    Anonymous,
    Token(#[secret] String),
    #[secret]
    Password { login: String, value: String },
}
```

## Choosing the mask

`with` replaces the mask, taking a string or a single character:

```rust
#[secret(with = "REDACTED")]
password: String,
```

`fixed` prints a set number of mask characters, hiding how long the value was.
A mask that tracks the real length discloses it, which matters for a PIN or a
CVV:

```rust
#[secret(fixed = 3)]
cvv: String,
```

`partial` exposes a little of each end, for values where the visible part is
operationally useful:

```rust
#[secret(partial)]
number: String,
```

```text
Card { number: 123■■■■■■■■■■456 }
```

Exposure is a fifth of the value at each end, never more than four characters,
so the proportion revealed falls as the value grows. Below eight characters
nothing is exposed at all and a fixed-width mask is printed instead — a few
characters of a short value narrow it too far, and a mask that tracked the
length would disclose that too.

`partial` reads the value, so the field must implement `Display`. The other
modes never look at it, and work on any type.

## Patterns

For shapes the modes above cannot express, `search` and `replacement` rewrite
the value. This needs the `regex` feature, which is off by default because a
regex engine is a large dependency to impose on users who only need a constant
mask:

```toml
kistelen = { version = "0.1", features = ["regex"] }
```

```rust
#[secret(search = r"([0-9]{4})([0-9]{8})([0-9]{4})", replacement = "****-****-****-$3")]
number: String,
```

```text
Card { number: ****-****-****-3456 }
```

Two rules keep a pattern from leaking what it was meant to hide:

- **The pattern must match the value end to end.** A partial match would leave
  everything outside it untouched, so a pattern written for one shape of value
  would print a different shape verbatim. Anything not fully matched is masked
  instead.
- **`$0` is rejected**, since it stands for the whole match and would print
  back the value the pattern had just matched.

Patterns are parsed when the macro expands, so an unusable one is a compile
error rather than a surprise on the first line of output that needs it. Each
is compiled at most once and reused.

Neither rule can catch a pattern that is simply too generous — a capture group
spanning the whole value will be substituted faithfully. Deciding what is safe
to expose remains yours.

## Options

A masked `Option` keeps its shape, because whether a value is set is usually
structural rather than sensitive:

```text
Account { password: Some(■■■) }
Account { password: None }
```

This is recognised by how the type is written. An alias for `Option<T>` cannot
be seen through at compile time, so it is masked whole — the safe reading of a
type the macro cannot inspect.

## Installation

```toml
[dependencies]
kistelen = "0.1"
```

## Scope

Masking applies to `Debug` and nothing else. A value can still leave the
process through `Display`, through serialisation, or by being read directly.
This is a guard on the accidental path — the implementation nobody wrote and
nobody reviews — not a guarantee about where a value can travel.

`Display` is deliberately left alone. It is meant to be human-readable, and
anything implementing it also gets `ToString`, so masking there would make
`to_string()` lossy in ways callers would not expect.

## Status

Early. Currently supported:

- structs, tuple structs, unit structs and enums, including generic ones
- `#[secret]` on a field, a type, or a single enum variant
- `#[secret(skip)]` to exempt a field from a wider rule
- `Option` masked through its `Some`
- both `{:?}` and `{:#?}`
- `with`, `fixed` and `partial` masking

Regex masking is available behind the `regex` feature.

Misuse is a compile error with an explanatory message rather than something
that silently does nothing — `skip` on a field no rule covers is rejected,
since it reads as protection that is not there.

The attribute is always spelled `#[secret]`. A project cannot nominate a name
of its own, because an attribute on a field must be registered by the derive
that reads it, and that list is fixed when this crate is compiled — see
[the spike](docs/custom-attribute-names.md) for what was tried.

## Related

[sekret](https://github.com/aafanasev/sekret) — the same idea for Kotlin,
implemented as a compiler plugin that rewrites the generated `toString()`.

## Licence

MIT
