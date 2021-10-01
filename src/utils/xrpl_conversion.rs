//! Conversions between XRP drops and native number types.

use rust_decimal::Decimal;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// Indivisible unit of XRP
pub const ONE_DROP: Decimal = Decimal::from_parts(1, 0, 0, false, 6);
/// 100 billion decimal XRP
pub const MAX_XRP: i64 = i64::pow(10, 11);
/// Maximum possible drops of XRP
pub const MAX_DROPS: i64 = i64::pow(10, 17);
/// Drops in one XRP
pub const XRP_DROPS: i64 = 1000000;

#[derive(Debug)]
/// Exception for invalid XRP Ledger time data.
pub struct XRPRangeException {
    message: String,
}

impl Display for XRPRangeException {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
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
/// use xrpl_rust::utils::xrpl_conversion::xrp_to_drops;
///
/// let drops = xrp_to_drops(Decimal::new(100000000, 6));
/// ```
pub fn xrp_to_drops(xrp: Decimal) -> Result<String, XRPRangeException> {
    if xrp < ONE_DROP && xrp != Decimal::ZERO {
        Err(XRPRangeException {
            message: format!("XRP amount {} is too small.", xrp),
        })
    } else if xrp.gt(&Decimal::new(MAX_XRP, 0)) {
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
/// use xrpl_rust::utils::xrpl_conversion::drops_to_xrp;
///
/// let xrp = drops_to_xrp("100000000");
/// ```
pub fn drops_to_xrp(drops: &str) -> Result<Decimal, XRPRangeException> {
    let drops_d: Decimal = Decimal::from_str(drops)?;
    let xrp = drops_d * ONE_DROP;

    if xrp.gt(&Decimal::new(MAX_XRP, 0)) {
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
    use std::str::FromStr;

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
}
