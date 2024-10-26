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

impl<'a> Model for XRPAmount<'a> {}

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

impl<'a> PartialOrd for XRPAmount<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for XRPAmount<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let self_decimal: BigDecimal = self.clone().try_into().unwrap();
        let other_decimal: BigDecimal = other.clone().try_into().unwrap();
        self_decimal.cmp(&other_decimal)
    }
}
