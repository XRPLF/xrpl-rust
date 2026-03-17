// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/accountSet.test.ts
//
// Scenarios:
//   - base: set domain field with hex-encoded value
//   - with_memo: attach a memo to the transaction

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
use xrpl::models::transactions::{account_set::AccountSet, Memo};

#[tokio::test]
async fn test_account_set_base() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;

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

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}

#[tokio::test]
async fn test_account_set_with_memo() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;

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

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}
