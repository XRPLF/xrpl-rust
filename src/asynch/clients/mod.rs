pub mod async_client;
pub mod client;
#[cfg(any(feature = "json-rpc-std", feature = "json-rpc"))]
pub mod json_rpc;
#[cfg(any(feature = "websocket-std", feature = "websocket"))]
pub mod websocket;

use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
pub type MultiExecutorMutex = CriticalSectionRawMutex;
pub type SingleExecutorMutex = NoopRawMutex;

pub use async_client::*;
pub use client::*;
#[cfg(any(feature = "json-rpc-std", feature = "json-rpc"))]
pub use json_rpc::*;
#[cfg(any(feature = "websocket-std", feature = "websocket"))]
pub use websocket::*;
