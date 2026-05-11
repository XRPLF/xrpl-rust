// End-to-end multisign integration test.
//
// Exercises:
//   - src/transaction/multisign.rs::multisign (combines signer-signed copies)
//   - src/asynch/transaction/mod.rs::sign(multisign=true)
// Both are at 0% integration coverage because every other transaction test uses
// the single-signer test_transaction helper.

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::{
    asynch::transaction::{autofill, sign, submit},
    models::{
        transactions::{
            payment::Payment,
            signer_list_set::{SignerEntry, SignerListSet},
        },
        Amount, XRPAmount,
    },
    wallet::Wallet,
};

#[tokio::test]
async fn test_multisign_payment() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let main_wallet = generate_funded_wallet().await;
        let signer_a = Wallet::create(None).expect("signer A wallet");
        let signer_b = Wallet::create(None).expect("signer B wallet");
        let recipient = Wallet::create(None).expect("recipient wallet");

        // Register a 2-of-2 signer list on the main account.
        let mut signer_list = SignerListSet::new(
            main_wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            2,
            Some(vec![
                SignerEntry::new(signer_a.classic_address.clone(), 1),
                SignerEntry::new(signer_b.classic_address.clone(), 1),
            ]),
        );
        crate::common::test_transaction(&mut signer_list, &main_wallet).await;

        // Build the payment but do not sign yet. autofill with signers_count=2 so
        // the fee accounts for the multisign cost.
        let mut payment = Payment::new(
            main_wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("20000000")),
            recipient.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
        );
        autofill(&mut payment, client, Some(2))
            .await
            .expect("autofill for multisign");

        // Each signer signs a copy of the transaction.
        let mut payment_signed_by_a = payment.clone();
        sign(&mut payment_signed_by_a, &signer_a, true).expect("signer A sign");
        let mut payment_signed_by_b = payment.clone();
        sign(&mut payment_signed_by_b, &signer_b, true).expect("signer B sign");

        // multisign() merges the signer-signed copies into the master transaction.
        // Box::leak the Vec because multisign's signature ties the borrow's lifetime
        // to T's 'a parameter, which the async block infers as 'static when the
        // Payment was built from string literals (e.g. XRPAmount::from("20000000")).
        // Without 'static here the borrow checker rejects the call. The leak is
        // intentional and bounded by the test runtime.
        let signer_signed_copies: &'static Vec<_> =
            Box::leak(Box::new(vec![payment_signed_by_a, payment_signed_by_b]));
        xrpl::transaction::multisign(&mut payment, signer_signed_copies)
            .expect("multisign combine");

        assert!(
            payment
                .common_fields
                .signers
                .as_ref()
                .is_some_and(|s| s.len() == 2),
            "multisigned payment should carry both signers"
        );

        // Submit the multisigned transaction directly (no further signing).
        let submit_result = submit(&payment, client)
            .await
            .expect("submit multisigned payment");
        assert_eq!(submit_result.engine_result, "tesSUCCESS");
        ledger_accept().await;
    })
    .await;
}
