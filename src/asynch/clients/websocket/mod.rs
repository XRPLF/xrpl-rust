use crate::{
    models::{requests::XRPLRequest, results::XRPLResponse},
    Err,
};
#[cfg(feature = "std")]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
use anyhow::Result;
#[cfg(not(feature = "std"))]
use core::fmt::Display;
#[cfg(not(feature = "std"))]
use embedded_io_async::{ErrorType, Read as EmbeddedIoRead, Write as EmbeddedIoWrite};
#[cfg(feature = "std")]
use futures::{Sink, SinkExt, Stream, StreamExt};

mod websocket_base;
use websocket_base::MessageHandler;

#[cfg(all(feature = "websocket", not(feature = "std")))]
mod _no_std;
mod exceptions;
pub use exceptions::*;
#[cfg(all(feature = "websocket", feature = "std"))]
mod _std;

#[cfg(all(feature = "websocket", not(feature = "std")))]
pub use _no_std::*;
#[cfg(all(feature = "websocket", feature = "std"))]
pub use _std::*;

pub struct WebSocketOpen;
pub struct WebSocketClosed;

#[allow(async_fn_in_trait)]
pub trait XRPLWebsocketIO {
    async fn xrpl_send(&mut self, message: XRPLRequest<'_>) -> Result<()>;

    async fn xrpl_receive(&mut self) -> Result<Option<XRPLResponse<'_>>>;
}

#[cfg(not(feature = "std"))]
impl<T: EmbeddedIoRead + EmbeddedIoWrite + MessageHandler> XRPLWebsocketIO for T
where
    <T as ErrorType>::Error: Display,
{
    async fn xrpl_send(&mut self, message: XRPLRequest<'_>) -> Result<()> {
        let message = match serde_json::to_string(&message) {
            Ok(message) => message,
            Err(error) => return Err!(error),
        };
        let message_buffer = message.as_bytes();
        match self.write(message_buffer).await {
            Ok(_) => Ok(()),
            Err(e) => Err!(e),
        }
    }

    async fn xrpl_receive(&mut self) -> Result<Option<XRPLResponse<'_>>> {
        let mut buffer = [0; 1024];
        loop {
            match self.read(&mut buffer).await {
                Ok(u_size) => {
                    // If the buffer is empty, continue to the next iteration.
                    if u_size == 0 {
                        continue;
                    }
                    let response_str = match core::str::from_utf8(&buffer[..u_size]) {
                        Ok(response_str) => response_str,
                        Err(error) => {
                            return Err!(XRPLWebsocketException::<anyhow::Error>::Utf8(error))
                        }
                    };
                    self.handle_message(response_str.to_string()).await?;
                    let message = self.pop_message().await;
                    match serde_json::from_str(&message) {
                        Ok(response) => return Ok(response),
                        Err(error) => return Err!(error),
                    }
                }
                Err(error) => return Err!(error),
            }
        }
    }
}

#[cfg(feature = "std")]
impl<T: ?Sized> XRPLWebsocketIO for T
where
    T: Stream<Item = Result<String>> + Sink<String, Error = anyhow::Error> + MessageHandler + Unpin,
{
    async fn xrpl_send(&mut self, message: XRPLRequest<'_>) -> Result<()> {
        let message = match serde_json::to_string(&message) {
            Ok(message) => message,
            Err(error) => return Err!(error),
        };
        match self.send(message).await {
            Ok(()) => Ok(()),
            Err(error) => Err!(error),
        }
    }

    async fn xrpl_receive(&mut self) -> Result<Option<XRPLResponse<'_>>> {
        match self.next().await {
            Some(Ok(item)) => {
                self.handle_message(item).await?;
                let message = self.pop_message().await;
                match serde_json::from_str(&message) {
                    Ok(response) => Ok(response),
                    Err(error) => Err!(error),
                }
            }
            Some(Err(error)) => Err!(error),
            None => Ok(None),
        }
    }
}
