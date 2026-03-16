// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/trustSet.test.ts
//
// Scenarios:
//   - base: set a USD trust line to a well-known issuer

use crate::common::{get_client, get_wallet, ledger_accept, with_blockchain_lock};
use xrpl::{
    asynch::transaction::submit_and_wait,
    models::{transactions::trust_set::TrustSet, IssuedCurrencyAmount},
};

#[tokio::test]
async fn test_trust_set_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = get_wallet().await;

        let mut tx = TrustSet::new(
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
            IssuedCurrencyAmount::new(
                "USD".into(),
                "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(), // Bitstamp issuer
                "1000".into(),
            ),
            None,
            None,
        );

        let result = submit_and_wait(&mut tx, client, Some(wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit TrustSet");

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
