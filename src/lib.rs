//! A [librsync](https://github.com/librsync/librsync)-compatible implementation of the
//! rsync algorithm in pure Rust, using SIMD operations where available.
//!
//! This crate is a fork of Dropbox's [`fast_rsync`](https://github.com/dropbox/fast_rsync)
//! that adds BLAKE2 signatures and stays wire-compatible with librsync.
//!
//! This crate offers three major APIs:
//!
//! 1. [Signature::calculate()], which takes a block of data and returns a
//!    "signature" of that data which is much smaller than the original data.
//! 2. [diff()], which takes a signature for some block A, and a block of data B, and
//!    returns a delta between block A and block B. If A and B are "similar", then
//!    the delta is usually much smaller than block B.
//! 3. [apply()], which takes a block A and a delta (as constructed by [diff()]), and
//!    (usually) returns the block B.
//!
//! Signatures and deltas use librsync's binary formats (compatible with librsync 2.2.1),
//! so any of these steps can be performed by librsync (or `rdiff`) instead. See the
//! README for the details of the compatibility guarantee.
//!
//! # Example
//!
//! ```
//! use librsync_oxide::{apply, diff, Signature, SignatureOptions, SignatureType};
//!
//! let base: Vec<u8> = (0..100_000u32).map(|i| (i % 251) as u8).collect();
//! let mut new = base.clone();
//! new[50_000..50_010].copy_from_slice(b"0123456789");
//!
//! // The receiver computes a signature of the data it already has.
//! let signature = Signature::calculate(
//!     &base,
//!     SignatureOptions {
//!         block_size: 1024,
//!         crypto_hash_size: 16,
//!         signature_type: SignatureType::Blake2,
//!     },
//! );
//!
//! // The sender computes a delta against that signature.
//! let mut delta = Vec::new();
//! diff(&signature.index(), &new, &mut delta).expect("diff failed");
//! assert!(delta.len() < new.len() / 10);
//!
//! // The receiver applies the delta to reconstruct the new data.
//! let mut reconstructed = Vec::new();
//! apply(&base, &delta, &mut reconstructed).expect("apply failed");
//! assert_eq!(reconstructed, new);
//! ```
#![allow(clippy::unreadable_literal)]
#![deny(missing_docs)]

mod consts;
mod crc;
mod diff;
mod hasher;
mod hashmap_variant;
mod md4;
mod patch;
mod signature;

#[cfg(test)]
mod tests;

pub use diff::{diff, DiffError};
pub use patch::{apply, apply_limited, ApplyError};
pub use signature::{
    IndexedSignature, Signature, SignatureOptions, SignatureParseError, SignatureType,
};
