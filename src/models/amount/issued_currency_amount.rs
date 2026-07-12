use crate::models::{Model, XRPLModelException, XRPLModelResult};
use alloc::borrow::Cow;
use bigdecimal::BigDecimal;
use core::convert::TryInto;
use core::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct IssuedCurrencyAmount<'a> {
    pub currency: Cow<'a, str>,
    pub issuer: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

impl<'a> Model for IssuedCurrencyAmount<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.value.parse::<f64>()?;

        Ok(())
    }
}

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
        // Compare numerically when both values parse; fall back to string order.
        // Then break ties with currency and issuer so that Ord is consistent
        // with the derived PartialEq (which compares all three fields).
        let value_cmp = match (
            self.value.parse::<BigDecimal>(),
            other.value.parse::<BigDecimal>(),
        ) {
            (Ok(a), Ok(b)) => a.partial_cmp(&b).unwrap_or(core::cmp::Ordering::Equal),
            _ => self.value.cmp(&other.value),
        };
        value_cmp
            .then_with(|| self.currency.cmp(&other.currency))
            .then_with(|| self.issuer.cmp(&other.issuer))
    }
}
