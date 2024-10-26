use core::fmt::Debug;

use alloc::string::ToString;
use serde::{de::DeserializeOwned, Serialize};
use strum::IntoEnumIterator;

use crate::{
    models::transactions::{exceptions::XRPLTransactionFieldException, Transaction},
    XRPLSerdeJsonError,
};

use super::exceptions::XRPLUtilsResult;

pub fn get_transaction_field_value<'a, F, T, R>(
    transaction: &T,
    field_name: &str,
) -> XRPLUtilsResult<R>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize,
    R: DeserializeOwned,
{
    let txn_value = serde_json::to_value(transaction)?;
    let common_field_value = txn_value
        .get(field_name)
        .ok_or(XRPLSerdeJsonError::InvalidNoneError(field_name.to_string()))?;

    Ok(serde_json::from_value::<R>(common_field_value.clone())?)
}

pub fn set_transaction_field_value<'a, F, T, V>(
    transaction: &mut T,
    field_name: &str,
    field_value: V,
) -> XRPLUtilsResult<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize + DeserializeOwned,
    V: Serialize,
{
    match serde_json::to_value(&mut *transaction) {
        Ok(mut transaction_json) => {
            transaction_json[field_name] = serde_json::to_value(field_value)?;
            match serde_json::from_value::<T>(transaction_json) {
                Ok(val) => {
                    *transaction = val;
                    Ok(())
                }
                Err(error) => Err(error.into()),
            }
        }
        Err(error) => Err(error.into()),
    }
}

pub fn validate_transaction_has_field<'a, T, F>(
    transaction: &T,
    field_name: &str,
) -> XRPLUtilsResult<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize,
{
    serde_json::to_value(transaction)?
        .get(field_name)
        .ok_or(XRPLSerdeJsonError::InvalidNoneError(field_name.to_string()))?;

    Ok(())
}

pub fn validate_common_fied(common_field_name: &str) -> XRPLUtilsResult<()> {
    match common_field_name {
        "Account" | "TransactionType" | "Fee" | "Sequence" | "AccountTxnID" | "Flags"
        | "LastLedgerSequence" | "Memos" | "NetworkID" | "Signers" | "SourceTag"
        | "SigningPubKey" | "TicketSequence" | "TxnSignature" => Ok(()),
        _ => Err(
            XRPLTransactionFieldException::InvalidCommonField(common_field_name.to_string()).into(),
        ),
    }
}
