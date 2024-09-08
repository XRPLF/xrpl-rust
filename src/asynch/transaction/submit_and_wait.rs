use core::fmt::Debug;

use alloc::{borrow::Cow, format};
use anyhow::{Ok, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use strum::IntoEnumIterator;

use crate::{
    asynch::{
        clients::AsyncClient,
        ledger::get_latest_validated_ledger_sequence,
        transaction::{
            autofill, check_txn_fee,
            exceptions::{XRPLSignTransactionException, XRPLSubmitAndWaitException},
            sign, submit,
        },
    },
    models::{requests, results, transactions::Transaction, Model},
    wallet::Wallet,
    Err,
};

pub async fn submit_and_wait<'a: 'b, 'b, T, F, C>(
    transaction: &'b mut T,
    client: &C,
    wallet: Option<&Wallet>,
    check_fee: Option<bool>,
    autofill: Option<bool>,
) -> Result<results::tx::Tx<'b>>
where
    T: Transaction<'a, F> + Model + Clone + DeserializeOwned + Debug,
    F: IntoEnumIterator + Serialize + Debug + PartialEq + Debug + Clone + 'a,
    C: AsyncClient,
{
    get_signed_transaction(transaction, client, wallet, check_fee, autofill).await?;
    send_reliable_submission(transaction, client).await
}

async fn send_reliable_submission<'a: 'b, 'b, T, F, C>(
    transaction: &'b mut T,
    client: &C,
) -> Result<results::tx::Tx<'b>>
where
    T: Transaction<'a, F> + Model + Clone + DeserializeOwned + Debug,
    F: IntoEnumIterator + Serialize + Debug + PartialEq + Debug + Clone + 'a,
    C: AsyncClient,
{
    let tx_hash = transaction.get_hash()?;
    let submit_response = submit(transaction, client).await?;
    let prelim_result = submit_response.engine_result;
    if &prelim_result[0..3] == "tem" {
        let message = format!(
            "{}: {}",
            prelim_result, submit_response.engine_result_message
        );
        Err!(XRPLSubmitAndWaitException::SubmissionFailed(message.into()))
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
) -> Result<results::tx::Tx<'b>>
where
    C: AsyncClient,
{
    let mut validated_ledger_sequence = 0;
    let mut c = 0;
    while validated_ledger_sequence < last_ledger_sequence {
        c += 1;
        if c > 20 {
            panic!()
        }
        validated_ledger_sequence = get_latest_validated_ledger_sequence(client).await?;
        // sleep for 1 second
        #[cfg(feature = "embassy-rt")]
        embassy_time::Timer::after_secs(1).await;
        #[cfg(any(
            feature = "tokio-rt",
            all(feature = "embassy-rt", feature = "tokio-rt")
        ))]
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let response = client
            .request(requests::tx::Tx::new(None, None, None, None, Some(tx_hash.clone())).into())
            .await?;
        if response.is_success() {
            if let Some(error) = response.error {
                if error == "txnNotFound" {
                    continue;
                } else {
                    return Err!(XRPLSubmitAndWaitException::SubmissionFailed(
                        format!("{}: {}", error, response.error_message.unwrap_or("".into()))
                            .into()
                    ));
                }
            } else {
                let opt_result = response.try_into_opt_result::<results::tx::Tx>()?;
                let validated = opt_result.try_get_typed("validated")?;
                if validated {
                    let result = opt_result.try_into_result()?;
                    let return_code = match result.meta.get("TransactionResult") {
                        Some(Value::String(s)) => s,
                        _ => {
                            return Err!(XRPLSubmitAndWaitException::ExpectedFieldInTxMeta(
                                "TransactionResult".into()
                            ));
                        }
                    };
                    if return_code != "tesSUCCESS" {
                        return Err!(XRPLSubmitAndWaitException::SubmissionFailed(
                            return_code.into()
                        ));
                    } else {
                        return Ok(result);
                    }
                }
            }
        }
    }
    return Err!(XRPLSubmitAndWaitException::SubmissionFailed(
        "Transaction not included in ledger".into()
    ));
}

async fn get_signed_transaction<'a, T, F, C>(
    transaction: &mut T,
    client: &C,
    wallet: Option<&Wallet>,
    do_check_fee: Option<bool>,
    do_autofill: Option<bool>,
) -> Result<()>
where
    T: Transaction<'a, F> + Model + Clone + DeserializeOwned + Debug,
    F: IntoEnumIterator + Serialize + Debug + PartialEq + Debug + Clone,
    C: AsyncClient,
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
        Err!(XRPLSignTransactionException::WalletRequired)
    }
}

#[cfg(all(
    feature = "std",
    feature = "json-rpc",
    feature = "wallet-helpers",
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
