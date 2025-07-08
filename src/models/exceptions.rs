//! General XRPL Model Exception.

use core::num::{ParseFloatError, ParseIntError};

use alloc::string::String;
use thiserror_no_std::Error;

use crate::XRPLSerdeJsonError;

use super::{
    results::exceptions::XRPLResultException,
    transactions::exceptions::{
        XRPLAccountSetException, XRPLNFTokenCancelOfferException, XRPLNFTokenCreateOfferException,
        XRPLPaymentException, XRPLSignerListSetException, XRPLTransactionException,
        XRPLXChainClaimException, XRPLXChainCreateBridgeException,
        XRPLXChainCreateClaimIDException, XRPLXChainModifyBridgeException,
    },
};

pub type XRPLModelResult<T, E = XRPLModelException> = core::result::Result<T, E>;

#[derive(Debug, PartialEq, Error)]
pub enum XRPLModelException {
    // Model validation errors
    #[error("Expected one of: {}", .0.join(", "))]
    ExpectedOneOf(&'static [&'static str]),
    #[error("Invalid field combination: {field} with {other_fields:?}")]
    InvalidFieldCombination {
        field: &'static str,
        other_fields: &'static [&'static str],
    },
    #[error("The value of the field `{field:?}` is defined above its maximum (max {max:?}, found {found:?})")]
    ValueTooHigh { field: String, max: u32, found: u32 },
    #[error("The value of the field `{field:?}` is defined below its minimum (min {min:?}, found {found:?})")]
    ValueTooLow { field: String, min: u32, found: u32 },
    #[error("The value of the field `{field:?}` does not have the correct format (expected {format:?}, found {found:?})")]
    InvalidValueFormat {
        field: String,
        format: String,
        found: String,
    },
    #[error("The value of the field `{field:?}` exceeds its maximum length of characters (max {max:?}, found {found:?})")]
    ValueTooLong {
        field: String,
        max: usize,
        found: usize,
    },
    #[error("The value of the field `{field:?}` is below its minimum length of characters (min {min:?}, found {found:?})")]
    ValueTooShort {
        field: String,
        min: usize,
        found: usize,
    },
    #[error("The value of the field `{field1:?}` is not allowed to be below the value of the field `{field2:?}` (max {field2_val:?}, found {field1_val:?})")]
    ValueBelowValue {
        field1: String,
        field2: String,
        field1_val: u32,
        field2_val: u32,
    },
    #[error("The value of the field `{field1:?}` is not allowed to be the same as the value of the field `{field2:?}`")]
    ValueEqualsValue { field1: String, field2: String },
    #[error("The value of the field `{0:?}` is not allowed to be zero")]
    ValueZero(String),
    #[error("If the field `{field1:?}` is defined, the field `{field2:?}` must also be defined")]
    FieldRequiresField { field1: String, field2: String },
    #[error("The value of the field `{field:?}` is not a valid value (expected: {expected:?}, found: {found:?})")]
    InvalidValue {
        field: String,
        expected: String,
        found: String,
    },

    #[error("Expected field `{0}` is missing")]
    MissingField(String),

    #[error("From hex error: {0}")]
    FromHexError(#[from] hex::FromHexError),
    #[error("Parse int error: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("Parse float error: {0}")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] XRPLSerdeJsonError),
    #[error("BigDecimal error: {0}")]
    BigDecimalError(#[from] bigdecimal::ParseBigDecimalError),
    #[error("{0}")]
    XRPLResultError(#[from] XRPLResultException),
    #[error("{0}")]
    XRPLTransactionError(#[from] XRPLTransactionException),
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLModelException {}

impl From<serde_json::Error> for XRPLModelException {
    fn from(error: serde_json::Error) -> Self {
        XRPLModelException::SerdeJsonError(error.into())
    }
}

impl From<XRPLAccountSetException> for XRPLModelException {
    fn from(error: XRPLAccountSetException) -> Self {
        XRPLModelException::XRPLTransactionError(error.into())
    }
}

impl From<XRPLNFTokenCancelOfferException> for XRPLModelException {
    fn from(error: XRPLNFTokenCancelOfferException) -> Self {
        XRPLModelException::XRPLTransactionError(error.into())
    }
}

impl From<XRPLNFTokenCreateOfferException> for XRPLModelException {
    fn from(error: XRPLNFTokenCreateOfferException) -> Self {
        XRPLModelException::XRPLTransactionError(error.into())
    }
}

impl From<XRPLPaymentException> for XRPLModelException {
    fn from(error: XRPLPaymentException) -> Self {
        XRPLModelException::XRPLTransactionError(error.into())
    }
}

impl From<XRPLSignerListSetException> for XRPLModelException {
    fn from(error: XRPLSignerListSetException) -> Self {
        XRPLModelException::XRPLTransactionError(error.into())
    }
}

impl From<XRPLXChainClaimException> for XRPLModelException {
    fn from(error: XRPLXChainClaimException) -> Self {
        XRPLModelException::XRPLTransactionError(error.into())
    }
}

impl From<XRPLXChainCreateBridgeException> for XRPLModelException {
    fn from(error: XRPLXChainCreateBridgeException) -> Self {
        XRPLModelException::XRPLTransactionError(error.into())
    }
}

impl From<XRPLXChainCreateClaimIDException> for XRPLModelException {
    fn from(error: XRPLXChainCreateClaimIDException) -> Self {
        XRPLModelException::XRPLTransactionError(error.into())
    }
}

impl From<XRPLXChainModifyBridgeException> for XRPLModelException {
    fn from(error: XRPLXChainModifyBridgeException) -> Self {
        XRPLModelException::XRPLTransactionError(error.into())
    }
}
