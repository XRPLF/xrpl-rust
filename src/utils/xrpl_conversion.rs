//! Conversions between XRP drops and native number types.

use crate::utils::exceptions::XRPRangeException;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use regex::Regex;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

/// Indivisible unit of XRP
pub(crate) const _ONE_DROP: Decimal = Decimal::from_parts(1, 0, 0, false, 6);

pub const ONE_DROP: &str = "0.000001";
/// 100 billion decimal XRP
pub const MAX_XRP: u64 = u64::pow(10, 11);
/// Maximum possible drops of XRP
pub const MAX_DROPS: u64 = u64::pow(10, 17);
/// Drops in one XRP
pub const XRP_DROPS: u64 = 1000000;
// Minimum IC exponent
pub const MIN_IOU_EXPONENT: i8 = -96;
// Maximum IC exponent
pub const MAX_IOU_EXPONENT: u8 = 80;
// Maximum IC precision
pub const MAX_IOU_PRECISION: u8 = 16;

/// TODO Fix
/// Get the precision of a number.
fn _calculate_precision(value: &str) -> Result<usize, XRPRangeException> {
    let decimal = Decimal::from_str(value)?;
    let string = decimal.normalize().to_string();
    let regex = Regex::new("[^0-9]").unwrap();

    if let Some(value) = decimal.checked_rem(Decimal::ONE) {
        let quantized = value.to_string();
        let stripped = regex.replace(&quantized, "").replace(".", "");

        Ok(stripped.len())
    } else {
        let stripped = regex.replace(&string, "").replace(".", "");
        Ok(stripped.len())
    }
}

/// Convert a numeric XRP amount to drops of XRP.
/// Return an equivalent amount in drops of XRP.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use xrpl::utils::xrpl_conversion::xrp_to_drops;
/// let drops = xrp_to_drops("100.000001");
/// ```
pub fn xrp_to_drops(xrp: &str) -> Result<String, XRPRangeException> {
    let xrp_d = Decimal::from_str(xrp)?;

    if xrp_d < _ONE_DROP && xrp_d != Decimal::ZERO {
        Err(XRPRangeException::InvalidXRPAmountTooSmall {
            min: ONE_DROP.to_string(),
            found: xrp.to_string(),
        })
    } else if xrp_d.gt(&Decimal::new(MAX_XRP as i64, 0)) {
        Err(XRPRangeException::InvalidDropsAmountTooLarge {
            max: MAX_XRP.to_string(),
            found: xrp.to_string(),
        })
    } else {
        Ok(format!("{}", (xrp_d / _ONE_DROP).trunc()))
    }
}

/// Convert from drops to decimal XRP.
/// Return an equivalent amount of XRP from drops.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use xrpl::utils::xrpl_conversion::drops_to_xrp;
/// let xrp = drops_to_xrp("100000000");
/// ```
pub fn drops_to_xrp(drops: &str) -> Result<String, XRPRangeException> {
    let drops_d = Decimal::from_str(drops)?;
    let xrp = drops_d * _ONE_DROP;

    if xrp.gt(&Decimal::new(MAX_XRP as i64, 0)) {
        Err(XRPRangeException::InvalidDropsAmountTooLarge {
            max: MAX_XRP.to_string(),
            found: drops.to_string(),
        })
    } else {
        Ok(xrp.normalize().to_string())
    }
}

/// Validate if a provided XRP amount is valid.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use xrpl::utils::xrpl_conversion::verify_valid_xrp_value;
/// use xrpl::utils::exceptions::XRPRangeException;
///
/// let valid: bool = match verify_valid_xrp_value("0.00000001") {
///     Ok(()) => true,
///     Err(e) => match e {
///         // Handle each case your own way
///         XRPRangeException::InvalidXRPAmountTooSmall { min: _, found: _ } => false,
///         XRPRangeException::InvalidXRPAmountTooLarge { max: _, found: _ } => false,
///         _ => false,
///     },
/// };
/// ```
pub fn verify_valid_xrp_value(xrp_value: &str) -> Result<(), XRPRangeException> {
    let decimal = Decimal::from_str(xrp_value)?;
    let max = Decimal::new(MAX_DROPS as i64, 0);

    match decimal {
        xrp if xrp.is_zero() => Ok(()),
        xrp if xrp.ge(&_ONE_DROP) && xrp.le(&max) => Ok(()),
        xrp if xrp.lt(&_ONE_DROP) => Err(XRPRangeException::InvalidXRPAmountTooSmall {
            min: ONE_DROP.to_string(),
            found: xrp.to_string(),
        }),
        xrp if xrp.gt(&max) => Err(XRPRangeException::InvalidDropsAmountTooLarge {
            max: MAX_XRP.to_string(),
            found: xrp.to_string(),
        }),
        // Should never occur
        _ => Err(XRPRangeException::InvalidXRPAmount),
    }
}

/// Validates the format of an issued currency amount value.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use xrpl::utils::xrpl_conversion::verify_valid_ic_value;
/// use xrpl::utils::exceptions::XRPRangeException;
/// ```
pub fn verify_valid_ic_value(ic_value: &str) -> Result<(), XRPRangeException> {
    let decimal = Decimal::from_str(ic_value)?.normalize();
    let scale = -(decimal.scale() as i32);

    extern crate std;
    std::println!(
        "{:?} {:?} {:?}",
        ic_value,
        scale,
        _calculate_precision(ic_value).unwrap()
    );
    match decimal {
        ic if ic.is_zero() => Ok(()),
        _ if scale > MAX_IOU_EXPONENT as i32 => {
            Err(XRPRangeException::InvalidICPrecisionTooLarge {
                max: MAX_IOU_EXPONENT as i32,
                found: scale,
            })
        }
        _ if scale < MIN_IOU_EXPONENT as i32 => {
            Err(XRPRangeException::InvalidICPrecisionTooSmall {
                min: MIN_IOU_EXPONENT as i32,
                found: scale as i32,
            })
        }
        // Should never occur
        _ => Err(XRPRangeException::InvalidXRPAmount),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::alloc::string::ToString;

    #[test]
    fn test_one_drop_decimal() {
        let test: Decimal = Decimal::from_str("0.000001").unwrap();
        assert_eq!(_ONE_DROP, test);
    }

    #[test]
    fn test_xrp_to_drops() {
        let xrp = xrp_to_drops("100.000001").unwrap();
        let drops = (100 * XRP_DROPS + 1).to_string();

        assert_eq!(xrp, drops);
    }

    #[test]
    fn test_drops_to_xrp() {
        let drops: String = drops_to_xrp("100000001").unwrap();
        let xrp: Decimal = Decimal::new(100000001, 6);

        assert_eq!(xrp.to_string(), drops);
    }

    #[test]
    fn test_verify_valid_xrp_value() {
        assert!(verify_valid_xrp_value(ONE_DROP).is_ok());
        assert!(verify_valid_xrp_value(&MAX_DROPS.to_string()).is_ok());
        assert!(verify_valid_xrp_value("0.0000001").is_err());
        assert!(verify_valid_xrp_value("100000000000000001").is_err());
    }

    // #[test]
    // fn test_verify_valid_ic_value() {
    //     // { zero, pos, negative } * fractional, large, small
    //     let valid = [
    //         "0",
    //         "0.0",
    //         "1",
    //         "1.1111",
    //         "-1",
    //         "-1.1",
    //         "1111111111111111.0",
    //         "-1111111111111111.0",
    //         "0.00000000001",
    //         "0.00000000001",
    //         "-0.00000000001",
    //         "1.111111111111111e-3",
    //         "-1.111111111111111e-3",
    //         "2E+2",
    //     ];

    //     for case in valid {
    //         verify_valid_ic_value(case);
    //     }
    // }

    #[test]
    fn accept_one_xrp() {
        assert_eq!("1000000".to_string(), xrp_to_drops("1").unwrap());
    }

    #[test]
    fn accept_zero_xrp() {
        assert_eq!("0".to_string(), xrp_to_drops("0").unwrap());
    }

    #[test]
    fn accept_min_xrp() {
        assert_eq!("1".to_string(), xrp_to_drops("0.000001").unwrap());
    }

    #[test]
    fn accept_max_xrp() {
        let xrp = MAX_XRP.to_string();
        assert_eq!(
            "100000000000000000".to_string(),
            xrp_to_drops(&xrp).unwrap()
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
        let xrp = Decimal::from_str(ONE_DROP).unwrap();
        assert_eq!(xrp.to_string(), drops_to_xrp("1").unwrap());
    }

    #[test]
    fn accept_zero_drops() {
        let xrp = Decimal::from_str("0").unwrap();
        assert_eq!(xrp.to_string(), drops_to_xrp("0").unwrap());
    }

    #[test]
    fn accept_1mil_drops() {
        let xrp = Decimal::new(1, 0);
        assert_eq!(xrp.to_string(), drops_to_xrp("1000000").unwrap());
    }

    #[test]
    fn accept_max_drops() {
        let xrp = Decimal::new(MAX_XRP as i64, 0);

        assert_eq!(
            xrp.to_string(),
            drops_to_xrp(&MAX_DROPS.to_string()).unwrap()
        );
    }

    #[test]
    fn accept_too_big_drops() {
        let drop = (MAX_XRP + 1).to_string();
        assert!(xrp_to_drops(&drop).is_err());
    }
}
