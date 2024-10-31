use thiserror_no_std::Error;

#[cfg(feature = "helpers")]
use crate::asynch::wallet::exceptions::XRPLFaucetException;
use crate::{models::XRPLModelException, XRPLSerdeJsonError};

#[cfg(feature = "json-rpc")]
use super::XRPLJsonRpcException;
#[cfg(feature = "websocket")]
use super::XRPLWebSocketException;

pub type XRPLClientResult<T, E = XRPLClientException> = core::result::Result<T, E>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum XRPLClientException {
    #[error("serde_json error: {0}")]
    XRPLSerdeJsonError(#[from] XRPLSerdeJsonError),
    #[error("XRPL Model error: {0}")]
    XRPLModelError(#[from] XRPLModelException),
    #[cfg(feature = "helpers")]
    #[error("XRPL Faucet error: {0}")]
    XRPLFaucetError(#[from] XRPLFaucetException),
    #[cfg(feature = "websocket")]
    #[error("XRPL WebSocket error: {0}")]
    XRPLWebSocketError(#[from] XRPLWebSocketException),
    #[cfg(feature = "json-rpc")]
    #[error("XRPL JSON-RPC error: {0}")]
    XRPLJsonRpcError(#[from] XRPLJsonRpcException),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[cfg(feature = "std")]
    #[error("I/O error: {0}")]
    IoError(#[from] alloc::io::Error),
}

impl From<serde_json::Error> for XRPLClientException {
    fn from(error: serde_json::Error) -> Self {
        XRPLClientException::XRPLSerdeJsonError(XRPLSerdeJsonError::from(error))
    }
}

#[cfg(all(not(feature = "std"), feature = "json-rpc"))]
impl From<reqwless::Error> for XRPLClientException {
    fn from(error: reqwless::Error) -> Self {
        XRPLClientException::XRPLJsonRpcError(XRPLJsonRpcException::ReqwlessError(error))
    }
}

#[cfg(all(feature = "std", feature = "websocket"))]
impl From<tokio_tungstenite::tungstenite::Error> for XRPLClientException {
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
        XRPLClientException::XRPLWebSocketError(XRPLWebSocketException::from(error))
    }
}

#[cfg(all(feature = "std", feature = "json-rpc"))]
impl From<reqwest::Error> for XRPLClientException {
    fn from(error: reqwest::Error) -> Self {
        XRPLClientException::XRPLJsonRpcError(XRPLJsonRpcException::ReqwestError(error))
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLClientException {}
