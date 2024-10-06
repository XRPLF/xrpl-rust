use super::Wallet;
use crate::asynch::{
    clients::{XRPLAsyncClient, XRPLFaucet},
    wallet::generate_faucet_wallet as async_generate_faucet_wallet,
};
use alloc::borrow::Cow;
use anyhow::Result;
use embassy_futures::block_on;
use url::Url;

pub use crate::asynch::wallet::get_faucet_url;

pub fn generate_faucet_wallet<'a, C>(
    client: &C,
    wallet: Option<Wallet>,
    faucet_host: Option<Url>,
    usage_context: Option<Cow<'a, str>>,
    user_agent: Option<Cow<'a, str>>,
) -> Result<Wallet>
where
    C: XRPLFaucet + XRPLAsyncClient,
{
    block_on(async_generate_faucet_wallet(
        client,
        wallet,
        faucet_host,
        usage_context,
        user_agent,
    ))
}
