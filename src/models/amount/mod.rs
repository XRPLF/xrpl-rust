pub mod exceptions;
pub mod issued_currency_amount;
pub mod xrp_amount;

use core::convert::TryInto;
pub use issued_currency_amount::*;
use rust_decimal::Decimal;
pub use xrp_amount::*;

use crate::models::Model;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Display)]
#[serde(untagged)]
pub enum Amount<'a> {
    IssuedCurrencyAmount(IssuedCurrencyAmount<'a>),
    XRPAmount(XRPAmount<'a>),
}

impl<'a> TryInto<Decimal> for Amount<'a> {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Decimal, Self::Error> {
        match self {
            Amount::IssuedCurrencyAmount(amount) => amount.try_into(),
            Amount::XRPAmount(amount) => amount.try_into(),
        }
    }
}

impl<'a> Model for Amount<'a> {}

impl<'a> Default for Amount<'a> {
    fn default() -> Self {
        Self::XRPAmount("0".into())
    }
}

impl<'a> Amount<'a> {
    pub fn is_xrp(&self) -> bool {
        match self {
            Amount::IssuedCurrencyAmount(_) => false,
            Amount::XRPAmount(_) => true,
        }
    }

    pub fn is_issued_currency(&self) -> bool {
        !self.is_xrp()
    }
}

impl<'a> From<IssuedCurrencyAmount<'a>> for Amount<'a> {
    fn from(value: IssuedCurrencyAmount<'a>) -> Self {
        Self::IssuedCurrencyAmount(value)
    }
}

impl<'a> From<XRPAmount<'a>> for Amount<'a> {
    fn from(value: XRPAmount<'a>) -> Self {
        Self::XRPAmount(value)
    }
}

impl<'a> From<&'a str> for Amount<'a> {
    fn from(value: &'a str) -> Self {
        Self::XRPAmount(value.into())
    }
}
