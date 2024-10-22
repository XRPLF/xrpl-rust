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

#[cfg(feature = "account-helpers")]
pub mod account;
#[cfg(any(
    feature = "json-rpc",
    feature = "websocket",
    feature = "account-helpers",
    feature = "ledger-helpers",
    feature = "transaction-helpers",
    feature = "wallet-helpers"
))]
pub mod asynch;
#[cfg(any(feature = "json-rpc", feature = "websocket"))]
pub mod clients;
pub mod constants;
#[cfg(feature = "core")]
pub mod core;
#[cfg(feature = "ledger-helpers")]
pub mod ledger;
pub mod macros;
#[cfg(any(
    feature = "ledger-models",
    feature = "request-models",
    feature = "result-models",
    feature = "transaction-models"
))]
pub mod models;
#[cfg(feature = "transaction-helpers")]
pub mod transaction;
#[cfg(feature = "utils")]
pub mod utils;
#[cfg(feature = "wallet")]
pub mod wallet;

pub extern crate serde_json;

#[cfg(any(
    feature = "ledger-models",
    feature = "request-models",
    feature = "result-models",
    feature = "transaction-models"
))]
mod _serde;

#[cfg(all(
    any(feature = "transaction-helpers", feature = "wallet-helpers"),
    not(any(
        feature = "tokio-rt",
        feature = "embassy-rt",
        feature = "actix-rt",
        feature = "async-std-rt",
        feature = "futures-rt",
        feature = "smol-rt"
    ))
))]
compile_error!("Cannot enable `transaction-helpers` or `wallet-helpers` without enabling a runtime feature (\"*-rt\"). This is required for sleeping between retries internally.");
