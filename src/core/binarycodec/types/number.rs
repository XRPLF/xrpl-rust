//! Codec for XRPL Number type (STNumber).
//!
//! Always encoded as 12 bytes: 8-byte signed mantissa (big-endian) + 4-byte signed exponent (big-endian).
//! Used for high-precision decimal fields like AssetsMaximum.

use alloc::string::String;
use alloc::string::ToString;
use serde::Serializer;

use crate::core::binarycodec::binary_wrappers::Parser;
use crate::core::exceptions::{XRPLCoreException, XRPLCoreResult};
use crate::core::BinaryParser;

use super::{SerializedType, TryFromParser, XRPLType};

const NUMBER_BYTE_LENGTH: usize = 12;
const MIN_MANTISSA: i128 = 1_000_000_000_000_000_000; // 10^18
const MAX_MANTISSA: i128 = 9_999_999_999_999_999_999; // 10^19 - 1
const MAX_INT64: i128 = 9_223_372_036_854_775_807; // 2^63 - 1
const MIN_EXPONENT: i32 = -32768;
const MAX_EXPONENT: i32 = 32768;
const DEFAULT_VALUE_EXPONENT: i32 = -2_147_483_648; // i32::MIN

#[derive(Debug, Clone)]
pub struct Number(SerializedType);

impl XRPLType for Number {
    type Error = XRPLCoreException;

    fn new(buffer: Option<&[u8]>) -> XRPLCoreResult<Self, Self::Error> {
        Ok(Number(SerializedType(
            buffer.unwrap_or(&[0u8; NUMBER_BYTE_LENGTH]).to_vec(),
        )))
    }
}

impl TryFromParser for Number {
    type Error = XRPLCoreException;

    fn from_parser(
        parser: &mut BinaryParser,
        _length: Option<usize>,
    ) -> XRPLCoreResult<Self, Self::Error> {
        let bytes = parser.read(NUMBER_BYTE_LENGTH)?;
        Ok(Number(SerializedType::from(bytes)))
    }
}

/// Parse a number string into (mantissa, exponent, is_negative).
fn extract_parts(val: &str) -> Result<(i128, i32, bool), XRPLCoreException> {
    // Regex-like manual parsing: [+-]?[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?
    let val = val.trim();
    let (sign, rest) = if let Some(stripped) = val.strip_prefix('-') {
        (true, stripped)
    } else if let Some(stripped) = val.strip_prefix('+') {
        (false, stripped)
    } else {
        (false, val)
    };

    // Split on 'e' or 'E'
    let (num_part, exp_part) = if let Some(pos) = rest.find(['e', 'E']) {
        (&rest[..pos], Some(&rest[pos + 1..]))
    } else {
        (rest, None)
    };

    // Split on '.'
    let (int_part, frac_part) = if let Some(pos) = num_part.find('.') {
        (&num_part[..pos], Some(&num_part[pos + 1..]))
    } else {
        (num_part, None)
    };

    // Clean leading zeros
    let clean_int = int_part.trim_start_matches('0');
    let clean_int = if clean_int.is_empty() { "0" } else { clean_int };

    let mut mantissa_str = String::from(clean_int);
    let mut exponent: i32 = 0;

    if let Some(frac) = frac_part {
        mantissa_str.push_str(frac);
        exponent -= frac.len() as i32;
    }
    if let Some(exp_s) = exp_part {
        exponent += exp_s
            .parse::<i32>()
            .map_err(|_| XRPLCoreException::XRPLUtilsError("Invalid exponent".into()))?;
    }

    // Remove trailing zeros from mantissa, adjust exponent
    while mantissa_str.len() > 1 && mantissa_str.ends_with('0') {
        mantissa_str.pop();
        exponent += 1;
    }

    let mut mantissa: i128 = mantissa_str
        .parse()
        .map_err(|_| XRPLCoreException::XRPLUtilsError("Invalid mantissa".into()))?;
    if sign {
        mantissa = -mantissa;
    }

    Ok((mantissa, exponent, mantissa < 0))
}

/// Normalize mantissa and exponent to XRPL constraints.
fn normalize(mantissa: i128, exponent: i32) -> Result<(i64, i32), XRPLCoreException> {
    let is_negative = mantissa < 0;
    let mut m = if is_negative { -mantissa } else { mantissa };
    let mut exp = exponent;

    if m == 0 {
        return Ok((0i64, DEFAULT_VALUE_EXPONENT));
    }

    // Grow mantissa until it reaches MIN_MANTISSA
    while m < MIN_MANTISSA && exp > MIN_EXPONENT {
        exp -= 1;
        m *= 10;
    }

    let mut last_digit: Option<i128> = None;

    // Shrink mantissa until it fits within MAX_MANTISSA
    while m > MAX_MANTISSA {
        if exp >= MAX_EXPONENT {
            return Err(XRPLCoreException::XRPLUtilsError(
                "Mantissa and exponent are too large".into(),
            ));
        }
        exp += 1;
        last_digit = Some(m % 10);
        m /= 10;
    }

    if exp < MIN_EXPONENT || m < MIN_MANTISSA {
        return Err(XRPLCoreException::XRPLUtilsError(
            "Underflow: value too small".into(),
        ));
    }
    if exp > MAX_EXPONENT {
        return Err(XRPLCoreException::XRPLUtilsError(
            "Exponent overflow".into(),
        ));
    }

    // Handle overflow if mantissa exceeds MAX_INT64
    if m > MAX_INT64 {
        if exp >= MAX_EXPONENT {
            return Err(XRPLCoreException::XRPLUtilsError(
                "Exponent overflow".into(),
            ));
        }
        exp += 1;
        last_digit = Some(m % 10);
        m /= 10;
    }

    // Rounding
    if let Some(ld) = last_digit {
        if ld >= 5 {
            m += 1;
            if m > MAX_INT64 {
                if exp >= MAX_EXPONENT {
                    return Err(XRPLCoreException::XRPLUtilsError(
                        "Exponent overflow".into(),
                    ));
                }
                let ld2 = m % 10;
                exp += 1;
                m /= 10;
                if ld2 >= 5 {
                    m += 1;
                }
            }
        }
    }

    let result = if is_negative { -(m as i64) } else { m as i64 };
    Ok((result, exp))
}

impl TryFrom<&str> for Number {
    type Error = XRPLCoreException;

    fn try_from(value: &str) -> XRPLCoreResult<Self, Self::Error> {
        let (mantissa, exponent, _) = extract_parts(value)?;
        let (norm_m, norm_e) = normalize(mantissa, exponent)?;

        let mut bytes = [0u8; NUMBER_BYTE_LENGTH];
        bytes[0..8].copy_from_slice(&norm_m.to_be_bytes());
        bytes[8..12].copy_from_slice(&norm_e.to_be_bytes());
        Ok(Number(SerializedType::from(bytes.to_vec())))
    }
}

/// Convert Number bytes back to a string representation.
pub fn number_to_string(bytes: &[u8]) -> Result<String, XRPLCoreException> {
    if bytes.len() != NUMBER_BYTE_LENGTH {
        return Err(XRPLCoreException::XRPLUtilsError(
            "Number must be 12 bytes".into(),
        ));
    }

    let mantissa = i64::from_be_bytes(bytes[0..8].try_into().unwrap());
    let exponent = i32::from_be_bytes(bytes[8..12].try_into().unwrap());

    // Canonical zero
    if mantissa == 0 && exponent == DEFAULT_VALUE_EXPONENT {
        return Ok("0".to_string());
    }

    // Validate exponent for non-zero mantissa: must be within normalized range.
    // Crafted binary with extreme exponents (e.g. i32::MIN) could cause integer
    // overflow in the offset arithmetic below (Rust-specific; JS doesn't overflow).
    if exponent < MIN_EXPONENT - 1 || exponent > MAX_EXPONENT + 1 {
        return Err(XRPLCoreException::XRPLUtilsError(
            "Invalid exponent in Number bytes".into(),
        ));
    }

    let is_negative = mantissa < 0;
    let mut mantissa_abs: i128 = if is_negative {
        -(mantissa as i128)
    } else {
        mantissa as i128
    };
    let mut exp = exponent;

    // Restore mantissa if it was shrunk for int64 serialization
    if mantissa_abs != 0 && mantissa_abs < MIN_MANTISSA {
        mantissa_abs *= 10;
        exp -= 1;
    }

    let range_log: i32 = 18;

    // Scientific notation for exponents outside [-28, -8] (when exp != 0)
    if exp != 0 && (exp < -(range_log + 10) || exp > -(range_log - 10)) {
        // Strip trailing zeros
        while mantissa_abs != 0 && mantissa_abs % 10 == 0 && exp < MAX_EXPONENT {
            mantissa_abs /= 10;
            exp += 1;
        }
        let sign = if is_negative { "-" } else { "" };
        return Ok(alloc::format!("{}{}e{}", sign, mantissa_abs, exp));
    }

    // Decimal rendering
    let pad_prefix = (range_log + 12) as usize; // 30
    let pad_suffix = (range_log + 8) as usize; // 26

    let mantissa_str = alloc::format!("{}", mantissa_abs);
    let mut raw_value = String::new();
    for _ in 0..pad_prefix {
        raw_value.push('0');
    }
    raw_value.push_str(&mantissa_str);
    for _ in 0..pad_suffix {
        raw_value.push('0');
    }

    let offset = (exp + pad_prefix as i32 + range_log + 1) as usize;
    let integer_part = raw_value[..offset].trim_start_matches('0');
    let integer_part = if integer_part.is_empty() {
        "0"
    } else {
        integer_part
    };
    let fraction_part = raw_value[offset..].trim_end_matches('0');

    let sign = if is_negative { "-" } else { "" };
    if fraction_part.is_empty() {
        Ok(alloc::format!("{}{}", sign, integer_part))
    } else {
        Ok(alloc::format!("{}{}.{}", sign, integer_part, fraction_part))
    }
}

impl serde::Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = number_to_string(self.as_ref()).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&s)
    }
}

impl AsRef<[u8]> for Number {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
