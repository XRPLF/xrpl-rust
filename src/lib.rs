// Copyright 2021 589Labs Developers.
// Licensed under the ISC License

//! Utilities for interacting with the XRP Ledger.
//!
//! A pure Rust implementation for interacting with the XRP Ledger. The
//! xrpl-rust crate simplifies the hardest parts of XRP Ledger interaction
//! including serialization and transaction signing while providing idiomatic
//! Rust functionality for XRP Ledger transactions and core server API
//! (rippled) objects.
//!
//! # Quick Start
//!
//! TODO
//!
//! # The XRP Ledger
//!
//! For the user guide and further documentation, please read
//! [XRP Ledger](https://xrpl.org/docs.html).
#![no_std]
#![allow(dead_code)] // Remove eventually

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

pub mod constants;
#[cfg(feature = "core")]
pub mod core;
pub mod macros;
#[cfg(feature = "models")]
pub mod models;
#[cfg(feature = "utils")]
pub mod utils;
pub mod wallet;

pub extern crate indexmap;
pub extern crate serde_json;

mod _anyhow;
mod _serde;
