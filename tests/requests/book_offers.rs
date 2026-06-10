// Scenarios:
//   - base: send a book_offers request for XRP/USD and verify the response

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::book_offers::BookOffers;
use xrpl::models::results::book_offers::BookOffers as BookOffersResult;
use xrpl::models::{Currency, IssuedCurrency, XRP};

#[tokio::test]
async fn test_book_offers_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let wallet = crate::common::generate_funded_wallet().await;

        let request = BookOffers::new(
            None,                      // id
            Currency::XRP(XRP::new()), // taker_gets
            Currency::IssuedCurrency(IssuedCurrency::new(
                // taker_pays
                "USD".into(),
                wallet.classic_address.clone().into(),
            )),
            None, // ledger_hash
            None, // ledger_index
            None, // limit
            None, // taker
        );

        let response = client
            .request(request.into())
            .await
            .expect("book_offers request failed");

        let result: BookOffersResult = response
            .try_into()
            .expect("failed to parse book_offers result");

        // Verify offers is empty (no offers on the book)
        assert!(result.offers.is_empty());
        // Verify ledger_current_index exists
        assert!(result.ledger_current_index.unwrap() > 0);
    })
    .await;
}
