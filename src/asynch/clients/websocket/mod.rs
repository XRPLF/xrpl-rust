use core::fmt::{Debug, Display};

use crate::{models::results::XRPLResponse, Err};
#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
use alloc::string::String;
#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
use alloc::string::ToString;
use anyhow::Result;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
use embedded_io_async::{ErrorType, Read as EmbeddedIoRead, Write as EmbeddedIoWrite};
#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
use futures::{Sink, SinkExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};

mod websocket_base;
use websocket_base::MessageHandler;

#[cfg(all(feature = "embedded-ws", feature = "std", not(feature = "tungstenite")))]
pub mod codec;
#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
mod embedded_websocket;
pub mod exceptions;
#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
mod tungstenite;

#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
pub use embedded_websocket::AsyncWebsocketClient;
#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
pub use tungstenite::AsyncWebsocketClient;

pub struct WebsocketOpen;
pub struct WebsocketClosed;

pub type MultiExecutorMutex = CriticalSectionRawMutex;
pub type SingleExecutorMutex = NoopRawMutex;

#[allow(async_fn_in_trait)]
pub trait XRPLWebsocketIO {
    async fn xrpl_send<Req: Serialize>(&mut self, message: Req) -> Result<()>;
    async fn xrpl_receive<
        Res: Serialize + for<'de> Deserialize<'de> + Debug,
        Req: Serialize + for<'de> Deserialize<'de> + Debug,
    >(
        &mut self,
    ) -> Result<Option<XRPLResponse<'_, Res, Req>>>;
}

#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
impl<T: EmbeddedIoRead + EmbeddedIoWrite + MessageHandler> XRPLWebsocketIO for T
where
    <T as ErrorType>::Error: Display,
{
    async fn xrpl_send<Req: Serialize>(&mut self, message: Req) -> Result<()> {
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

    async fn xrpl_receive<
        Res: Serialize + for<'de> Deserialize<'de> + Debug,
        Req: Serialize + for<'de> Deserialize<'de> + Debug,
    >(
        &mut self,
    ) -> Result<XRPLResponse<'_, Res, Req>> {
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

#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
impl<T> XRPLWebsocketIO for T
where
    T: Stream<Item = Result<String>> + Sink<String> + MessageHandler + Unpin,
    <T as Sink<String>>::Error: Debug + Display,
{
    async fn xrpl_send<Req: Serialize>(&mut self, message: Req) -> Result<()> {
        let message = match serde_json::to_string(&message) {
            Ok(message) => message,
            Err(error) => return Err!(error),
        };
        match self.send(message).await {
            Ok(()) => Ok(()),
            Err(error) => Err!(error),
        }
    }

    async fn xrpl_receive<
        Res: Serialize + for<'de> Deserialize<'de> + Debug,
        Req: Serialize + for<'de> Deserialize<'de> + Debug,
    >(
        &mut self,
    ) -> Result<Option<XRPLResponse<'_, Res, Req>>> {
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
