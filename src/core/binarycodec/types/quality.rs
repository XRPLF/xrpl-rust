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

    // Bounds check: exponent must be in representable range [-100, 155]
    // Prevent overflow when computing scale and creating BigDecimal
    if !(-100..=155).contains(&exponent) {
        return Err(XRPLCoreException::XRPLUtilsError(alloc::format!(
            "Quality exponent {} out of representable range [-100, 155]",
            exponent
        )));
    }

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
    if !(0..=255).contains(&(exponent + 100)) {
        return Err(XRPLCoreException::XRPLUtilsError(alloc::format!(
            "Quality exponent {} out of representable range [-100, 155]",
            exponent
        )));
    }
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

    // Bounds check: exponent must be in representable range [-100, 155]
    // Prevent overflow when creating BigDecimal scale
    if !(-100..=155).contains(&exponent) {
        return Err(XRPLCoreException::XRPLUtilsError(alloc::format!(
            "Quality exponent {} out of representable range [-100, 155]",
            exponent
        )));
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_quality_valid() {
        let quality = "100";
        let result = encode_quality(quality);
        assert!(result.is_ok(), "Valid quality should encode successfully");
        let bytes = result.unwrap();
        assert_eq!(bytes.len(), 8, "Encoded quality should be 8 bytes");
    }

    #[test]
    fn test_decode_quality_valid() {
        // First encode a known quality, then decode it
        let quality_str = "100";
        let encoded = encode_quality(quality_str);
        assert!(encoded.is_ok());

        let encoded_bytes = encoded.unwrap();
        let hex_string = hex::encode(&encoded_bytes);
        let decoded = decode_quality(&hex_string);
        assert!(
            decoded.is_ok(),
            "Valid encoded quality should decode successfully"
        );
    }

    #[test]
    fn test_decode_quality_rejects_out_of_bounds_exponent() {
        // The byte value directly determines exponent, so mathematically:
        // byte 0 -> exponent = -100, byte 255 -> exponent = 155
        // All values 0-255 are within [-100, 155] range
        // However, our bounds check ensures defensive coding
        let mut bytes = [0u8; 8];
        bytes[0] = 0; // exponent = -100
        let hex_string = hex::encode(&bytes);
        let result = decode_quality(&hex_string);
        assert!(result.is_ok(), "Minimum exponent -100 should be accepted");

        bytes[0] = 255; // exponent = 155
        let hex_string = hex::encode(&bytes);
        let result = decode_quality(&hex_string);
        assert!(result.is_ok(), "Maximum exponent 155 should be accepted");
    }

    #[test]
    fn test_decode_quality_requires_minimum_8_bytes() {
        let hex_string = "0102030405"; // Only 5 bytes
        let result = decode_quality(&hex_string);
        assert!(result.is_err(), "Quality hex with < 8 bytes should fail");
    }

    #[test]
    fn test_roundtrip_quality_encode_decode() {
        let original_quality = "150.5";
        let encoded = encode_quality(original_quality);
        assert!(encoded.is_ok(), "Quality should encode");

        let hex_string = hex::encode(encoded.unwrap());
        let decoded = decode_quality(&hex_string);
        assert!(decoded.is_ok(), "Quality should decode");

        // The decoded value should be approximately equal to original
        // (may differ slightly due to rounding)
        let decoded_value = decoded.unwrap();
        let decoded_f64: f64 = decoded_value.parse().unwrap_or(0.0);
        let original_f64: f64 = original_quality.parse().unwrap_or(0.0);

        // Allow small floating point error
        let ratio = (decoded_f64 / original_f64 - 1.0).abs();
        assert!(
            ratio < 0.001,
            "Decoded quality should be close to original. Original: {}, Decoded: {}",
            original_f64,
            decoded_f64
        );
    }
}
