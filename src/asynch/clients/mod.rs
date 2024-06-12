mod async_client;
mod client;
mod json_rpc;
mod websocket;

use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};

pub type MultiExecutorMutex = CriticalSectionRawMutex;
pub type SingleExecutorMutex = NoopRawMutex;

pub use async_client::*;
pub use json_rpc::*;
pub use websocket::*;
