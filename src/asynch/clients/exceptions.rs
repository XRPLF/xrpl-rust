use core::fmt::Debug;
use core::str::Utf8Error;
use embedded_websocket::framer_async::FramerError;
use thiserror_no_std::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum XRPLWebsocketException<E: Debug> {
    // FramerError
    #[error("I/O error: {0:?}")]
    Io(E),
    #[error("Frame too large (size: {0:?})")]
    FrameTooLarge(usize),
    #[error("Failed to interpret u8 to string (error: {0:?})")]
    Utf8(Utf8Error),
    #[error("Invalid HTTP header")]
    HttpHeader,
    #[error("Websocket error: {0:?}")]
    WebSocket(embedded_websocket::Error),
    #[error("Disconnected")]
    Disconnected,
    #[error("Read buffer is too small (size: {0:?})")]
    RxBufferTooSmall(usize),
}

impl<E: Debug> From<FramerError<E>> for XRPLWebsocketException<E> {
    fn from(value: FramerError<E>) -> Self {
        match value {
            FramerError::Io(e) => XRPLWebsocketException::Io(e),
            FramerError::FrameTooLarge(e) => XRPLWebsocketException::FrameTooLarge(e),
            FramerError::Utf8(e) => XRPLWebsocketException::Utf8(e),
            FramerError::HttpHeader(_) => XRPLWebsocketException::HttpHeader,
            FramerError::WebSocket(e) => XRPLWebsocketException::WebSocket(e),
            FramerError::Disconnected => XRPLWebsocketException::Disconnected,
            FramerError::RxBufferTooSmall(e) => XRPLWebsocketException::RxBufferTooSmall(e),
        }
    }
}

#[cfg(feature = "std")]
impl<E: Debug> alloc::error::Error for XRPLWebsocketException<E> {}
