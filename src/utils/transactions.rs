use core::fmt::Debug;

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use strum::IntoEnumIterator;

use crate::{
    models::transactions::{exceptions::XRPLTransactionFieldException, Transaction},
    Err,
};

pub fn get_transaction_field_value<'a, F, T, R>(transaction: &T, field_name: &str) -> Result<R>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize,
    R: DeserializeOwned,
{
    match serde_json::to_value(transaction) {
        Ok(transaction_json) => match transaction_json.get(field_name) {
            Some(common_field_value) => {
                match serde_json::from_value::<R>(common_field_value.clone()) {
                    Ok(val) => Ok(val),
                    Err(error) => Err!(error),
                }
            }
            None => Err!(XRPLTransactionFieldException::FieldMissing(field_name)),
        },
        Err(error) => Err!(error),
    }
}

pub fn set_transaction_field_value<'a, F, T, V>(
    transaction: &mut T,
    field_name: &str,
    field_value: V,
) -> Result<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize + DeserializeOwned,
    V: Serialize,
{
    match serde_json::to_value(&mut *transaction) {
        Ok(mut transaction_json) => {
            transaction_json[field_name] = match serde_json::to_value(field_value) {
                Ok(json_value) => json_value,
                Err(error) => return Err!(error),
            };
            match serde_json::from_value::<T>(transaction_json) {
                Ok(val) => {
                    *transaction = val;
                    Ok(())
                }
                Err(error) => Err!(error),
            }
        }
        Err(error) => Err!(error),
    }
}

pub fn validate_transaction_has_field<'a, T, F>(transaction: &T, field_name: &str) -> Result<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize,
{
    match serde_json::to_value(transaction) {
        Ok(transaction_json) => match transaction_json.get(field_name) {
            Some(_) => Ok(()),
            None => Err!(XRPLTransactionFieldException::FieldMissing(field_name)),
        },
        Err(error) => Err!(error),
    }
}

pub fn validate_common_fied(common_field_name: &str) -> Result<()> {
    match common_field_name {
        "Account" | "TransactionType" | "Fee" | "Sequence" | "AccountTxnID" | "Flags"
        | "LastLedgerSequence" | "Memos" | "NetworkID" | "Signers" | "SourceTag"
        | "SigningPubKey" | "TicketSequence" | "TxnSignature" => Ok(()),
        _ => Err!(XRPLTransactionFieldException::InvalidCommonField(
            common_field_name
        )),
    }
}
