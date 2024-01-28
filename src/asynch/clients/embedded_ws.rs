use super::{
    exceptions::XRPLWebsocketException,
    {WebsocketClosed, WebsocketOpen},
};
use crate::Err;
use anyhow::Result;
use core::marker::PhantomData;
use core::{fmt::Debug, ops::Deref};
pub use embedded_websocket::{
    framer_async::{
        Framer as EmbeddedWebsocketFramer, FramerError as EmbeddedWebsocketFramerError,
        ReadResult as EmbeddedWebsocketReadMessageType,
    },
    Client as EmbeddedWebsocketClient, Error as EmbeddedWebsocketError,
    WebSocket as EmbeddedWebsocket, WebSocketCloseStatusCode as EmbeddedWebsocketCloseStatusCode,
    WebSocketOptions as EmbeddedWebsocketOptions,
    WebSocketSendMessageType as EmbeddedWebsocketSendMessageType,
    WebSocketState as EmbeddedWebsocketState,
};
use futures::{Sink, Stream};
use rand_core::RngCore;

pub struct AsyncWebsocketClient<Rng: RngCore, Status = WebsocketClosed> {
    inner: EmbeddedWebsocketFramer<Rng, EmbeddedWebsocketClient>,
    status: PhantomData<Status>,
}

impl<Rng: RngCore, Status> AsyncWebsocketClient<Rng, Status> {
    pub fn is_open(&self) -> bool {
        core::any::type_name::<Status>() == core::any::type_name::<WebsocketOpen>()
    }
}

impl<Rng: RngCore> AsyncWebsocketClient<Rng, WebsocketClosed> {
    /// Open a websocket connection.
    pub async fn open<'a, B, E>(
        stream: &mut (impl Stream<Item = Result<B, E>> + Sink<&'a [u8], Error = E> + Unpin),
        buffer: &'a mut [u8],
        rng: Rng,
        websocket_options: &EmbeddedWebsocketOptions<'_>,
    ) -> Result<AsyncWebsocketClient<Rng, WebsocketOpen>>
    where
        B: AsRef<[u8]>,
        E: Debug,
    {
        let websocket = EmbeddedWebsocket::<Rng, EmbeddedWebsocketClient>::new_client(rng);
        let mut framer = EmbeddedWebsocketFramer::new(websocket);
        match framer.connect(stream, buffer, websocket_options).await {
            Ok(Some(_)) => {}
            Ok(None) => {}
            Err(error) => return Err!(XRPLWebsocketException::from(error)),
        }

        Ok(AsyncWebsocketClient {
            inner: framer,
            status: PhantomData::<WebsocketOpen>,
        })
    }
}

impl<Rng: RngCore> AsyncWebsocketClient<Rng, WebsocketOpen> {
    /// Encode a message to be sent over the websocket.
    pub fn encode<E>(
        &mut self,
        message_type: EmbeddedWebsocketSendMessageType,
        end_of_message: bool,
        from: &[u8],
        to: &mut [u8],
    ) -> Result<usize>
    where
        E: Debug,
    {
        match self
            .inner
            .encode::<E>(message_type, end_of_message, from, to)
        {
            Ok(bytes_written) => Ok(bytes_written),
            Err(error) => Err!(XRPLWebsocketException::from(error)),
        }
    }

    /// Send a message over the websocket.
    pub async fn send<'b, E, R: serde::Serialize>(
        &mut self,
        stream: &mut (impl Sink<&'b [u8], Error = E> + Unpin),
        stream_buf: &'b mut [u8],
        end_of_message: bool,
        frame_buf: R,
    ) -> Result<()>
    where
        E: Debug,
    {
        match serde_json::to_vec(&frame_buf) {
            Ok(frame_buf) => match self
                .inner
                .write(
                    stream,
                    stream_buf,
                    EmbeddedWebsocketSendMessageType::Text,
                    end_of_message,
                    frame_buf.as_slice(),
                )
                .await
            {
                Ok(()) => Ok(()),
                Err(error) => Err!(XRPLWebsocketException::from(error)),
            },
            Err(error) => Err!(error),
        }
    }

    /// Close the websocket connection.
    pub async fn close<'b, E>(
        &mut self,
        stream: &mut (impl Sink<&'b [u8], Error = E> + Unpin),
        stream_buf: &'b mut [u8],
        close_status: EmbeddedWebsocketCloseStatusCode,
        status_description: Option<&str>,
    ) -> Result<()>
    where
        E: Debug,
    {
        match self
            .inner
            .close(stream, stream_buf, close_status, status_description)
            .await
        {
            Ok(()) => Ok(()),
            Err(error) => Err!(XRPLWebsocketException::from(error)),
        }
    }

    /// Read a message from the websocket.
    pub async fn next<'a, B: Deref<Target = [u8]>, E>(
        &'a mut self,
        stream: &mut (impl Stream<Item = Result<B, E>> + Sink<&'a [u8], Error = E> + Unpin),
        buffer: &'a mut [u8],
    ) -> Option<Result<EmbeddedWebsocketReadMessageType<'_>>>
    // TODO: Change to Response as soon as implemented
    where
        E: Debug,
    {
        match self.inner.read(stream, buffer).await {
            Some(Ok(read_result)) => Some(Ok(read_result)),
            Some(Err(error)) => Some(Err!(XRPLWebsocketException::from(error))),
            None => None,
        }
    }

    /// Read a message from the websocket.
    ///
    /// This is similar to the `next` method, but returns a `Result<Option<EmbeddedWebsocketReadMessageType>>` rather than an `Option<Result<EmbeddedWebsocketReadMessageType>>`, making for easy use with the ? operator.
    pub async fn try_next<'a, B: Deref<Target = [u8]>, E>(
        &'a mut self,
        stream: &mut (impl Stream<Item = Result<B, E>> + Sink<&'a [u8], Error = E> + Unpin),
        buffer: &'a mut [u8],
    ) -> Result<Option<EmbeddedWebsocketReadMessageType<'_>>>
    // TODO: Change to Response as soon as implemented
    where
        E: Debug,
    {
        match self.inner.read(stream, buffer).await {
            Some(Ok(read_result)) => Ok(Some(read_result)),
            Some(Err(error)) => Err!(XRPLWebsocketException::from(error)),
            None => Ok(None),
        }
    }
}
