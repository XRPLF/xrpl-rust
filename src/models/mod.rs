//! Top-level modules for the models package.
//!
//! Order of models:
//! 1. Type of model
//! 2. Required common fields in alphabetical order
//! 3. Optional common fields in alphabetical order
//! 4. Required specific fields in alphabetical order
//! 5. Optional specific fields in alphabetical order

pub mod exceptions;
#[cfg(feature = "ledger")]
pub mod ledger;
pub mod model;
#[cfg(feature = "requests")]
#[allow(clippy::too_many_arguments)]
pub mod requests;
#[cfg(feature = "transactions")]
#[allow(clippy::too_many_arguments)]
pub mod transactions;

#[cfg(feature = "amounts")]
pub mod amount;
#[cfg(feature = "currencies")]
pub mod currency;
pub mod utils;

use derive_new::new;
pub use model::Model;

use crate::models::currency::{Currency, XRP};
use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

/// Represents the object types that an AccountObjects
/// Request can ask for.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum AccountObjectType {
    Check,
    DepositPreauth,
    Escrow,
    Offer,
    PaymentChannel,
    SignerList,
    RippleState,
    Ticket,
}

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

/// Returns a Currency as XRP for the currency, without a value.
fn default_xrp_currency<'a>() -> Currency<'a> {
    Currency::XRP(XRP::new())
}

/// For use with serde defaults.
fn default_true() -> Option<bool> {
    Some(true)
}

/// For use with serde defaults.
fn default_false() -> Option<bool> {
    Some(false)
}

/// For use with serde defaults.
fn default_limit_200() -> Option<u16> {
    Some(200)
}

/// For use with serde defaults.
fn default_limit_300() -> Option<u16> {
    Some(300)
}

/// For use with serde defaults.
fn default_fee_mult_max() -> Option<u32> {
    Some(10)
}

/// For use with serde defaults.
fn default_fee_div_max() -> Option<u32> {
    Some(1)
}

// pub trait SignAndSubmitError {
//     fn _get_field_error(&self) -> Result<(), XRPLSignAndSubmitException>;
//     fn _get_key_type_error(&self) -> Result<(), XRPLSignAndSubmitException>;
// }
//
// pub trait SignForError {
//     fn _get_field_error(&self) -> Result<(), XRPLSignForException>;
//     fn _get_key_type_error(&self) -> Result<(), XRPLSignForException>;
// }
//
// pub trait SignError {
//     fn _get_field_error(&self) -> Result<(), XRPLSignException>;
//     fn _get_key_type_error(&self) -> Result<(), XRPLSignException>;
// }
