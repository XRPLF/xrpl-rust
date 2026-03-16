// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/accountSet.test.ts
//
// Scenarios:
//   - base: set domain field with hex-encoded value
//   - with_memo: attach a memo to the transaction

use crate::common::{get_client, get_wallet, ledger_accept, with_blockchain_lock};
use xrpl::{
    asynch::transaction::submit_and_wait,
    models::transactions::{account_set::AccountSet, Memo},
};

#[tokio::test]
async fn test_account_set_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = get_wallet().await;

        let mut tx = AccountSet::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("6578616d706c652e636f6d".into()), // hex("example.com")
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let result = submit_and_wait(&mut tx, client, Some(wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit AccountSet");

        assert_eq!(
            result
                .get_transaction_metadata()
                .expect("Expected metadata")
                .transaction_result,
            "tesSUCCESS"
        );

        ledger_accept().await;
    })
    .await;
}

#[tokio::test]
async fn test_account_set_with_memo() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = get_wallet().await;

        let mut tx = AccountSet::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            Some(vec![Memo::new(
                Some(hex::encode("Hello, XRPL!").into()),
                Some(hex::encode("text/plain").into()),
                Some(hex::encode("application/json").into()),
            )]),
            None,
            None,
            None,
            None,
            None,
            Some("6578616d706c652e636f6d".into()),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let result = submit_and_wait(&mut tx, client, Some(wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit AccountSet with memo");

        assert_eq!(
            result
                .get_transaction_metadata()
                .expect("Expected metadata")
                .transaction_result,
            "tesSUCCESS"
        );

        ledger_accept().await;
    })
    .await;
}
