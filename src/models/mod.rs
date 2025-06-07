//! Top-level modules for the models package.
//!
//! Order of models:
//! 1. Type of model
//! 2. Required common fields in alphabetical order
//! 3. Optional common fields in alphabetical order
//! 4. Required specific fields in alphabetical order
//! 5. Optional specific fields in alphabetical order

#[cfg(feature = "models")]
#[allow(clippy::too_many_arguments)]
pub mod ledger;
#[cfg(feature = "models")]
#[allow(clippy::too_many_arguments)]
pub mod requests;
#[cfg(feature = "models")]
#[allow(clippy::too_many_arguments)]
pub mod results;
#[cfg(feature = "models")]
#[allow(clippy::too_many_arguments)]
pub mod transactions;

mod amount;
mod currency;
mod exceptions;
mod flag_collection;
mod model;

pub use amount::*;
pub use currency::*;
pub use exceptions::*;
pub use flag_collection::*;
pub use model::*;

use alloc::borrow::Cow;
use derive_new::new;
use serde::{Deserialize, Serialize};

/// A PathStep represents an individual step along a Path.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default, Clone, new)]
#[serde(rename_all = "PascalCase")]
pub struct PathStep<'a> {
    account: Option<Cow<'a, str>>,
    currency: Option<Cow<'a, str>>,
    issuer: Option<Cow<'a, str>>,
    r#type: Option<u8>,
    type_hex: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, derive_new::new)]
#[serde(rename_all = "PascalCase")]
pub struct XChainBridge<'a> {
    pub issuing_chain_door: Cow<'a, str>,
    pub issuing_chain_issue: Currency<'a>,
    pub locking_chain_door: Cow<'a, str>,
    pub locking_chain_issue: Currency<'a>,
}

/// For use with serde defaults.
fn default_false() -> Option<bool> {
    Some(false)
}

/// Trait for validating currencies in models. This is needed to use xrpl-rust-macros for deriving validation methods.
pub trait ValidateCurrencies {
    fn validate_currencies(&self) -> crate::models::XRPLModelResult<()>;
}
