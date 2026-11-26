use alloc::string::String;
#[cfg(feature = "std")]
use alloc::string::ToString;
use core::fmt::Debug;
use core::str::Utf8Error;
#[cfg(all(feature = "websocket", not(feature = "std")))]
use embedded_io_async::{Error as EmbeddedIoError, ErrorKind};
#[cfg(all(feature = "websocket", not(feature = "std")))]
use embedded_websocket_embedded_io::framer_async::FramerError;
use futures::channel::oneshot::Canceled;
use thiserror_no_std::Error;

use crate::asynch::clients::exceptions::XRPLNetworkErrorKind;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum XRPLWebSocketException {
    // FramerError
    #[error("I/O error: {0:?}")]
    Io(String),
    #[error("Frame too large (size: {0:?})")]
    FrameTooLarge(usize),
    #[error("Failed to interpret u8 to string (error: {0:?})")]
    Utf8(Utf8Error),
    #[error("Invalid HTTP header")]
    HttpHeader,
    #[cfg(all(feature = "websocket", not(feature = "std")))]
    #[error("Websocket error: {0:?}")]
    WebSocket(embedded_websocket_embedded_io::Error),
    #[error("Disconnected")]
    Disconnected,
    #[error("Connection closed by peer: {0}")]
    ConnectionClosed(String),
    #[error("Read buffer is too small (size: {0:?})")]
    RxBufferTooSmall(usize),
    #[error("Unexpected message type")]
    UnexpectedMessageType,
    #[cfg(all(feature = "websocket", not(feature = "std")))]
    #[error("Embedded I/O error: {0:?}")]
    EmbeddedIoError(ErrorKind),
    #[error("Missing request channel sender.")]
    MissingRequestSender,
    #[error("Missing request channel receiver.")]
    MissingRequestReceiver,
    #[error("Invalid message.")]
    InvalidMessage,
    #[error("Failed to send message through channel: {0:?}")]
    MessageChannelError(String),
    #[error("Failed to receive message through channel: {0:?}")]
    Canceled(#[from] Canceled),
    #[cfg(feature = "std")]
    #[error("WebSocket I/O error: {0:?}")]
    TungsteniteIo(alloc::io::Error),
    #[cfg(feature = "std")]
    #[error("WebSocket connection closed")]
    ConnectionClosed,
    #[cfg(feature = "std")]
    #[error("WebSocket connection already closed")]
    AlreadyClosed,
    #[cfg(feature = "std")]
    #[error("WebSocket protocol error: {0}")]
    Protocol(String),
    #[cfg(feature = "std")]
    #[error("WebSocket capacity error: {0}")]
    Capacity(String),
    #[cfg(feature = "std")]
    #[error("WebSocket TLS error: {0}")]
    Tls(String),
    #[cfg(feature = "std")]
    #[error("Tungstenite error: {0}")]
    Tungstenite(String),
}

#[cfg(all(feature = "websocket", not(feature = "std")))]
impl<E: Debug> From<FramerError<E>> for XRPLWebSocketException {
    fn from(value: FramerError<E>) -> Self {
        use alloc::format;

        match value {
            FramerError::Io(e) => XRPLWebSocketException::Io(format!("{:?}", e)),
            FramerError::FrameTooLarge(e) => XRPLWebSocketException::FrameTooLarge(e),
            FramerError::Utf8(e) => XRPLWebSocketException::Utf8(e),
            FramerError::HttpHeader(_) => XRPLWebSocketException::HttpHeader,
            FramerError::WebSocket(e) => XRPLWebSocketException::WebSocket(e),
            FramerError::Disconnected => XRPLWebSocketException::Disconnected,
            FramerError::RxBufferTooSmall(e) => XRPLWebSocketException::RxBufferTooSmall(e),
        }
    }
}

#[cfg(all(feature = "websocket", not(feature = "std")))]
impl EmbeddedIoError for XRPLWebSocketException {
    fn kind(&self) -> ErrorKind {
        match self {
            XRPLWebSocketException::EmbeddedIoError(e) => e.kind(),
            _ => ErrorKind::Other,
        }
    }
}

#[cfg(feature = "std")]
impl From<tokio_tungstenite::tungstenite::Error> for XRPLWebSocketException {
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
        use tokio_tungstenite::tungstenite::Error;

        match error {
            Error::ConnectionClosed => XRPLWebSocketException::ConnectionClosed,
            Error::AlreadyClosed => XRPLWebSocketException::AlreadyClosed,
            Error::Io(error) => XRPLWebSocketException::TungsteniteIo(error),
            Error::Tls(error) => XRPLWebSocketException::Tls(error.to_string()),
            Error::Capacity(error) => XRPLWebSocketException::Capacity(error.to_string()),
            Error::Protocol(error) => XRPLWebSocketException::Protocol(error.to_string()),
            error => XRPLWebSocketException::Tungstenite(error.to_string()),
        }
    }
}

impl XRPLWebSocketException {
    /// Return a typed network error category, if this websocket failure is transport-related.
    pub fn network_error_kind(&self) -> Option<XRPLNetworkErrorKind> {
        match self {
            XRPLWebSocketException::Disconnected => Some(XRPLNetworkErrorKind::ConnectionClosed),
            #[cfg(feature = "std")]
            XRPLWebSocketException::ConnectionClosed => {
                Some(XRPLNetworkErrorKind::ConnectionClosed)
            }
            #[cfg(feature = "std")]
            XRPLWebSocketException::AlreadyClosed => Some(XRPLNetworkErrorKind::AlreadyClosed),
            #[cfg(feature = "std")]
            XRPLWebSocketException::TungsteniteIo(error) => Some(match error.kind() {
                alloc::io::ErrorKind::ConnectionRefused => XRPLNetworkErrorKind::ConnectionRefused,
                alloc::io::ErrorKind::ConnectionReset => XRPLNetworkErrorKind::ConnectionReset,
                alloc::io::ErrorKind::TimedOut => XRPLNetworkErrorKind::TimedOut,
                _ => XRPLNetworkErrorKind::OtherIo,
            }),
            #[cfg(feature = "std")]
            XRPLWebSocketException::Tls(_) => Some(XRPLNetworkErrorKind::Tls),
            #[cfg(feature = "std")]
            XRPLWebSocketException::Protocol(_) => Some(XRPLNetworkErrorKind::Protocol),
            XRPLWebSocketException::InvalidMessage
            | XRPLWebSocketException::UnexpectedMessageType => {
                Some(XRPLNetworkErrorKind::InvalidResponse)
            }
            _ => None,
        }
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLWebSocketException {}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn maps_tungstenite_io_error_kind() {
        let error = tokio_tungstenite::tungstenite::Error::Io(alloc::io::Error::from(
            alloc::io::ErrorKind::ConnectionRefused,
        ));
        let error = XRPLWebSocketException::from(error);
        assert_eq!(
            error.network_error_kind(),
            Some(XRPLNetworkErrorKind::ConnectionRefused)
        );
    }

    #[test]
    fn maps_std_websocket_network_error_kinds() {
        let cases = [
            (
                XRPLWebSocketException::Disconnected,
                Some(XRPLNetworkErrorKind::ConnectionClosed),
            ),
            (
                XRPLWebSocketException::ConnectionClosed,
                Some(XRPLNetworkErrorKind::ConnectionClosed),
            ),
            (
                XRPLWebSocketException::AlreadyClosed,
                Some(XRPLNetworkErrorKind::AlreadyClosed),
            ),
            (
                XRPLWebSocketException::TungsteniteIo(alloc::io::Error::from(
                    alloc::io::ErrorKind::ConnectionReset,
                )),
                Some(XRPLNetworkErrorKind::ConnectionReset),
            ),
            (
                XRPLWebSocketException::TungsteniteIo(alloc::io::Error::from(
                    alloc::io::ErrorKind::TimedOut,
                )),
                Some(XRPLNetworkErrorKind::TimedOut),
            ),
            (
                XRPLWebSocketException::TungsteniteIo(alloc::io::Error::from(
                    alloc::io::ErrorKind::Other,
                )),
                Some(XRPLNetworkErrorKind::OtherIo),
            ),
            (
                XRPLWebSocketException::Tls("tls".into()),
                Some(XRPLNetworkErrorKind::Tls),
            ),
            (
                XRPLWebSocketException::Protocol("protocol".into()),
                Some(XRPLNetworkErrorKind::Protocol),
            ),
            (
                XRPLWebSocketException::InvalidMessage,
                Some(XRPLNetworkErrorKind::InvalidResponse),
            ),
            (
                XRPLWebSocketException::UnexpectedMessageType,
                Some(XRPLNetworkErrorKind::InvalidResponse),
            ),
            (XRPLWebSocketException::Capacity("capacity".into()), None),
        ];

        for (error, expected) in cases {
            assert_eq!(error.network_error_kind(), expected);
        }
    }
}
