//! Low-level FFI bindings to the [mpt-crypto](https://github.com/XRPLF/mpt-crypto)
//! C library, which implements the cryptographic primitives for
//! [XLS-0096 Confidential MPTs](https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0096-confidential-mpt).
//!
//! **Not a stable public API.** Use the high-level `mpt-crypto` wrapper crate
//! instead — that crate provides idiomatic Rust types, error handling, and
//! secret-scrubbing.
//!
//! See [`build.rs`] for how the native library is located on the filesystem.

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]

include!("bindings.rs");
