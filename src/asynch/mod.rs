pub mod exceptions;

#[cfg(feature = "helpers")]
pub mod account;
#[cfg(any(feature = "websocket", feature = "json-rpc"))]
pub mod clients;
#[cfg(feature = "helpers")]
pub mod ledger;
#[cfg(feature = "helpers")]
pub mod transaction;
#[cfg(feature = "helpers")]
pub mod wallet;

async fn wait_seconds(_seconds: u64) {
    #[cfg(feature = "tokio-rt")]
    {
        tokio::time::sleep(tokio::time::Duration::from_secs(_seconds)).await;
    }
    #[cfg(feature = "embassy-rt")]
    {
        embassy_time::Timer::after_secs(1).await;
    }
    #[cfg(feature = "actix-rt")]
    {
        use core::time::Duration;
        actix_rt::time::sleep(Duration::from_secs(_seconds)).await;
    }
    #[cfg(feature = "async-std-rt")]
    {
        use core::time::Duration;
        async_std::task::sleep(Duration::from_secs(_seconds)).await;
    }
    #[cfg(feature = "futures-rt")]
    {
        use core::time::Duration;
        futures_timer::Delay::new(Duration::from_secs(_seconds)).await;
    }
    #[cfg(feature = "smol-rt")]
    {
        use core::time::Duration;
        smol::Timer::after(Duration::from_secs(_seconds)).await;
    }
}
