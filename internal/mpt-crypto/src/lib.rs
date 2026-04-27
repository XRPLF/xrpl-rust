//! Safe Rust wrappers around [`mpt_crypto_sys`] for XLS-0096 Confidential MPTs.
//!
//! The sys crate exposes ~88 raw `unsafe extern "C"` functions; this crate
//! curates the ~12 a client library actually needs, with idiomatic Rust types,
//! `Result`-based errors, and `Zeroize`-on-drop secrets.
//!
//! # Module map
//!
//! | Module | Purpose |
//! |---|---|
//! | [`keypair`] | Generate ElGamal holder keys |
//! | [`encrypt`] | Encrypt / decrypt amounts; produce blinding factors |
//! | [`commit`]  | Pedersen commitments |
//! | [`context`] | Per-transaction-type context hashes (replay binding) |
//! | [`prove`]   | Generate the four per-transaction-type proofs |
//! | [`types`]   | Strongly-typed byte-array wrappers (Privkey, Pubkey, etc.) |
//! | [`error`]   | [`Error`] and [`Result`] |
//!
//! # Example
//!
//! ```ignore
//! // `ignore`: this code is for documentation only — see Cargo.toml's
//! // `doctest = false`. Real end-to-end coverage lives in
//! // `tests/integration.rs`.
//! use mpt_crypto::{keypair, encrypt};
//!
//! let (sk, pk) = keypair::generate().unwrap();
//! let r        = encrypt::random_blinding_factor().unwrap();
//! let ct       = encrypt::encrypt(1_000, &pk, &r).unwrap();
//! let m        = encrypt::decrypt(&ct, &sk).unwrap();
//! assert_eq!(m, 1_000);
//! ```

pub mod commit;
pub mod context;
pub mod encrypt;
pub mod error;
pub mod keypair;
pub mod prove;
pub mod types;

pub use error::{Error, Result};
pub use types::*;
