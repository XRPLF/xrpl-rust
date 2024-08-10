use crate::models::Model;
use crate::{models::amount::exceptions::XRPLAmountException, Err};
use alloc::borrow::Cow;
use core::convert::TryInto;
use core::str::FromStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
pub struct IssuedCurrencyAmount<'a> {
    pub currency: Cow<'a, str>,
    pub issuer: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

impl<'a> Model for IssuedCurrencyAmount<'a> {}

impl<'a> IssuedCurrencyAmount<'a> {
    pub fn new(currency: Cow<'a, str>, issuer: Cow<'a, str>, value: Cow<'a, str>) -> Self {
        Self {
            currency,
            issuer,
            value,
        }
    }
}

impl<'a> TryInto<Decimal> for IssuedCurrencyAmount<'a> {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Decimal, Self::Error> {
        match Decimal::from_str(&self.value) {
            Ok(decimal) => Ok(decimal),
            Err(decimal_error) => Err!(XRPLAmountException::ToDecimalError(decimal_error)),
        }
    }
}

impl<'a> PartialOrd for IssuedCurrencyAmount<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<'a> Ord for IssuedCurrencyAmount<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}
