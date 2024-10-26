use thiserror_no_std::Error;

use crate::{
    core::exceptions::XRPLCoreException,
    models::{transactions::exceptions::XRPLTransactionFieldException, XRPLModelException},
    transaction::exceptions::XRPLMultisignException,
    utils::exceptions::XRPLUtilsException,
    wallet::exceptions::XRPLWalletException,
    XRPLSerdeJsonError,
};

use super::{
    clients::exceptions::XRPLClientException,
    transaction::exceptions::{
        XRPLSignTransactionException, XRPLSubmitAndWaitException, XRPLTransactionHelperException,
    },
    wallet::exceptions::XRPLFaucetException,
};

pub type XRPLHelperResult<T, E = XRPLHelperException> = core::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum XRPLHelperException {
    #[error("XRPL Wallet error: {0}")]
    XRPLWalletError(#[from] XRPLWalletException),
    #[error("XRPL Faucet error: {0}")]
    XRPLFaucetError(#[from] XRPLFaucetException),
    #[error("XRPL Transaction Helper error: {0}")]
    XRPLTransactionHelperError(#[from] XRPLTransactionHelperException),
    #[error("XRPL Model error: {0}")]
    XRPLModelError(#[from] XRPLModelException),
    #[error("XRPL Core error: {0}")]
    XRPLCoreError(#[from] XRPLCoreException),
    #[error("XRPL Transaction Field error: {0}")]
    XRPLTransactionFieldError(#[from] XRPLTransactionFieldException),
    #[error("XRPL Utils error: {0}")]
    XRPLUtilsError(#[from] XRPLUtilsException),
    #[error("XRPL MultiSign error: {0}")]
    XRPLMultiSignError(#[from] XRPLMultisignException),
    #[error("XRPL Client error: {0}")]
    XRPLClientError(#[from] XRPLClientException),
    #[error("serde_json error: {0}")]
    XRPLSerdeJsonError(#[from] XRPLSerdeJsonError),
    #[error("From hex error: {0}")]
    FromHexError(#[from] hex::FromHexError),
}

impl From<serde_json::Error> for XRPLHelperException {
    fn from(error: serde_json::Error) -> Self {
        XRPLHelperException::XRPLSerdeJsonError(XRPLSerdeJsonError::SerdeJsonError(error))
    }
}

impl From<XRPLSignTransactionException> for XRPLHelperException {
    fn from(error: XRPLSignTransactionException) -> Self {
        XRPLHelperException::XRPLTransactionHelperError(
            XRPLTransactionHelperException::XRPLSignTransactionError(error),
        )
    }
}

impl From<XRPLSubmitAndWaitException> for XRPLHelperException {
    fn from(error: XRPLSubmitAndWaitException) -> Self {
        XRPLHelperException::XRPLTransactionHelperError(
            XRPLTransactionHelperException::XRPLSubmitAndWaitError(error),
        )
    }
}
