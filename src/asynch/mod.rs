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

#[allow(unused_imports)]
#[allow(clippy::needless_return)]
async fn wait_seconds(seconds: u64) {
    use core::time::Duration;

    #[cfg(feature = "tokio-rt")]
    {
        tokio::time::sleep(tokio::time::Duration::from_secs(seconds)).await;
        return;
    }
    #[cfg(feature = "embassy-rt")]
    {
        embassy_time::Timer::after_secs(1).await;
        return;
    }
    #[cfg(feature = "actix-rt")]
    {
        actix_rt::time::sleep(Duration::from_secs(seconds)).await;
        return;
    }
    #[cfg(feature = "async-std-rt")]
    {
        async_std::task::sleep(Duration::from_secs(seconds)).await;
        return;
    }
    #[cfg(feature = "futures-rt")]
    {
        futures_timer::Delay::new(Duration::from_secs(seconds)).await;
        return;
    }
    #[cfg(feature = "smol-rt")]
    {
        smol::Timer::after(Duration::from_secs(seconds)).await;
        return;
    }
}
