//! Codec for serializing and deserializing Amount fields.
//!
//! See Amount Fields:
//! `<https://xrpl.org/serialization.html#amount-fields>`

use super::exceptions::XRPLTypeException;
use super::AccountId;
use super::Currency;
use super::TryFromParser;
use super::XRPLType;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::core::exceptions::XRPLCoreException;
use crate::core::exceptions::XRPLCoreResult;
use crate::core::BinaryParser;
use crate::core::Parser;
use crate::utils::exceptions::XRPRangeException;
use crate::utils::*;
use crate::XRPLSerdeJsonError;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use bigdecimal::{BigDecimal, Signed, Zero};
use core::convert::TryFrom;
use core::convert::TryInto;
use core::fmt::Display;
use core::str::FromStr;
use rust_decimal::prelude::ToPrimitive;
use serde::ser::Error;
use serde::ser::SerializeMap;
use serde::Serializer;
use serde::{Deserialize, Serialize};

const _MIN_MANTISSA: u128 = u128::pow(10, 15);
const _MAX_MANTISSA: u128 = u128::pow(10, 16) - 1;

const _NOT_XRP_BIT_MASK: u8 = 0x80;
const _POS_SIGN_BIT_MASK: i64 = 0x4000000000000000;
const _ZERO_CURRENCY_AMOUNT_HEX: u64 = 0x8000000000000000;
const _NATIVE_AMOUNT_BYTE_LENGTH: u8 = 8;
const _CURRENCY_AMOUNT_BYTE_LENGTH: u8 = 48;

/// Normally when using bigdecimal "serde_json" feature a `1` will be serialized as `1.000000000000000`.
/// This function normalizes a `BigDecimal` before serializing to a string.
pub fn serialize_bigdecimal<S: Serializer>(
    value: &BigDecimal,
    s: S,
) -> XRPLCoreResult<S::Ok, S::Error> {
    let trimmed_str = value.normalized().to_string();
    s.serialize_str(&trimmed_str)
}

/// An Issued Currency object.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct IssuedCurrency {
    #[serde(serialize_with = "serialize_bigdecimal")]
    pub value: BigDecimal,
    pub currency: Currency,
    pub issuer: AccountId,
}

/// Codec for serializing and deserializing Amount fields.
///
/// See Amount Fields:
/// `<https://xrpl.org/serialization.html#amount-fields>`
#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "&str")]
pub struct Amount(Vec<u8>);

/// Returns True if the given string contains a
/// decimal point character.
fn _contains_decimal(string: &str) -> bool {
    string.contains('.')
}

/// Serializes the value field of an issued currency amount
/// to its bytes representation.
fn _serialize_issued_currency_value(decimal: BigDecimal) -> XRPLCoreResult<[u8; 8]> {
    verify_valid_ic_value(&decimal.to_scientific_notation())
        .map_err(|e| XRPLCoreException::XRPLUtilsError(e.to_string()))?;

    if decimal.is_zero() {
        return Ok((_ZERO_CURRENCY_AMOUNT_HEX).to_be_bytes());
    };

    let is_positive: bool = decimal.is_positive();
    let (mantissa_str, scale) = decimal.normalized().as_bigint_and_exponent();
    let mut exp: i32 = -(scale as i32);
    let mut mantissa: u128 = mantissa_str.abs().to_u128().unwrap();

    while mantissa < _MIN_MANTISSA && exp > MIN_IOU_EXPONENT {
        mantissa *= 10;
        exp -= 1;
    }

    while mantissa > _MAX_MANTISSA {
        if exp >= MAX_IOU_EXPONENT {
            return Err(XRPLBinaryCodecException::from(
                XRPRangeException::UnexpectedICAmountOverflow {
                    max: MAX_IOU_EXPONENT as usize,
                    found: exp as usize,
                },
            )
            .into());
        } else {
            mantissa /= 10;
            exp += 1;
        }
    }

    if exp < MIN_IOU_EXPONENT || mantissa < _MIN_MANTISSA {
        // Round to zero
        Ok((_ZERO_CURRENCY_AMOUNT_HEX).to_be_bytes())
    } else if exp > MAX_IOU_EXPONENT || mantissa > _MAX_MANTISSA {
        Err(
            XRPLBinaryCodecException::from(XRPRangeException::UnexpectedICAmountOverflow {
                max: MAX_IOU_EXPONENT as usize,
                found: exp as usize,
            })
            .into(),
        )
    } else {
        // "Not XRP" bit set
        let mut serial: i128 = _ZERO_CURRENCY_AMOUNT_HEX as i128;

        // "Is positive" bit set
        if is_positive {
            serial |= _POS_SIGN_BIT_MASK as i128;
        };

        // next 8 bits are exponents
        serial |= ((exp as i64 + 97) << 54) as i128;
        // last 54 bits are mantissa
        serial |= mantissa as i128;

        Ok((serial as u64).to_be_bytes())
    }
}

/// Serializes an XRP amount.
fn _serialize_xrp_amount(value: &str) -> XRPLCoreResult<[u8; 8]> {
    verify_valid_xrp_value(value).map_err(|e| XRPLCoreException::XRPLUtilsError(e.to_string()))?;

    let decimal = bigdecimal::BigDecimal::from_str(value)
        .map_err(XRPLTypeException::BigDecimalError)?
        .normalized();

    if let Some(result) = decimal.to_i64() {
        let value_with_pos_bit = result | _POS_SIGN_BIT_MASK;
        Ok(value_with_pos_bit.to_be_bytes())
    } else {
        // Safety, should never occur
        Err(XRPLCoreException::XRPLUtilsError(
            XRPRangeException::InvalidXRPAmount.to_string(),
        ))
    }
}

/// Serializes an issued currency amount.
fn _serialize_issued_currency_amount(issused_currency: IssuedCurrency) -> XRPLCoreResult<[u8; 48]> {
    let mut bytes = vec![];
    let amount_bytes = _serialize_issued_currency_value(issused_currency.value)?;
    let currency_bytes: &[u8] = issused_currency.currency.as_ref();
    let issuer_bytes: &[u8] = issused_currency.issuer.as_ref();

    bytes.extend_from_slice(&amount_bytes);
    bytes.extend_from_slice(currency_bytes);
    bytes.extend_from_slice(issuer_bytes);

    if bytes.len() != 48 {
        Err(
            XRPLBinaryCodecException::from(XRPRangeException::InvalidICSerializationLength {
                expected: 48,
                found: bytes.len(),
            })
            .into(),
        )
    } else {
        Ok(bytes.try_into().expect("_serialize_issued_currency_amount"))
    }
}

impl Amount {
    /// Deserialize native asset amount.
    fn _deserialize_native_amount(&self) -> String {
        let mut sized: [u8; 8] = Default::default();

        sized.copy_from_slice(&self.as_ref()[..8]);
        (u64::from_be_bytes(sized) & 0x3FFFFFFFFFFFFFFF).to_string()
    }

    /// Returns True if this amount is a native XRP amount.
    pub fn is_native(&self) -> bool {
        self.0[0] & 0x80 == 0
    }

    /// Returns true if 2nd bit in 1st byte is set to 1
    /// (positive amount).
    pub fn is_positive(&self) -> bool {
        self.0[1] & 0x40 > 0
    }
}

impl IssuedCurrency {
    /// Deserialize the issued currency amount.
    fn _deserialize_issued_currency_amount(
        parser: &mut BinaryParser,
    ) -> XRPLCoreResult<BigDecimal> {
        let mut value: BigDecimal;
        let bytes = parser.read(8)?;

        // Some wizardry by Amie Corso
        let exp = ((bytes[0] as i32 & 0x3F) << 2) + ((bytes[1] as i32 & 0xFF) >> 6) - 97;

        if exp < MIN_IOU_EXPONENT {
            value = BigDecimal::from(0);
        } else {
            let hex_mantissa = hex::encode([&[bytes[1] & 0x3F], &bytes[2..]].concat());
            let int_mantissa = i128::from_str_radix(&hex_mantissa, 16)
                .map_err(XRPLBinaryCodecException::ParseIntError)?;

            // Adjust scale using the exponent
            let scale = exp.unsigned_abs();
            value = BigDecimal::new(int_mantissa.into(), scale as i64);

            // Handle the sign
            if bytes[0] & 0x40 > 0 {
                // Set the value to positive (BigDecimal assumes positive by default)
                value = value.abs();
            } else {
                // Set the value to negative
                value = -value.abs();
            }
        }
        verify_valid_ic_value(&value.to_string())
            .map_err(|e| XRPLCoreException::XRPLUtilsError(e.to_string()))?;

        Ok(value)
    }
}

impl XRPLType for Amount {
    type Error = hex::FromHexError;

    /// Construct an Amount from given bytes.
    fn new(buffer: Option<&[u8]>) -> XRPLCoreResult<Self, Self::Error> {
        if let Some(data) = buffer {
            Ok(Amount(data.to_vec()))
        } else {
            Ok(Amount(vec![]))
        }
    }
}

impl TryFromParser for Amount {
    type Error = XRPLCoreException;

    /// Build Amount from a BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        _length: Option<usize>,
    ) -> XRPLCoreResult<Amount, Self::Error> {
        let parser_first_byte = parser.peek();
        let num_bytes = match parser_first_byte {
            None => _CURRENCY_AMOUNT_BYTE_LENGTH,
            Some(_) => _NATIVE_AMOUNT_BYTE_LENGTH,
        };

        Ok(Amount(parser.read(num_bytes as usize)?))
    }
}

impl TryFromParser for IssuedCurrency {
    type Error = XRPLCoreException;

    /// Build IssuedCurrency from a BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        _length: Option<usize>,
    ) -> XRPLCoreResult<IssuedCurrency, Self::Error> {
        Ok(IssuedCurrency {
            value: IssuedCurrency::_deserialize_issued_currency_amount(parser)?,
            currency: Currency::from_parser(parser, None)?,
            issuer: AccountId::from_parser(parser, None)?,
        })
    }
}

impl Serialize for Amount {
    /// Construct a JSON object representing this Amount.
    fn serialize<S>(&self, serializer: S) -> XRPLCoreResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.is_native() {
            serializer.serialize_str(&self._deserialize_native_amount())
        } else {
            let mut parser = BinaryParser::from(self.as_ref());

            if let Ok(ic) = IssuedCurrency::from_parser(&mut parser, None) {
                let mut builder = serializer.serialize_map(Some(3))?;

                builder.serialize_entry("value", &ic.value.normalized())?;
                builder.serialize_entry("currency", &ic.currency)?;
                builder.serialize_entry("issuer", &ic.issuer)?;
                builder.end()
            } else {
                Err(S::Error::custom(
                    XRPLBinaryCodecException::InvalidReadFromBytesValue,
                ))
            }
        }
    }
}

impl TryFrom<&str> for Amount {
    type Error = XRPLCoreException;

    /// Construct an Amount object from a hex string.
    fn try_from(value: &str) -> XRPLCoreResult<Self, Self::Error> {
        let serialized = _serialize_xrp_amount(value)?;
        Ok(Amount::new(Some(&serialized))?)
    }
}

impl TryFrom<IssuedCurrency> for Amount {
    type Error = XRPLCoreException;

    /// Construct an Amount object from an IssuedCurrency.
    fn try_from(value: IssuedCurrency) -> XRPLCoreResult<Self, Self::Error> {
        let serialized = _serialize_issued_currency_amount(value)?;
        Ok(Amount::new(Some(&serialized))?)
    }
}

impl TryFrom<serde_json::Value> for Amount {
    type Error = XRPLCoreException;

    /// Construct an Amount object from a Serde JSON Value.
    fn try_from(value: serde_json::Value) -> XRPLCoreResult<Self, Self::Error> {
        if value.is_string() {
            let xrp_value = value.as_str().ok_or(XRPLTypeException::InvalidNoneValue)?;
            Self::try_from(xrp_value)
        } else if value.is_object() {
            Ok(Self::try_from(IssuedCurrency::try_from(value)?)?)
        } else {
            Err(XRPLCoreException::SerdeJsonError(
                XRPLSerdeJsonError::UnexpectedValueType {
                    expected: "String/Object".into(),
                    found: value,
                },
            ))
        }
    }
}

impl TryFrom<serde_json::Value> for IssuedCurrency {
    type Error = XRPLCoreException;

    /// Construct an IssuedCurrency object from a Serde JSON Value.
    fn try_from(json: serde_json::Value) -> XRPLCoreResult<Self, Self::Error> {
        let value = BigDecimal::from_str(
            json["value"]
                .as_str()
                .ok_or(XRPLTypeException::InvalidNoneValue)?,
        )
        .map_err(XRPLTypeException::BigDecimalError)?;
        let currency = Currency::try_from(
            json["currency"]
                .as_str()
                .ok_or(XRPLTypeException::InvalidNoneValue)?,
        )?;
        let issuer = AccountId::try_from(
            json["issuer"]
                .as_str()
                .ok_or(XRPLTypeException::InvalidNoneValue)?,
        )?;

        Ok(IssuedCurrency {
            value,
            currency,
            issuer,
        })
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&hex::encode_upper(self.as_ref()), f)
    }
}

impl AsRef<[u8]> for Amount {
    /// Get a reference of the byte representation.
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::binarycodec::test_cases::load_data_tests;
    use crate::core::binarycodec::types::test_cases::IOUCase;
    use crate::core::binarycodec::types::test_cases::TEST_XRP_CASES;
    use alloc::format;

    const IOU_TEST: &str = include_str!("../test_data/iou-tests.json");

    #[test]
    fn test_contains_decimal() {
        assert!(_contains_decimal("1.00"));
        assert!(!_contains_decimal("100"));
    }

    #[test]
    fn test_amount_new() {
        let json: Vec<IOUCase> = serde_json::from_str(IOU_TEST).expect("");

        for case in json {
            let bytes = hex::decode(case.1).expect("");
            let amount: Amount = Amount::new(Some(&bytes)).unwrap();

            assert_eq!(hex::encode_upper(bytes), amount.to_string())
        }
    }

    #[test]
    fn test_amount_try_from_issuedcurrency() {
        let json: Vec<IOUCase> = serde_json::from_str(IOU_TEST).expect("");

        for case in json {
            let amount = Amount::try_from(case.0).unwrap();
            assert_eq!(case.1, amount.to_string())
        }
    }

    #[test]
    fn test_amount_try_from_string() {
        for (xrp, result) in TEST_XRP_CASES {
            let amount = Amount::try_from(xrp).unwrap();
            assert_eq!(result, amount.to_string())
        }
    }

    #[test]
    fn accept_amount_serde_encode_decode() {
        let json: Vec<IOUCase> = serde_json::from_str(IOU_TEST).expect("");

        for case in json {
            let expect = serde_json::to_string(&case.0).expect("");
            let bytes = hex::decode(case.1).expect("");
            let amount: Amount = Amount::new(Some(&bytes)).unwrap();
            let serialize = serde_json::to_string(&amount).unwrap();

            assert_eq!(serialize, expect);
        }

        for (xrp, result) in TEST_XRP_CASES {
            let bytes = hex::decode(result).expect("");
            let amount: Amount = Amount::new(Some(&bytes)).unwrap();
            let serialize = serde_json::to_string(&amount).unwrap();

            assert_eq!(serialize, format!("\"{xrp}\""));
        }
    }

    #[test]
    fn accept_amount_value_tests() {
        let tests = load_data_tests(Some("Amount"));

        for test in tests {
            let amount = Amount::try_from(test.test_json);

            if test.error.is_none() {
                assert_eq!(test.expected_hex, Some(amount.unwrap().to_string()));
            } else {
                assert!(amount.is_err());
            }
        }
    }
}
