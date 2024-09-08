pub mod async_client;
pub mod client;
#[cfg(feature = "json-rpc")]
mod json_rpc;
#[cfg(feature = "websocket")]
mod websocket;

use alloc::borrow::Cow;
use anyhow::Result;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use serde::{Deserialize, Serialize};
use url::Url;

pub use async_client::*;
pub use client::*;
#[cfg(feature = "json-rpc")]
pub use json_rpc::*;
#[cfg(feature = "websocket")]
pub use websocket::*;

pub type MultiExecutorMutex = CriticalSectionRawMutex;
pub type SingleExecutorMutex = NoopRawMutex;

#[cfg(feature = "wallet-helpers")]
use crate::{asynch::wallet::get_faucet_url, models::requests::FundFaucet};
#[allow(async_fn_in_trait)]
#[cfg(feature = "wallet-helpers")]
pub trait XRPLFaucet: Client {
    fn get_faucet_url(&self, url: Option<Url>) -> Result<Url>
    where
        Self: Sized + Client,
    {
        get_faucet_url(self, url)
    }

    async fn request_funding(&self, url: Option<Url>, request: FundFaucet<'_>) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonFields<'a> {
    pub build_version: Option<Cow<'a, str>>,
    pub network_id: Option<u32>,
}
