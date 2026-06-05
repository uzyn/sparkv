# SparKV

SparKV is an expirable in-memory key-value store for Rust.

## Features

1. Flexible expiration duration (a.k.a. time-to-live or TTL) per entry instead of database-wide common TTL.
    1. This is similar to that of DNS where each entries of the same domain can have its own unique TTL.
2. Automatically clears expired entries by default.
3. Generic over the value type, defaulting to `String`. Store any type you like.
4. Fast data entry enforcements, including ensuring entry size, database size and max TTL.
5. SparKV is intentionally not an LRU cache.
6. Configurable.

## Usage

Add SparKV crate to your Cargo dependencies:

```sh
$ cargo add sparkv
```

Quick start

```rust
use sparkv::SparKV;

// The value type defaults to String.
let mut sparkv = SparKV::new();
sparkv.set("your-key", "your-value".to_string()).unwrap(); // write
let value = sparkv.get("your-key").unwrap(); // read

// Write with unique TTL
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

See `config.rs` for more configuration options. `max_item_size` is an
`Option<usize>` — set it to `None` to disable per-entry size enforcement.


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


## TODO

1. Documentations

## License

MIT License<br>
Copyright © 2024 U-Zyn Chua
