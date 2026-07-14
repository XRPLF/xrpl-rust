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
        let sv = BigDecimal::from_str(&self.value).unwrap_or_default();
        let ov = BigDecimal::from_str(&other.value).unwrap_or_default();
        sv.cmp(&ov)
            .then_with(|| self.currency.cmp(&other.currency))
            .then_with(|| self.issuer.cmp(&other.issuer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cmp::Ordering;

    fn ica(
        currency: &'static str,
        issuer: &'static str,
        value: &'static str,
    ) -> IssuedCurrencyAmount<'static> {
        IssuedCurrencyAmount::new(currency.into(), issuer.into(), value.into())
    }

    // Ord == Equal ↔ Eq: same value+currency+issuer
    #[test]
    fn test_ord_eq_consistent_equal() {
        let a = ica("USD", "rA", "100");
        let b = ica("USD", "rA", "100");
        assert_eq!(a, b);
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    // Different currency, same value: Eq says unequal, Ord must NOT say Equal
    #[test]
    fn test_ord_eq_consistent_different_currency() {
        let a = ica("USD", "rA", "100");
        let b = ica("EUR", "rA", "100");
        assert_ne!(a, b);
        assert_ne!(
            a.cmp(&b),
            Ordering::Equal,
            "same value, different currency must not be Ord-Equal"
        );
    }

    // Different issuer, same value: Eq says unequal, Ord must NOT say Equal
    #[test]
    fn test_ord_eq_consistent_different_issuer() {
        let a = ica("USD", "rA", "100");
        let b = ica("USD", "rB", "100");
        assert_ne!(a, b);
        assert_ne!(
            a.cmp(&b),
            Ordering::Equal,
            "same value, different issuer must not be Ord-Equal"
        );
    }

    // Numeric ordering: 10 > 9 (lexicographic "10" < "9" was the old bug)
    #[test]
    fn test_numeric_ordering() {
        let nine = ica("USD", "rA", "9");
        let ten = ica("USD", "rA", "10");
        assert!(ten > nine, "10 must be greater than 9 numerically");
    }

    // PartialOrd consistent with Ord
    #[test]
    fn test_partial_ord_consistent() {
        let a = ica("USD", "rA", "50");
        let b = ica("USD", "rA", "100");
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    }
}
