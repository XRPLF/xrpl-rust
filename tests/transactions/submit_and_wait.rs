// Direct end-to-end test of asynch::transaction::submit_and_wait.
//
// Other transaction tests use the shared `test_transaction` helper, which calls
// `sign_and_submit` and then `ledger_accept`. submit_and_wait does its own polling
// for ledger validation, so it needs a separate test. Standalone rippled does not
// auto-close ledgers, so a background task drives `ledger_accept` while the poll
// loop runs.

use core::time::Duration;

use crate::common::{
    generate_funded_wallet, get_client, ledger_accept, payment::xrp_payment, with_blockchain_lock,
};
use xrpl::asynch::{
    exceptions::XRPLHelperException,
    transaction::{
        exceptions::{XRPLSubmitAndWaitException, XRPLTransactionHelperException},
        submit_and_wait,
    },
};
use xrpl::wallet::Wallet;

/// Match on the typed `SubmissionFailed { result_code, message }` shape
/// through the wrapped exception layers.
macro_rules! assert_submission_failed_matches {
    ($err:expr, |$code:ident, $msg:ident| $body:block) => {
        match $err {
            XRPLHelperException::XRPLTransactionHelperError(
                XRPLTransactionHelperException::XRPLSubmitAndWaitError(
                    XRPLSubmitAndWaitException::SubmissionFailed {
                        result_code: $code,
                        message: $msg,
                    },
                ),
            ) => $body,
            other => panic!(
                "expected SubmissionFailed {{ result_code, message }}, got {:?}",
                other
            ),
        }
    };
}

#[tokio::test]
async fn test_submit_and_wait_payment() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let sender = generate_funded_wallet().await;
        let recipient = Wallet::create(None).expect("recipient wallet");

        // 20 XRP covers the standalone base reserve for the new recipient.
        let mut payment = xrp_payment(
            sender.classic_address.clone(),
            recipient.classic_address.clone(),
            "20000000",
        );

        // Drive ledger closes while submit_and_wait polls for validation.
        let ledger_driver = tokio::spawn(async {
            loop {
                ledger_accept().await;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });

        let validated_tx =
            submit_and_wait(&mut payment, client, Some(&sender), Some(true), Some(true))
                .await
                .expect("submit_and_wait should return a validated transaction");

        ledger_driver.abort();
        let _ = ledger_driver.await;

        let metadata = validated_tx
            .get_transaction_metadata()
            .expect("validated transaction should have metadata");

        assert_eq!(metadata.transaction_result, "tesSUCCESS");
    })
    .await;
}

/// Validated `tec*` path: sending more XRP than the sender holds is accepted
/// by rippled at submit time (fee is consumed), but the transactor rejects it
/// once validated with `tecUNFUNDED_PAYMENT`. `submit_and_wait` should surface
/// that code via the polling branch.
#[tokio::test]
async fn test_submit_and_wait_validated_tec_error() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let sender = generate_funded_wallet().await;
        let recipient = Wallet::create(None).expect("recipient wallet");

        // Sender was funded with 400 XRP; asking for 1000 XRP (10^9 drops)
        // exceeds the balance and reserves, so the transactor rejects with
        // tecUNFUNDED_PAYMENT. Kept below 2^32 drops to avoid the u32-parsing
        // overflow in XRPAmount's client-side check.
        let mut payment = xrp_payment(
            sender.classic_address.clone(),
            recipient.classic_address.clone(),
            "1000000000",
        );

        let ledger_driver = tokio::spawn(async {
            loop {
                ledger_accept().await;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });

        let err = submit_and_wait(&mut payment, client, Some(&sender), Some(true), Some(true))
            .await
            .expect_err("over-payment should be rejected once validated");

        ledger_driver.abort();
        let _ = ledger_driver.await;

        assert_submission_failed_matches!(err, |result_code, message| {
            assert_eq!(result_code, "tecUNFUNDED_PAYMENT");
            // Validated tec/tef path only has the code, no separate message.
            assert!(message.is_none(), "tec* path should have message: None");
        });
    })
    .await;
}

/// Polling timeout: if no ledger closes while `submit_and_wait` polls, the
/// retry counter (`c > 20`) trips and returns `SubmissionTimeout`. This is
/// the pre-existing variant orthogonal to the `SubmissionFailed` restructure,
/// but exercising it here documents the timeout contract end-to-end.
#[tokio::test]
async fn test_submit_and_wait_polling_timeout() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let sender = generate_funded_wallet().await;
        let recipient = Wallet::create(None).expect("recipient wallet");

        let mut payment = xrp_payment(
            sender.classic_address.clone(),
            recipient.classic_address.clone(),
            "20000000",
        );

        // No ledger driver spawned → validated_ledger_sequence never advances,
        // so the poll loop hits its 20-iteration retry cap after ~20s.
        let err = submit_and_wait(&mut payment, client, Some(&sender), Some(true), Some(true))
            .await
            .expect_err("polling without ledger_accept should time out");

        match err {
            XRPLHelperException::XRPLTransactionHelperError(
                XRPLTransactionHelperException::XRPLSubmitAndWaitError(
                    XRPLSubmitAndWaitException::SubmissionTimeout { prelim_result, .. },
                ),
            ) => {
                assert_eq!(prelim_result, "Transaction not included in ledger");
            }
            other => panic!("expected SubmissionTimeout, got {other:?}"),
        }

        // Close the ledger the timed-out tx sat in so subsequent tests
        // (serialized on the blockchain lock) see a clean state.
        ledger_accept().await;
    })
    .await;
}
