use super::{
    exceptions::XRPLWebsocketException,
    {WebsocketClosed, WebsocketOpen},
};
use crate::Err;

use anyhow::Result;
use core::marker::PhantomData;
use core::{pin::Pin, task::Poll};
use futures::{Sink, Stream};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async as tungstenite_connect_async, MaybeTlsStream as TungsteniteMaybeTlsStream,
    WebSocketStream as TungsteniteWebsocketStream,
};
use url::Url;

pub use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

pub struct AsyncWebsocketClient<Status = WebsocketClosed> {
    inner: TungsteniteWebsocketStream<TungsteniteMaybeTlsStream<TcpStream>>,
    status: PhantomData<Status>,
}

impl<Status> AsyncWebsocketClient<Status> {
    pub fn is_open(&self) -> bool {
        core::any::type_name::<Status>() == core::any::type_name::<WebsocketOpen>()
    }
}

impl<I> Sink<I> for AsyncWebsocketClient<WebsocketOpen>
where
    I: serde::Serialize,
{
    type Error = anyhow::Error;

    fn poll_ready(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<()>> {
        match Pin::new(&mut self.inner).poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(error)) => Poll::Ready(Err!(error)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn start_send(mut self: core::pin::Pin<&mut Self>, item: I) -> Result<()> {
        match serde_json::to_string(&item) {
            Ok(json) => {
                match Pin::new(&mut self.inner).start_send(TungsteniteMessage::Text(json)) {
                    Ok(()) => Ok(()),
                    Err(error) => Err!(error),
                }
            }
            Err(error) => Err!(error),
        }
    }

    fn poll_flush(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<()>> {
        match Pin::new(&mut self.inner).poll_flush(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(error)) => Poll::Ready(Err!(error)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_close(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<()>> {
        match Pin::new(&mut self.inner).poll_close(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(error)) => Poll::Ready(Err!(error)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Stream for AsyncWebsocketClient<WebsocketOpen> {
    type Item = <TungsteniteWebsocketStream<TungsteniteMaybeTlsStream<TcpStream>> as Stream>::Item;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some(item)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWebsocketClient<WebsocketClosed> {
    pub async fn open(uri: Url) -> Result<AsyncWebsocketClient<WebsocketOpen>> {
        match tungstenite_connect_async(uri).await {
            Ok((websocket_stream, _)) => Ok(AsyncWebsocketClient {
                inner: websocket_stream,
                status: PhantomData::<WebsocketOpen>,
            }),
            Err(error) => {
                Err!(XRPLWebsocketException::UnableToConnect::<anyhow::Error>(
                    error
                ))
            }
        }
    }
}
