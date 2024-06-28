use super::exceptions::XRPLWebsocketException;
use super::{WebsocketClosed, WebsocketOpen};
use crate::asynch::clients::client::Client;
use crate::asynch::clients::websocket::websocket_base::{MessageHandler, WebsocketBase};
use crate::asynch::clients::SingleExecutorMutex;
use crate::models::requests::Request;
use crate::models::results::XRPLResponse;
use crate::Err;

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use anyhow::Result;
use core::marker::PhantomData;
use core::{pin::Pin, task::Poll};
use embassy_futures::block_on;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::mutex::Mutex;
use futures::{Sink, SinkExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

pub struct AsyncWebsocketClient<T, M = SingleExecutorMutex, Status = WebsocketClosed>
where
    M: RawMutex,
{
    websocket: Arc<Mutex<M, T>>,
    websocket_base: Arc<Mutex<M, WebsocketBase<M>>>,
    status: PhantomData<Status>,
}

impl<T, M> Sink<String> for AsyncWebsocketClient<T, M, WebsocketOpen>
where
    T: Sink<TungsteniteMessage, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
    M: RawMutex,
{
    type Error = anyhow::Error;

    fn poll_ready(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<()>> {
        let mut guard = block_on(self.websocket.lock());
        match Pin::new(&mut *guard).poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(error)) => Poll::Ready(Err!(error)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn start_send(self: core::pin::Pin<&mut Self>, item: String) -> Result<()> {
        let mut guard = block_on(self.websocket.lock());
        match Pin::new(&mut *guard).start_send(TungsteniteMessage::Text(item)) {
            Ok(()) => Ok(()),
            Err(error) => Err!(error),
        }
    }

    fn poll_flush(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<()>> {
        let mut guard = block_on(self.websocket.lock());
        match Pin::new(&mut *guard).poll_flush(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(error)) => Poll::Ready(Err!(error)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_close(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<()>> {
        let mut guard = block_on(self.websocket.lock());
        match Pin::new(&mut *guard).poll_close(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(error)) => Poll::Ready(Err!(error)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T, M> Stream for AsyncWebsocketClient<T, M, WebsocketOpen>
where
    T: Stream<Item = Result<TungsteniteMessage, tokio_tungstenite::tungstenite::Error>> + Unpin,
    M: RawMutex,
{
    type Item = Result<String>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut guard = block_on(self.websocket.lock());
        match Pin::new(&mut *guard).poll_next(cx) {
            Poll::Ready(Some(item)) => match item {
                Ok(message) => match message {
                    TungsteniteMessage::Text(response) => Poll::Ready(Some(Ok(response))),
                    TungsteniteMessage::Binary(response) => {
                        let response_string = match String::from_utf8(response) {
                            Ok(string) => string,
                            Err(error) => {
                                return Poll::Ready(Some(Err!(XRPLWebsocketException::<
                                    anyhow::Error,
                                >::Utf8(
                                    error.utf8_error()
                                ))));
                            }
                        };
                        Poll::Ready(Some(Ok(response_string)))
                    }
                    TungsteniteMessage::Close(_) => Poll::Ready(Some(Err!(
                        XRPLWebsocketException::<anyhow::Error>::Disconnected
                    ))),
                    _ => Poll::Ready(Some(Err!(
                        XRPLWebsocketException::<anyhow::Error>::UnexpectedMessageType
                    ))),
                },
                Err(error) => Poll::Ready(Some(Err!(error))),
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T, M> AsyncWebsocketClient<T, M, WebsocketClosed>
where
    M: RawMutex,
{
    pub async fn open(stream: T) -> Result<AsyncWebsocketClient<T, M, WebsocketOpen>> {
        Ok(AsyncWebsocketClient {
            websocket: Arc::new(Mutex::new(stream)),
            websocket_base: Arc::new(Mutex::new(WebsocketBase::new())),
            status: PhantomData::<WebsocketOpen>,
        })
    }
}

impl<T, M> MessageHandler for AsyncWebsocketClient<T, M, WebsocketOpen>
where
    M: RawMutex,
{
    async fn setup_request_future(&mut self, id: String) {
        let mut websocket_base = self.websocket_base.lock().await;
        websocket_base.setup_request_future(id).await;
    }

    async fn handle_message(&mut self, message: String) -> Result<()> {
        let mut websocket_base = self.websocket_base.lock().await;
        websocket_base.handle_message(message).await
    }

    async fn pop_message(&mut self) -> String {
        let mut websocket_base = self.websocket_base.lock().await;
        websocket_base.pop_message().await
    }

    async fn try_recv_request(&mut self, id: String) -> Result<Option<String>> {
        let mut websocket_base = self.websocket_base.lock().await;
        websocket_base.try_recv_request(id).await
    }
}

impl<T, M> Client for AsyncWebsocketClient<T, M, WebsocketOpen>
where
    T: Stream<Item = Result<TungsteniteMessage, tokio_tungstenite::tungstenite::Error>>
        + Sink<TungsteniteMessage, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin,
    M: RawMutex,
{
    async fn request_impl<
        'a: 'b,
        'b,
        Res: Serialize + for<'de> Deserialize<'de>,
        Req: Serialize + for<'de> Deserialize<'de> + Request<'a>,
    >(
        &self,
        mut request: Req,
    ) -> Result<XRPLResponse<'b, Res, Req>> {
        // setup request future
        let request_id = self.set_request_id::<Res, Req>(&mut request);
        let mut websocket_base = self.websocket_base.lock().await;
        websocket_base
            .setup_request_future(request_id.to_string())
            .await;
        // send request
        let mut websocket = self.websocket.lock().await;
        let request_string = match serde_json::to_string(&request) {
            Ok(request_string) => request_string,
            Err(error) => return Err!(error),
        };
        if let Err(error) = websocket
            .send(TungsteniteMessage::Text(request_string))
            .await
        {
            return Err!(error);
        }
        // wait for response
        loop {
            let message = websocket.next().await;
            match message {
                Some(Ok(TungsteniteMessage::Text(message))) => {
                    websocket_base.handle_message(message).await?;
                    let message_opt = websocket_base
                        .try_recv_request(request_id.to_string())
                        .await?;
                    if let Some(message) = message_opt {
                        let response = match serde_json::from_str(&message) {
                            Ok(response) => response,
                            Err(error) => return Err!(error),
                        };
                        return Ok(response);
                    }
                }
                Some(Ok(TungsteniteMessage::Binary(response))) => {
                    let message = match String::from_utf8(response) {
                        Ok(string) => string,
                        Err(error) => {
                            return Err!(XRPLWebsocketException::<anyhow::Error>::Utf8(
                                error.utf8_error()
                            ));
                        }
                    };
                    match serde_json::from_str(&message) {
                        Ok(response) => return Ok(response),
                        Err(error) => return Err!(error),
                    }
                }
                Some(Ok(TungsteniteMessage::Close(_))) => {
                    return Err!(XRPLWebsocketException::<anyhow::Error>::Disconnected)
                }
                Some(Ok(_)) => {
                    return Err!(XRPLWebsocketException::<anyhow::Error>::UnexpectedMessageType);
                }
                Some(Err(error)) => return Err!(error),
                None => continue,
            }
        }
    }
}
