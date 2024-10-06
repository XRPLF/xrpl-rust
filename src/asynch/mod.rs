#[cfg(all(
    feature = "account-helpers",
    any(feature = "websocket", feature = "json-rpc")
))]
pub mod account;
#[cfg(any(feature = "websocket", feature = "json-rpc"))]
pub mod clients;
#[cfg(all(
    feature = "ledger-helpers",
    any(feature = "websocket", feature = "json-rpc")
))]
pub mod ledger;
#[cfg(all(
    feature = "transaction-helpers",
    any(feature = "websocket", feature = "json-rpc")
))]
pub mod transaction;
#[cfg(all(
    feature = "wallet-helpers",
    any(feature = "websocket", feature = "json-rpc")
))]
pub mod wallet;

use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum XRPLFaucetException {
    #[error(
        "Cannot fund an account on an issuing chain. Accounts must be created via the bridge."
    )]
    CannotFundSidechainAccount,
    #[error("Cannot derive a faucet URL from the client host.")]
    CannotDeriveFaucetUrl,
    #[error("Funding request timed out.")]
    FundingTimeout,
}

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
