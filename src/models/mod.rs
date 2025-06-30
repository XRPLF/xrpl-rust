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

impl<'a> PathStep<'a> {
    /// Set the account field
    pub fn with_account(mut self, account: Cow<'a, str>) -> Self {
        self.account = Some(account);
        self
    }

    /// Set the currency field
    pub fn with_currency(mut self, currency: Cow<'a, str>) -> Self {
        self.currency = Some(currency);
        self
    }

    /// Set the issuer field
    pub fn with_issuer(mut self, issuer: Cow<'a, str>) -> Self {
        self.issuer = Some(issuer);
        self
    }

    /// Set the type field
    pub fn with_type(mut self, r#type: u8) -> Self {
        self.r#type = Some(r#type);
        self
    }

    /// Set the type_hex field
    pub fn with_type_hex(mut self, type_hex: Cow<'a, str>) -> Self {
        self.type_hex = Some(type_hex);
        self
    }
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
