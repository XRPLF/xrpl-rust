use crate::models::{Model, XRPLModelException, XRPLModelResult};
use alloc::borrow::Cow;
use bigdecimal::BigDecimal;
use core::convert::TryInto;
use core::str::FromStr;
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

impl<'a> TryInto<BigDecimal> for IssuedCurrencyAmount<'a> {
    type Error = XRPLModelException;

    fn try_into(self) -> XRPLModelResult<BigDecimal, Self::Error> {
        Ok(BigDecimal::from_str(&self.value)?)
    }
}

impl<'a> PartialOrd for IssuedCurrencyAmount<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for IssuedCurrencyAmount<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}
