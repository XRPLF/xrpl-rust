use crate::{
    asynch::clients::{exceptions::XRPLClientResult, CommonFields, XRPLClient},
    models::{requests::XRPLRequest, results::XRPLResponse},
};

pub use crate::asynch::clients::{SingleExecutorMutex, XRPLFaucet};

pub trait XRPLSyncClient: XRPLClient {
    fn request<'a: 'b, 'b>(&self, request: XRPLRequest<'a>) -> XRPLClientResult<XRPLResponse<'b>>;

    fn get_common_fields(&self) -> XRPLClientResult<CommonFields<'_>>;
}

#[cfg(all(feature = "json-rpc", feature = "std"))]
pub mod json_rpc {
    use tokio::runtime::Runtime;
    use url::Url;

    use crate::{
        asynch::clients::{
            exceptions::XRPLClientResult, AsyncJsonRpcClient, CommonFields, XRPLAsyncClient,
            XRPLClient, XRPLFaucet,
        },
        models::{
            requests::{FundFaucet, XRPLRequest},
            results::XRPLResponse,
        },
    };

    use super::XRPLSyncClient;

    pub struct JsonRpcClient(AsyncJsonRpcClient);
    impl JsonRpcClient {
        pub fn connect(url: Url) -> Self {
            Self(AsyncJsonRpcClient::connect(url))
        }
    }

    impl XRPLClient for JsonRpcClient {
        async fn request_impl<'a: 'b, 'b>(
            &self,
            request: XRPLRequest<'a>,
        ) -> XRPLClientResult<XRPLResponse<'b>> {
            self.0.request_impl(request).await
        }

        fn get_host(&self) -> Url {
            self.0.get_host()
        }

        fn get_random_id<'a>(&self) -> alloc::borrow::Cow<'a, str> {
            self.0.get_random_id()
        }
    }

    impl XRPLSyncClient for JsonRpcClient {
        fn request<'a: 'b, 'b>(
            &self,
            request: XRPLRequest<'a>,
        ) -> XRPLClientResult<XRPLResponse<'b>> {
            match Runtime::new() {
                Ok(rt) => rt.block_on(self.0.request_impl(request)),
                Err(e) => Err(e.into()),
            }
        }

        fn get_common_fields(&self) -> XRPLClientResult<CommonFields<'_>> {
            match Runtime::new() {
                Ok(rt) => rt.block_on(self.0.get_common_fields()),
                Err(e) => Err(e.into()),
            }
        }
    }

    impl XRPLFaucet for JsonRpcClient {
        async fn request_funding(
            &self,
            url: Option<Url>,
            request: FundFaucet<'_>,
        ) -> XRPLClientResult<()> {
            self.0.request_funding(url, request).await
        }
    }
}

#[cfg(all(feature = "json-rpc", not(feature = "std")))]
pub mod json_rpc {
    use embassy_sync::blocking_mutex::raw::RawMutex;
    use embedded_nal_async::{Dns, TcpConnect};
    use url::Url;

    use crate::{
        asynch::clients::{
            exceptions::XRPLClientResult, AsyncJsonRpcClient, XRPLClient, XRPLFaucet,
        },
        models::{
            requests::{FundFaucet, XRPLRequest},
            results::XRPLResponse,
        },
    };

    pub struct JsonRpcClient<'a, const BUF: usize, T, D, M>(
        pub(crate) AsyncJsonRpcClient<'a, BUF, T, D, M>,
    )
    where
        M: RawMutex,
        T: TcpConnect + 'a,
        D: Dns + 'a;

    impl<'a, const BUF: usize, T, D, M> JsonRpcClient<'a, BUF, T, D, M>
    where
        M: RawMutex,
        T: TcpConnect + 'a,
        D: Dns + 'a,
    {
        pub fn connect(url: Url, tcp: &'a T, dns: &'a D) -> Self {
            Self(AsyncJsonRpcClient::connect(url, tcp, dns))
        }
    }

    impl<const BUF: usize, T, D, M> XRPLClient for JsonRpcClient<'_, BUF, T, D, M>
    where
        M: RawMutex,
        T: TcpConnect,
        D: Dns,
    {
        async fn request_impl<'a: 'b, 'b>(
            &self,
            request: XRPLRequest<'a>,
        ) -> XRPLClientResult<XRPLResponse<'b>> {
            self.0.request_impl(request).await
        }

        fn get_host(&self) -> Url {
            self.0.get_host()
        }
    }

    impl<'a, const BUF: usize, T, D, M> XRPLFaucet for JsonRpcClient<'a, BUF, T, D, M>
    where
        M: RawMutex,
        T: TcpConnect + 'a,
        D: Dns + 'a,
    {
        async fn request_funding(
            &self,
            url: Option<Url>,
            request: FundFaucet<'_>,
        ) -> XRPLClientResult<()> {
            self.0.request_funding(url, request).await
        }
    }
}

pub trait XRPLSyncWebsocketIO {
    fn xrpl_send(&mut self, message: XRPLRequest<'_>) -> XRPLClientResult<()>;

    fn xrpl_receive(&mut self) -> XRPLClientResult<Option<XRPLResponse<'_>>>;
}

#[cfg(all(feature = "websocket", feature = "std"))]
pub mod websocket {
    use embassy_sync::blocking_mutex::raw::RawMutex;
    use tokio::runtime::Runtime;
    use url::Url;

    use super::{XRPLSyncClient, XRPLSyncWebsocketIO};
    use crate::{
        asynch::clients::{
            exceptions::XRPLClientResult, AsyncWebSocketClient, CommonFields, XRPLAsyncClient,
            XRPLAsyncWebsocketIO, XRPLClient,
        },
        models::{requests::XRPLRequest, results::XRPLResponse},
    };

    pub use crate::asynch::clients::{WebSocketClosed, WebSocketOpen};

    pub struct WebSocketClient<M: RawMutex, Status = WebSocketClosed> {
        pub(crate) inner: AsyncWebSocketClient<M, Status>,
        rt: Runtime,
    }

    impl<M: RawMutex> WebSocketClient<M, WebSocketClosed> {
        pub fn open(url: Url) -> XRPLClientResult<WebSocketClient<M, WebSocketOpen>> {
            match Runtime::new() {
                Ok(rt) => {
                    let client: AsyncWebSocketClient<M, WebSocketOpen> =
                        rt.block_on(AsyncWebSocketClient::open(url))?;

                    Ok(WebSocketClient { inner: client, rt })
                }
                Err(e) => Err(e.into()),
            }
        }
    }

    impl<M> XRPLClient for WebSocketClient<M, WebSocketOpen>
    where
        M: RawMutex,
    {
        fn get_host(&self) -> Url {
            self.inner.get_host()
        }

        async fn request_impl<'a: 'b, 'b>(
            &self,
            request: XRPLRequest<'a>,
        ) -> XRPLClientResult<XRPLResponse<'b>> {
            match Runtime::new() {
                Ok(rt) => rt.block_on(self.inner.request_impl(request)),
                Err(e) => Err(e.into()),
            }
        }
    }

    impl<M> XRPLSyncClient for WebSocketClient<M, WebSocketOpen>
    where
        M: RawMutex,
    {
        fn request<'a: 'b, 'b>(
            &self,
            request: XRPLRequest<'a>,
        ) -> XRPLClientResult<XRPLResponse<'b>> {
            self.rt.block_on(self.inner.request_impl(request))
        }

        fn get_common_fields(&self) -> XRPLClientResult<CommonFields<'_>> {
            self.rt.block_on(self.inner.get_common_fields())
        }
    }

    impl<M> XRPLSyncWebsocketIO for WebSocketClient<M, WebSocketOpen>
    where
        M: RawMutex,
    {
        fn xrpl_send(&mut self, message: XRPLRequest<'_>) -> XRPLClientResult<()> {
            self.rt.block_on(self.inner.xrpl_send(message))
        }

        fn xrpl_receive(&mut self) -> XRPLClientResult<Option<XRPLResponse<'_>>> {
            self.rt.block_on(self.inner.xrpl_receive())
        }
    }
}

#[cfg(all(feature = "websocket", not(feature = "std")))]
pub mod websocket {
    use super::XRPLSyncWebsocketIO;
    use embassy_futures::block_on;
    use embassy_sync::blocking_mutex::raw::RawMutex;
    use embedded_io_async::{Read, Write};
    use rand::RngCore;
    use url::Url;

    use crate::{
        asynch::clients::{
            exceptions::XRPLClientResult, AsyncWebSocketClient, WebSocketOpen,
            XRPLAsyncWebsocketIO, XRPLClient,
        },
        models::{requests::XRPLRequest, results::XRPLResponse},
    };

    pub struct WebSocketClient<const BUF: usize, Tcp, Rng, M, Status = WebSocketOpen>(
        pub(crate) AsyncWebSocketClient<BUF, Tcp, Rng, M, Status>,
    )
    where
        Tcp: Read + Write + Unpin,
        Rng: RngCore,
        M: RawMutex;

    impl<const BUF: usize, Tcp, Rng, M> XRPLClient for WebSocketClient<BUF, Tcp, Rng, M, WebSocketOpen>
    where
        Tcp: Read + Write + Unpin,
        Rng: RngCore,
        M: RawMutex,
    {
        fn get_host(&self) -> Url {
            self.0.get_host()
        }

        async fn request_impl<'a: 'b, 'b>(
            &self,
            request: XRPLRequest<'a>,
        ) -> XRPLClientResult<XRPLResponse<'b>> {
            block_on(self.0.request_impl(request))
        }
    }

    impl<const BUF: usize, Tcp, Rng, M> XRPLSyncWebsocketIO
        for WebSocketClient<BUF, Tcp, Rng, M, WebSocketOpen>
    where
        Tcp: Read + Write + Unpin,
        Rng: RngCore,
        M: RawMutex,
    {
        fn xrpl_send(
            &mut self,
            message: crate::models::requests::XRPLRequest<'_>,
        ) -> XRPLClientResult<()> {
            block_on(self.0.xrpl_send(message))
        }

        fn xrpl_receive(
            &mut self,
        ) -> XRPLClientResult<Option<crate::models::results::XRPLResponse<'_>>> {
            block_on(self.0.xrpl_receive())
        }
    }
}
