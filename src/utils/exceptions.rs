//! Exception for invalid XRP Ledger amount data.

use alloc::string::String;

#[derive(Debug)]
#[non_exhaustive]
pub enum XRPRangeException {
    InvalidXRPAmountTooSmall { min: String, found: String },
    InvalidXRPAmountTooLarge { max: u64, found: u64 },
    InvalidDropsAmountTooLarge { max: String, found: String },
    DecimalError(rust_decimal::Error),
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ISOCodeException {
    InvalidISOCode,
    InvalidISOLength,
    InvalidXRPBytes,
    HexError(hex::FromHexError),
}

impl From<rust_decimal::Error> for XRPRangeException {
    fn from(err: rust_decimal::Error) -> Self {
        XRPRangeException::DecimalError(err)
    }
}

impl From<hex::FromHexError> for ISOCodeException {
    fn from(err: hex::FromHexError) -> Self {
        ISOCodeException::HexError(err)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for XRPRangeException {}

#[cfg(feature = "std")]
impl std::error::Error for ISOCodeException {}
