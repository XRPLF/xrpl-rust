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

const TEST_FAUCET_URL: &str = "https://faucet.altnet.rippletest.net/accounts";
const DEV_FAUCET_URL: &str = "https://faucet.devnet.rippletest.net/accounts";

use crate::{asynch::XRPLFaucetException, models::requests::FundFaucet, Err};
#[allow(async_fn_in_trait)]
pub trait XRPLFaucet: XRPLClient {
    fn get_faucet_url(&self, url: Option<Url>) -> Result<Url>
    where
        Self: Sized + XRPLClient,
    {
        if let Some(url) = url {
            Ok(url)
        } else {
            let host = self.get_host();
            let host_str = host.host_str().unwrap();
            if host_str.contains("altnet") || host_str.contains("testnet") {
                match Url::parse(TEST_FAUCET_URL) {
                    Ok(url) => Ok(url),
                    Err(error) => Err!(error),
                }
            } else if host_str.contains("devnet") {
                match Url::parse(DEV_FAUCET_URL) {
                    Ok(url) => Ok(url),
                    Err(error) => Err!(error),
                }
            } else if host_str.contains("sidechain-net2") {
                Err!(XRPLFaucetException::CannotFundSidechainAccount)
            } else {
                Err!(XRPLFaucetException::CannotDeriveFaucetUrl)
            }
        }
    }

    async fn request_funding(&self, url: Option<Url>, request: FundFaucet<'_>) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonFields<'a> {
    pub build_version: Option<Cow<'a, str>>,
    pub network_id: Option<u32>,
}
