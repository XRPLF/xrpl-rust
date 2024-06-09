use core::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use alloc::{
    dbg, panic,
    string::{String, ToString},
    sync::Arc,
};
use anyhow::Result;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::mutex::Mutex;
use embedded_io_async::{ErrorType, Read, Write};
use embedded_websocket::{
    framer_async::{Framer, FramerError, ReadResult},
    Client, WebSocketClient, WebSocketOptions, WebSocketSendMessageType,
};
use futures_core::Stream;
use futures_sink::Sink;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{SingleExecutorMutex, WebsocketClosed, WebsocketOpen};
use crate::{
    asynch::clients::{
        client::Client as ClientTrait,
        websocket::websocket_base::{MessageHandler, WebsocketBase},
    },
    models::{requests::Request, results::XRPLResponse},
    Err,
};

use super::exceptions::XRPLWebsocketException;

pub struct AsyncWebsocketClient<
    const BUF: usize,
    Tcp,
    B,
    E,
    Rng: RngCore,
    M = SingleExecutorMutex,
    Status = WebsocketClosed,
> where
    M: RawMutex,
    B: Deref<Target = [u8]> + AsRef<[u8]>,
    Tcp: Stream<Item = Result<B, E>> + for<'a> Sink<&'a [u8], Error = E> + Unpin,
{
    tcp: Arc<Mutex<M, Tcp>>,
    websocket: Arc<Mutex<M, Framer<Rng, Client>>>,
    tx_buffer: [u8; BUF],
    websocket_base: Arc<Mutex<M, WebsocketBase<M>>>,
    status: PhantomData<Status>,
}

impl<const BUF: usize, M, Tcp, B, E, Rng: RngCore>
    AsyncWebsocketClient<BUF, Tcp, B, E, Rng, M, WebsocketClosed>
where
    M: RawMutex,
    B: Deref<Target = [u8]> + AsRef<[u8]>,
    E: Debug + Display,
    Tcp: Stream<Item = Result<B, E>> + for<'a> Sink<&'a [u8], Error = E> + Unpin,
{
    pub async fn open(
        rng: Rng,
        tcp: Tcp,
        url: Url,
    ) -> Result<AsyncWebsocketClient<BUF, Tcp, B, E, Rng, M, WebsocketOpen>> {
        // replace the scheme with http or https
        let scheme = match url.scheme() {
            "wss" => "https",
            "ws" => "http",
            _ => url.scheme(),
        };
        let port = match url.port() {
            Some(port) => port,
            None => match url.scheme() {
                "wss" => 443,
                "ws" => 80,
                _ => 80,
            },
        }
        .to_string();
        let path = url.path();
        let host = match url.host_str() {
            Some(host) => host,
            None => return Err!(XRPLWebsocketException::<E>::Disconnected),
        };
        let origin = scheme.to_string() + "://" + host + ":" + &port + path;
        let websocket_options = WebSocketOptions {
            path,
            host,
            origin: &origin,
            sub_protocols: None,
            additional_headers: None,
        };
        let websocket = Arc::new(Mutex::new(Framer::new(WebSocketClient::new_client(rng))));
        let tcp = Arc::new(Mutex::new(tcp));
        let mut buffer = [0; BUF];
        if let Err(error) = websocket
            .lock()
            .await
            .connect(
                tcp.lock().await.deref_mut(),
                &mut buffer,
                &websocket_options,
            )
            .await
        {
            match error {
                // FramerError::WebSocket(embedded_websocket::Error::HttpResponseCodeInvalid(
                //     Some(308),
                // )) => (),
                error => return Err!(XRPLWebsocketException::from(error)),
            }
        }

        Ok(AsyncWebsocketClient {
            tcp,
            websocket,
            tx_buffer: buffer,
            websocket_base: Arc::new(Mutex::new(WebsocketBase::new())),
            status: PhantomData::<WebsocketOpen>,
        })
    }
}

impl<const BUF: usize, M, Tcp, B, E, Rng: RngCore>
    AsyncWebsocketClient<BUF, Tcp, B, E, Rng, M, WebsocketOpen>
where
    M: RawMutex,
    B: Deref<Target = [u8]> + AsRef<[u8]>,
    E: Debug + Display,
    Tcp: Stream<Item = Result<B, E>> + for<'a> Sink<&'a [u8], Error = E> + Unpin,
{
    async fn do_write(&self, buf: &[u8]) -> Result<usize, <Self as ErrorType>::Error> {
        let mut inner = self.websocket.lock().await;
        let mut tcp = self.tcp.lock().await;
        let mut buffer = self.tx_buffer;
        match inner
            .write(
                tcp.deref_mut(),
                &mut buffer,
                WebSocketSendMessageType::Text,
                false,
                buf,
            )
            .await
        {
            Ok(()) => Ok(buf.len()),
            Err(error) => Err(XRPLWebsocketException::<E>::from(error)),
        }
    }
}

impl<const BUF: usize, M, Tcp, B, E, Rng: RngCore> ErrorType
    for AsyncWebsocketClient<BUF, Tcp, B, E, Rng, M, WebsocketOpen>
where
    M: RawMutex,
    B: Deref<Target = [u8]> + AsRef<[u8]>,
    E: Debug + Display,
    Tcp: Stream<Item = Result<B, E>> + for<'a> Sink<&'a [u8], Error = E> + Unpin,
{
    type Error = XRPLWebsocketException<E>;
}

impl<const BUF: usize, M, Tcp, B, E, Rng: RngCore> Write
    for AsyncWebsocketClient<BUF, Tcp, B, E, Rng, M, WebsocketOpen>
where
    M: RawMutex,
    B: Deref<Target = [u8]> + AsRef<[u8]>,
    E: Debug + Display,
    Tcp: Stream<Item = Result<B, E>> + for<'a> Sink<&'a [u8], Error = E> + Unpin,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.do_write(buf).await
    }
}

impl<const BUF: usize, M, Tcp, B, E, Rng: RngCore> Read
    for AsyncWebsocketClient<BUF, Tcp, B, E, Rng, M, WebsocketOpen>
where
    M: RawMutex,
    B: Deref<Target = [u8]> + AsRef<[u8]>,
    E: Debug + Display,
    Tcp: Stream<Item = Result<B, E>> + for<'a> Sink<&'a [u8], Error = E> + Unpin,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut inner = self.websocket.lock().await;
        let mut tcp = self.tcp.lock().await;
        match inner.read(tcp.deref_mut(), buf).await {
            Some(Ok(ReadResult::Text(t))) => Ok(t.len()),
            Some(Ok(ReadResult::Binary(b))) => Ok(b.len()),
            Some(Ok(ReadResult::Ping(_))) => Ok(0),
            Some(Ok(ReadResult::Pong(_))) => Ok(0),
            Some(Ok(ReadResult::Close(_))) => Err(XRPLWebsocketException::<E>::Disconnected),
            Some(Err(error)) => Err(XRPLWebsocketException::<E>::from(error)),
            None => Err(XRPLWebsocketException::<E>::Disconnected),
        }
    }
}

impl<const BUF: usize, M, Tcp, B, E, Rng: RngCore> MessageHandler
    for AsyncWebsocketClient<BUF, Tcp, B, E, Rng, M, WebsocketOpen>
where
    M: RawMutex,
    B: Deref<Target = [u8]> + AsRef<[u8]>,
    E: Debug + Display,
    Tcp: Stream<Item = Result<B, E>> + for<'a> Sink<&'a [u8], Error = E> + Unpin,
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

    async fn request_impl(&mut self, id: String) -> Result<String> {
        let mut websocket_base = self.websocket_base.lock().await;
        websocket_base.request_impl(id).await
    }
}

impl<const BUF: usize, M, Tcp, B, E, Rng: RngCore> ClientTrait
    for AsyncWebsocketClient<BUF, Tcp, B, E, Rng, M, WebsocketOpen>
where
    M: RawMutex,
    B: Deref<Target = [u8]> + AsRef<[u8]>,
    E: Debug + Display,
    Tcp: Stream<Item = Result<B, E>> + for<'a> Sink<&'a [u8], Error = E> + Unpin,
{
    async fn request_impl<
        Res: Serialize + for<'de> Deserialize<'de>,
        Req: Serialize + for<'de> Deserialize<'de> + for<'a> Request<'a>,
    >(
        &self,
        mut request: Req,
    ) -> Result<XRPLResponse<'_, Res, Req>> {
        let request_id = self.set_request_id::<Res, Req>(&mut request);
        let request_string = match serde_json::to_string(&request) {
            Ok(request_string) => request_string,
            Err(error) => return Err!(error),
        };
        if let Err(error) = self.do_write(request_string.as_bytes()).await {
            return Err!(error);
        }
        let mut websocket_base = self.websocket_base.lock().await;
        let message = websocket_base.request_impl(request_id.to_string()).await?;
        let response = match serde_json::from_str(&message) {
            Ok(response) => response,
            Err(error) => return Err!(error),
        };
        Ok(response)
    }
}
