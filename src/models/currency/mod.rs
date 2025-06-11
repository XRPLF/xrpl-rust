pub mod issued_currency;
pub mod xrp;

use crate::models::Model;
use alloc::borrow::Cow;
pub use issued_currency::*;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
pub use xrp::*;

use super::{IssuedCurrencyAmount, XRPAmount};

pub trait ToAmount<'a, A> {
    fn to_amount(&self, value: Cow<'a, str>) -> A;
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Display)]
#[serde(untagged)]
pub enum Currency<'a> {
    IssuedCurrency(IssuedCurrency<'a>),
    XRP(XRP<'a>),
}

impl<'a> Model for Currency<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        match self {
            Currency::IssuedCurrency(issued_currency) => issued_currency.get_errors(),
            Currency::XRP(xrp) => xrp.get_errors(),
        }
    }
}

impl<'a> Default for Currency<'a> {
    fn default() -> Self {
        Self::XRP(XRP::new())
    }
}

impl<'a> From<IssuedCurrency<'a>> for Currency<'a> {
    fn from(value: IssuedCurrency<'a>) -> Self {
        Self::IssuedCurrency(value)
    }
}

impl<'a> From<XRP<'a>> for Currency<'a> {
    fn from(value: XRP<'a>) -> Self {
        Self::XRP(value)
    }
}

impl<'a> From<IssuedCurrencyAmount<'a>> for Currency<'a> {
    fn from(value: IssuedCurrencyAmount<'a>) -> Self {
        IssuedCurrency::new(value.currency, value.issuer).into()
    }
}

impl<'a> From<XRPAmount<'a>> for Currency<'a> {
    fn from(_value: XRPAmount<'a>) -> Self {
        XRP::new().into()
    }
}

impl<'a> From<&IssuedCurrencyAmount<'a>> for Currency<'a> {
    fn from(value: &IssuedCurrencyAmount<'a>) -> Self {
        IssuedCurrency::new(value.currency.clone(), value.issuer.clone()).into()
    }
}

impl<'a> From<&XRPAmount<'a>> for Currency<'a> {
    fn from(_value: &XRPAmount<'a>) -> Self {
        XRP::new().into()
    }
}
