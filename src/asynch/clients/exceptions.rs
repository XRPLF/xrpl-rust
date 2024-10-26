use thiserror_no_std::Error;

use crate::{
    asynch::wallet::exceptions::XRPLFaucetException, models::XRPLModelException, XRPLSerdeJsonError,
};

use super::{XRPLJsonRpcException, XRPLWebSocketException};

pub type XRPLClientResult<T, E = XRPLClientException> = core::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum XRPLClientException {
    #[error("serde_json error: {0}")]
    XRPLSerdeJsonError(#[from] XRPLSerdeJsonError),
    #[error("XRPL Model error: {0}")]
    XRPLModelError(#[from] XRPLModelException),
    #[error("XRPL Faucet error: {0}")]
    XRPLFaucetError(#[from] XRPLFaucetException),
    #[error("XRPL WebSocket error: {0}")]
    XRPLWebSocketError(#[from] XRPLWebSocketException),
    #[error("XRPL JSON-RPC error: {0}")]
    XRPLJsonRpcError(#[from] XRPLJsonRpcException),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("I/O error: {0}")]
    IoError(#[from] alloc::io::Error),
}

impl From<serde_json::Error> for XRPLClientException {
    fn from(error: serde_json::Error) -> Self {
        XRPLClientException::XRPLSerdeJsonError(XRPLSerdeJsonError::from(error))
    }
}

#[cfg(feature = "std")]
impl From<tokio_tungstenite::tungstenite::Error> for XRPLClientException {
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
        XRPLClientException::XRPLWebSocketError(XRPLWebSocketException::from(error))
    }
}

#[cfg(feature = "std")]
impl From<reqwest::Error> for XRPLClientException {
    fn from(error: reqwest::Error) -> Self {
        XRPLClientException::XRPLJsonRpcError(XRPLJsonRpcException::ReqwestError(error))
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLClientException {}
