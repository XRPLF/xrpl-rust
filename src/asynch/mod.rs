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

// async-std has been discontinued (RUSTSEC-2025-0052). Emit a compile-time error so
// callers get a clear message instead of a runtime panic, and avoid unreachable-code
// lint when --all-features is used alongside other runtime feature flags.
#[cfg(feature = "async-std-rt")]
compile_error!(
    "The async-std-rt feature is deprecated and no longer supported. \
     async-std has been discontinued (RUSTSEC-2025-0052). \
     Use the smol-rt feature instead."
);

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
