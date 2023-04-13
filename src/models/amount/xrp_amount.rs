use crate::models::amount::exceptions::XRPLAmountException;
use crate::models::Model;
use alloc::borrow::Cow;
use core::convert::TryInto;
use core::str::FromStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
pub struct XRPAmount<'a>(pub Cow<'a, str>);

impl<'a> Model for XRPAmount<'a> {}

impl<'a> From<Cow<'a, str>> for XRPAmount<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}

impl<'a> From<&'a str> for XRPAmount<'a> {
    fn from(value: &'a str) -> Self {
        Self(value.into())
    }
}

impl<'a> TryInto<Decimal> for XRPAmount<'a> {
    type Error = XRPLAmountException;

    fn try_into(self) -> Result<Decimal, Self::Error> {
        match Decimal::from_str(&self.0) {
            Ok(decimal) => Ok(decimal),
            Err(decimal_error) => Err(XRPLAmountException::ToDecimalError(decimal_error)),
        }
    }
}
