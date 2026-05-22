// Scenarios:
//   - base: send a ripple_path_find request between two funded wallets and
//     verify destination_account and destination_currencies

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::ripple_path_find::RipplePathFind as RipplePathFindRequest,
    results::ripple_path_find::RipplePathFind as RipplePathFindResult, Amount,
};

#[tokio::test]
async fn test_ripple_path_find_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let wallet1 = crate::common::generate_funded_wallet().await;
        let wallet2 = crate::common::generate_funded_wallet().await;

        let request = RipplePathFindRequest::new(
            None,                                   // id
            wallet2.classic_address.clone().into(), // destination_account
            Amount::XRPAmount("100".into()),        // destination_amount (XRP drops)
            wallet1.classic_address.clone().into(), // source_account
            None,                                   // ledger_hash
            None,                                   // ledger_index
            None,                                   // send_max
            None,                                   // source_currencies
        );

        let response = client
            .request(request.into())
            .await
            .expect("ripple_path_find request failed");

        let result: RipplePathFindResult = response
            .try_into()
            .expect("failed to parse ripple_path_find result");

        // Verify the destination account matches
        assert_eq!(
            result.destination_account.as_ref(),
            wallet2.classic_address.as_str()
        );
        // Verify destination_currencies is not empty (should at least contain XRP)
        assert!(!result.destination_currencies.is_empty());
    })
    .await;
}
