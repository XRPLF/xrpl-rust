use thiserror_no_std::Error;

use crate::core::exceptions::XRPLCoreException;

pub type XRPLWalletResult<T, E = XRPLWalletException> = core::result::Result<T, E>;

#[derive(Debug, PartialEq, Error)]
pub enum XRPLWalletException {
    #[error("XRPL Core error: {0}")]
    XRPLCoreError(#[from] XRPLCoreException),
}
