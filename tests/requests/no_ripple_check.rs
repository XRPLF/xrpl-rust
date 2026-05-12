// Scenarios:
//   - base: send a noripple_check request for a funded wallet with role=gateway
//     and transactions=true, verify problems and transactions arrays

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::no_ripple_check::{NoRippleCheck as NoRippleCheckRequest, NoRippleCheckRole},
    requests::LedgerIndex,
    results::no_ripple_check::NoRippleCheck as NoRippleCheckResult,
};

#[tokio::test]
async fn test_no_ripple_check_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let wallet = crate::common::generate_funded_wallet().await;

        let request = NoRippleCheckRequest::new(
            None,                                     // id
            wallet.classic_address.clone().into(),    // account
            NoRippleCheckRole::Gateway,               // role
            None,                                     // ledger_hash
            Some(LedgerIndex::Str("current".into())), // ledger_index
            None,                                     // limit
            Some(true),                               // transactions
        );

        let response = client
            .request(request.into())
            .await
            .expect("noripple_check request failed");

        let result: NoRippleCheckResult = response
            .try_into()
            .expect("failed to parse noripple_check result");

        // A newly funded account with gateway role should have at least one problem
        // (the default ripple flag recommendation)
        assert!(!result.problems.is_empty());
        assert!(result.problems[0].contains("default ripple"));

        // With transactions=true, we should get suggested fix transactions
        assert!(result.transactions.is_some());
        let transactions = result.transactions.unwrap();
        assert!(!transactions.is_empty());
    })
    .await;
}
