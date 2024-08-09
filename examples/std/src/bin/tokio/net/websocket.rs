/// Utilizing the no_std `AsyncWebsocketClient` of xrpl-rust with a tokio tcp stream.
use rand::rngs::OsRng;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use xrpl::asynch::clients::{websocket::codec::Codec, AsyncWebsocketClient, SingleExecutorMutex};
use xrpl::models::requests::{StreamParameter, Subscribe};

#[tokio::main]
async fn main() {
    let tcp_stream = tokio::net::TcpStream::connect("ws.vi-server.org:80")
        .await
        .unwrap();
    let framed_stream = Framed::new(tcp_stream, Codec);
    let rng = OsRng;
    let url = "wss://xrplcluster.com/".parse().unwrap();
    let mut websocket = AsyncWebsocketClient::<
        4096,
        Framed<TcpStream, Codec>,
        _,
        _,
        OsRng,
        SingleExecutorMutex,
    >::open(rng, framed_stream, url)
    .await
    .unwrap();

    let subscribe = Subscribe::new(
        Some("my_id".into()),
        None,
        None,
        None,
        Some(vec![StreamParameter::Ledger]),
        None,
        None,
        None,
    );
    websocket.xrpl_send(subscribe.into()).await.unwrap();
    loop {
        let account_info_echo = websocket.xrpl_receive().await.unwrap().unwrap();
        println!("subscription message: {:?}", account_info_echo);
        // break;
    }
}
