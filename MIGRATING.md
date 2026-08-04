# Migrating from 1.x to 2.0

Most of this migration is mechanical: renamed methods produce compile errors,
and renamed features produce "unknown feature" errors. Start with the section
below, which covers the changes that do *not* announce themselves.

## Changes that compile silently

### `Error::ErrorString` and the blanket `From` impl are gone

1.x had a catch-all `Error::ErrorString(String)` variant, plus:

```rust
impl<S: Into<String>> From<S> for Error
```

Both are removed. `Error` now has specific variants for each failure
(`InvalidCharacter`, `NoDigits`, `DuplicatedDecimalPoint`, and so on).

Code that matched on `ErrorString` will fail to compile, which is easy to spot.
The quieter case is code that relied on the blanket impl to turn a `&str` or
`String` into an `Error` via `?` or `.into()` — that conversion no longer
exists.

`Error` is also now `#[non_exhaustive]`, so `match` expressions over it need a
wildcard arm.

### Binary formats no longer need a feature flag

In 1.x, deserializing from a non-self-describing format (`bincode`, `postcard`,
MessagePack) required the `serde-str` feature — or `serde-bincode`, which was an
alias for it. Without it, deserialization failed at *runtime* with
`DeserializeAnyNotSupported`, while serialization appeared to work fine.

2.0 detects such formats automatically via `Deserializer::is_human_readable()`.

**Remove `serde-str` and `serde-bincode` from your `Cargo.toml`. Your code keeps
working.** The encoded bytes are unchanged, so data written by 1.x still reads.

If you were *not* using those features and had given up on binary formats, they
now work with no configuration.

## Renamed methods

| 1.x | 2.0 |
| --- | --- |
| `Decimal::new` | `Decimal::from_i64_with_scale` |
| `Decimal::try_new` | `Decimal::try_from_i64_with_scale` |
| `Decimal::from_scientific` | `Decimal::from_scientific_exact` |

`new` and `try_new` remain as deprecated aliases throughout 2.x and will be
removed in 3.0, so existing call sites keep compiling with a warning. Note that
`#[deprecated]` does not carry a machine-applicable fix, so `cargo fix` cannot
perform the rename — it is a find/replace of `Decimal::new(` →
`Decimal::from_i64_with_scale(`.

`from_scientific_exact` is a straight rename of `from_scientific`; behaviour is
unchanged. `from_scientific_lossy` is also unchanged.

## Removed methods

These were deprecated in 1.x and are now gone.

| 1.x | 2.0 |
| --- | --- |
| `Decimal::min_value()` | `Decimal::MIN` |
| `Decimal::max_value()` | `Decimal::MAX` |
| `Decimal::is_negative()` | `Decimal::is_sign_negative()` |
| `Decimal::is_positive()` | `Decimal::is_sign_positive()` |
| `Decimal::set_sign()` | `set_sign_positive()` / `set_sign_negative()` |

## Renamed and removed features

| 1.x | 2.0 |
| --- | --- |
| `serde-float` | `serde-default-number` |
| `serde-arbitrary-precision` | `serde-default-exact` |
| `serde-with-arbitrary-precision` | `serde-default-exact` |
| `serde-with-float`, `serde-with-str` | *removed — always available* |
| `serde-str`, `serde-bincode` | *removed — now automatic* |
| `tokio-pg` | `db-tokio-postgres` |
| `db-diesel2-mysql` | `db-diesel-mysql` |
| `db-diesel2-postgres` | `db-diesel-postgres` |
| `rand-0_9` | `rand-0_10` |

Cargo has no mechanism for deprecating a feature, so all of the above are hard
removals that produce an "unknown feature" error.

### The serde behaviour features

There are now exactly two, and they are independent. All four combinations are
supported:

| Features | `1.0000` serializes as | reads unquoted 28-digit |
| --- | --- | --- |
| *(neither)* | `"1.0000"` | error |
| `serde-default-number` | `1.0` | error |
| `serde-default-exact` | `"1.0000"` | exact |
| both | `1.0000` | exact |

`serde-default-number` controls *shape* — unquoted number instead of quoted
string. `serde-default-exact` controls *fidelity* — exact rather than via `f64`.
Fidelity always affects reading; it affects writing only alongside
`serde-default-number`.

Note that `serde-default-exact` on its own is a useful configuration: it writes
quoted strings while still accepting high-precision numbers on input.

If you previously used `serde-with-float` or `serde-with-str` to get the
per-field helper modules, drop the feature — the helpers are now always
available under `serde`:

```rust
#[serde(with = "rust_decimal::serde::str")]
```

`arbitrary_precision` and `arbitrary_precision_option` still require
`serde-default-exact`, because they pull in `serde_json` and `zmij`.

## Other changes

- `rust_decimal::str` is no longer a public module. Its only public item was
  `overflow_128`, an internal parsing helper.
- `UnpackedDecimal` is `#[non_exhaustive]`. Fields remain readable; construct it
  via `Decimal::unpack()` rather than a struct literal.
- `Error` implements `core::error::Error` rather than `std::error::Error`, so it
  is now usable in `no_std` builds.
- Rocket integration, `rand` 0.8 and 0.9 support, and the legacy `ops` module
  have been removed.
- Edition 2024; minimum supported Rust version is 1.85.0.

## Note for upgrades from 1.41 or earlier

`Decimal::from_str` gained support for scientific notation in **1.42.0**:

```rust
Decimal::from_str("1e5")   // 1.41 and earlier: Err
                           // 1.42.0 onwards:   Ok(100000)
```

This is not a 2.0 change, but if you are jumping from 1.41 or earlier straight
to 2.0 you will encounter it. It matters if you relied on a parse failure to
reject scientific notation as invalid input.
