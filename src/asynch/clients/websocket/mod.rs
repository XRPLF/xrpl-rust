use crate::models::{requests::XRPLRequest, results::XRPLResponse};
#[cfg(feature = "std")]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
use embedded_io_async::Error;
#[cfg(not(feature = "std"))]
use embedded_io_async::{Read as EmbeddedIoRead, Write as EmbeddedIoWrite};
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

use super::exceptions::XRPLClientResult;

pub struct WebSocketOpen;
pub struct WebSocketClosed;

#[allow(async_fn_in_trait)]
pub trait XRPLAsyncWebsocketIO {
    async fn xrpl_send(&mut self, message: XRPLRequest<'_>) -> XRPLClientResult<()>;

    async fn xrpl_receive(&mut self) -> XRPLClientResult<Option<XRPLResponse<'_>>>;
}

#[cfg(not(feature = "std"))]
impl<T: EmbeddedIoRead + EmbeddedIoWrite + MessageHandler> XRPLAsyncWebsocketIO for T {
    async fn xrpl_send(&mut self, message: XRPLRequest<'_>) -> XRPLClientResult<()> {
        let message = serde_json::to_string(&message)?;
        let message_buffer = message.as_bytes();
        self.write(message_buffer)
            .await
            .map_err(|e| XRPLWebSocketException::EmbeddedIoError(e.kind()))?;

        Ok(())
    }

    async fn xrpl_receive(&mut self) -> XRPLClientResult<Option<XRPLResponse<'_>>> {
        let mut buffer = [0; 1024];
        loop {
            match self.read(&mut buffer).await {
                Ok(u_size) => {
                    // If the buffer is empty, continue to the next iteration.
                    if u_size == 0 {
                        continue;
                    }
                    let response_str = core::str::from_utf8(&buffer[..u_size])
                        .map_err(|e| XRPLWebSocketException::Utf8(e))?;
                    self.handle_message(response_str.to_string()).await?;
                    let message = self.pop_message().await;

                    return Ok(serde_json::from_str(&message)?);
                }
                Err(error) => {
                    return Err(XRPLWebSocketException::EmbeddedIoError(error.kind()).into())
                }
            }
        }
    }
}

#[cfg(feature = "std")]
impl<T: ?Sized> XRPLAsyncWebsocketIO for T
where
    T: Stream<Item = XRPLClientResult<String>>
        + Sink<String, Error = XRPLClientException>
        + MessageHandler
        + Unpin,
{
    async fn xrpl_send(&mut self, message: XRPLRequest<'_>) -> XRPLClientResult<()> {
        let message = serde_json::to_string(&message)?;

        self.send(message).await
    }

    async fn xrpl_receive(&mut self) -> XRPLClientResult<Option<XRPLResponse<'_>>> {
        match self.next().await {
            Some(Ok(item)) => {
                self.handle_message(item).await?;
                let message = self.pop_message().await;

                Ok(serde_json::from_str(&message)?)
            }
            Some(Err(error)) => Err(error),
            None => Ok(None),
        }
    }
}
