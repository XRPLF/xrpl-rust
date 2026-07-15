use crate::models::{Model, XRPLModelException, XRPLModelResult};
use alloc::{
    borrow::Cow,
    string::{String, ToString},
};
use bigdecimal::BigDecimal;
use core::str::FromStr;
use core::{
    convert::{TryFrom, TryInto},
    fmt::Display,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents an amount of XRP in Drops.
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct XRPAmount<'a>(pub Cow<'a, str>);

impl<'a> Model for XRPAmount<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.0.parse::<u32>()?;

        Ok(())
    }
}

impl Default for XRPAmount<'_> {
    fn default() -> Self {
        Self("0".into())
    }
}

impl Display for XRPAmount<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// implement Deserializing from Cow<str>, &str, String, Decimal, f64, u32, and Value
impl<'de, 'a> Deserialize<'de> for XRPAmount<'a> {
    fn deserialize<D>(deserializer: D) -> XRPLModelResult<XRPAmount<'a>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let amount_string = Value::deserialize(deserializer)?;
        XRPAmount::try_from(amount_string).map_err(serde::de::Error::custom)
    }
}

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

impl<'a> From<String> for XRPAmount<'a> {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl<'a> From<BigDecimal> for XRPAmount<'a> {
    fn from(value: BigDecimal) -> Self {
        Self(value.to_string().into())
    }
}

impl<'a> From<f64> for XRPAmount<'a> {
    fn from(value: f64) -> Self {
        Self(value.to_string().into())
    }
}

impl<'a> From<u32> for XRPAmount<'a> {
    fn from(value: u32) -> Self {
        Self(value.to_string().into())
    }
}

impl<'a> TryFrom<Value> for XRPAmount<'a> {
    type Error = XRPLModelException;

    fn try_from(value: Value) -> XRPLModelResult<Self, Self::Error> {
        // Reject non-string and non-number JSON types (objects, arrays, null, booleans)
        if !value.is_string() && !value.is_number() {
            return Err(XRPLModelException::InvalidValue {
                field: "XRPAmount".into(),
                expected: "string or number".into(),
                found: match &value {
                    Value::Object(_) => "object".into(),
                    Value::Array(_) => "array".into(),
                    Value::Null => "null".into(),
                    Value::Bool(_) => "boolean".into(),
                    _ => "unknown".into(),
                },
            });
        }

        match serde_json::to_string(&value) {
            Ok(amount_string) => {
                let amount_string = amount_string.clone().replace("\"", "");
                Ok(Self(amount_string.into()))
            }
            Err(serde_error) => Err(serde_error.into()),
        }
    }
}

impl<'a> TryInto<f64> for XRPAmount<'a> {
    type Error = XRPLModelException;

    fn try_into(self) -> XRPLModelResult<f64, Self::Error> {
        Ok(self.0.parse::<f64>()?)
    }
}

impl<'a> TryInto<u32> for XRPAmount<'a> {
    type Error = XRPLModelException;

    fn try_into(self) -> XRPLModelResult<u32, Self::Error> {
        Ok(self.0.parse::<u32>()?)
    }
}

impl<'a> TryInto<BigDecimal> for XRPAmount<'a> {
    type Error = XRPLModelException;

    fn try_into(self) -> XRPLModelResult<BigDecimal, Self::Error> {
        Ok(BigDecimal::from_str(&self.0)?)
    }
}

impl<'a> TryInto<Cow<'a, str>> for XRPAmount<'a> {
    type Error = XRPLModelException;

    fn try_into(self) -> XRPLModelResult<Cow<'a, str>, Self::Error> {
        Ok(self.0)
    }
}

impl<'a> XRPAmount<'a> {
    /// Compare two XRP amounts numerically.
    ///
    /// Returns an error if either side is not a valid numeric XRP amount. Use this
    /// when callers need to distinguish malformed input from an ordering result.
    pub fn checked_cmp(&self, other: &Self) -> XRPLModelResult<core::cmp::Ordering> {
        let self_decimal: BigDecimal = <Self as Clone>::clone(self).try_into()?;
        let other_decimal: BigDecimal = <Self as Clone>::clone(other).try_into()?;
        Ok(self_decimal.cmp(&other_decimal))
    }
}

impl<'a> PartialOrd for XRPAmount<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for XRPAmount<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.checked_cmp(other)
            .expect("cannot compare invalid XRPAmount values")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{format, vec};
    use core::cmp::Ordering;

    #[test]
    fn test_cmp_valid_amounts() {
        let amount1 = XRPAmount("100".into());
        let amount2 = XRPAmount("200".into());
        let amount3 = XRPAmount("100".into());

        assert_eq!(amount1.cmp(&amount2), Ordering::Less);
        assert_eq!(amount2.cmp(&amount1), Ordering::Greater);
        assert_eq!(amount1.cmp(&amount3), Ordering::Equal);
    }

    #[test]
    fn test_cmp_zero() {
        let zero = XRPAmount("0".into());
        let positive = XRPAmount("100".into());

        assert_eq!(zero.cmp(&positive), Ordering::Less);
        assert_eq!(positive.cmp(&zero), Ordering::Greater);
    }

    #[test]
    fn test_checked_cmp_invalid_vs_valid_returns_error() {
        let valid = XRPAmount("100".into());
        let invalid = XRPAmount("not-a-number".into());

        assert!(valid.checked_cmp(&invalid).is_err());
        assert!(invalid.checked_cmp(&valid).is_err());
    }

    #[test]
    fn test_checked_cmp_both_invalid_returns_error() {
        let invalid1 = XRPAmount("not-a-number".into());
        let invalid2 = XRPAmount("also-invalid".into());

        assert!(invalid1.checked_cmp(&invalid2).is_err());
    }

    #[test]
    #[should_panic(expected = "cannot compare invalid XRPAmount values")]
    fn test_cmp_panics_on_malformed() {
        let valid = XRPAmount("100".into());
        let malformed = XRPAmount("xyz".into());

        let _ = valid.cmp(&malformed);
    }

    #[test]
    fn test_partial_ord_consistency() {
        let amount1 = XRPAmount("100".into());
        let amount2 = XRPAmount("200".into());

        // PartialOrd should be consistent with Ord
        assert_eq!(amount1.partial_cmp(&amount2), Some(amount1.cmp(&amount2)));
    }

    #[test]
    fn test_sorting_valid_amounts() {
        let mut amounts = vec![
            XRPAmount("50".into()),
            XRPAmount("100".into()),
            XRPAmount("25".into()),
        ];

        amounts.sort();

        assert_eq!(amounts[0].0.as_ref(), "25");
        assert_eq!(amounts[1].0.as_ref(), "50");
        assert_eq!(amounts[2].0.as_ref(), "100");
    }

    #[test]
    fn test_try_from_value_rejects_object() {
        let obj_value = serde_json::json!({"key": "value"});
        let result = XRPAmount::try_from(obj_value);
        assert!(result.is_err(), "Object should be rejected");
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("object"));
    }

    #[test]
    fn test_try_from_value_rejects_array() {
        let array_value = serde_json::json!([1, 2, 3]);
        let result = XRPAmount::try_from(array_value);
        assert!(result.is_err(), "Array should be rejected");
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("array"));
    }

    #[test]
    fn test_try_from_value_rejects_null() {
        let null_value = serde_json::Value::Null;
        let result = XRPAmount::try_from(null_value);
        assert!(result.is_err(), "Null should be rejected");
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("null"));
    }

    #[test]
    fn test_try_from_value_rejects_boolean() {
        let bool_value = serde_json::json!(true);
        let result = XRPAmount::try_from(bool_value);
        assert!(result.is_err(), "Boolean should be rejected");
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("boolean"));
    }

    #[test]
    fn test_try_from_value_accepts_string() {
        let string_value = serde_json::json!("100");
        let result = XRPAmount::try_from(string_value);
        assert!(result.is_ok(), "String should be accepted");
        assert_eq!(result.unwrap().0.as_ref(), "100");
    }

    #[test]
    fn test_try_from_value_accepts_number() {
        let number_value = serde_json::json!(100);
        let result = XRPAmount::try_from(number_value);
        assert!(result.is_ok(), "Number should be accepted");
    }
}
