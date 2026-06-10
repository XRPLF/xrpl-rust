use thiserror_no_std::Error;

#[cfg(any(feature = "json-rpc", feature = "websocket"))]
use super::clients::exceptions::XRPLClientException;
#[cfg(feature = "helpers")]
use super::{
    transaction::exceptions::{XRPLSubmitAndWaitException, XRPLTransactionHelperException},
    wallet::exceptions::XRPLFaucetException,
};
#[cfg(feature = "wallet")]
use crate::wallet::exceptions::XRPLWalletException;
// Available whenever `signing` is compiled (which requires core+models+wallet),
// or whenever `helpers` is on — the former is a subset of the latter.
#[cfg(all(feature = "core", feature = "models", feature = "wallet"))]
use crate::{
    core::exceptions::XRPLCoreException,
    models::transactions::exceptions::XRPLTransactionFieldException,
    signing::exceptions::{XRPLMultisignException, XRPLSignTransactionException},
    utils::exceptions::XRPLUtilsException,
};
use crate::{models::XRPLModelException, XRPLSerdeJsonError};

pub type XRPLHelperResult<T, E = XRPLHelperException> = core::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum XRPLHelperException {
    #[cfg(feature = "wallet")]
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
    #[cfg(all(feature = "core", feature = "models", feature = "wallet"))]
    #[error("XRPL Core error: {0}")]
    XRPLCoreError(#[from] XRPLCoreException),
    #[cfg(all(feature = "core", feature = "models", feature = "wallet"))]
    #[error("XRPL Transaction Field error: {0}")]
    XRPLTransactionFieldError(#[from] XRPLTransactionFieldException),
    #[cfg(all(feature = "core", feature = "models", feature = "wallet"))]
    #[error("XRPL Utils error: {0}")]
    XRPLUtilsError(#[from] XRPLUtilsException),
    #[cfg(all(feature = "core", feature = "models", feature = "wallet"))]
    #[error("XRPL MultiSign error: {0}")]
    XRPLMultiSignError(#[from] XRPLMultisignException),
    #[cfg(all(feature = "core", feature = "models", feature = "wallet"))]
    #[error("XRPL Sign Transaction error: {0}")]
    XRPLSignTransactionError(#[from] XRPLSignTransactionException),
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

// `XRPLSignTransactionException` is now a direct variant of
// `XRPLHelperException` (see `XRPLSignTransactionError`), so the `From` impl
// is derived automatically.

#[cfg(feature = "helpers")]
impl From<XRPLSubmitAndWaitException> for XRPLHelperException {
    fn from(error: XRPLSubmitAndWaitException) -> Self {
        XRPLHelperException::XRPLTransactionHelperError(
            XRPLTransactionHelperException::XRPLSubmitAndWaitError(error),
        )
    }
}
