//! Codec for XRPL Quality type.
//!
//! Used for encoding/decoding offer quality from BookDirectory 8-byte sequences.
//! The first byte is `exponent + 100` and the remaining 7 bytes are the mantissa.

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use bigdecimal::BigDecimal;
use core::str::FromStr;

use crate::core::exceptions::{XRPLCoreException, XRPLCoreResult};

/// Encode a quality string into 8 bytes.
///
/// Mirrors `xrpl.js` `quality.encode(quality: string) -> Uint8Array`.
///
/// Algorithm:
/// 1. Parse the quality string as a BigDecimal
/// 2. Compute exponent = floor(log10(value)) - 15
/// 3. Compute mantissa = value * 10^(-exponent) (as integer)
/// 4. Write mantissa as big-endian u64 (8 bytes)
/// 5. Set first byte to (exponent + 100)
pub fn encode_quality(quality: &str) -> XRPLCoreResult<Vec<u8>> {
    let decimal = BigDecimal::from_str(quality)
        .map_err(|_| XRPLCoreException::XRPLUtilsError("Invalid quality string".into()))?;

    // Get the exponent: BigDecimal stores (unscaled_value, scale) where value = unscaled * 10^(-scale)
    // We need to find the "e" (floor log10) like BigNumber.e in JavaScript
    // In BigNumber.js, `.e` is the exponent in scientific notation (floor of log10 of abs value)
    // For "195796912.5171664", BigNumber.e = 8
    // Then quality exponent = e - 15
    let (digits, scale) = decimal.as_bigint_and_exponent();
    let digit_str = digits.to_string();
    let digit_str = digit_str.trim_start_matches('-');
    let num_digits = digit_str.len() as i64;
    // e = num_digits - 1 - scale (JavaScript BigNumber.e equivalent)
    let e = num_digits - 1 - scale;

    let exponent = e - 15;

    // mantissa = decimal * 10^(-exponent), take absolute value
    let shift = BigDecimal::from_str(&alloc::format!("1e{}", -exponent))
        .map_err(|_| XRPLCoreException::XRPLUtilsError("Internal error".into()))?;
    let mantissa_decimal = &decimal * &shift;
    let mantissa_str = mantissa_decimal.abs().to_string();
    // Parse as u64 (strip any trailing ".0" etc)
    let mantissa_clean = mantissa_str.split('.').next().unwrap_or("0");
    let mantissa_u64: u64 = mantissa_clean
        .parse()
        .map_err(|_| XRPLCoreException::XRPLUtilsError("Mantissa too large for u64".into()))?;

    let mut bytes = mantissa_u64.to_be_bytes().to_vec();
    bytes[0] = (exponent + 100) as u8;

    Ok(bytes)
}

/// Decode quality from a hex string (typically 64-char BookDirectory hash).
///
/// Mirrors `xrpl.js` `quality.decode(quality: string) -> BigNumber`.
///
/// Takes the last 8 bytes of the hex string:
/// - First byte: exponent = byte - 100
/// - Remaining 7 bytes: mantissa (big-endian)
/// - Result: mantissa * 10^exponent
pub fn decode_quality(hex_str: &str) -> XRPLCoreResult<String> {
    let all_bytes = hex::decode(hex_str)
        .map_err(|_| XRPLCoreException::XRPLUtilsError("Invalid hex string".into()))?;
    if all_bytes.len() < 8 {
        return Err(XRPLCoreException::XRPLUtilsError(
            "Quality hex must be at least 8 bytes".into(),
        ));
    }
    let bytes = &all_bytes[all_bytes.len() - 8..];
    let exponent = bytes[0] as i32 - 100;

    // Extract mantissa from bytes[1..8] as big-endian u64
    let mut mantissa_bytes = [0u8; 8];
    mantissa_bytes[1..8].copy_from_slice(&bytes[1..8]);
    let mantissa = u64::from_be_bytes(mantissa_bytes);

    let mantissa_bd = BigDecimal::from(mantissa);
    let multiplier = BigDecimal::from_str(&alloc::format!("1e{}", exponent))
        .map_err(|_| XRPLCoreException::XRPLUtilsError("Internal error".into()))?;
    let result = mantissa_bd * multiplier;

    // Normalize to remove trailing zeros
    Ok(result.normalized().to_string())
}
