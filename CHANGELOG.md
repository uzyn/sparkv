# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-06-16

This release makes SparKV generic over its value type and hardens the expiry
machinery. The `SparKV` and `KvEntry` names still default to their `String` forms,
so most call sites are unaffected — see the
[migration guide](./README.md#migrating-from-01-to-02) for the two breaking changes.

### Added

- Generic value type: `SparKV<V>` (defaulting to `String`). `get` / `delete` return
  `Option<V>`, and `get_item` returns `Option<&KvEntry<V>>`.
- `ValueSize` trait so custom types can participate in `max_item_size` enforcement,
  with built-in impls for `String`, `str`, `&str`, `Vec<u8>`, `[u8]`, `&[u8]`, and the
  scalar types (`bool`, `char`, `u8`–`u128`, `i8`–`i128`, `usize`, `isize`, `f32`, `f64`).
- Migration guide and a configuration reference in the README.

### Changed

- **Breaking:** `set` / `set_with_ttl` now take the value by value (`V`) instead of `&str`.
- **Breaking:** `Config.max_item_size` is now `Option<usize>` (was `usize`); `None`
  disables per-entry size enforcement.
- `contains_key` is now expiry-aware: it returns `false` for an expired-but-unswept
  entry, consistent with `get`.

### Fixed

- `clear_expired` no longer panics. It previously `unwrap`ed a key that could already
  be gone (e.g. after an overwrite or delete) and could crash; sweeping is now a safe
  no-op when there is nothing to remove.
- Removed re-entrant recursion in the auto-clear path that could overflow the stack;
  the sweep now removes entries directly rather than re-entering `delete`.
- Restored the `Eq` derive on `KvEntry`.

### Performance

- Replaced the `BinaryHeap`-based expiry index with a 1:1 `BTreeMap` keyed by
  `(expired_at, key)`. The index now stays bounded to the live set — repeatedly
  overwriting the same key no longer leaks stale heap entries that lingered until
  expiry, fixing unbounded heap growth.

## [0.1.1] - 2024-05-07

### Added

- Max-TTL enforcement and a quick-start guide in the README.

### Fixed

- Corrected sample code in the documentation.

## [0.1.0] - 2024-05-07

- Initial release: expirable in-memory key-value store with per-entry TTL,
  automatic expiry clearing, and configurable capacity, item-size, and TTL limits.

[0.2.0]: https://github.com/uzyn/sparkv/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/uzyn/sparkv/releases/tag/0.1.1
[0.1.0]: https://crates.io/crates/sparkv/0.1.0
