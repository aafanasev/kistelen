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

Planned: custom mask strings, fixed-width masks that hide length, partial
masking, regex-driven masking.

Misuse is a compile error with an explanatory message rather than something
that silently does nothing — `skip` on a field no rule covers is rejected,
since it reads as protection that is not there.

## Related

[sekret](https://github.com/aafanasev/sekret) — the same idea for Kotlin,
implemented as a compiler plugin that rewrites the generated `toString()`.

## Licence

MIT
