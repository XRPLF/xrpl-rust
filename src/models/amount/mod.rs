mod issued_currency_amount;
mod mpt_amount;
mod xrp_amount;

pub use issued_currency_amount::*;
pub use mpt_amount::*;
pub use xrp_amount::*;

use alloc::string::ToString;
use core::convert::{TryFrom, TryInto};
use core::str::FromStr;

use bigdecimal::BigDecimal;
use serde::{
    de::{Error as DeError, Unexpected},
    Deserialize, Deserializer, Serialize,
};
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

const AMOUNT_OBJECT_FIELDS: &[&str] = &["mpt_issuance_id", "currency", "issuer", "value"];
const MPT_AMOUNT_OBJECT_FIELDS: &[&str] = &["mpt_issuance_id", "value"];
const ISSUED_CURRENCY_AMOUNT_OBJECT_FIELDS: &[&str] =
    &["mpt_issuance_id", "currency", "issuer", "value"];

fn unexpected_json_value(value: &serde_json::Value) -> Unexpected<'_> {
    match value {
        serde_json::Value::Null => Unexpected::Unit,
        serde_json::Value::Bool(value) => Unexpected::Bool(*value),
        serde_json::Value::Number(number) => {
            if let Some(value) = number.as_u64() {
                Unexpected::Unsigned(value)
            } else if let Some(value) = number.as_i64() {
                Unexpected::Signed(value)
            } else if let Some(value) = number.as_f64() {
                Unexpected::Float(value)
            } else {
                Unexpected::Other("number")
            }
        }
        serde_json::Value::String(value) => Unexpected::Str(value),
        serde_json::Value::Array(_) => Unexpected::Seq,
        serde_json::Value::Object(_) => Unexpected::Map,
    }
}

fn reject_unknown_object_fields<E>(
    obj: &serde_json::Map<alloc::string::String, serde_json::Value>,
    expected: &'static [&'static str],
) -> Result<(), E>
where
    E: DeError,
{
    if let Some(field) = obj.keys().find(|field| !expected.contains(&field.as_str())) {
        Err(E::unknown_field(field, expected))
    } else {
        Ok(())
    }
}

fn deserialize_amount_object<'a, E>(
    value: serde_json::Value,
    obj: &serde_json::Map<alloc::string::String, serde_json::Value>,
) -> Result<Amount<'a>, E>
where
    E: DeError,
{
    match (
        obj.contains_key("mpt_issuance_id"),
        obj.contains_key("currency"),
        obj.contains_key("issuer"),
        obj.contains_key("value"),
    ) {
        // Pure MPT amount shape: accept only `{ mpt_issuance_id, value }`.
        // The `currency` and `issuer` booleans must both be false here so a hybrid
        // object cannot be treated as MPT by silently discarding issued-currency keys.
        (true, false, false, true) => {
            reject_unknown_object_fields::<E>(obj, MPT_AMOUNT_OBJECT_FIELDS)?;
            serde_json::from_value::<MPTAmount>(value)
                .map(Amount::MPTAmount)
                .map_err(E::custom)
        }
        // `mpt_issuance_id` by itself identifies the MPT, but an Amount still needs
        // `value` to be usable. Report the missing required field directly.
        (true, false, false, false) => Err(E::missing_field("value")),

        // Issued-currency amount shape: accept `{ currency, issuer, value }`.
        // The first tuple element is ignored intentionally: if `mpt_issuance_id` is
        // also present, this arm only succeeds after every ICA-required field exists,
        // then `reject_unknown_object_fields` rejects the extra MPT key. That makes
        // hybrid MPT+ICA objects fail loudly instead of falling back to MPT or ICA.
        (_, true, true, true) => {
            reject_unknown_object_fields::<E>(obj, ISSUED_CURRENCY_AMOUNT_OBJECT_FIELDS)?;
            serde_json::from_value::<IssuedCurrencyAmount>(value)
                .map(Amount::IssuedCurrencyAmount)
                .map_err(E::custom)
        }
        // Partial ICA shapes get typed missing-field errors. Order matters: `currency`
        // is the discriminator for issued currency, so report it before `issuer`/`value`
        // when callers provide only one of the non-currency ICA fields.
        (_, false, true, _) | (_, false, _, true) => Err(E::missing_field("currency")),
        (_, true, false, _) => Err(E::missing_field("issuer")),
        (_, true, true, false) => Err(E::missing_field("value")),

        _ => {
            if let Some(field) = obj.keys().next() {
                Err(E::unknown_field(field, AMOUNT_OBJECT_FIELDS))
            } else {
                Err(E::invalid_length(
                    0,
                    &"an Amount object with mpt_issuance_id/value or currency/issuer/value",
                ))
            }
        }
    }
}

impl<'de, 'a> Deserialize<'de> for Amount<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        if let Some(obj) = value.as_object() {
            return deserialize_amount_object::<D::Error>(value.clone(), obj);
        }

        // XRPAmount: string or number.
        if value.is_string() || value.is_number() {
            return serde_json::from_value::<XRPAmount>(value)
                .map(Amount::XRPAmount)
                .map_err(D::Error::custom);
        }

        Err(D::Error::invalid_type(
            unexpected_json_value(&value),
            &"a string/number for XRPAmount, an object with currency/issuer/value for IssuedCurrencyAmount, or an object with mpt_issuance_id/value for MPTAmount",
        ))
    }
}

impl<'a> TryInto<BigDecimal> for Amount<'a> {
    type Error = XRPLModelException;

    fn try_into(self) -> XRPLModelResult<BigDecimal, Self::Error> {
        match self {
            Amount::MPTAmount(amount) => {
                // Match MPTAmount validation: unsigned decimal digits only, then enforce
                // the XLS-33 / xrpld limit of i64::MAX = 9223372036854775807.
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

impl<'a> TryFrom<f64> for Amount<'a> {
    type Error = XRPLModelException;

    fn try_from(value: f64) -> XRPLModelResult<Self, Self::Error> {
        if !value.is_finite() {
            return Err(XRPLModelException::InvalidValue {
                field: "amount".into(),
                expected: "finite f64 XRP amount".into(),
                found: value.to_string(),
            });
        }
        // Use BigDecimal for fixed-point arithmetic to avoid floating-point precision loss.
        // Convert f64 to string first to preserve exact decimal representation.
        let value_bd = BigDecimal::from_str(&value.to_string()).map_err(|error| {
            XRPLModelException::InvalidValueFormat {
                field: "amount".into(),
                format: alloc::format!("finite decimal f64 ({error})"),
                found: value.to_string(),
            }
        })?;
        let drops_bd = BigDecimal::from(XRP_DROPS);
        let result = value_bd * drops_bd;

        Ok(Self::XRPAmount(result.to_string().into()))
    }
}

impl<'a> From<BigDecimal> for Amount<'a> {
    fn from(value: BigDecimal) -> Self {
        Self::XRPAmount((value * XRP_DROPS).to_string().into())
    }
}

#[cfg(test)]
mod tests_amount_enum {
    use alloc::format;

    use super::*;
    use crate::models::transactions::test_fixtures::{ISSUER_ACCOUNT_ALT, MPT_ISSUANCE_ID};

    #[test]
    fn test_amount_deserialize_valid_xrp_string() {
        let json = "\"100\"";
        let amount: Result<Amount, _> = serde_json::from_str(json);
        assert!(amount.is_ok());
        assert!(amount.unwrap().is_xrp());
    }

    #[test]
    fn test_amount_deserialize_valid_issued_currency() {
        let json = format!(
            r#"{{"currency":"USD","issuer":"{}","value":"100"}}"#,
            ISSUER_ACCOUNT_ALT
        );
        let amount: Result<Amount, _> = serde_json::from_str(&json);
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
        let amount = Amount::try_from(xrp).unwrap();

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
        let amount = Amount::try_from(xrp).unwrap();

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

    const MPT_ID: &str = MPT_ISSUANCE_ID;

    #[test]
    fn test_amount_deserialize_valid_mpt() {
        let json = format!(
            r#"{{"value":"100","mpt_issuance_id":"{}"}}"#,
            MPT_ISSUANCE_ID
        );
        let amount: Amount = serde_json::from_str(&json).unwrap();
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
        let json = format!(
            r#"{{"currency":"USD","issuer":"{}","value":"100"}}"#,
            ISSUER_ACCOUNT_ALT
        );
        let amount: Amount = serde_json::from_str(&json).unwrap();
        assert!(amount.is_issued_currency() && !amount.is_mpt());
    }

    #[test]
    fn test_amount_mpt_hybrid_object_parses_as_issued_currency() {
        // A hybrid object with both mpt_issuance_id AND currency/issuer must NOT silently
        // discard the ICA fields — it should fall through to IssuedCurrencyAmount.
        let json = format!(
            r#"{{"mpt_issuance_id":"{}","currency":"USD","issuer":"{}","value":"100"}}"#,
            MPT_ISSUANCE_ID, ISSUER_ACCOUNT_ALT
        );
        let amount: Amount = serde_json::from_str(&json).unwrap();
        assert!(
            amount.is_issued_currency(),
            "hybrid object must parse as ICA, not MPT"
        );
        assert!(!amount.is_mpt());
    }

    #[test]
    fn test_amount_mpt_object_with_ica_field_does_not_parse_as_mpt() {
        let json = format!(
            r#"{{"mpt_issuance_id":"{}","currency":"USD","value":"100"}}"#,
            MPT_ISSUANCE_ID
        );
        let err = serde_json::from_str::<Amount>(&json).unwrap_err();
        assert!(
            err.to_string().contains("missing field `issuer`"),
            "MPT object with ICA fields must not be accepted as MPT; got: {err}"
        );
    }

    #[test]
    fn test_amount_mpt_object_rejects_unknown_fields() {
        let json = format!(
            r#"{{"mpt_issuance_id":"{}","value":"100","extra":"ignored?"}}"#,
            MPT_ISSUANCE_ID
        );
        let err = serde_json::from_str::<Amount>(&json).unwrap_err();
        assert!(
            err.to_string().contains("unknown field `extra`"),
            "MPT object must not silently ignore unknown fields; got: {err}"
        );
    }

    #[test]
    fn test_amount_issued_currency_object_rejects_unknown_fields() {
        let json = format!(
            r#"{{"currency":"USD","issuer":"{}","value":"100","extra":"ignored?"}}"#,
            ISSUER_ACCOUNT_ALT
        );
        let err = serde_json::from_str::<Amount>(&json).unwrap_err();
        assert!(
            err.to_string().contains("unknown field `extra`"),
            "IssuedCurrencyAmount must not silently ignore unknown fields; got: {err}"
        );
    }

    #[test]
    fn test_amount_hybrid_object_rejects_unknown_fields() {
        let json = format!(
            r#"{{"mpt_issuance_id":"{}","currency":"USD","issuer":"{}","value":"100","extra":"ignored?"}}"#,
            MPT_ISSUANCE_ID, ISSUER_ACCOUNT_ALT
        );
        let err = serde_json::from_str::<Amount>(&json).unwrap_err();
        assert!(
            err.to_string().contains("unknown field `extra`"),
            "Hybrid Amount object must not silently ignore unknown fields; got: {err}"
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
    fn test_amount_try_from_f64_rejects_nan() {
        let error = Amount::try_from(f64::NAN).unwrap_err();
        assert!(error.to_string().contains("finite f64 XRP amount"));
    }

    #[test]
    fn test_amount_try_from_f64_rejects_infinity() {
        let error = Amount::try_from(f64::INFINITY).unwrap_err();
        assert!(error.to_string().contains("finite f64 XRP amount"));
    }
}
