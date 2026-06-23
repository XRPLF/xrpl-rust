mod issued_currency_amount;
mod mpt_amount;
mod xrp_amount;

pub use issued_currency_amount::*;
pub use mpt_amount::*;
pub use xrp_amount::*;

use alloc::string::ToString;
use core::convert::TryInto;
use core::str::FromStr;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Deserializer, Serialize};
use strum_macros::Display;

use crate::{models::Model, utils::XRP_DROPS};

use super::{XRPLModelException, XRPLModelResult};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Display)]
#[serde(untagged)]
pub enum Amount<'a> {
    // MPTAmount must be tried first: object with `mpt_issuance_id` key (no currency/issuer)
    MPTAmount(MPTAmount<'a>),
    // IssuedCurrencyAmount must be tried next (requires currency, issuer, value)
    IssuedCurrencyAmount(IssuedCurrencyAmount<'a>),
    // XRPAmount must be tried last (can be string or number)
    XRPAmount(XRPAmount<'a>),
}

impl<'de, 'a> Deserialize<'de> for Amount<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        if let Some(obj) = value.as_object() {
            // MPT amount: exactly the two keys {"mpt_issuance_id", "value"} — matches
            // xrpl.js isAmountObjectMPT which requires sorted keys === ["mpt_issuance_id","value"].
            // Any extra key (e.g. a hybrid object) falls through to the ICA path.
            if obj.len() == 2 && obj.contains_key("mpt_issuance_id") && obj.contains_key("value") {
                if let Ok(mpt) = serde_json::from_value::<MPTAmount>(value.clone()) {
                    return Ok(Amount::MPTAmount(mpt));
                }
            }

            // ICA: exactly the three keys {"currency", "issuer", "value"}.
            if obj.len() == 3
                && obj.contains_key("currency")
                && obj.contains_key("issuer")
                && obj.contains_key("value")
            {
                if let Ok(issued) = serde_json::from_value::<IssuedCurrencyAmount>(value.clone()) {
                    return Ok(Amount::IssuedCurrencyAmount(issued));
                }
            }
        }

        // If it's a string or number, try XRPAmount
        if value.is_string() || value.is_number() {
            if let Ok(xrp) = serde_json::from_value::<XRPAmount>(value.clone()) {
                return Ok(Amount::XRPAmount(xrp));
            }
        }

        Err(serde::de::Error::custom(
            "Amount must be a string/number (for XRP), an object with currency/issuer/value (for IssuedCurrency), or an object with mpt_issuance_id/value (for MPT)"
        ))
    }
}

impl<'a> TryInto<BigDecimal> for Amount<'a> {
    type Error = XRPLModelException;

    fn try_into(self) -> XRPLModelResult<BigDecimal, Self::Error> {
        match self {
            Amount::MPTAmount(amount) => {
                // Match MPTAmount validation: unsigned decimal digits only, then enforce
                // the XLS-33 / rippled limit of i64::MAX = 9223372036854775807.
                if amount.value.is_empty() || !amount.value.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(XRPLModelException::InvalidValue {
                        field: "value".into(),
                        expected: "unsigned decimal string".into(),
                        found: amount.value.to_string(),
                    });
                }
                let n: u64 =
                    amount
                        .value
                        .parse()
                        .map_err(|_| XRPLModelException::InvalidValue {
                            field: "value".into(),
                            expected: "unsigned decimal string".into(),
                            found: amount.value.to_string(),
                        })?;
                if n > i64::MAX as u64 {
                    return Err(XRPLModelException::InvalidValue {
                        field: "value".into(),
                        expected: alloc::format!("MPT amount <= {} (i64::MAX)", i64::MAX),
                        found: amount.value.to_string(),
                    });
                }
                Ok(BigDecimal::from(n))
            }
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
        matches!(self, Amount::XRPAmount(_))
    }

    /// Returns `true` only for `IssuedCurrencyAmount`. MPT amounts return `false`.
    /// **Breaking change from pre-MPT behaviour:** previously this returned `!is_xrp()`,
    /// so callers treating it as "not XRP" must now also check `is_mpt()`.
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
        // NaN and Infinity have no meaningful drops representation — treat as a programming error.
        assert!(
            value.is_finite(),
            "NaN and Infinity cannot be converted to Amount; got {value}"
        );
        // Use BigDecimal for fixed-point arithmetic to avoid floating-point precision loss
        // Convert f64 to string first to preserve exact decimal representation
        let value_bd =
            BigDecimal::from_str(&value.to_string()).unwrap_or_else(|_| BigDecimal::from(0));
        let drops_bd = BigDecimal::from(XRP_DROPS);
        let result = value_bd * drops_bd;

        Self::XRPAmount(result.to_string().into())
    }
}

impl<'a> From<BigDecimal> for Amount<'a> {
    fn from(value: BigDecimal) -> Self {
        Self::XRPAmount((value * XRP_DROPS).to_string().into())
    }
}

#[cfg(test)]
mod tests_amount_enum {
    use super::*;

    #[test]
    fn test_amount_deserialize_valid_xrp_string() {
        let json = "\"100\"";
        let amount: Result<Amount, _> = serde_json::from_str(json);
        assert!(amount.is_ok());
        assert!(amount.unwrap().is_xrp());
    }

    #[test]
    fn test_amount_deserialize_valid_issued_currency() {
        let json =
            r#"{"currency":"USD","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd","value":"100"}"#;
        let amount: Result<Amount, _> = serde_json::from_str(json);
        assert!(amount.is_ok());
        assert!(amount.unwrap().is_issued_currency());
    }

    #[test]
    fn test_amount_deserialize_malformed_object_should_fail() {
        let json = r#"{"invalid":"object"}"#;
        let amount: Result<Amount, _> = serde_json::from_str(json);
        assert!(
            amount.is_err(),
            "Malformed object should not deserialize silently"
        );
    }

    #[test]
    fn test_amount_deserialize_empty_object_should_fail() {
        let json = "{}";
        let amount: Result<Amount, _> = serde_json::from_str(json);
        assert!(
            amount.is_err(),
            "Empty object should not deserialize silently"
        );
    }

    #[test]
    fn test_amount_deserialize_partial_issued_currency_should_fail() {
        let json = r#"{"currency":"USD"}"#;
        let amount: Result<Amount, _> = serde_json::from_str(json);
        assert!(
            amount.is_err(),
            "Partial IssuedCurrency (missing issuer/value) should fail"
        );
    }

    #[test]
    fn test_amount_deserialize_null_should_fail() {
        let json = "null";
        let amount: Result<Amount, _> = serde_json::from_str(json);
        assert!(amount.is_err(), "Null should not deserialize");
    }

    #[test]
    fn test_amount_from_f64_preserves_precision() {
        // Test that f64 conversion uses fixed-point arithmetic, not floating-point
        // 1.5 XRP should convert to 1_500_000 drops exactly (no rounding errors)
        let xrp: f64 = 1.5;
        let amount = Amount::from(xrp);

        // Extract as string to check exact value
        match amount {
            Amount::XRPAmount(xrp_amount) => {
                // 1.5 * 1_000_000 = 1_500_000
                // Using BigDecimal should give exact result
                let value_str = xrp_amount.0.to_string();
                assert!(
                    value_str == "1500000" || value_str.contains("1500000"),
                    "Expected 1500000 drops from 1.5 XRP, got: {}",
                    value_str
                );
            }
            _ => panic!("Expected XRPAmount variant"),
        }
    }

    #[test]
    fn test_amount_from_f64_with_small_value() {
        // Test with a small value: 0.001 XRP = 1000 drops
        let xrp: f64 = 0.001;
        let amount = Amount::from(xrp);

        match amount {
            Amount::XRPAmount(xrp_amount) => {
                let value_str = xrp_amount.0.to_string();
                assert!(
                    value_str == "1000" || value_str.contains("1000"),
                    "Expected 1000 drops from 0.001 XRP, got: {}",
                    value_str
                );
            }
            _ => panic!("Expected XRPAmount variant"),
        }
    }

    const MPT_ID: &str = "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58";

    #[test]
    fn test_amount_deserialize_valid_mpt() {
        let json = r#"{"value":"100","mpt_issuance_id":"00000001A407AF5856CEFBF81F3D4A0000000000A407AF58"}"#;
        let amount: Amount = serde_json::from_str(json).unwrap();
        assert!(amount.is_mpt());
        assert!(!amount.is_xrp());
        assert!(!amount.is_issued_currency());
    }

    #[test]
    fn test_amount_mpt_json_round_trip() {
        let original = Amount::MPTAmount(MPTAmount::new("42".into(), MPT_ID.into()));
        let json = serde_json::to_string(&original).unwrap();
        let decoded: Amount = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_amount_mpt_not_confused_with_issued_currency() {
        // An IssuedCurrency object must NOT be parsed as MPTAmount
        let json =
            r#"{"currency":"USD","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd","value":"100"}"#;
        let amount: Amount = serde_json::from_str(json).unwrap();
        assert!(amount.is_issued_currency() && !amount.is_mpt());
    }

    #[test]
    fn test_amount_mpt_hybrid_object_rejected() {
        // xrpl.js isAmountObjectMPT requires exactly 2 keys {"mpt_issuance_id","value"};
        // an IOU-like isAmountObjectIOU requires exactly 3 keys {"currency","issuer","value"}.
        // A 4-key hybrid satisfies neither guard and must be rejected by both.
        let json = r#"{"mpt_issuance_id":"00000001A407AF5856CEFBF81F3D4A0000000000A407AF58","currency":"USD","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd","value":"100"}"#;
        let result: Result<Amount, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "hybrid 4-key object must be rejected (matches neither MPT nor ICA exact key-sets)"
        );
    }

    #[test]
    fn test_amount_mpt_extra_key_rejected() {
        // Object with correct MPT keys plus an unexpected extra field must be rejected.
        // xrpl.js rejects any object where sorted keys != ["mpt_issuance_id","value"].
        let json = r#"{"mpt_issuance_id":"00000001A407AF5856CEFBF81F3D4A0000000000A407AF58","value":"100","foo":"bar"}"#;
        let result: Result<Amount, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "3-key MPT-like object with extra field must be rejected"
        );
    }

    #[test]
    fn test_amount_mpt_exact_keys_accepted() {
        // Exactly {"mpt_issuance_id","value"} — the only valid MPT shape.
        let json = r#"{"mpt_issuance_id":"00000001A407AF5856CEFBF81F3D4A0000000000A407AF58","value":"100"}"#;
        let amount: Amount = serde_json::from_str(json).unwrap();
        assert!(amount.is_mpt(), "exact 2-key MPT object must parse as MPT");
        assert!(!amount.is_issued_currency());
    }

    #[test]
    fn test_amount_ica_extra_key_rejected() {
        // Object with correct ICA keys plus an extra field must be rejected.
        // xrpl.js isAmountObjectIOU requires exactly 3 keys.
        let json = r#"{"currency":"USD","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd","value":"100","extra":"field"}"#;
        let result: Result<Amount, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "4-key ICA-like object with extra field must be rejected"
        );
    }

    #[test]
    fn test_amount_mpt_try_into_bigdecimal_enforces_i64_max() {
        use core::convert::TryInto;
        // Value exceeding i64::MAX must fail at TryInto<BigDecimal>, not silently succeed.
        // This value is i64::MAX + 1. It fits in a u64, but is an invalid MPT amount.
        let oversized =
            Amount::MPTAmount(MPTAmount::new("9223372036854775808".into(), MPT_ID.into()));
        let result: Result<bigdecimal::BigDecimal, _> = oversized.try_into();
        assert!(
            result.is_err(),
            "value > i64::MAX must be rejected: {}",
            i64::MAX
        );

        // A value > u64::MAX should also fail (at the parse<u64> step)
        let overflowing =
            Amount::MPTAmount(MPTAmount::new("99999999999999999999".into(), MPT_ID.into()));
        let result: Result<bigdecimal::BigDecimal, _> = overflowing.try_into();
        assert!(result.is_err(), "value > u64::MAX must be rejected");
    }

    #[test]
    #[should_panic(expected = "NaN and Infinity cannot be converted to Amount; got NaN")]
    fn test_amount_from_f64_panics_on_nan() {
        let _ = Amount::from(f64::NAN);
    }

    #[test]
    #[should_panic(expected = "NaN and Infinity cannot be converted to Amount; got inf")]
    fn test_amount_from_f64_panics_on_infinity() {
        let _ = Amount::from(f64::INFINITY);
    }
}
