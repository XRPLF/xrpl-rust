// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/trustSet.test.ts
//
// Scenarios:
//   - base: set a USD trust line to a locally funded issuer
//
// NOTE: Bitstamp (rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B) does not exist in standalone Docker mode.
// A fresh issuer wallet is funded from genesis instead.

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
use xrpl::models::{transactions::trust_set::TrustSet, IssuedCurrencyAmount};

#[tokio::test]
async fn test_trust_set_base() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let issuer = generate_funded_wallet().await;

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
                issuer.classic_address.clone().into(),
                "1000".into(),
            ),
            None,
            None,
        );

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}
