# librsync\_oxide

[![Crates.io](https://img.shields.io/crates/v/librsync_oxide.svg)](https://crates.io/crates/librsync_oxide)
[![Build Status](https://github.com/mriise/librsync_xodie/workflows/Rust/badge.svg)](https://github.com/mriise/librsync_xodie/actions)

[Documentation](https://docs.rs/librsync_oxide)

A [librsync](https://github.com/librsync/librsync)-compatible implementation of
the rsync algorithm (signature / delta / patch) in pure Rust, using SIMD
operations where available.

`librsync_oxide` is a fork of Dropbox's
[`fast_rsync`](https://github.com/dropbox/fast_rsync). It adds BLAKE2b-256
signatures (librsync's default hash) alongside MD4 and stays wire-compatible
with librsync (see [librsync compatibility](#librsync-compatibility)).

SIMD-accelerated MD4 is currently supported on x86, x86-64, and aarch64
targets; vector extensions are detected at runtime.

## Usage

```toml
[dependencies]
librsync_oxide = "0.3"
```

```rust
use librsync_oxide::{apply, diff, Signature, SignatureOptions, SignatureType};

let base: Vec<u8> = (0..100_000u32).map(|i| (i % 251) as u8).collect();
let mut new = base.clone();
new[50_000..50_010].copy_from_slice(b"0123456789");

// The receiver computes a signature of the data it already has.
let signature = Signature::calculate(
    &base,
    SignatureOptions {
        block_size: 1024,
        crypto_hash_size: 16,
        signature_type: SignatureType::Blake2,
    },
);

// The sender computes a delta against that signature.
let mut delta = Vec::new();
diff(&signature.index(), &new, &mut delta).expect("diff failed");

// The receiver applies the delta to reconstruct the new data.
let mut reconstructed = Vec::new();
apply(&base, &delta, &mut reconstructed).expect("apply failed");
assert_eq!(reconstructed, new);
```

Use `Signature::serialized()` / `Signature::deserialize()` to move signatures
across the wire, and `apply_limited` instead of `apply` when the delta comes
from an untrusted source.

## The rsync algorithm
This crate offers three major APIs:

1. `Signature::calculate`, which takes a block of data and returns a
   "signature" of that data which is much smaller than the original data.
2. `diff`, which takes a signature for some block A, and a block of data B, and
   returns a delta between block A and block B. If A and B are "similar", then
   the delta is usually much smaller than block B.
3. `apply`, which takes a block A and a delta (as constructed by `diff`), and
   (usually) returns the block B.

These functions can be used to implement a protocol for efficiently
transferring data over a network. Suppose hosts A and B have similar versions
of some file `foo`, and host B would like to acquire A's copy.
1. Host B calculates the `Signature` of `foo_B` and sends it to A. This is
   cheap because the signature can be 1000X smaller than `foo_B` itself. (The
   precise factor is configurable and creates a tradeoff between signature size
   and usefulness. A larger signature enables the creation of smaller and more
   precise deltas.)
2. Host A calculates a `diff` from B's signature and `foo_A`, and sends it to
   `B`.
3. Host B attempts to `apply` the delta to `foo_B`. The resulting data is
   _probably_ (\*) equal to `foo_A`.

(\*) Note the caveat. Blocks are matched by a weak rolling checksum plus a
(possibly truncated) strong hash, and `SignatureType::Md4` uses the broken MD4
algorithm. You should not assume that `diff` produces a correct delta, whatever
the signature type. Always verify the output of `apply` by some other means,
such as a cryptographic hash of the whole file.

## librsync compatibility

`librsync_oxide` uses librsync's binary formats, so any step of the protocol
can be done by librsync (or its `rdiff` CLI) instead of this crate, and vice
versa. Changes that break wire compatibility with librsync will not be
accepted.

| Format | librsync name | Magic | Supported |
| --- | --- | --- | --- |
| MD4 signature (rollsum) | `RS_MD4_SIG_MAGIC` | `0x72730136` | yes (`SignatureType::Md4`) |
| BLAKE2 signature (rollsum) | `RS_BLAKE2_SIG_MAGIC` | `0x72730137` | yes (`SignatureType::Blake2`) |
| MD4 signature (RabinKarp) | `RS_RK_MD4_SIG_MAGIC` | `0x72730146` | no |
| BLAKE2 signature (RabinKarp) | `RS_RK_BLAKE2_SIG_MAGIC` | `0x72730147` | no |
| Delta | `RS_DELTA_MAGIC` | `0x72730236` | yes |

- Signatures produced by `Signature::calculate` are byte-for-byte identical to
  librsync's for the same block size, strong-sum length and hash.
- Deltas produced by `diff` can be applied by librsync, and deltas produced by
  librsync can be applied by `apply` / `apply_limited`.
- Only the classic "rollsum" rolling checksum is implemented. librsync ≥ 2.2
  defaults to the RabinKarp rollsum, so when generating a signature with
  `rdiff` for consumption by this crate pass `-R rollsum` (and `-H md4` or
  `-H blake2` as desired). Signatures with a RabinKarp magic are rejected by
  `Signature::deserialize`.
- `diff` operates on in-memory buffers; there is no streaming equivalent of
  librsync's job API.

`librsync_oxide` is compatible with **librsync 2.2.1**. The test suite
verifies all of the above against the real librsync library, through the
[`librsync`](https://crates.io/crates/librsync) crate (0.2.5), which builds a
vendored librsync 2.2.2 development snapshot (2.2.1 plus unreleased fixes,
none of which touch the wire formats). The rollsum signature and delta formats
have not changed since librsync 2.0, and they are what `rdiff` 2.x reads and
writes with `-R rollsum`.

## Benchmarks

Performance depends on the hardware (SIMD width, cache sizes), so no numbers
are published here. Run the benchmarks yourself:

```
cargo bench
```

The benchmarks live in `benches/rsync_bench.rs` and compare signature
calculation, delta computation and delta application against `librsync`.

## Contributing
Issues and pull requests are welcome at
[mriise/librsync_xodie](https://github.com/mriise/librsync_xodie). Changes
must keep the librsync interoperability tests passing.

## License
This project is licensed under [the Apache-2.0
license](http://www.apache.org/licenses/LICENSE-2.0).

Copyright (c) 2025 mriise  
Copyright (c) 2019 Dropbox, Inc.  
Copyright (c) 2016 bacher09, Artyom Pavlov (RustCrypto/hashes/MD4).
