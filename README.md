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


## TODO

1. Documentations

## License

MIT License<br>
Copyright © 2024 U-Zyn Chua
