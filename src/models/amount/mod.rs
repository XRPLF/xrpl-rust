mod issued_currency_amount;
mod mpt_amount;
mod xrp_amount;

pub use issued_currency_amount::*;
pub use mpt_amount::*;
pub use xrp_amount::*;

use alloc::string::ToString;
use core::convert::TryInto;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::{models::Model, utils::XRP_DROPS};

use super::{XRPLModelException, XRPLModelResult};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Display)]
#[serde(untagged)]
pub enum Amount<'a> {
    MPTAmount(MPTAmount<'a>),
    IssuedCurrencyAmount(IssuedCurrencyAmount<'a>),
    XRPAmount(XRPAmount<'a>),
}

impl<'a> TryInto<BigDecimal> for Amount<'a> {
    type Error = XRPLModelException;

    fn try_into(self) -> XRPLModelResult<BigDecimal, Self::Error> {
        match self {
            Amount::MPTAmount(amount) => amount.try_into(),
            Amount::IssuedCurrencyAmount(amount) => amount.try_into(),
            Amount::XRPAmount(amount) => amount.try_into(),
        }
    }
}

impl<'a> Model for Amount<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        match self {
            Amount::MPTAmount(amount) => amount.get_errors(),
            Amount::IssuedCurrencyAmount(amount) => amount.get_errors(),
            Amount::XRPAmount(amount) => amount.get_errors(),
        }
    }
}

impl<'a> Default for Amount<'a> {
    fn default() -> Self {
        Self::XRPAmount("0".into())
    }
}

impl<'a> Amount<'a> {
    pub fn is_xrp(&self) -> bool {
        match self {
            Amount::MPTAmount(_) => false,
            Amount::IssuedCurrencyAmount(_) => false,
            Amount::XRPAmount(_) => true,
        }
    }

    pub fn is_issued_currency(&self) -> bool {
        matches!(self, Amount::IssuedCurrencyAmount(_))
    }

    pub fn is_mpt(&self) -> bool {
        matches!(self, Amount::MPTAmount(_))
    }
}

impl<'a> From<MPTAmount<'a>> for Amount<'a> {
    fn from(value: MPTAmount<'a>) -> Self {
        Self::MPTAmount(value)
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

impl<'a> From<u32> for Amount<'a> {
    fn from(value: u32) -> Self {
        Self::XRPAmount(value.to_string().into())
    }
}

impl<'a> From<u64> for Amount<'a> {
    fn from(value: u64) -> Self {
        Self::XRPAmount(value.to_string().into())
    }
}

impl<'a> From<f64> for Amount<'a> {
    fn from(value: f64) -> Self {
        let drops = XRP_DROPS as f64;
        let result = value * drops;

        Self::XRPAmount(result.to_string().into())
    }
}

impl<'a> From<BigDecimal> for Amount<'a> {
    fn from(value: BigDecimal) -> Self {
        Self::XRPAmount((value * XRP_DROPS).to_string().into())
    }
}
