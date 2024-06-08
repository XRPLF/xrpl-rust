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
#[cfg(feature = "results")]
pub mod results;
#[cfg(feature = "transactions")]
#[allow(clippy::too_many_arguments)]
pub mod transactions;

#[cfg(feature = "amounts")]
pub mod amount;
#[cfg(feature = "currencies")]
pub mod currency;
pub mod utils;

use core::convert::TryFrom;

use derive_new::new;
pub use model::Model;
use strum::IntoEnumIterator;

use crate::models::currency::{Currency, XRP};
use crate::models::exceptions::XRPLFlagsException;
use crate::Err;
use alloc::{borrow::Cow, vec::Vec};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter};

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

/// Represents the type of flags when the XRPL model has no flags.
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize, Deserialize, Display, AsRefStr, EnumIter, Copy,
)]
pub enum NoFlags {}

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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, new)]
pub struct FlagCollection<T>(pub(crate) Vec<T>)
where
    T: IntoEnumIterator;

impl<T> Iterator for FlagCollection<T>
where
    T: IntoEnumIterator,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<T> Default for FlagCollection<T>
where
    T: IntoEnumIterator,
{
    fn default() -> Self {
        FlagCollection(Vec::new())
    }
}

impl<T> From<Vec<T>> for FlagCollection<T>
where
    T: IntoEnumIterator,
{
    fn from(flags: Vec<T>) -> Self {
        FlagCollection(flags)
    }
}

impl<T> TryFrom<u32> for FlagCollection<T>
where
    T: IntoEnumIterator + Serialize,
{
    type Error = anyhow::Error;

    fn try_from(flags: u32) -> Result<Self> {
        let mut flag_collection = Vec::new();
        for flag in T::iter() {
            let flag_as_u32 = flag_to_u32(&flag)?;
            if flags & flag_as_u32 == flag_as_u32 {
                flag_collection.push(flag);
            }
        }
        Ok(FlagCollection::new(flag_collection))
    }
}

impl<T> TryFrom<FlagCollection<T>> for u32
where
    T: IntoEnumIterator + Serialize,
{
    type Error = anyhow::Error;

    fn try_from(flag_collection: FlagCollection<T>) -> Result<Self> {
        let mut flags = 0;
        for flag in flag_collection {
            let flag_as_u32 = flag_to_u32(&flag)?;
            flags |= flag_as_u32;
        }
        Ok(flags)
    }
}

fn flag_to_u32<T>(flag: &T) -> Result<u32>
where
    T: Serialize,
{
    match serde_json::to_string(flag) {
        Ok(flag_as_string) => match flag_as_string.parse::<u32>() {
            Ok(flag_as_u32) => Ok(flag_as_u32),
            Err(_error) => Err!(XRPLFlagsException::CannotConvertFlagToU32),
        },
        Err(_error) => Err!(XRPLFlagsException::CannotConvertFlagToU32),
    }
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
