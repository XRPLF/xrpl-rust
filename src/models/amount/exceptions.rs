use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum XRPLAmountException {
    #[error("Unable to convert amount `value` into `Decimal`.")]
    ToDecimalError(#[from] rust_decimal::Error),
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLAmountException {}
