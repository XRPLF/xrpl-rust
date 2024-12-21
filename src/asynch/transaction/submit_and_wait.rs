use core::fmt::Debug;

use alloc::{borrow::Cow, format};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
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
        results::tx::{TxMetaV1, TxV1, TxVersionMap},
        transactions::Transaction,
        Model,
    },
    wallet::Wallet,
};

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
        let message = format!(
            "{}: {}",
            prelim_result, submit_response.engine_result_message
        );
        Err(XRPLSubmitAndWaitException::SubmissionFailed(message).into())
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
            if let Some(error) = response.error {
                if error == "txnNotFound" {
                    continue;
                } else {
                    return Err(XRPLSubmitAndWaitException::SubmissionFailed(format!(
                        "{}: {}",
                        error,
                        response.error_message.unwrap_or("".into())
                    ))
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
                        TxVersionMap::V1(TxV1 {
                            meta: Some(TxMetaV1::Json(ref meta)),
                            ..
                        }) => Some(meta.clone()),
                        _ => None,
                    };
                    let meta = meta.unwrap(); // safe to unwrap because we requested using non-binary mode and we checked that the transaction was validated
                    let return_code = match meta.get("TransactionResult") {
                        Some(Value::String(s)) => s,
                        _ => {
                            return Err(XRPLSubmitAndWaitException::ExpectedFieldInTxMeta(
                                "TransactionResult".into(),
                            )
                            .into());
                        }
                    };
                    if return_code != "tesSUCCESS" {
                        return Err(XRPLSubmitAndWaitException::SubmissionFailed(
                            return_code.into(),
                        )
                        .into());
                    } else {
                        return Ok(result);
                    }
                }
            }
        }
    }
    Err(
        XRPLSubmitAndWaitException::SubmissionFailed("Transaction not included in ledger".into())
            .into(),
    )
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
mod test_submit_and_wait {
    use super::*;
    use crate::{
        asynch::{clients::AsyncJsonRpcClient, wallet::generate_faucet_wallet},
        models::transactions::account_set::AccountSet,
    };

    #[tokio::test]
    async fn test_submit_and_wait() {
        let client = AsyncJsonRpcClient::connect("https://testnet.xrpl-labs.com/".parse().unwrap());
        let wallet = generate_faucet_wallet(&client, None, None, None, None)
            .await
            .unwrap();
        let mut tx = AccountSet::new(
            Cow::from(wallet.classic_address.clone()),
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
            Some("6578616d706c652e636f6d".into()), // "example.com"
            None,
            None,
            None,
            None,
            None,
            None,
        );
        submit_and_wait(&mut tx, &client, Some(&wallet), Some(true), Some(true))
            .await
            .unwrap();
    }
}
