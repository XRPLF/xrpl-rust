//! Conversions between XRP drops and native number types.

use alloc::fmt::Display;
use alloc::fmt::Formatter;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

/// Indivisible unit of XRP
pub const ONE_DROP: Decimal = Decimal::from_parts(1, 0, 0, false, 6);
/// 100 billion decimal XRP
pub const MAX_XRP: u64 = u64::pow(10, 11);
/// Maximum possible drops of XRP
pub const MAX_DROPS: u64 = u64::pow(10, 17);
/// Drops in one XRP
pub const XRP_DROPS: u64 = 1000000;

#[derive(Debug)]
/// Exception for invalid XRP Ledger time data.
pub struct XRPRangeException {
    message: String,
}

impl Display for XRPRangeException {
    fn fmt(&self, f: &mut Formatter) -> alloc::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<rust_decimal::Error> for XRPRangeException {
    fn from(err: rust_decimal::Error) -> Self {
        XRPRangeException {
            message: err.to_string(),
        }
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
/// use rust_decimal::Decimal;
/// use xrpl::utils::xrpl_conversion::xrp_to_drops;
///
/// let drops = xrp_to_drops(Decimal::new(100000000, 6));
/// ```
pub fn xrp_to_drops(xrp: Decimal) -> Result<String, XRPRangeException> {
    if xrp < ONE_DROP && xrp != Decimal::ZERO {
        Err(XRPRangeException {
            message: format!("XRP amount {} is too small.", xrp),
        })
    } else if xrp.gt(&Decimal::new(MAX_XRP as i64, 0)) {
        Err(XRPRangeException {
            message: format!("XRP amount {} is too large.", xrp),
        })
    } else {
        Ok(format!("{}", (xrp / ONE_DROP).trunc()))
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
///
/// let xrp = drops_to_xrp("100000000");
/// ```
pub fn drops_to_xrp(drops: &str) -> Result<Decimal, XRPRangeException> {
    let drops_d: Decimal = Decimal::from_str(drops)?;
    let xrp = drops_d * ONE_DROP;

    if xrp.gt(&Decimal::new(MAX_XRP as i64, 0)) {
        Err(XRPRangeException {
            message: format!("Drops amount {} is too large.", drops),
        })
    } else {
        Ok(xrp)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_one_drop_decimal() {
        let test: Decimal = Decimal::from_str("0.000001").unwrap();
        assert_eq!(ONE_DROP, test);
    }

    #[test]
    fn test_xrp_to_drops() {
        let xrp: String = xrp_to_drops(Decimal::new(100000001, 6)).unwrap();
        let drops: String = (100 * XRP_DROPS + 1).to_string();

        assert_eq!(xrp, drops);
    }

    #[test]
    fn test_drops_to_xrp() {
        let drops: Decimal = drops_to_xrp("100000001").unwrap();
        let xrp: Decimal = Decimal::new(100000001, 6);

        assert_eq!(xrp, drops);
    }

    #[test]
    fn accept_one_xrp() {
        let xrp = Decimal::new(1, 0);
        assert_eq!("1000000".to_string(), xrp_to_drops(xrp).unwrap());
    }

    #[test]
    fn accept_zero_xrp() {
        let xrp = Decimal::new(0, 0);
        assert_eq!("0".to_string(), xrp_to_drops(xrp).unwrap());
    }

    #[test]
    fn accept_min_xrp() {
        let xrp = Decimal::from_str("0.000001").unwrap();
        assert_eq!("1".to_string(), xrp_to_drops(xrp).unwrap());
    }

    #[test]
    fn accept_max_xrp() {
        let xrp = Decimal::new(i64::pow(10, 11), 0);
        assert_eq!("100000000000000000".to_string(), xrp_to_drops(xrp).unwrap());
    }

    #[test]
    fn accept_too_small_xrp() {
        let xrp = Decimal::from_str("0.0000001").unwrap();
        assert!(xrp_to_drops(xrp).is_err());
    }

    #[test]
    fn accept_too_big_xrp() {
        let xrp = Decimal::new(i64::pow(10, 11) + 1, 0);
        assert!(xrp_to_drops(xrp).is_err());
    }

    #[test]
    fn accept_one_drop() {
        let xrp = Decimal::from_str("0.000001").unwrap();
        assert_eq!(xrp, drops_to_xrp("1").unwrap());
    }

    #[test]
    fn accept_zero_drops() {
        let xrp = Decimal::from_str("0").unwrap();
        assert_eq!(xrp, drops_to_xrp("0").unwrap());
    }

    #[test]
    fn accept_1mil_drops() {
        let xrp = Decimal::new(1, 0);
        assert_eq!(xrp, drops_to_xrp("1000000").unwrap());
    }

    #[test]
    fn accept_max_drops() {
        let xrp = Decimal::new(i64::pow(10, 11), 0);
        assert_eq!(xrp, drops_to_xrp(&u64::pow(10, 17).to_string()).unwrap());
    }

    #[test]
    fn accept_too_big_drops() {
        let drop = Decimal::new(i64::pow(10, 11) + 1, 0);
        assert!(xrp_to_drops(drop).is_err());
    }
}
