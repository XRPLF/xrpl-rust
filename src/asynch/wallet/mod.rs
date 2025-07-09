pub mod exceptions;

use alloc::borrow::Cow;
use exceptions::XRPLFaucetException;
use url::Url;

use crate::{
    asynch::{account::get_next_valid_seq_number, wait_seconds},
    models::{requests::FundFaucet, XRPAmount},
    wallet::Wallet,
};

use super::{
    account::get_xrp_balance,
    clients::{XRPLClient, XRPLFaucet},
    exceptions::XRPLHelperResult,
};

const TIMEOUT_SECS: u8 = 40;

pub async fn generate_faucet_wallet<'a, C>(
    client: &C,
    wallet: Option<Wallet>,
    faucet_host: Option<Url>,
    usage_context: Option<Cow<'a, str>>,
    user_agent: Option<Cow<'a, str>>,
) -> XRPLHelperResult<Wallet>
where
    C: XRPLFaucet + XRPLClient,
{
    let faucet_url = get_faucet_url(client, faucet_host)?;
    let wallet = match wallet {
        Some(wallet) => wallet,
        None => Wallet::create(None)?,
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

    Err(XRPLFaucetException::FundingTimeout.into())
}

pub fn get_faucet_url<C>(client: &C, url: Option<Url>) -> XRPLHelperResult<Url>
where
    C: XRPLFaucet + XRPLClient,
{
    Ok(client.get_faucet_url(url)?)
}

async fn check_balance<'a: 'b, 'b, C>(client: &'a C, address: Cow<'a, str>) -> XRPAmount<'b>
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
) -> XRPLHelperResult<()>
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
    use crate::{
        asynch::clients::AsyncJsonRpcClient,
        handle_test_result,
        utils::testing::{
            assertions, is_known_network_error, test_constants, test_network_operation,
            TestTimeouts,
        },
    };

    #[tokio::test]
    async fn test_generate_faucet_wallet() {
        let client = AsyncJsonRpcClient::connect(test_constants::TESTNET_URL.parse().unwrap());

        let result = test_network_operation(
            generate_faucet_wallet(&client, None, None, None, None),
            TestTimeouts::FAUCET,
            "faucet wallet generation",
        )
        .await;

        let wallet = handle_test_result!(result, "test_generate_faucet_wallet");
        assertions::assert_valid_wallet(&wallet);
    }

    #[test]
    fn test_wallet_creation_parameters() {
        // Test that we can create wallets and URLs without network calls
        let wallet = Wallet::create(None).unwrap();
        assertions::assert_valid_wallet(&wallet);

        // Test URL parsing
        let url1 = test_constants::TESTNET_URL.parse::<Url>().unwrap();
        assert_eq!(url1.scheme(), "https");
        assert_eq!(url1.host_str(), Some("testnet.xrpl-labs.com"));

        let url2 = test_constants::ALT_TESTNET_URL.parse::<Url>().unwrap();
        assert_eq!(url2.scheme(), "https");
        assert_eq!(url2.host_str(), Some("faucet.altnet.rippletest.net"));
        assert_eq!(url2.port_or_known_default(), Some(443));

        // Test a URL with explicit non-default port
        let url3 = "https://custom-faucet.example.com:8080/api"
            .parse::<Url>()
            .unwrap();
        assert_eq!(url3.scheme(), "https");
        assert_eq!(url3.host_str(), Some("custom-faucet.example.com"));
        assert_eq!(url3.port(), Some(8080)); // Non-default port should be explicit
        assert_eq!(url3.port_or_known_default(), Some(8080));

        // Test HTTP with default port
        let url4 = "http://example.com:80/path".parse::<Url>().unwrap();
        assert_eq!(url4.scheme(), "http");
        assert_eq!(url4.host_str(), Some("example.com"));
        assert_eq!(url4.port(), None); // Default port 80 for HTTP
        assert_eq!(url4.port_or_known_default(), Some(80));

        println!("✅ test_wallet_creation_parameters passed");
    }

    #[test]
    fn test_error_detection() {
        // Test that our error detection works correctly
        assert!(is_known_network_error("dns error occurred"));
        assert!(is_known_network_error(
            "failed to lookup address information"
        ));
        assert!(is_known_network_error("Connection refused"));
        assert!(is_known_network_error("Network is unreachable"));
        assert!(is_known_network_error("expected value"));
        assert!(is_known_network_error("ConnectError"));

        assert!(!is_known_network_error("some other error"));
        assert!(!is_known_network_error("validation failed"));
    }
}
