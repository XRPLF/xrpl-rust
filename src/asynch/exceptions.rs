use thiserror_no_std::Error;

#[cfg(any(feature = "json-rpc", feature = "websocket"))]
use super::clients::exceptions::XRPLClientException;
#[cfg(feature = "helpers")]
use super::{
    transaction::exceptions::{
        XRPLSignTransactionException, XRPLSubmitAndWaitException, XRPLTransactionHelperException,
    },
    wallet::exceptions::XRPLFaucetException,
};
#[cfg(feature = "helpers")]
use crate::{
    core::exceptions::XRPLCoreException,
    models::transactions::exceptions::XRPLTransactionFieldException,
    transaction::exceptions::XRPLMultisignException, utils::exceptions::XRPLUtilsException,
    wallet::exceptions::XRPLWalletException,
};
use crate::{models::XRPLModelException, XRPLSerdeJsonError};

pub type XRPLHelperResult<T, E = XRPLHelperException> = core::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum XRPLHelperException {
    #[cfg(feature = "helpers")]
    #[error("XRPL Wallet error: {0}")]
    XRPLWalletError(#[from] XRPLWalletException),
    #[cfg(feature = "helpers")]
    #[error("XRPL Faucet error: {0}")]
    XRPLFaucetError(#[from] XRPLFaucetException),
    #[cfg(feature = "helpers")]
    #[error("XRPL Transaction Helper error: {0}")]
    XRPLTransactionHelperError(#[from] XRPLTransactionHelperException),
    #[error("XRPL Model error: {0}")]
    XRPLModelError(#[from] XRPLModelException),
    #[cfg(feature = "helpers")]
    #[error("XRPL Core error: {0}")]
    XRPLCoreError(#[from] XRPLCoreException),
    #[cfg(feature = "helpers")]
    #[error("XRPL Transaction Field error: {0}")]
    XRPLTransactionFieldError(#[from] XRPLTransactionFieldException),
    #[cfg(feature = "helpers")]
    #[error("XRPL Utils error: {0}")]
    XRPLUtilsError(#[from] XRPLUtilsException),
    #[cfg(feature = "helpers")]
    #[error("XRPL MultiSign error: {0}")]
    XRPLMultiSignError(#[from] XRPLMultisignException),
    #[cfg(any(feature = "json-rpc", feature = "websocket"))]
    #[error("XRPL Client error: {0}")]
    XRPLClientError(#[from] XRPLClientException),
    #[error("serde_json error: {0}")]
    XRPLSerdeJsonError(#[from] XRPLSerdeJsonError),
    #[error("From hex error: {0}")]
    FromHexError(#[from] hex::FromHexError),
    #[cfg(feature = "std")]
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
}

impl From<serde_json::Error> for XRPLHelperException {
    fn from(error: serde_json::Error) -> Self {
        XRPLHelperException::XRPLSerdeJsonError(XRPLSerdeJsonError::SerdeJsonError(error))
    }
}

#[cfg(feature = "helpers")]
impl From<XRPLSignTransactionException> for XRPLHelperException {
    fn from(error: XRPLSignTransactionException) -> Self {
        XRPLHelperException::XRPLTransactionHelperError(
            XRPLTransactionHelperException::XRPLSignTransactionError(error),
        )
    }
}

#[cfg(feature = "helpers")]
impl From<XRPLSubmitAndWaitException> for XRPLHelperException {
    fn from(error: XRPLSubmitAndWaitException) -> Self {
        XRPLHelperException::XRPLTransactionHelperError(
            XRPLTransactionHelperException::XRPLSubmitAndWaitError(error),
        )
    }
}
