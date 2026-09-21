use core::fmt::Debug;

use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use serde::{de::DeserializeOwned, Serialize};
use strum::IntoEnumIterator;

use crate::{
    asynch::{
        clients::XRPLAsyncClient,
        exceptions::XRPLHelperResult,
        ledger::get_latest_validated_ledger_sequence,
        transaction::{
            autofill, check_txn_fee,
            exceptions::{XRPLSignTransactionException, XRPLSubmitAndWaitException},
            sign, submit,
        },
        wait_seconds,
    },
    models::{
        requests::{self},
        results::{tx::TxVersionMap, XRPLRpcError},
        transactions::Transaction,
        Model,
    },
    wallet::Wallet,
};

/// Build a `SubmissionFailed` exception. Extracted so all four failure sites
/// (prelim `tem*`, RPC error mid-poll, validated `tec*`/`tef*`, poll timeout)
/// share the same construction shape.
fn submission_failed(
    result_code: impl Into<String>,
    message: Option<String>,
) -> XRPLSubmitAndWaitException {
    XRPLSubmitAndWaitException::SubmissionFailed {
        result_code: result_code.into(),
        message,
    }
}

pub async fn submit_and_wait<'a: 'b, 'b, T, F, C>(
    transaction: &'b mut T,
    client: &C,
    wallet: Option<&Wallet>,
    check_fee: Option<bool>,
    autofill: Option<bool>,
) -> XRPLHelperResult<TxVersionMap<'b>>
where
    T: Transaction<'a, F> + Model + Clone + DeserializeOwned + Debug,
    F: IntoEnumIterator + Serialize + Debug + PartialEq + Debug + Clone + 'a,
    C: XRPLAsyncClient,
{
    get_signed_transaction(transaction, client, wallet, check_fee, autofill).await?;
    send_reliable_submission(transaction, client).await
}

async fn send_reliable_submission<'a: 'b, 'b, T, F, C>(
    transaction: &'b mut T,
    client: &C,
) -> XRPLHelperResult<TxVersionMap<'b>>
where
    T: Transaction<'a, F> + Model + Clone + DeserializeOwned + Debug,
    F: IntoEnumIterator + Serialize + Debug + PartialEq + Debug + Clone + 'a,
    C: XRPLAsyncClient,
{
    let tx_hash = transaction.get_hash()?;
    let submit_response = submit(transaction, client).await?;
    let prelim_result = submit_response.engine_result;
    if &prelim_result[0..3] == "tem" {
        Err(submission_failed(
            prelim_result,
            Some(submit_response.engine_result_message.to_string()),
        )
        .into())
    } else {
        wait_for_final_transaction_result(
            tx_hash,
            client,
            transaction
                .get_common_fields()
                .last_ledger_sequence
                .unwrap(), // safe to unwrap because we autofilled the transaction
        )
        .await
    }
}

async fn wait_for_final_transaction_result<'a: 'b, 'b, C>(
    tx_hash: Cow<'a, str>,
    client: &C,
    last_ledger_sequence: u32,
) -> XRPLHelperResult<TxVersionMap<'b>>
where
    C: XRPLAsyncClient,
{
    let mut validated_ledger_sequence = 0;
    let mut c = 0;
    while validated_ledger_sequence < last_ledger_sequence {
        c += 1;
        if c > 20 {
            return Err(XRPLSubmitAndWaitException::SubmissionTimeout {
                last_ledger_sequence,
                validated_ledger_sequence,
                prelim_result: "Transaction not included in ledger".into(),
            }
            .into());
        }
        validated_ledger_sequence = get_latest_validated_ledger_sequence(client).await?;
        // sleep for 1 second
        wait_seconds(1).await;
        let response = client
            .request(requests::tx::Tx::new(None, None, None, None, Some(tx_hash.clone())).into())
            .await?;
        if response.is_success() {
            if let Some(error) = response.error.as_ref() {
                if response.rpc_error() == Some(XRPLRpcError::TxnNotFound) {
                    continue;
                } else {
                    // Non-`txnNotFound` RPC error while polling — treat the
                    // rippled `error` name as the result code, and put the
                    // human-readable `error_message` alongside it.
                    return Err(submission_failed(
                        error.to_string(),
                        response.error_message.map(|m| m.to_string()),
                    )
                    .into());
                }
            } else {
                let result: TxVersionMap = response.try_into()?;
                let base = match &result {
                    TxVersionMap::Default(tx) => tx.base.clone(),
                    TxVersionMap::V1(tx) => tx.base.clone(),
                };
                let validated = base.validated.unwrap_or(false);
                if validated {
                    let meta = match result {
                        TxVersionMap::Default(ref tx) => tx.meta.clone(),
                        TxVersionMap::V1(ref tx) => tx.meta.clone(),
                    };
                    let meta = meta.expect("Expected field in the transaction metadata: meta");
                    if meta.transaction_result != "tesSUCCESS" {
                        // The ledger validated the tx and the transactor
                        // rejected it (tec*/tef*). The transaction_result is
                        // the code; no separate message is available on the
                        // meta object.
                        return Err(submission_failed(meta.transaction_result, None).into());
                    } else {
                        return Ok(result);
                    }
                }
            }
        }
    }
    // Polling loop exited without validating the tx (validated ledger
    // caught up with `last_ledger_sequence`). Semantically the same "we
    // gave up waiting" as the `c > 20` retry-cap path above, so surface
    // both via the same `SubmissionTimeout` variant with the ledger
    // context — keeping `SubmissionFailed` reserved for definite failures
    // that carry an actual rippled result code.
    Err(XRPLSubmitAndWaitException::SubmissionTimeout {
        last_ledger_sequence,
        validated_ledger_sequence,
        prelim_result: "Transaction not included in ledger".into(),
    }
    .into())
}

async fn get_signed_transaction<'a, T, F, C>(
    transaction: &mut T,
    client: &C,
    wallet: Option<&Wallet>,
    do_check_fee: Option<bool>,
    do_autofill: Option<bool>,
) -> XRPLHelperResult<()>
where
    T: Transaction<'a, F> + Model + Clone + DeserializeOwned + Debug,
    F: IntoEnumIterator + Serialize + Debug + PartialEq + Debug + Clone,
    C: XRPLAsyncClient,
{
    if transaction.get_common_fields().is_signed() {
        return Ok(());
    }
    if let Some(wallet) = wallet {
        if let Some(check_fee) = do_check_fee {
            if check_fee {
                check_txn_fee(transaction, client).await?;
            }
        }
        if let Some(do_autofill) = do_autofill {
            if do_autofill {
                autofill(transaction, client, None).await?;
            }
        }
        if transaction.get_common_fields().signers.as_ref().is_some() {
            sign(transaction, wallet, true)
        } else {
            sign(transaction, wallet, false)
        }
    } else {
        Err(XRPLSignTransactionException::WalletRequired.into())
    }
}

#[cfg(all(
    feature = "std",
    feature = "json-rpc",
    feature = "helpers",
    feature = "models",
    feature = "tokio-rt"
))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        asynch::{clients::AsyncJsonRpcClient, wallet::generate_faucet_wallet},
        handle_test_result,
        models::transactions::{account_set::AccountSet, CommonFields, TransactionType},
        utils::testing::{
            assertions, test_constants, test_network_operation, test_wallets, TestTimeouts,
        },
    };

    #[cfg(feature = "integration")]
    #[tokio::test]
    async fn test_submit_and_wait() {
        let client = AsyncJsonRpcClient::connect(test_constants::TESTNET_URL.parse().unwrap());

        // First try to generate a faucet wallet with timeout and error handling
        let wallet_result = test_network_operation(
            generate_faucet_wallet(&client, None, None, None, None),
            TestTimeouts::FAUCET,
            "faucet wallet generation for submit_and_wait test",
        )
        .await;

        let wallet = handle_test_result!(wallet_result, "test_submit_and_wait - wallet generation");

        // Create transaction using the new builder pattern
        let mut tx = AccountSet {
            common_fields: CommonFields::from_account(&wallet.classic_address)
                .with_transaction_type(TransactionType::AccountSet),
            domain: Some(test_constants::EXAMPLE_COM_HEX.into()),
            ..Default::default()
        };

        // Try submit_and_wait with timeout and error handling
        let submit_result = test_network_operation(
            submit_and_wait(&mut tx, &client, Some(&wallet), Some(true), Some(true)),
            TestTimeouts::TRANSACTION, // Longer timeout for transaction processing
            "submit and wait",
        )
        .await;

        handle_test_result!(submit_result, "test_submit_and_wait - submit operation");

        // Verify the transaction was properly processed using generic assertions
        assertions::assert_transaction_autofilled(&tx);
        assertions::assert_transaction_signed(&tx);
    }

    #[test]
    fn test_transaction_creation() {
        // Test the transaction builder pattern without network calls
        let wallet = test_wallets::create_test_wallet_unwrap();

        let tx = AccountSet {
            common_fields: CommonFields::from_account(&wallet.classic_address)
                .with_transaction_type(TransactionType::AccountSet)
                .with_fee("12".into())
                .with_sequence(100),
            domain: Some(test_constants::EXAMPLE_COM_HEX.into()),
            ..Default::default()
        };

        assert_eq!(tx.common_fields.account, wallet.classic_address);
        assert_eq!(
            tx.common_fields.transaction_type,
            TransactionType::AccountSet
        );
        assert_eq!(tx.common_fields.fee, Some("12".into()));
        assert_eq!(tx.common_fields.sequence, Some(100));
        assert_eq!(tx.domain, Some(test_constants::EXAMPLE_COM_HEX.into()));

        // Test that we can get common fields
        let common_fields = tx.get_common_fields();
        assert_eq!(common_fields.account, wallet.classic_address);
        assert!(!common_fields.is_signed()); // Should not be signed yet
    }

    #[test]
    fn test_submit_and_wait_parameters() {
        // Test parameter validation without network calls
        use crate::models::transactions::account_set::AccountSetFlag;

        let wallet = test_wallets::create_test_wallet_unwrap();

        // Test different parameter combinations
        let tx1 = AccountSet {
            common_fields: CommonFields::<AccountSetFlag>::from_account(&wallet.classic_address)
                .with_transaction_type(TransactionType::AccountSet)
                .with_fee("10".into())
                .with_sequence(1),
            domain: Some(test_constants::EXAMPLE_COM_HEX.into()),
            ..Default::default()
        };

        // Verify transaction structure
        assert_eq!(tx1.common_fields.account, wallet.classic_address);
        assert_eq!(tx1.common_fields.fee, Some("10".into()));
        assert_eq!(tx1.common_fields.sequence, Some(1));
        assert_eq!(tx1.domain, Some(test_constants::EXAMPLE_COM_HEX.into()));
    }

    #[tokio::test]
    async fn test_wait_for_final_transaction_result_retries_typed_txn_not_found() {
        use std::sync::Mutex;

        use crate::{
            asynch::clients::{exceptions::XRPLClientResult, XRPLClient},
            models::{requests::XRPLRequest, results::XRPLResponse},
        };
        use url::Url;

        struct MockClient {
            request_count: Mutex<usize>,
        }

        const HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

        impl XRPLClient for MockClient {
            async fn request_impl<'a: 'b, 'b>(
                &self,
                _request: XRPLRequest<'a>,
            ) -> XRPLClientResult<XRPLResponse<'b>> {
                let mut request_count = self.request_count.lock().unwrap();
                *request_count += 1;
                if *request_count == 1 {
                    Ok(serde_json::from_str(&alloc::format!(
                        r#"{{
                            "status":"success",
                            "result":{{
                                "ledger":{{
                                    "account_hash":"{HASH}",
                                    "close_flags":0,
                                    "close_time":0,
                                    "close_time_resolution":10,
                                    "closed":true,
                                    "ledger_hash":"{HASH}",
                                    "ledger_index":"2",
                                    "parent_close_time":0,
                                    "parent_hash":"{HASH}",
                                    "total_coins":"0",
                                    "transaction_hash":"{HASH}"
                                }},
                                "ledger_hash":"{HASH}",
                                "ledger_index":2,
                                "validated":true
                            }}
                        }}"#
                    ))?)
                } else {
                    let response: XRPLResponse<'b> = serde_json::from_str(
                        r#"{
                            "status":"success",
                            "error":"txnNotFound",
                            "error_code":29,
                            "error_message":"Transaction not found."
                        }"#,
                    )?;
                    assert_eq!(response.rpc_error(), Some(XRPLRpcError::TxnNotFound));
                    Ok(response)
                }
            }

            fn get_host(&self) -> Url {
                "http://127.0.0.1:5005".parse().unwrap()
            }
        }

        let client = MockClient {
            request_count: Mutex::new(0),
        };

        let result = wait_for_final_transaction_result(HASH.into(), &client, 1).await;
        match result {
            Err(crate::asynch::exceptions::XRPLHelperException::XRPLTransactionHelperError(
                crate::asynch::transaction::exceptions::XRPLTransactionHelperException::XRPLSubmitAndWaitError(
                    XRPLSubmitAndWaitException::SubmissionTimeout { prelim_result, .. },
                ),
            )) => {
                assert_eq!(prelim_result, "Transaction not included in ledger");
            }
            other => panic!("expected typed txnNotFound retry path, got {other:?}"),
        }
        assert_eq!(*client.request_count.lock().unwrap(), 2);
    }

    /// Verifies that when rippled returns a validated transaction with a
    /// non-tesSUCCESS `TransactionResult` (a `tec*` code, i.e. the ledger
    /// accepted the tx and the transactor rejected it), the polling loop
    /// surfaces it as `SubmissionFailed { result_code: "tec...", .. }` so
    /// callers can match on the code without substring-parsing.
    #[tokio::test]
    async fn test_wait_for_final_transaction_result_surfaces_tec_result_code() {
        use std::sync::Mutex;

        use crate::{
            asynch::clients::{exceptions::XRPLClientResult, XRPLClient},
            models::{requests::XRPLRequest, results::XRPLResponse},
        };
        use url::Url;

        struct MockClient {
            request_count: Mutex<usize>,
        }

        const HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

        impl XRPLClient for MockClient {
            async fn request_impl<'a: 'b, 'b>(
                &self,
                _request: XRPLRequest<'a>,
            ) -> XRPLClientResult<XRPLResponse<'b>> {
                let mut request_count = self.request_count.lock().unwrap();
                *request_count += 1;
                // First call is get_latest_validated_ledger_sequence, second
                // is the tx lookup. Serve a validated tx with a tec* meta on
                // the second call.
                if *request_count == 1 {
                    Ok(serde_json::from_str(&alloc::format!(
                        r#"{{
                            "status":"success",
                            "result":{{
                                "ledger":{{
                                    "account_hash":"{HASH}",
                                    "close_flags":0,
                                    "close_time":0,
                                    "close_time_resolution":10,
                                    "closed":true,
                                    "ledger_hash":"{HASH}",
                                    "ledger_index":"2",
                                    "parent_close_time":0,
                                    "parent_hash":"{HASH}",
                                    "total_coins":"0",
                                    "transaction_hash":"{HASH}"
                                }},
                                "ledger_hash":"{HASH}",
                                "ledger_index":2,
                                "validated":true
                            }}
                        }}"#
                    ))?)
                } else {
                    Ok(serde_json::from_str(&alloc::format!(
                        r#"{{
                            "status":"success",
                            "result":{{
                                "hash":"{HASH}",
                                "validated":true,
                                "meta":{{
                                    "AffectedNodes":[],
                                    "TransactionIndex":0,
                                    "TransactionResult":"tecBYTECODE_REJECTED"
                                }}
                            }}
                        }}"#
                    ))?)
                }
            }

            fn get_host(&self) -> Url {
                "http://127.0.0.1:5005".parse().unwrap()
            }
        }

        let client = MockClient {
            request_count: Mutex::new(0),
        };

        let result = wait_for_final_transaction_result(HASH.into(), &client, 1).await;
        match result {
            Err(crate::asynch::exceptions::XRPLHelperException::XRPLTransactionHelperError(
                crate::asynch::transaction::exceptions::XRPLTransactionHelperException::XRPLSubmitAndWaitError(
                    XRPLSubmitAndWaitException::SubmissionFailed { result_code, message },
                ),
            )) => {
                assert_eq!(result_code, "tecBYTECODE_REJECTED");
                // A validated-but-rejected tx has no separate engine_result_message
                // on the meta object, so message is None.
                assert_eq!(message, None);
            }
            other => panic!("expected tec* SubmissionFailed, got {other:?}"),
        }
    }

    /// Prelim `tem*` path (site 1): when `submit()` comes back with an
    /// `engine_result` starting with "tem", `send_reliable_submission` should
    /// short-circuit to `SubmissionFailed { result_code, message }` without
    /// entering the poll loop. Uses a mocked client so we can guarantee the
    /// tem* engine_result — real rippled either normalises typical bad-tx
    /// inputs to `tel*`/`tec*` or fails at the RPC layer before the submit
    /// response is materialised, making this branch impractical to exercise
    /// against a live node.
    #[tokio::test]
    async fn test_send_reliable_submission_surfaces_tem_engine_result() {
        use std::sync::Mutex;

        use crate::{
            asynch::clients::{exceptions::XRPLClientResult, XRPLClient},
            models::{requests::XRPLRequest, results::XRPLResponse},
        };
        use url::Url;

        struct MockClient {
            submit_calls: Mutex<usize>,
        }

        impl XRPLClient for MockClient {
            async fn request_impl<'a: 'b, 'b>(
                &self,
                _request: XRPLRequest<'a>,
            ) -> XRPLClientResult<XRPLResponse<'b>> {
                *self.submit_calls.lock().unwrap() += 1;
                Ok(serde_json::from_str(
                    r#"{
                        "status":"success",
                        "result":{
                            "engine_result":"temBAD_SIGNATURE",
                            "engine_result_code":-186,
                            "engine_result_message":"Bad signature.",
                            "tx_blob":"00",
                            "tx_json":{}
                        }
                    }"#,
                )?)
            }

            fn get_host(&self) -> Url {
                "http://127.0.0.1:5005".parse().unwrap()
            }
        }

        let client = MockClient {
            submit_calls: Mutex::new(0),
        };

        // Pre-signed AccountSet skips get_signed_transaction's autofill+sign.
        let wallet = test_wallets::create_test_wallet_unwrap();
        let mut tx = AccountSet {
            common_fields: CommonFields::from_account(&wallet.classic_address)
                .with_transaction_type(TransactionType::AccountSet)
                .with_fee("10".into())
                .with_sequence(1),
            ..Default::default()
        };
        tx.common_fields.last_ledger_sequence = Some(1);
        tx.common_fields.txn_signature = Some("00".into());
        tx.common_fields.signing_pub_key = Some("00".into());

        let result = submit_and_wait(&mut tx, &client, None, Some(false), Some(false)).await;
        match result {
            Err(crate::asynch::exceptions::XRPLHelperException::XRPLTransactionHelperError(
                crate::asynch::transaction::exceptions::XRPLTransactionHelperException::XRPLSubmitAndWaitError(
                    XRPLSubmitAndWaitException::SubmissionFailed { result_code, message },
                ),
            )) => {
                assert_eq!(result_code, "temBAD_SIGNATURE");
                assert_eq!(message.as_deref(), Some("Bad signature."));
            }
            other => panic!("expected tem* SubmissionFailed, got {other:?}"),
        }

        // send_reliable_submission should short-circuit on tem* without
        // entering the poll loop, so exactly one client request fired.
        assert_eq!(*client.submit_calls.lock().unwrap(), 1);
    }
}
