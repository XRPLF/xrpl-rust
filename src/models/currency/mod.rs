pub mod issued_currency;
pub mod mpt_currency;
pub mod xrp;

use crate::models::Model;
use alloc::borrow::Cow;
pub use issued_currency::*;
pub use mpt_currency::*;
use serde::{Deserialize, Deserializer, Serialize};
use strum_macros::Display;
pub use xrp::*;

use super::{IssuedCurrencyAmount, MPTAmount, XRPAmount};

pub trait ToAmount<'a, A> {
    fn to_amount(&self, value: Cow<'a, str>) -> A;
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Display)]
#[serde(untagged)]
pub enum Currency<'a> {
    /// MPTCurrency variant must be checked first: object with only `mpt_issuance_id`
    MPTCurrency(MPTCurrency<'a>),
    /// IssuedCurrency variant (requires both currency and issuer fields)
    IssuedCurrency(IssuedCurrency<'a>),
    /// XRP variant (only requires currency field set to "XRP")
    XRP(XRP<'a>),
}

impl<'de, 'a> Deserialize<'de> for Currency<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        // Check if it's an object (all Currency variants are objects)
        if let Some(obj) = value.as_object() {
            let has_currency = obj.contains_key("currency");
            let has_issuer = obj.contains_key("issuer");

            // Try MPTCurrency first: object with `mpt_issuance_id` and no currency/issuer keys.
            // Guard prevents a hybrid object from silently discarding ICA fields.
            if obj.contains_key("mpt_issuance_id")
                && !obj.contains_key("currency")
                && !obj.contains_key("issuer")
            {
                if let Ok(mpt) = serde_json::from_value::<MPTCurrency>(value.clone()) {
                    return Ok(Currency::MPTCurrency(mpt));
                }
            }

            // Try to deserialize as IssuedCurrency if issuer field exists (more specific variant)
            if has_issuer {
                if let Ok(ic) = serde_json::from_value::<IssuedCurrency>(value.clone()) {
                    return Ok(Currency::IssuedCurrency(ic));
                }
            }

            // Try to deserialize as XRP if currency field exists and equals "XRP"
            if has_currency {
                if let Ok(xrp) = serde_json::from_value::<XRP>(value.clone()) {
                    // Validate that XRP currency is actually "XRP"
                    if xrp.currency == "XRP" {
                        return Ok(Currency::XRP(xrp));
                    }
                }
            }

            // If we got here with an object but it doesn't match any variant, error
            return Err(serde::de::Error::custom(
                "Invalid Currency object: must have 'mpt_issuance_id' for MPTCurrency, 'issuer' for IssuedCurrency, or 'currency'='XRP' for XRP"
            ));
        }

        Err(serde::de::Error::custom("Currency must be a JSON object"))
    }
}

impl<'a> Model for Currency<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        match self {
            Currency::MPTCurrency(mpt) => mpt.get_errors(),
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

impl<'a> From<MPTCurrency<'a>> for Currency<'a> {
    fn from(value: MPTCurrency<'a>) -> Self {
        Self::MPTCurrency(value)
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

impl<'a> From<&MPTAmount<'a>> for Currency<'a> {
    fn from(value: &MPTAmount<'a>) -> Self {
        MPTCurrency::new(value.mpt_issuance_id.clone()).into()
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

#[cfg(test)]
mod tests_currency_enum {
    use crate::models::Model;

    use super::*;

    const VALID_ID: &str = "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58";

    #[test]
    fn test_currency_deserialize_mpt() {
        let json = alloc::format!(r#"{{"mpt_issuance_id":"{VALID_ID}"}}"#);
        let cur: Currency = serde_json::from_str(&json).unwrap();
        assert!(matches!(cur, Currency::MPTCurrency(_)));
    }

    #[test]
    fn test_currency_mpt_json_round_trip() {
        let original = Currency::MPTCurrency(MPTCurrency::new(VALID_ID.into()));
        let json = serde_json::to_string(&original).unwrap();
        let decoded: Currency = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_currency_deserialize_issued_not_mpt() {
        let json = r#"{"currency":"USD","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd"}"#;
        let cur: Currency = serde_json::from_str(json).unwrap();
        assert!(matches!(cur, Currency::IssuedCurrency(_)));
    }

    #[test]
    fn test_currency_mpt_get_errors_valid() {
        let cur = Currency::MPTCurrency(MPTCurrency::new(VALID_ID.into()));
        assert!(cur.get_errors().is_ok());
    }

    #[test]
    fn test_currency_mpt_get_errors_bad_id() {
        let cur = Currency::MPTCurrency(MPTCurrency::new("TOOSHORT".into()));
        assert!(cur.get_errors().is_err());
    }
}
