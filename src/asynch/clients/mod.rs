pub mod exceptions;

pub struct WebsocketOpen;
pub struct WebsocketClosed;

#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
mod embedded_websocket;
#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
mod tokio_tungstenite;

#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
pub use embedded_websocket::*;
#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
pub use tokio_tungstenite::*;
