pub mod async_client;
pub mod client;
pub mod json_rpc;
#[cfg(any(feature = "tungstenite", feature = "embedded-ws"))]
pub mod websocket;

use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
pub type MultiExecutorMutex = CriticalSectionRawMutex;
pub type SingleExecutorMutex = NoopRawMutex;

pub use async_client::*;
pub use client::*;
pub use json_rpc::*;
#[cfg(any(feature = "tungstenite", feature = "embedded-ws"))]
pub use websocket::*;
