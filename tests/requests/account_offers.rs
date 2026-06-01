// Scenarios:
//   - base: send an account_offers request for a funded wallet and verify
//     the response returns an empty offers list

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_offers::AccountOffers;
use xrpl::models::results::account_offers::AccountOffers as AccountOffersResult;

#[tokio::test]
async fn test_account_offers_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let wallet = crate::common::generate_funded_wallet().await;

        let request = AccountOffers::new(
            None,                                  // id
            wallet.classic_address.clone().into(), // account
            None,                                  // ledger_hash
            None,                                  // ledger_index
            None,                                  // limit
            Some(true),                            // strict
            None,                                  // marker
        );

        let response = client
            .request(request.into())
            .await
            .expect("account_offers request failed");

        let result: AccountOffersResult = response
            .try_into()
            .expect("failed to parse account_offers result");

        // Verify account matches
        assert_eq!(result.account.as_ref(), wallet.classic_address.as_str());
        // Verify offers is empty
        assert!(result.offers.is_empty());
        // Verify ledger_current_index exists (no specific ledger requested)
        assert!(result.ledger_current_index.unwrap() > 0);
    })
    .await;
}
