// Scenarios:
//   - base: send an account_channels request for a funded wallet and verify
//     the response returns an empty channels list (no channels created)

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_channels::AccountChannels;
use xrpl::models::requests::LedgerIndex;
use xrpl::models::results::account_channels::AccountChannels as AccountChannelsResult;

#[tokio::test]
async fn test_account_channels_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let wallet = crate::common::generate_funded_wallet().await;

        let request = AccountChannels::new(
            None,                                       // id
            wallet.classic_address.clone().into(),      // account
            None,                                       // destination_account
            None,                                       // ledger_hash
            Some(LedgerIndex::Str("validated".into())), // ledger_index
            None,                                       // limit
            None,                                       // marker
        );

        let response = client
            .request(request.into())
            .await
            .expect("account_channels request failed");

        let result: AccountChannelsResult = response
            .try_into()
            .expect("failed to parse account_channels result");

        // Verify account matches
        assert_eq!(result.account.as_ref(), wallet.classic_address.as_str());
        // Verify channels is empty (no channels created)
        assert!(result.channels.is_empty());
        // Verify validated
        assert!(result.validated);
        // Verify ledger_hash exists
        assert!(result.ledger_hash.is_some());
        // Verify ledger_index is valid
        assert!(result.ledger_index > 0);
    })
    .await;
}
