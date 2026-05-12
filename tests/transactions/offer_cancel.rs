// Scenarios:
//   - base: create an offer then cancel it by sequence number

use crate::common::{
    generate_funded_wallet, get_client, ledger_accept, test_transaction, with_blockchain_lock,
};
use xrpl::{
    asynch::transaction::sign_and_submit,
    models::{
        transactions::{offer_cancel::OfferCancel, offer_create::OfferCreate, Transaction},
        Amount, IssuedCurrencyAmount, XRPAmount,
    },
};

#[tokio::test]
async fn test_offer_cancel_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        // Fresh wallet: this test creates and cancels an offer, modifying sequence state.
        let wallet = generate_funded_wallet().await;

        // Step 1: place an offer so we have a sequence number to cancel.
        let mut create = OfferCreate::new(
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
            Amount::XRPAmount(XRPAmount::from("100")),
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                "10".into(),
            )),
            None,
            None,
        );

        sign_and_submit(&mut create, client, &wallet, true, true)
            .await
            .expect("Failed to submit OfferCreate");

        ledger_accept().await;

        // Step 2: cancel the offer using the sequence number filled in during autofill.
        let offer_sequence = create
            .get_common_fields()
            .sequence
            .expect("Sequence should be set after autofill");

        let mut cancel = OfferCancel::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            offer_sequence,
        );

        test_transaction(&mut cancel, &wallet).await;
    })
    .await;
}
