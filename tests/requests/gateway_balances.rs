// Scenarios:
//   - base: send a gateway_balances request for a funded wallet and verify
//     the response (no issued currencies, so balances/obligations are empty)

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::gateway_balances::GatewayBalances;
use xrpl::models::requests::LedgerIndex;
use xrpl::models::results::gateway_balances::GatewayBalances as GatewayBalancesResult;

#[tokio::test]
async fn test_gateway_balances_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let wallet = crate::common::generate_funded_wallet().await;

        let request = GatewayBalances::new(
            None,                                       // id
            wallet.classic_address.clone().into(),      // account
            None,                                       // hotwallet
            None,                                       // ledger_hash
            Some(LedgerIndex::Str("validated".into())), // ledger_index
            Some(true),                                 // strict
        );

        let response = client
            .request(request.into())
            .await
            .expect("gateway_balances request failed");

        let result: GatewayBalancesResult = response
            .try_into()
            .expect("failed to parse gateway_balances result");

        // Verify account matches
        assert_eq!(result.account.as_ref(), wallet.classic_address.as_str());
        // Verify ledger_hash exists
        assert!(result.ledger_hash.is_some());
        // Verify ledger_index is valid
        assert!(result.ledger_index.unwrap() > 0);
        // Verify no obligations (no issued currencies)
        assert!(result.obligations.is_none());
    })
    .await;
}
