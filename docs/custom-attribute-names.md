# Spike: custom attribute names

## The question

The Kotlin plugin lets a project nominate its own annotation:

```groovy
sekret {
    annotations = ["com.example.Confidential"]
}
```

A domain model can then carry `@Confidential` and depend on nothing. The
masking library is a build-time concern that the model never mentions.

Can `kistelen` do the same — let a project write `#[confidential]` instead of
`#[secret]`?

## Finding

No, not for a name the user chooses. The constraint is structural rather than
a gap in the implementation.

An attribute on a struct field must be *registered* by a derive macro, in the
list given when the macro is declared:

```rust
#[proc_macro_derive(Secret, attributes(secret))]
```

Anything not in that list is rejected by name resolution, before the macro is
ever invoked:

```
error: cannot find attribute `confidential` in this scope
 --> src/main.rs:5:7
  |
5 |     #[confidential]
  |       ^^^^^^^^^^^^
```

Adding the name to the list does make it work — verified by hand, masking and
all:

```rust
#[proc_macro_derive(Secret, attributes(secret, confidential))]
```

```text
Account { password: ■■■ }
```

But that list is baked in when `kistelen-macros` is compiled, which happens
before any user code is parsed. A downstream crate cannot extend it.

## Why the Kotlin approach does not carry over

Sekret's configuration works because a Gradle plugin sits between the build and
the compiler, and can pass options into the compiler plugin. Cargo has no
equivalent channel: a proc macro receives a token stream and nothing else. It
can read environment variables at expansion time, but that does not help — the
registered attribute list is fixed earlier, when the macro crate itself is
compiled, so no runtime lookup can extend it.

Attribute macros on fields would sidestep registration, but they are not
permitted on stable Rust. `#![register_tool(...)]`, which would allow
`#[mytool::confidential]`, is nightly-only.

## What is possible instead

**A fixed set of aliases.** `attributes(secret, confidential, sensitive)` would
work, optionally behind cargo features so a project opts into the spelling it
wants. This is a menu the library chooses, not a name the user chooses.
Cheap, but it satisfies the letter of the request rather than its point.

**A wrapper derive.** A project can write its own derive that re-emits the type
with `#[secret]` attached and delegates. The domain model then mentions only
the project's own macro. This works today and needs nothing from `kistelen`,
but it costs a proc-macro crate per project, which is far more than the
annotation was worth.

## Recommendation

Close this as not supportable, and say so in the documentation rather than
leaving it as an implied gap for anyone arriving from the Kotlin plugin.

The motivation behind the Kotlin feature — keeping a masking library out of a
domain model — is also weaker here. `#[secret]` is an inert attribute; the
dependency it implies is a `use` of the derive in the same file, not a runtime
coupling. The Kotlin version avoids a jar on the classpath. There is no
equivalent weight to avoid.

If the fixed-alias option is ever wanted, it is a few lines. It is left undone
deliberately: offering `#[confidential]` and `#[sensitive]` as synonyms would
give three spellings for one idea, which is worse for anyone reading a codebase
than the single spelling is for anyone writing one.
