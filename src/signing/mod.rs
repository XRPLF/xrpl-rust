//! Pure cryptographic transaction signing.
//!
//! These functions don't touch the network — they only need the wallet's
//! private key plus the transaction. They live here (rather than under
//! `asynch::transaction`) so they compile and unit-test without enabling the
//! `helpers`/`json-rpc`/`websocket` features that pull in async client code.
//!
//! Re-exported from the legacy locations (`asynch::transaction::sign`,
//! `transaction::multisign`) for backward compatibility.

pub mod exceptions;

use core::fmt::Debug;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use serde::Serialize;
use serde::{de::DeserializeOwned, Deserialize};
use strum::IntoEnumIterator;

use crate::asynch::exceptions::XRPLHelperResult;
use crate::core::{
    addresscodec::{decode_classic_address, is_valid_xaddress, xaddress_to_classic_address},
    binarycodec::{encode_for_multisigning, encode_for_signing},
    keypairs::sign as keypairs_sign,
};
use crate::models::transactions::loan_set::CounterpartySignature;
use crate::models::transactions::loan_set::LoanSet;
use crate::models::XRPLModelException;
use crate::models::{
    transactions::{Signer, Transaction},
    Model,
};
use crate::utils::transactions::{
    get_transaction_field_value, set_transaction_field_value, validate_transaction_has_field,
};
use crate::wallet::Wallet;

use exceptions::{XRPLMultisignException, XRPLSignTransactionException};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
enum AccountFieldType {
    Account,
    Destination,
}

/// Sign a transaction with the given wallet's key.
///
/// Pure crypto — does not contact the network. When `multisign` is true the
/// signature is appended as a `Signer` entry; otherwise it goes into
/// `TxnSignature` directly.
pub fn sign<'a, T, F>(transaction: &mut T, wallet: &Wallet, multisign: bool) -> XRPLHelperResult<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Model + Serialize + DeserializeOwned + Clone + Debug,
{
    transaction.validate()?;

    if multisign {
        let serialized_for_signing =
            encode_for_multisigning(transaction, wallet.classic_address.clone().into())?;
        let serialized_bytes = hex::decode(serialized_for_signing)?;
        let signature = keypairs_sign(&serialized_bytes, &wallet.private_key)?;
        let signer = Signer::new(
            wallet.classic_address.clone(),
            signature,
            wallet.public_key.clone(),
        );
        transaction.get_mut_common_fields().signers = Some(vec![signer]);

        Ok(())
    } else {
        prepare_transaction(transaction, wallet)?;
        let serialized_for_signing = encode_for_signing(transaction)?;
        let serialized_bytes = hex::decode(serialized_for_signing)?;
        let signature = keypairs_sign(&serialized_bytes, &wallet.private_key)?;
        transaction.get_mut_common_fields().txn_signature = Some(signature.into());

        Ok(())
    }
}

/// Combine signer-signed copies of `transaction` into a single multisigned
/// transaction. `tx_list` must contain copies of `transaction` each signed by
/// a different signer.
pub fn multisign<'a, T, F>(transaction: &mut T, tx_list: &'a Vec<T>) -> XRPLHelperResult<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq + 'a,
    T: Transaction<'a, F>,
{
    let mut decoded_tx_signers = Vec::new();
    for tx in tx_list {
        let tx_signers = match tx.get_common_fields().signers.as_ref() {
            Some(signers) => signers,
            None => return Err(XRPLMultisignException::NoSigners.into()),
        };
        let tx_signer = match tx_signers.first() {
            Some(signer) => signer,
            None => return Err(XRPLMultisignException::NoSigners.into()),
        };
        decoded_tx_signers.push(tx_signer.clone());
    }
    decoded_tx_signers
        .sort_by_key(|signer| decode_classic_address(signer.account.as_ref()).unwrap());
    transaction.get_mut_common_fields().signers = Some(decoded_tx_signers);
    transaction.get_mut_common_fields().signing_pub_key = Some("".into());

    Ok(())
}

/// Signs a LoanSet transaction as the counterparty.
/// This function adds a counterparty signature to a LoanSet transaction that has
/// already been signed by the first party. The counterparty uses their wallet to
/// sign the transaction, which is required for multi-party loan agreements on the
/// XRP Ledger.
pub fn sign_loan_set_by_counterparty<'a>(
    transaction: &mut LoanSet<'a>,
    wallet: &Wallet,
    multisign: bool,
) -> XRPLHelperResult<()> {
    transaction.validate()?;

    reject_if_already_signed(transaction, wallet, multisign)?;

    if transaction.get_common_fields().txn_signature.is_none()
        || transaction.get_common_fields().signing_pub_key.is_none()
    {
        Err(XRPLModelException::MissingField(
            "txn_signature or signing_pub_key is required as first party signing".into(),
        ))?
    }

    if multisign {
        sign_multisign(transaction, wallet)?;
    } else {
        sign_single(transaction, wallet)?;
    }

    Ok(())
}

pub(crate) fn prepare_transaction<'a, T, F>(
    transaction: &mut T,
    wallet: &Wallet,
) -> XRPLHelperResult<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize + DeserializeOwned + Clone,
{
    let common_fields = transaction.get_mut_common_fields();
    common_fields.signing_pub_key = Some(wallet.public_key.clone().into());

    validate_account_xaddress(transaction, AccountFieldType::Account)?;
    if validate_transaction_has_field(transaction, "Destination").is_ok() {
        validate_account_xaddress(transaction, AccountFieldType::Destination)?;
    }

    let _ = convert_to_classic_address(transaction, "Unauthorize");
    let _ = convert_to_classic_address(transaction, "Authorize");
    // EscrowCancel, EscrowFinish
    let _ = convert_to_classic_address(transaction, "Owner");
    // SetRegularKey
    let _ = convert_to_classic_address(transaction, "RegularKey");

    Ok(())
}

fn validate_account_xaddress<'a, T, F>(
    prepared_transaction: &mut T,
    account_field: AccountFieldType,
) -> XRPLHelperResult<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize + DeserializeOwned + Clone,
{
    let (account_field_name, tag_field_name) = match account_field {
        AccountFieldType::Account => ("Account", "SourceTag"),
        AccountFieldType::Destination => ("Destination", "DestinationTag"),
    };
    let account_address = match account_field {
        AccountFieldType::Account => prepared_transaction.get_common_fields().account.clone(),
        AccountFieldType::Destination => {
            get_transaction_field_value(prepared_transaction, "Destination")?
        }
    };

    if is_valid_xaddress(&account_address) {
        let (address, tag, _) = xaddress_to_classic_address(&account_address)?;
        validate_transaction_has_field(prepared_transaction, account_field_name)?;
        set_transaction_field_value(prepared_transaction, account_field_name, address)?;

        if validate_transaction_has_field(prepared_transaction, tag_field_name).is_ok()
            && get_transaction_field_value(prepared_transaction, tag_field_name).unwrap_or(Some(0))
                != tag
        {
            Err(XRPLSignTransactionException::TagFieldMismatch(tag_field_name.to_string()).into())
        } else {
            set_transaction_field_value(prepared_transaction, tag_field_name, tag)?;

            Ok(())
        }
    } else {
        Ok(())
    }
}

fn convert_to_classic_address<'a, T, F>(
    transaction: &mut T,
    field_name: &str,
) -> XRPLHelperResult<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize + DeserializeOwned + Clone,
{
    let address = get_transaction_field_value::<F, _, String>(transaction, field_name)?;
    if is_valid_xaddress(&address) {
        let classic_address = xaddress_to_classic_address(&address)?.0;
        Ok(set_transaction_field_value(
            transaction,
            field_name,
            classic_address,
        )?)
    } else {
        Ok(())
    }
}

fn sign_single<'a>(transaction: &mut LoanSet<'a>, wallet: &Wallet) -> XRPLHelperResult<()> {
    let txn_signature = crate::core::keypairs::sign(
        &hex::decode(encode_for_signing(&transaction)?)?,
        &wallet.private_key,
    )?;
    transaction.counterparty_signature = Some(CounterpartySignature {
        signing_pub_key: Some(wallet.public_key.clone().into()),
        txn_signature: Some(txn_signature.into()),
        signers: None,
    });
    Ok(())
}

fn sign_multisign<'a>(transaction: &mut LoanSet<'a>, wallet: &Wallet) -> XRPLHelperResult<()> {
    let txn_signature = crate::core::keypairs::sign(
        &hex::decode(encode_for_multisigning(
            &transaction,
            wallet.classic_address.as_str().into(),
        )?)?,
        &wallet.private_key,
    )?;
    let signer = Signer {
        account: wallet.classic_address.clone(),
        signing_pub_key: wallet.public_key.clone(),
        txn_signature,
    };

    let cs = transaction
        .counterparty_signature
        .get_or_insert(CounterpartySignature {
            signing_pub_key: None,
            txn_signature: None,
            signers: None,
        });

    cs.signing_pub_key = None;
    cs.txn_signature = None;
    let signers = cs.signers.get_or_insert_with(Vec::new);
    signers.push(signer);

    // fallible sort instead of unwrap()-in-key-fn
    let mut keyed = signers
        .drain(..)
        .map(|s| crate::core::addresscodec::decode_classic_address(&s.account).map(|k| (k, s)))
        .collect::<Result<Vec<_>, _>>()?;

    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    *signers = keyed.into_iter().map(|(_, s)| s).collect();

    Ok(())
}

fn reject_if_already_signed<'a>(
    transaction: &LoanSet<'a>,
    wallet: &Wallet,
    multisign: bool,
) -> XRPLHelperResult<()> {
    let Some(cs) = transaction.counterparty_signature.as_ref() else {
        return Ok(());
    };

    match &cs.signers {
        Some(signers) if !multisign => Err(XRPLSignTransactionException::TransactionSigned(
            "Transaction already has multisign counterparty signatures; \
             cannot apply a single-sign counterparty signature."
                .into(),
        )
        .into()),
        Some(signers) if signers.iter().any(|s| s.account == wallet.classic_address) => {
            Err(XRPLSignTransactionException::TransactionSigned(
                "This counterparty account has already signed.".into(),
            )
            .into())
        }
        Some(_) => Ok(()),
        None => Err(XRPLSignTransactionException::TransactionSigned(
            "Transaction is already signed by the counterparty.".into(),
        )
        .into()),
    }
}
