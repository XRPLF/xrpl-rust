use crate::models::Model;
use crate::{models::amount::exceptions::XRPLAmountException, Err};
use alloc::{
    borrow::Cow,
    string::{String, ToString},
};
use anyhow::Result;
use core::convert::{TryFrom, TryInto};
use core::str::FromStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents an amount of XRP in Drops.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Default)]
pub struct XRPAmount<'a>(pub Cow<'a, str>);

impl<'a> Model for XRPAmount<'a> {}

// implement Deserializing from Cow<str>, &str, String, Decimal, f64, u32, and Value
impl<'de, 'a> Deserialize<'de> for XRPAmount<'a> {
    fn deserialize<D>(deserializer: D) -> Result<XRPAmount<'a>, D::Error>
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

impl<'a> From<Decimal> for XRPAmount<'a> {
    fn from(value: Decimal) -> Self {
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
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match serde_json::to_string(&value) {
            Ok(amount_string) => {
                let amount_string = amount_string.clone().replace("\"", "");
                Ok(Self(amount_string.into()))
            }
            Err(serde_error) => Err!(XRPLAmountException::FromSerdeError(serde_error)),
        }
    }
}

impl<'a> TryInto<f64> for XRPAmount<'a> {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<f64, Self::Error> {
        match self.0.parse::<f64>() {
            Ok(f64_value) => Ok(f64_value),
            Err(parse_error) => Err!(XRPLAmountException::ToFloatError(parse_error)),
        }
    }
}

impl<'a> TryInto<u32> for XRPAmount<'a> {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<u32, Self::Error> {
        match self.0.parse::<u32>() {
            Ok(u32_value) => Ok(u32_value),
            Err(parse_error) => Err!(XRPLAmountException::ToIntError(parse_error)),
        }
    }
}

impl<'a> TryInto<Decimal> for XRPAmount<'a> {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Decimal, Self::Error> {
        match Decimal::from_str(&self.0) {
            Ok(decimal) => Ok(decimal),
            Err(decimal_error) => Err!(XRPLAmountException::ToDecimalError(decimal_error)),
        }
    }
}

impl<'a> PartialOrd for XRPAmount<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl<'a> Ord for XRPAmount<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
