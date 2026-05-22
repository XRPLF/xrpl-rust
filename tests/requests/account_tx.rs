// Scenarios:
//   - base: send an account_tx request for a funded wallet and verify the
//     response contains the funding transaction (Payment from genesis)

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_tx::AccountTx;
use xrpl::models::requests::LedgerIndex;
use xrpl::models::results::account_tx::AccountTxVersionMap;

#[tokio::test]
async fn test_account_tx_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let wallet = crate::common::generate_funded_wallet().await;

        let request = AccountTx::new(
            None,                                       // id
            wallet.classic_address.clone().into(),      // account
            None,                                       // ledger_hash
            Some(LedgerIndex::Str("validated".into())), // ledger_index
            None,                                       // binary
            None,                                       // forward
            None,                                       // ledger_index_min
            None,                                       // ledger_index_max
            None,                                       // limit
            None,                                       // marker
        );

        let response = client
            .request(request.into())
            .await
            .expect("account_tx request failed");

        let result: AccountTxVersionMap = response
            .try_into()
            .expect("failed to parse account_tx result");

        // Standalone rippled returns API v1 format by default (tx object
        // instead of tx_json), so we handle both variants.
        match result {
            AccountTxVersionMap::Default(tx_result) => {
                // API v2 format
                assert_eq!(
                    tx_result.base.account.as_ref(),
                    wallet.classic_address.as_str()
                );
                assert!(
                    !tx_result.base.transactions.is_empty(),
                    "Expected at least one transaction"
                );
                let first_tx = &tx_result.base.transactions[0];
                assert!(!first_tx.hash.is_empty());
                let tx_json = first_tx.tx_json.as_ref().expect("tx_json should exist");
                assert_eq!(tx_json["TransactionType"].as_str().unwrap(), "Payment");
                assert_eq!(
                    tx_json["Destination"].as_str().unwrap(),
                    wallet.classic_address.as_str()
                );
            }
            AccountTxVersionMap::V1(tx_result) => {
                // API v1 format (standalone node default)
                assert_eq!(
                    tx_result.base.account.as_ref(),
                    wallet.classic_address.as_str()
                );
                assert!(
                    !tx_result.base.transactions.is_empty(),
                    "Expected at least one transaction"
                );
                let first_tx = &tx_result.base.transactions[0];
                let tx = first_tx.tx.as_ref().expect("tx should exist");
                assert_eq!(tx["TransactionType"].as_str().unwrap(), "Payment");
                assert_eq!(
                    tx["Destination"].as_str().unwrap(),
                    wallet.classic_address.as_str()
                );
            }
        }
    })
    .await;
}
