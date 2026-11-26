use crate::models::{Model, XRPLModelException, XRPLModelResult};
use alloc::borrow::Cow;
use bigdecimal::BigDecimal;
use core::convert::TryInto;
use core::str::FromStr;
use serde::{Deserialize, Serialize};

// PartialEq and Eq are implemented manually (not derived) so that Eq agrees with
// Ord: two amounts with numerically equal values, same currency, and same issuer
// are considered equal regardless of their string representation (e.g. "100" and
// "100.0" compare Equal). When a value is not a valid decimal, comparison falls
// back to byte-wise string equality — consistent with the Ord fallback.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct IssuedCurrencyAmount<'a> {
    pub currency: Cow<'a, str>,
    pub issuer: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

impl<'a> PartialEq for IssuedCurrencyAmount<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.currency != other.currency || self.issuer != other.issuer {
            return false;
        }
        // Delegate to Ord::cmp so the Ord/Eq invariant holds by construction:
        // any change to cmp is automatically reflected here with no risk of divergence.
        self.cmp(other) == core::cmp::Ordering::Equal
    }
}

impl<'a> Eq for IssuedCurrencyAmount<'a> {}

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
        // Parse values as BigDecimal for numeric comparison. Valid decimals sort
        // before malformed strings so the ordering is total and transitive:
        //   (valid, valid)   → numeric order
        //   (valid, invalid) → valid sorts first (Less)
        //   (invalid, valid) → invalid sorts last (Greater)
        //   (invalid, invalid) → lexicographic fallback on the raw strings
        let value_ord = match (
            BigDecimal::from_str(&self.value),
            BigDecimal::from_str(&other.value),
        ) {
            (Ok(sv), Ok(ov)) => sv.cmp(&ov),
            (Ok(_), Err(_)) => core::cmp::Ordering::Less,
            (Err(_), Ok(_)) => core::cmp::Ordering::Greater,
            (Err(_), Err(_)) => self.value.cmp(&other.value),
        };
        value_ord
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

    // "100" and "100.0" must be Eq-equal under numeric comparison
    #[test]
    fn test_eq_canonical_forms_numeric() {
        let a = ica("USD", "rA", "100");
        let b = ica("USD", "rA", "100.0");
        assert_eq!(
            a, b,
            "'100' and '100.0' must be Eq-equal with numeric comparison"
        );
        assert_eq!(
            a.cmp(&b),
            Ordering::Equal,
            "'100' and '100.0' must be Ord-Equal"
        );
    }

    // Currency tiebreak: EUR < USD when values are numerically equal
    #[test]
    fn test_ord_tiebreak_currency_when_value_equal() {
        let eur = ica("EUR", "rA", "100");
        let usd = ica("USD", "rA", "100");
        assert!(eur < usd, "EUR sorts before USD when values are equal");
        assert_ne!(eur, usd, "different currencies must not be Eq-equal");
    }

    // Numerically-equal values with different string representations ("1.0" vs
    // "1.00") must be Eq-equal AND Ord-Equal so that cmp == Equal ↔ eq always holds.
    // This guards against a scale-sensitive BigDecimal PartialEq ever being used
    // instead of the Ord-based comparison in the eq implementation.
    #[test]
    fn test_eq_and_ord_agree_for_different_decimal_repr() {
        let a = ica("USD", "rA", "1.0");
        let b = ica("USD", "rA", "1.00");
        assert_eq!(
            a, b,
            "'1.0' and '1.00' must be Eq-equal (numeric comparison)"
        );
        assert_eq!(
            a.cmp(&b),
            Ordering::Equal,
            "'1.0' and '1.00' must be Ord-Equal (numeric comparison)"
        );
    }

    // Malformed value does not silently sort as zero — it falls back to
    // lexicographic string comparison rather than mapping to the zero BigDecimal.
    #[test]
    fn test_ord_malformed_value_not_silent_zero() {
        let malformed = ica("USD", "rA", "not-a-number");
        let zero = ica("USD", "rA", "0");
        // "not-a-number" fails to parse; "0" is valid.
        // Hits the (Err, Ok) arm → Greater (invalid sorts after valid).
        // Key invariant: malformed != zero (it is NOT silently treated as 0).
        assert_ne!(malformed, zero, "malformed value must not equal zero");
        // And they have a defined, stable ordering (not zero-based).
        let ord = malformed.cmp(&zero);
        assert!(
            ord != Ordering::Equal,
            "malformed value must not compare Equal to zero"
        );
    }
}
