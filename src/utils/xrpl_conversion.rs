//! Conversions between XRP drops and native number types.

use crate::utils::exceptions::XRPRangeException;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use bigdecimal::BigDecimal;
use regex::Regex;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

use super::exceptions::XRPLUtilsResult;

/// Indivisible unit of XRP
pub(crate) const _ONE_DROP: Decimal = Decimal::from_parts(1, 0, 0, false, 6);

/// One drop in decimal form.
pub const ONE_DROP: &str = "0.000001";
/// 100 billion decimal XRP
pub const MAX_XRP: u64 = u64::pow(10, 11);
/// Maximum possible drops of XRP
pub const MAX_DROPS: u64 = u64::pow(10, 17);
/// Drops in one XRP
pub const XRP_DROPS: u64 = 1000000;
/// Minimum IC exponent
pub const MIN_IOU_EXPONENT: i32 = -96;
/// Maximum IC exponent
pub const MAX_IOU_EXPONENT: i32 = 80;
/// Maximum IC precision
pub const MAX_IOU_PRECISION: u8 = 16;

/// Checked remainder. Computes self % other, returning None if overflow occurred.
fn checked_rem(first: &BigDecimal, second: &BigDecimal) -> Option<BigDecimal> {
    // If second is zero, return None to avoid division by zero
    if second.is_zero() {
        return None;
    }

    // Perform the division and handle the case where there's no overflow
    match checked_div(first, second) {
        Some(div) => {
            // Get the integer part of the division
            let int_part = div.with_scale(0); // Truncate to integer

            // Calculate remainder: remainder = first - (int_part * second)
            let rem = first - &(int_part * second);

            Some(rem)
        }
        None => None, // If the division fails, return None (overflow case)
    }
}

/// Checked division. Computes self / other, returning None if overflow occurred.
fn checked_div(first: &BigDecimal, second: &BigDecimal) -> Option<BigDecimal> {
    // If second is zero, return None to avoid division by zero
    if second.is_zero() {
        return None;
    }

    // Perform the division and return Some(result) if successful
    Some(first / second)
}

/// Checked multiplication. Computes self * other, returning None if overflow occurred.
fn checked_mul(first: &BigDecimal, second: &BigDecimal) -> Option<BigDecimal> {
    // Perform the multiplication
    let result = first * second;

    // Since BigDecimal supports arbitrary precision, we don't need to check for overflow.
    // Simply return the result wrapped in Some.
    Some(result)
}

/// TODO Make less bootleg
/// Get the precision of a number.
fn _calculate_precision(value: &str) -> XRPLUtilsResult<usize> {
    let decimal = BigDecimal::from_str(value)?.normalized();
    let regex = Regex::new("[^1-9]").expect("_calculate_precision");

    if checked_rem(&decimal, &BigDecimal::one()).is_some() {
        let stripped = regex
            .replace(&decimal.to_string(), "")
            .replace(['.', '0'], "");
        Ok(stripped.len())
    } else {
        let quantized = decimal.with_scale(2);
        let stripped = regex
            .replace(&quantized.to_string(), "")
            .replace(['.', '0'], "");
        Ok(stripped.len())
    }
}

/// Ensure that the value after being multiplied by the
/// exponent does not contain a decimal.
fn _verify_no_decimal(decimal: BigDecimal) -> XRPLUtilsResult<()> {
    let (mantissa, scale) = decimal.as_bigint_and_exponent();
    let decimal = BigDecimal::from_i64(scale).expect("_verify_no_decimal");

    let value: String = if decimal == BigDecimal::zero() {
        mantissa.to_string()
    } else {
        (&decimal * &decimal).to_string()
        // decimal
        //     .checked_mul(decimal)
        //     .unwrap_or(Decimal::ZERO)
        //     .to_string()
    };

    if value.contains('.') {
        Err(XRPRangeException::InvalidValueContainsDecimal.into())
    } else {
        Ok(())
    }
}

/// Convert a numeric XRP amount to drops of XRP.
/// Return an equivalent amount in drops of XRP.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::utils::xrp_to_drops;
/// use xrpl::utils::exceptions::{XRPRangeException, XRPLUtilsException};
///
/// let xrp: &str = "100.000001";
/// let drops: String = "100000001".to_string();
///
/// let conversion: Option<String> = match xrp_to_drops(xrp) {
///     Ok(xrp) => Some(xrp),
///     Err(e) => match e {
///         XRPLUtilsException::XRPRangeError(XRPRangeException::InvalidXRPAmountTooLarge { max: _, found: _ }) => None,
///         XRPLUtilsException::XRPRangeError(XRPRangeException::InvalidXRPAmountTooSmall { min: _, found: _ }) => None,
///         _ => None,
///     },
/// };
///
/// assert_eq!(Some(drops), conversion);
/// ```
pub fn xrp_to_drops(xrp: &str) -> XRPLUtilsResult<String> {
    let xrp_d = Decimal::from_str(xrp)?;

    if xrp_d < _ONE_DROP && xrp_d != Decimal::ZERO {
        Err(XRPRangeException::InvalidXRPAmountTooSmall {
            min: ONE_DROP.to_string(),
            found: xrp.to_string(),
        }
        .into())
    } else if xrp_d.gt(&Decimal::new(MAX_XRP as i64, 0)) {
        Err(XRPRangeException::InvalidXRPAmountTooLarge {
            max: MAX_XRP,
            found: xrp.into(),
        }
        .into())
    } else {
        Ok(format!("{}", (xrp_d / _ONE_DROP).trunc()))
    }
}

/// Convert from drops to decimal XRP.
/// Return an equivalent amount of XRP from drops.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::utils::drops_to_xrp;
/// use xrpl::utils::exceptions::{XRPRangeException, XRPLUtilsException};
///
/// let drops: &str = "100000000";
/// let xrp: String = "100".to_string();
///
/// let conversion: Option<String> = match drops_to_xrp(drops) {
///     Ok(xrp) => Some(xrp),
///     Err(e) => match e {
///         XRPLUtilsException::XRPRangeError(XRPRangeException::InvalidDropsAmountTooLarge { max: _, found: _ }) => None,
///         _ => None,
///     },
/// };
///
/// assert_eq!(Some(xrp), conversion);
/// ```
pub fn drops_to_xrp(drops: &str) -> XRPLUtilsResult<String> {
    let drops_d = Decimal::from_str(drops)?;
    let xrp = drops_d * _ONE_DROP;

    if xrp.gt(&Decimal::new(MAX_XRP as i64, 0)) {
        Err(XRPRangeException::InvalidDropsAmountTooLarge {
            max: MAX_XRP.to_string(),
            found: drops.to_string(),
        }
        .into())
    } else {
        Ok(xrp.normalize().to_string())
    }
}

/// Validate if a provided XRP amount is valid.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::utils::verify_valid_xrp_value;
/// use xrpl::utils::exceptions::{XRPRangeException, XRPLUtilsException};
///
/// let valid: bool = match verify_valid_xrp_value("0.000001") {
///     Ok(()) => true,
///     Err(e) => match e {
///         XRPLUtilsException::XRPRangeError(XRPRangeException::InvalidXRPAmountTooSmall { min: _, found: _ }) => false,
///         XRPLUtilsException::XRPRangeError(XRPRangeException::InvalidXRPAmountTooLarge { max: _, found: _ }) => false,
///         _ => false,
///     },
/// };
///
/// assert!(valid);
/// ```
pub fn verify_valid_xrp_value(xrp_value: &str) -> XRPLUtilsResult<()> {
    let decimal = Decimal::from_str(xrp_value)?;
    let max = Decimal::new(MAX_DROPS as i64, 0);

    match decimal {
        xrp if xrp.is_zero() => Ok(()),
        xrp if xrp.ge(&_ONE_DROP) && xrp.le(&max) => Ok(()),
        xrp if xrp.lt(&_ONE_DROP) => Err(XRPRangeException::InvalidXRPAmountTooSmall {
            min: ONE_DROP.to_string(),
            found: xrp.to_string(),
        }
        .into()),
        xrp if xrp.gt(&max) => Err(XRPRangeException::InvalidDropsAmountTooLarge {
            max: MAX_XRP.to_string(),
            found: xrp.to_string(),
        }
        .into()),
        // Should never occur
        _ => Err(XRPRangeException::InvalidXRPAmount.into()),
    }
}

/// Validates the format of an issued currency amount value.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::utils::verify_valid_ic_value;
/// use xrpl::utils::exceptions::{XRPRangeException, XRPLUtilsException};
///
/// let valid: bool = match verify_valid_ic_value("1111111111111111.0") {
///     Ok(()) => true,
///     Err(e) => match e {
///         XRPLUtilsException::XRPRangeError(XRPRangeException::InvalidICPrecisionTooSmall { min: _, found: _ }) => false,
///         XRPLUtilsException::XRPRangeError(XRPRangeException::InvalidICPrecisionTooLarge { max: _, found: _ }) => false,
///         _ => false,
///     },
/// };
///
/// assert!(valid);
/// ```
pub fn verify_valid_ic_value(ic_value: &str) -> XRPLUtilsResult<()> {
    let decimal = BigDecimal::from_str(ic_value)?.normalized();
    let scale = -(decimal.fractional_digit_count() as i32);
    let prec = _calculate_precision(ic_value)?;

    match decimal {
        ic if ic.is_zero() => Ok(()),
        _ if prec > MAX_IOU_PRECISION as usize || scale > MAX_IOU_EXPONENT => {
            Err(XRPRangeException::InvalidICPrecisionTooLarge {
                max: MAX_IOU_EXPONENT,
                found: scale,
            }
            .into())
        }
        _ if prec > MAX_IOU_PRECISION as usize || scale < MIN_IOU_EXPONENT => {
            Err(XRPRangeException::InvalidICPrecisionTooSmall {
                min: MIN_IOU_EXPONENT,
                found: scale,
            }
            .into())
        }
        _ => _verify_no_decimal(decimal),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::alloc::string::ToString;
    extern crate std;

    #[test]
    fn test_one_drop_decimal() {
        assert_eq!(Ok(_ONE_DROP), Decimal::from_str("0.000001"));
    }

    #[test]
    fn test_xrp_to_drops() {
        assert_eq!(
            Ok((100 * XRP_DROPS + 1).to_string()),
            xrp_to_drops("100.000001")
        );
    }

    #[test]
    fn test_drops_to_xrp() {
        assert_eq!(
            drops_to_xrp("100000001"),
            Ok(Decimal::new(100000001, 6).to_string())
        );
    }

    #[test]
    fn test_verify_valid_xrp_value() {
        assert!(verify_valid_xrp_value(ONE_DROP).is_ok());
        assert!(verify_valid_xrp_value(&MAX_DROPS.to_string()).is_ok());
        assert!(verify_valid_xrp_value("0.0000001").is_err());
        assert!(verify_valid_xrp_value("100000000000000001").is_err());
    }

    #[test]
    fn test_verify_valid_ic_value() {
        // { zero, pos, negative } * fractional, large, small
        let valid = [
            "0",
            "0.0",
            "1",
            "1.1111",
            "-1",
            "-1.1",
            "1111111111111111.0",
            "-1111111111111111.0",
            "0.00000000001",
            "0.00000000001",
            "-0.00000000001",
            "0.001111111111111111",
            "-0.001111111111111111",
        ];

        let invalid = ["-0.0011111111111111111"];

        for case in valid {
            assert!(verify_valid_ic_value(case).is_ok());
        }

        for case in invalid {
            assert!(verify_valid_ic_value(case).is_err());
        }
    }

    #[test]
    fn accept_one_xrp() {
        assert_eq!(xrp_to_drops("1"), Ok("1000000".to_string()));
    }

    #[test]
    fn accept_zero_xrp() {
        assert_eq!(xrp_to_drops("0"), Ok("0".to_string()));
    }

    #[test]
    fn accept_min_xrp() {
        assert_eq!(xrp_to_drops("0.000001"), Ok("1".to_string()));
    }

    #[test]
    fn accept_max_xrp() {
        assert_eq!(
            xrp_to_drops(&MAX_XRP.to_string()),
            Ok("100000000000000000".to_string())
        );
    }

    #[test]
    fn accept_too_small_xrp() {
        assert!(xrp_to_drops("0.0000001").is_err());
    }

    #[test]
    fn accept_too_big_xrp() {
        let xrp = (MAX_XRP + 1).to_string();
        assert!(xrp_to_drops(&xrp).is_err());
    }

    #[test]
    fn accept_one_drop() {
        assert_eq!(drops_to_xrp("1"), Ok(ONE_DROP.to_string()));
    }

    #[test]
    fn accept_zero_drops() {
        assert_eq!(drops_to_xrp("0"), Ok("0".to_string()));
    }

    #[test]
    fn accept_1mil_drops() {
        assert_eq!(drops_to_xrp("1000000"), Ok(Decimal::new(1, 0).to_string()));
    }

    #[test]
    fn accept_max_drops() {
        assert_eq!(
            drops_to_xrp(&MAX_DROPS.to_string()),
            Ok(Decimal::new(MAX_XRP as i64, 0).to_string())
        );
    }

    #[test]
    fn accept_too_big_drops() {
        assert!(xrp_to_drops(&(MAX_XRP + 1).to_string()).is_err());
    }
}
