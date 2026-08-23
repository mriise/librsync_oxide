# fast\_rsync

[![Crates.io](https://img.shields.io/crates/v/librsync_oxide.svg)](https://crates.io/crates/librsync_oxide)
[![Build Status](https://github.com/mriise/librsync_xodie/workflows/Rust/badge.svg)](https://github.com/mriise/librsync_xodie/actions)

[Documentation](https://docs.rs/librsync_oxide)

A faster implementation of [librsync](https://github.com/librsync/librsync) in
pure Rust, using SIMD operations where available.

SIMD is currently supported on x86, x86-64, and aarch64 targets.

## The rsync algorithm
This crate offers three major APIs:

1. `Signature::calculate`, which takes a block of data and returns a
   "signature" of that data which is much smaller than the original data.
2. `diff`, which takes a signature for some block A, and a block of data B, and
   returns a delta between block A and block B. If A and B are "similar", then
   the delta is usually much smaller than block B.
3. `apply`, which takes a block A and a delta (as constructed by `diff`), and
   (usually) returns the block B.

These functions can be used to implement an protocol for efficiently
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

(\*) Note the caveat. `librsync_oxide` can use the insecure MD4 algorithm.
Therefore, you should not always trust that `diff` will produce a correct delta. You
must always verify the integrity of the output of `apply` using some other
mechanism, such as a cryptographic hash function like SHA-256 or by using blake2 signatures.

## Benchmarks

Performance is hardware-dependent (SIMD width, cache sizes), so rather than
publish numbers we encourage you to run the benchmarks yourself:

```
cargo bench
```

The benchmarks live in `benches/rsync_bench.rs` and compare signature
calculation, delta computation and delta application against `librsync`.
Note that `librsync_oxide` detects available vector extensions at runtime and
uses them as appropriate; `-C target-cpu` is not required.

## Contributing
Pull requests are welcome! We ask that you agree to [Dropbox's Contributor
License Agreement](https://opensource.dropbox.com/cla/) for your changes to be
merged.

## License
This project is licensed under [the Apache-2.0
license](http://www.apache.org/licenses/LICENSE-2.0).

Copyright (c) 2019 Dropbox, Inc.  
Copyright (c) 2016 bacher09, Artyom Pavlov (RustCrypto/hashes/MD4).
