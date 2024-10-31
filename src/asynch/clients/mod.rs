pub mod async_client;
pub mod client;
pub mod exceptions;
#[cfg(feature = "json-rpc")]
mod json_rpc;
#[cfg(feature = "websocket")]
mod websocket;

use alloc::borrow::Cow;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use serde::{Deserialize, Serialize};

#[cfg(feature = "helpers")]
use exceptions::XRPLClientResult;
#[cfg(feature = "helpers")]
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

#[cfg(feature = "helpers")]
use crate::{asynch::wallet::exceptions::XRPLFaucetException, models::requests::FundFaucet};

#[cfg(feature = "helpers")]
#[allow(async_fn_in_trait)]
pub trait XRPLFaucet: XRPLClient {
    fn get_faucet_url(&self, url: Option<Url>) -> XRPLClientResult<Url>
    where
        Self: Sized + XRPLClient,
    {
        if let Some(url) = url {
            Ok(url)
        } else {
            let host = self.get_host();
            let host_str = host.host_str().unwrap();
            if host_str.contains("altnet") || host_str.contains("testnet") {
                Ok(Url::parse(TEST_FAUCET_URL)?)
            } else if host_str.contains("devnet") {
                Ok(Url::parse(DEV_FAUCET_URL)?)
            } else if host_str.contains("sidechain-net2") {
                Err(XRPLFaucetException::CannotFundSidechainAccount.into())
            } else {
                Err(XRPLFaucetException::CannotDeriveFaucetUrl.into())
            }
        }
    }

    async fn request_funding(
        &self,
        url: Option<Url>,
        request: FundFaucet<'_>,
    ) -> XRPLClientResult<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonFields<'a> {
    pub build_version: Option<Cow<'a, str>>,
    pub network_id: Option<u32>,
}
