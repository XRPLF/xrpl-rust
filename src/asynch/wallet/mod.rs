use alloc::borrow::Cow;
use anyhow::Result;
use url::Url;

use crate::{
    asynch::{account::get_next_valid_seq_number, wait_seconds, XRPLFaucetException},
    models::{requests::FundFaucet, XRPAmount},
    wallet::Wallet,
    Err,
};

use super::{
    account::get_xrp_balance,
    clients::{XRPLClient, XRPLFaucet},
};

const TIMEOUT_SECS: u8 = 40;

pub async fn generate_faucet_wallet<'a, C>(
    client: &C,
    wallet: Option<Wallet>,
    faucet_host: Option<Url>,
    usage_context: Option<Cow<'a, str>>,
    user_agent: Option<Cow<'a, str>>,
) -> Result<Wallet>
where
    C: XRPLFaucet + XRPLClient,
{
    let faucet_url = get_faucet_url(client, faucet_host)?;
    let wallet = match wallet {
        Some(wallet) => wallet,
        None => match Wallet::create(None) {
            Ok(wallet) => wallet,
            Err(error) => return Err!(error),
        },
    };
    let address = &wallet.classic_address;
    let starting_balance = check_balance(client, address.into()).await;
    let user_agent = user_agent.unwrap_or("xrpl-rust".into());
    fund_wallet(
        client,
        faucet_url,
        address.into(),
        usage_context,
        Some(user_agent),
    )
    .await?;
    let mut is_funded = false;
    for _ in 0..TIMEOUT_SECS {
        // wait 1 second
        wait_seconds(1).await;
        if !is_funded {
            let balance = check_balance(client, address.into()).await;
            if balance > starting_balance {
                is_funded = true;
            }
        } else {
            // wait until the ledger knows about the wallets existence
            match get_next_valid_seq_number(address.into(), client, None).await {
                Ok(_sequence) => {
                    return Ok(wallet);
                }
                Err(_) => continue,
            }
        }
    }

    Err!(XRPLFaucetException::FundingTimeout)
}

pub fn get_faucet_url<C>(client: &C, url: Option<Url>) -> Result<Url>
where
    C: XRPLFaucet + XRPLClient,
{
    client.get_faucet_url(url)
}

async fn check_balance<'a: 'b, 'b, C>(client: &C, address: Cow<'a, str>) -> XRPAmount<'b>
where
    C: XRPLClient,
{
    get_xrp_balance(address, client, None)
        .await
        .unwrap_or(XRPAmount::default())
}

async fn fund_wallet<'a: 'b, 'b, C>(
    client: &C,
    faucet_url: Url,
    address: Cow<'a, str>,
    usage_context: Option<Cow<'a, str>>,
    user_agent: Option<Cow<'a, str>>,
) -> Result<()>
where
    C: XRPLFaucet + XRPLClient,
{
    let request = FundFaucet {
        destination: address,
        usage_context,
        user_agent,
    };
    client.request_funding(Some(faucet_url), request).await?;

    Ok(())
}

#[cfg(all(feature = "json-rpc", feature = "std"))]
#[cfg(test)]
mod test_faucet_wallet_generation {
    use super::*;
    use crate::asynch::clients::AsyncJsonRpcClient;
    use url::Url;

    #[tokio::test]
    async fn test_generate_faucet_wallet() {
        let client =
            AsyncJsonRpcClient::connect(Url::parse("https://testnet.xrpl-labs.com/").unwrap());
        let wallet = generate_faucet_wallet(&client, None, None, None, None)
            .await
            .unwrap();
        let balance = get_xrp_balance(wallet.classic_address.clone().into(), &client, None)
            .await
            .unwrap();
        assert!(balance > 0.into());
    }
}
