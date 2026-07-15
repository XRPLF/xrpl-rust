#[cfg(all(not(feature = "std"), feature = "websocket"))]
use alloc::boxed::Box;
use thiserror_no_std::Error;

#[cfg(feature = "helpers")]
use crate::asynch::wallet::exceptions::XRPLFaucetException;
use crate::{models::XRPLModelException, XRPLSerdeJsonError};

#[cfg(feature = "json-rpc")]
use super::XRPLJsonRpcException;
#[cfg(feature = "websocket")]
use super::XRPLWebSocketException;

pub type XRPLClientResult<T, E = XRPLClientException> = core::result::Result<T, E>;

/// Granular network/transport error categories for client failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum XRPLNetworkErrorKind {
    ConnectionClosed,
    AlreadyClosed,
    ConnectionRefused,
    ConnectionReset,
    TimedOut,
    Dns,
    Tls,
    Protocol,
    InvalidResponse,
    OtherIo,
    Other,
}

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
    XRPLWebSocketError(Box<XRPLWebSocketException>),
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

#[cfg(feature = "websocket")]
impl From<XRPLWebSocketException> for XRPLClientException {
    fn from(error: XRPLWebSocketException) -> Self {
        XRPLClientException::XRPLWebSocketError(Box::new(error))
    }
}

#[cfg(all(feature = "std", feature = "websocket"))]
impl From<tokio_tungstenite::tungstenite::Error> for XRPLClientException {
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
        XRPLClientException::XRPLWebSocketError(Box::new(XRPLWebSocketException::from(error)))
    }
}

#[cfg(all(feature = "std", feature = "json-rpc"))]
impl From<reqwest::Error> for XRPLClientException {
    fn from(error: reqwest::Error) -> Self {
        XRPLClientException::XRPLJsonRpcError(XRPLJsonRpcException::ReqwestError(error))
    }
}

impl XRPLClientException {
    /// Return a typed network error category, if this is a transport failure.
    pub fn network_error_kind(&self) -> Option<XRPLNetworkErrorKind> {
        match self {
            #[cfg(feature = "websocket")]
            XRPLClientException::XRPLWebSocketError(error) => error.network_error_kind(),
            #[cfg(feature = "json-rpc")]
            XRPLClientException::XRPLJsonRpcError(error) => error.network_error_kind(),
            #[cfg(feature = "std")]
            XRPLClientException::IoError(error) => Some(match error.kind() {
                alloc::io::ErrorKind::ConnectionRefused => XRPLNetworkErrorKind::ConnectionRefused,
                alloc::io::ErrorKind::ConnectionReset => XRPLNetworkErrorKind::ConnectionReset,
                alloc::io::ErrorKind::TimedOut => XRPLNetworkErrorKind::TimedOut,
                _ => XRPLNetworkErrorKind::OtherIo,
            }),
            _ => None,
        }
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLClientException {}

#[cfg(all(test, feature = "std", feature = "websocket", feature = "json-rpc"))]
mod tests {
    use alloc::boxed::Box;

    use super::*;
    use crate::asynch::clients::{XRPLJsonRpcException, XRPLWebSocketException};

    #[test]
    fn maps_client_network_error_kinds() {
        let cases = [
            (
                XRPLClientException::XRPLWebSocketError(Box::new(
                    XRPLWebSocketException::ConnectionClosed,
                )),
                Some(XRPLNetworkErrorKind::ConnectionClosed),
            ),
            (
                XRPLClientException::XRPLJsonRpcError(XRPLJsonRpcException::RequestError(
                    "not transport".into(),
                )),
                None,
            ),
            (
                XRPLClientException::IoError(alloc::io::Error::from(
                    alloc::io::ErrorKind::ConnectionRefused,
                )),
                Some(XRPLNetworkErrorKind::ConnectionRefused),
            ),
            (
                XRPLClientException::IoError(alloc::io::Error::from(
                    alloc::io::ErrorKind::ConnectionReset,
                )),
                Some(XRPLNetworkErrorKind::ConnectionReset),
            ),
            (
                XRPLClientException::IoError(alloc::io::Error::from(
                    alloc::io::ErrorKind::TimedOut,
                )),
                Some(XRPLNetworkErrorKind::TimedOut),
            ),
            (
                XRPLClientException::IoError(alloc::io::Error::from(alloc::io::ErrorKind::Other)),
                Some(XRPLNetworkErrorKind::OtherIo),
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(error.network_error_kind(), expected);
        }
    }
}
