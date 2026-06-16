# SparKV

[![Crates.io](https://img.shields.io/badge/crates.io-v0.2.0-orange.svg)](https://crates.io/crates/sparkv)
[![Documentation](https://docs.rs/sparkv/badge.svg)](https://docs.rs/sparkv)
[![CI](https://github.com/uzyn/sparkv/actions/workflows/rust.yml/badge.svg)](https://github.com/uzyn/sparkv/actions/workflows/rust.yml)
[![Dependencies](https://deps.rs/crate/sparkv/latest/status.svg)](https://deps.rs/crate/sparkv)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

SparKV is an expirable in-memory key-value store for Rust with **no dependencies**.

## Features

1. **Zero dependencies** — pure Rust `std`, nothing added to your dependency tree.
2. Flexible expiration duration (a.k.a. time-to-live or TTL) per entry instead of a database-wide common TTL.
    1. This is similar to DNS, where each entry of the same domain can have its own unique TTL.
3. Automatically clears expired entries by default.
4. Generic over the value type, defaulting to `String`. Store any type you like.
5. Fast data-entry enforcements, including entry size, database size and max TTL.
6. SparKV is intentionally not an LRU cache.
7. Configurable.

## Usage

Add SparKV to your Cargo dependencies:

```sh
$ cargo add sparkv
```

Quick start:

```rust
use sparkv::SparKV;

// The value type defaults to String.
let mut sparkv = SparKV::new();
sparkv.set("your-key", "your-value".to_string()).unwrap(); // write
let value = sparkv.get("your-key").unwrap(); // read

// Write with a unique TTL
sparkv.set_with_ttl("diff-ttl", "your-value".to_string(), std::time::Duration::from_secs(60)).unwrap();
```

`set` takes the value by value, and `get` returns an owned clone (so the value type
must implement `Clone`).

### Generic value types

SparKV is generic over the value type. Built-in types (integers, `Vec<u8>`, etc.)
work out of the box:

```rust
use sparkv::SparKV;

let mut counters: SparKV<i64> = SparKV::new();
counters.set("hits", 42).unwrap();
assert_eq!(counters.get("hits"), Some(42));
```

To store your own type, implement `ValueSize` so SparKV can enforce `max_item_size`
(return `0` if you don't care about size enforcement):

```rust
use sparkv::{SparKV, ValueSize};

#[derive(Clone)]
struct Session {
    user_id: u64,
    csrf: String,
}

impl ValueSize for Session {
    fn value_size(&self) -> usize {
        std::mem::size_of::<u64>() + self.csrf.len()
    }
}

let mut sessions: SparKV<Session> = SparKV::new();
sessions
    .set_with_ttl(
        "sid",
        Session { user_id: 1, csrf: "abc".to_string() },
        std::time::Duration::from_secs(60),
    )
    .unwrap();
```

## API overview

| Method | Description |
| --- | --- |
| `set(key, value)` | Insert/overwrite using `config.default_ttl`. |
| `set_with_ttl(key, value, ttl)` | Insert/overwrite with an explicit TTL. |
| `get(key) -> Option<V>` | Owned clone of a live (non-expired) value. |
| `get_item(key) -> Option<&KvEntry<V>>` | Borrow the live entry, including its `expired_at`. |
| `delete(key) -> Option<V>` | Remove and return the value. |
| `contains_key(key) -> bool` | Expiry-aware presence check (agrees with `get`). |
| `len() / is_empty()` | O(1) physical count (may include expired-but-unswept entries). |
| `clear_expired() -> usize` | Sweep expired entries, returning how many were removed. |

`set` / `set_with_ttl` return `Result<(), Error>`, where [`Error`](https://docs.rs/sparkv/latest/sparkv/enum.Error.html)
is one of `CapacityExceeded`, `ItemSizeExceeded`, or `TTLTooLong`.

## Configuration

Construct a store with defaults via `SparKV::new()`, or supply your own `Config`
with `SparKV::with_config(config)`:

```rust
use sparkv::{Config, SparKV};

let mut config = Config::new();
config.max_items = 50_000;
config.max_item_size = None; // disable per-entry size enforcement
config.default_ttl = std::time::Duration::from_secs(30);

let mut sparkv: SparKV = SparKV::with_config(config);
```

| Field | Type | Default | Purpose |
| --- | --- | --- | --- |
| `max_items` | `usize` | `10_000` | Maximum number of entries before `set` returns `CapacityExceeded`. |
| `max_item_size` | `Option<usize>` | `Some(500_000)` | Maximum per-entry size (via `ValueSize`); `None` disables the check. |
| `max_ttl` | `Duration` | `1 hour` | Upper bound on any entry's TTL; exceeding it returns `TTLTooLong`. |
| `default_ttl` | `Duration` | `5 minutes` | TTL applied by `set` when none is given. |
| `auto_clear_expired` | `bool` | `true` | Sweep expired entries automatically on writes/deletes. |

## Migrating from 0.1 to 0.2

`0.2.0` makes the value type generic (`SparKV<V>`, defaulting to `String`). The store
behaves exactly as before; the breaking changes are limited to how values are passed in
and how `max_item_size` is configured. The `SparKV` and `KvEntry` type names still
resolve to their `String` forms, so type annotations and struct fields are unaffected.

**1. `set` / `set_with_ttl` now take the value by value.** Pass an owned value instead
of a `&str`:

```rust
// 0.1
kv.set("key", "value");
// 0.2
kv.set("key", "value".to_string()); // or "value".into()
```

`get` and `delete` still return `Option<String>` for the default `String` store, so
your read sites do not change.

**2. `Config.max_item_size` is now `Option<usize>`.** Wrap the limit in `Some`, or use
`None` to disable size enforcement:

```rust
// 0.1
config.max_item_size = 500_000;
// 0.2
config.max_item_size = Some(500_000); // or None to disable
```

**3. Custom value types need a `ValueSize` impl** — only relevant if you switch away from
`String`. Built-in types (integers, `Vec<u8>`, `&[u8]`, etc.) already implement it; see
the generic example above.

No other call sites change.

## Changelog

See [CHANGELOG.md](./CHANGELOG.md) for the release history.

## License

MIT License<br>
Copyright © 2024 U-Zyn Chua
