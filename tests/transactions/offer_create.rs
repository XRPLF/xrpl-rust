// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/offerCreate.test.ts
//
// Scenarios:
//   - base: place an XRP/USD offer on the DEX

use crate::common::{get_client, get_wallet, ledger_accept, with_blockchain_lock};
use xrpl::{
    asynch::transaction::submit_and_wait,
    models::{transactions::offer_create::OfferCreate, Amount, IssuedCurrencyAmount, XRPAmount},
};

#[tokio::test]
async fn test_offer_create_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = get_wallet().await;

        let mut tx = OfferCreate::new(
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
            Amount::XRPAmount(XRPAmount::from("100")), // taker_pays: 100 XRP drops
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(), // Bitstamp issuer
                "10".into(),
            )),
            None,
            None,
        );

        let result = submit_and_wait(&mut tx, client, Some(wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit OfferCreate");

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
