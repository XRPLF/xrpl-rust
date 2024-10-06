//! Top-level modules for the models package.
//!
//! Order of models:
//! 1. Type of model
//! 2. Required common fields in alphabetical order
//! 3. Optional common fields in alphabetical order
//! 4. Required specific fields in alphabetical order
//! 5. Optional specific fields in alphabetical order

#[cfg(feature = "ledger-models")]
#[allow(clippy::too_many_arguments)]
pub mod ledger;
#[cfg(feature = "request-models")]
#[allow(clippy::too_many_arguments)]
pub mod requests;
#[cfg(feature = "result-models")]
#[allow(clippy::too_many_arguments)]
pub mod results;
#[cfg(feature = "transaction-models")]
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

/// For use with serde defaults.
fn default_false() -> Option<bool> {
    Some(false)
}
