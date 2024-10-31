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

use ::core::fmt::Display;

use alloc::string::{String, ToString};
use thiserror_no_std::Error;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

#[cfg(feature = "helpers")]
pub mod account;
#[cfg(any(feature = "json-rpc", feature = "websocket", feature = "helpers"))]
pub mod asynch;
#[cfg(any(feature = "json-rpc", feature = "websocket"))]
pub mod clients;
pub mod constants;
#[cfg(feature = "core")]
pub mod core;
#[cfg(feature = "helpers")]
pub mod ledger;
pub mod macros;
#[cfg(any(feature = "models"))]
pub mod models;
#[cfg(feature = "helpers")]
pub mod transaction;
#[cfg(feature = "utils")]
pub mod utils;
#[cfg(feature = "wallet")]
pub mod wallet;

pub extern crate serde_json;

#[cfg(any(feature = "models"))]
mod _serde;

#[cfg(all(
    feature = "helpers",
    not(any(
        feature = "tokio-rt",
        feature = "embassy-rt",
        feature = "actix-rt",
        feature = "async-std-rt",
        feature = "futures-rt",
        feature = "smol-rt"
    ))
))]
compile_error!("Cannot enable `helpers` without enabling a runtime feature (\"*-rt\"). This is required for sleeping between retries internally.");
#[cfg(all(
    feature = "helpers",
    not(any(feature = "json-rpc", feature = "websocket",))
))]
compile_error!("Cannot enable `helpers` without enabling a client feature (\"json-rpc\", \"websocket\"). This is required for interacting with the XRP Ledger.");

#[derive(Debug, Error)]
pub enum XRPLSerdeJsonError {
    SerdeJsonError(serde_json::Error),
    InvalidNoneError(String),
    UnexpectedValueType {
        expected: String,
        found: serde_json::Value,
    },
}

impl Display for XRPLSerdeJsonError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            XRPLSerdeJsonError::SerdeJsonError(err) => write!(f, "{}", err),
            XRPLSerdeJsonError::InvalidNoneError(err) => {
                write!(f, "Invalid None value on field: {}", err)
            }
            XRPLSerdeJsonError::UnexpectedValueType { expected, found } => {
                write!(
                    f,
                    "Unexpected value type (expected: {}, found: {})",
                    expected, found
                )
            }
        }
    }
}

impl From<serde_json::Error> for XRPLSerdeJsonError {
    fn from(err: serde_json::Error) -> Self {
        XRPLSerdeJsonError::SerdeJsonError(err)
    }
}

impl PartialEq for XRPLSerdeJsonError {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}
