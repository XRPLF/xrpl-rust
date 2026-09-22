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
    binarycodec::{
        encode_for_multisigning, encode_for_multisigning_counterparty, encode_for_signing,
        encode_for_signing_counterparty,
    },
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
///
/// # Verify the terms first
///
/// The signature commits the counterparty to every signing field of the
/// transaction — the principal, all fees and rates, the payment schedule and the
/// broker — so the loan terms cannot change afterwards without invalidating it.
/// This function only checks that a first-party signature is *present*; it does
/// not verify it, and it cannot know whether the terms are the ones your user
/// agreed to. A caller that receives a `LoanSet` from the other party is
/// responsible for showing the terms to its user before calling this.
pub fn sign_loan_set_by_counterparty<'a>(
    transaction: &mut LoanSet<'a>,
    wallet: &Wallet,
    multisign: bool,
) -> XRPLHelperResult<()> {
    transaction.validate()?;

    reject_if_already_signed(transaction, wallet, multisign)?;

    let cf = transaction.get_common_fields();

    let has_single_sig = cf.txn_signature.is_some() && cf.signing_pub_key.is_some();
    let has_multi_sig = cf.signers.as_ref().is_some_and(|s| !s.is_empty());

    if !has_single_sig && !has_multi_sig {
        return Err(XRPLModelException::MissingField(
            "first-party signature (TxnSignature+SigningPubKey) or Signers is required before counterparty signing".into(),
        )
        .into());
    }

    if multisign {
        sign_multisign(transaction, wallet)?;
    } else {
        sign_single(transaction, wallet)?;
    }

    Ok(())
}

/// Combines the counterparty signatures of several copies of the same
/// `LoanSet` into one transaction.
///
/// Each input must be the same transaction, signed by the first party, and
/// carrying exactly the counterparty `Signers` contributed by one counterparty
/// signer — which is what [`sign_loan_set_by_counterparty`] with
/// `multisign = true` produces. Use this when the counterparty signers sign
/// independently (on separate machines, say) and the results have to be merged
/// before submission; when every signer's wallet is available in one place,
/// calling [`sign_loan_set_by_counterparty`] once per signer on the same
/// transaction accumulates them directly.
///
/// The combined `Signers` list is sorted by decoded account ID, as rippled
/// requires.
pub fn combine_loan_set_counterparty_signers<'a>(
    transaction: &mut LoanSet<'a>,
    tx_list: &[LoanSet<'a>],
) -> XRPLHelperResult<()> {
    if tx_list.is_empty() {
        return Err(XRPLSignTransactionException::CombineCounterpartySigners(
            "there are 0 transactions to combine".into(),
        )
        .into());
    }

    let mut combined_signers: Vec<Signer> = Vec::new();
    for tx in tx_list {
        tx.validate()?;

        let cf = tx.get_common_fields();
        if cf.txn_signature.is_none() || cf.signing_pub_key.is_none() {
            return Err(XRPLSignTransactionException::CombineCounterpartySigners(
                "every transaction must first be signed by the first party".into(),
            )
            .into());
        }

        let signers = tx
            .counterparty_signature
            .as_ref()
            .and_then(|cs| cs.signers.as_ref())
            .filter(|signers| !signers.is_empty())
            .ok_or_else(|| {
                XRPLSignTransactionException::CombineCounterpartySigners(
                    "every transaction must carry counterparty Signers".into(),
                )
            })?;

        for signer in signers {
            if combined_signers
                .iter()
                .any(|existing| existing.account == signer.account)
            {
                return Err(XRPLSignTransactionException::CombineCounterpartySigners(
                    "the same counterparty account signed more than once".into(),
                )
                .into());
            }
            combined_signers.push(signer.clone());
        }
    }

    // The transactions have to be otherwise identical, or the signatures cover
    // different terms. Compare them with the counterparty signature stripped,
    // which is the only field allowed to differ.
    let mut reference = tx_list[0].clone();
    reference.counterparty_signature = None;
    for tx in &tx_list[1..] {
        let mut candidate = tx.clone();
        candidate.counterparty_signature = None;
        if candidate != reference {
            return Err(XRPLSignTransactionException::CombineCounterpartySigners(
                "the transactions to combine are not the same transaction".into(),
            )
            .into());
        }
    }

    // Fallible sort instead of unwrap()-in-key-fn.
    let mut keyed = combined_signers
        .drain(..)
        .map(|s| decode_classic_address(&s.account).map(|k| (k, s)))
        .collect::<Result<Vec<_>, _>>()?;
    keyed.sort_by(|a, b| a.0.cmp(&b.0));

    transaction.counterparty_signature = Some(CounterpartySignature {
        signing_pub_key: None,
        txn_signature: None,
        signers: Some(keyed.into_iter().map(|(_, s)| s).collect()),
    });

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
    // The counterparty role signs under its own `fixCleanup3_4_0` prefix, not the
    // transaction prefix.
    let txn_signature = crate::core::keypairs::sign(
        &hex::decode(encode_for_signing_counterparty(&transaction)?)?,
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
        &hex::decode(encode_for_multisigning_counterparty(
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
        Some(_) if !multisign => Err(XRPLSignTransactionException::TransactionSigned(
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

#[cfg(test)]
mod test_sign_loan_set_by_counterparty {
    use alloc::borrow::Cow;
    use alloc::string::ToString;

    use super::{combine_loan_set_counterparty_signers, sign, sign_loan_set_by_counterparty};
    use crate::models::transactions::loan_set::{CounterpartySignature, LoanSet};
    use crate::models::transactions::Signer;
    use crate::models::transactions::Transaction;
    use crate::wallet::Wallet;

    const LOAN_BROKER_ID: &str = "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD";

    fn broker_wallet() -> Wallet {
        Wallet::new("sEdSkooMk31MeTjbHVE7vLvgCpEMAdB", 0).unwrap()
    }

    fn borrower_wallet() -> Wallet {
        Wallet::new("sEdTLQkHAWpdS7FDk7EvuS7Mz8aSMRh", 0).unwrap()
    }

    fn second_borrower_wallet() -> Wallet {
        Wallet::new("sEd7DXaHkGQD8mz8xcRLDxfMLqCurif", 0).unwrap()
    }

    /// A `LoanSet` carrying the loan terms, as the first party would build it
    /// before signing.
    fn loan_set<'a>(broker: &Wallet, borrower: &Wallet) -> LoanSet<'a> {
        let mut tx = LoanSet::default();
        tx.common_fields.account = Cow::from(broker.classic_address.clone());
        tx.common_fields.transaction_type = crate::models::transactions::TransactionType::LoanSet;
        tx.common_fields.fee = Some("12".into());
        tx.common_fields.sequence = Some(8);
        tx.loan_broker_id = LOAN_BROKER_ID.into();
        tx.counterparty = Some(Cow::from(borrower.classic_address.clone()));
        tx.principal_requested = "1000".into();
        tx
    }

    #[test]
    fn test_single_sign_by_counterparty() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();
        let mut tx = loan_set(&broker, &borrower);

        sign(&mut tx, &broker, false).unwrap();
        sign_loan_set_by_counterparty(&mut tx, &borrower, false).unwrap();

        let cs = tx.counterparty_signature.as_ref().unwrap();
        assert_eq!(
            cs.signing_pub_key.as_deref(),
            Some(borrower.public_key.as_str())
        );
        assert!(cs.txn_signature.is_some());
        assert!(cs.signers.is_none());
    }

    /// Multisigned counterparty signatures accumulate in `Signers`, sorted by
    /// the decoded account id the way rippled expects.
    #[test]
    fn test_multisign_by_counterparty_sorts_signers() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();
        let second = second_borrower_wallet();
        let mut tx = loan_set(&broker, &borrower);

        sign(&mut tx, &broker, false).unwrap();
        sign_loan_set_by_counterparty(&mut tx, &borrower, true).unwrap();
        sign_loan_set_by_counterparty(&mut tx, &second, true).unwrap();

        let cs = tx.counterparty_signature.as_ref().unwrap();
        // The single-sign fields are cleared: the two modes are exclusive.
        assert!(cs.signing_pub_key.is_none());
        assert!(cs.txn_signature.is_none());

        let signers = cs.signers.as_ref().unwrap();
        assert_eq!(signers.len(), 2);
        let mut sorted = signers.clone();
        sorted.sort_by_key(|s| {
            crate::core::addresscodec::decode_classic_address(&s.account).unwrap()
        });
        assert_eq!(signers, &sorted);
    }

    /// The counterparty signs over a transaction the first party already signed,
    /// so signing an unsigned transaction is rejected.
    #[test]
    fn test_rejects_missing_first_party_signature() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();
        let mut tx = loan_set(&broker, &borrower);

        let err = sign_loan_set_by_counterparty(&mut tx, &borrower, false).unwrap_err();
        assert!(
            err.to_string().contains("first-party signature"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_rejects_second_single_sign() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();
        let mut tx = loan_set(&broker, &borrower);

        sign(&mut tx, &broker, false).unwrap();
        sign_loan_set_by_counterparty(&mut tx, &borrower, false).unwrap();

        let err = sign_loan_set_by_counterparty(&mut tx, &borrower, false).unwrap_err();
        assert!(
            err.to_string()
                .contains("already signed by the counterparty"),
            "unexpected error: {}",
            err
        );
    }

    /// A single-sign counterparty signature cannot be added on top of multisign
    /// counterparty signatures.
    #[test]
    fn test_rejects_single_sign_over_multisign() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();
        let mut tx = loan_set(&broker, &borrower);

        sign(&mut tx, &broker, false).unwrap();
        sign_loan_set_by_counterparty(&mut tx, &borrower, true).unwrap();

        let err = sign_loan_set_by_counterparty(&mut tx, &borrower, false).unwrap_err();
        assert!(
            err.to_string()
                .contains("multisign counterparty signatures"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_rejects_duplicate_multisign_by_same_account() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();
        let mut tx = loan_set(&broker, &borrower);

        sign(&mut tx, &broker, false).unwrap();
        sign_loan_set_by_counterparty(&mut tx, &borrower, true).unwrap();

        let err = sign_loan_set_by_counterparty(&mut tx, &borrower, true).unwrap_err();
        assert!(
            err.to_string()
                .contains("This counterparty account has already signed"),
            "unexpected error: {}",
            err
        );
    }

    /// A first-party multisigned transaction (`Signers` set, no `TxnSignature`)
    /// is also a valid starting point for counterparty signing.
    #[test]
    fn test_accepts_multisigned_first_party() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();
        let mut tx = loan_set(&broker, &borrower);

        sign(&mut tx, &broker, true).unwrap();
        assert!(tx.get_common_fields().signers.is_some());

        sign_loan_set_by_counterparty(&mut tx, &borrower, false).unwrap();
        assert!(tx
            .counterparty_signature
            .as_ref()
            .unwrap()
            .txn_signature
            .is_some());
    }

    /// An empty `Signers` list on the counterparty signature is not a signature,
    /// so the model validation run by `sign_loan_set_by_counterparty` rejects it.
    #[test]
    fn test_rejects_invalid_counterparty_signature_model() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();
        let mut tx = loan_set(&broker, &borrower);

        sign(&mut tx, &broker, false).unwrap();
        tx.counterparty_signature = Some(CounterpartySignature {
            signing_pub_key: None,
            txn_signature: None,
            signers: Some(alloc::vec::Vec::new()),
        });

        assert!(sign_loan_set_by_counterparty(&mut tx, &borrower, false).is_err());
    }

    /// Counterparty signers that signed independently can be merged into one
    /// transaction.
    ///
    /// xrpl.js reference: `combineLoanSetCounterpartySigners` in
    /// `packages/xrpl/src/Wallet/counterpartySigner.ts`.
    #[test]
    fn test_combine_counterparty_signers() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();
        let second = second_borrower_wallet();

        let mut base = loan_set(&broker, &borrower);
        sign(&mut base, &broker, false).unwrap();

        // Each counterparty signs its own copy, as it would on its own machine.
        let mut first_copy = base.clone();
        sign_loan_set_by_counterparty(&mut first_copy, &borrower, true).unwrap();
        let mut second_copy = base.clone();
        sign_loan_set_by_counterparty(&mut second_copy, &second, true).unwrap();

        let mut combined = base.clone();
        combine_loan_set_counterparty_signers(&mut combined, &[first_copy, second_copy]).unwrap();

        let cs = combined.counterparty_signature.as_ref().unwrap();
        assert!(cs.signing_pub_key.is_none());
        assert!(cs.txn_signature.is_none());

        let signers = cs.signers.as_ref().unwrap();
        assert_eq!(signers.len(), 2);
        let mut sorted = signers.clone();
        sorted.sort_by_key(|s| {
            crate::core::addresscodec::decode_classic_address(&s.account).unwrap()
        });
        assert_eq!(
            signers, &sorted,
            "Signers must be sorted by decoded account id"
        );
    }

    #[test]
    fn test_combine_rejects_empty_list() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();
        let mut combined = loan_set(&broker, &borrower);

        let err = combine_loan_set_counterparty_signers(&mut combined, &[]).unwrap_err();
        assert!(
            err.to_string().contains("0 transactions to combine"),
            "unexpected error: {}",
            err
        );
    }

    /// A single-signed counterparty signature carries no `Signers`, so it
    /// cannot take part in a combine.
    #[test]
    fn test_combine_rejects_single_signed_counterparty() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();

        let mut single = loan_set(&broker, &borrower);
        sign(&mut single, &broker, false).unwrap();
        sign_loan_set_by_counterparty(&mut single, &borrower, false).unwrap();

        let mut combined = loan_set(&broker, &borrower);
        let err = combine_loan_set_counterparty_signers(&mut combined, &[single]).unwrap_err();
        assert!(
            err.to_string().contains("must carry counterparty Signers"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_combine_rejects_unsigned_first_party() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();

        // Counterparty Signers present, but the first party never signed.
        let mut unsigned = loan_set(&broker, &borrower);
        unsigned.counterparty_signature = Some(CounterpartySignature {
            signing_pub_key: None,
            txn_signature: None,
            signers: Some(alloc::vec![Signer {
                account: borrower.classic_address.clone(),
                signing_pub_key: borrower.public_key.clone(),
                txn_signature: "DEADBEEF".into(),
            }]),
        });

        let mut combined = loan_set(&broker, &borrower);
        let err = combine_loan_set_counterparty_signers(&mut combined, &[unsigned]).unwrap_err();
        assert!(
            err.to_string().contains("signed by the first party"),
            "unexpected error: {}",
            err
        );
    }

    /// The signatures cover the loan terms, so combining copies that disagree
    /// on those terms would produce a transaction nobody signed.
    #[test]
    fn test_combine_rejects_mismatched_transactions() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();
        let second = second_borrower_wallet();

        let mut first_copy = loan_set(&broker, &borrower);
        sign(&mut first_copy, &broker, false).unwrap();
        sign_loan_set_by_counterparty(&mut first_copy, &borrower, true).unwrap();

        let mut tampered = loan_set(&broker, &borrower);
        tampered.principal_requested = "999999".into();
        sign(&mut tampered, &broker, false).unwrap();
        sign_loan_set_by_counterparty(&mut tampered, &second, true).unwrap();

        let mut combined = loan_set(&broker, &borrower);
        let err = combine_loan_set_counterparty_signers(&mut combined, &[first_copy, tampered])
            .unwrap_err();
        assert!(
            err.to_string().contains("not the same transaction"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_combine_rejects_duplicate_signer() {
        let broker = broker_wallet();
        let borrower = borrower_wallet();

        let mut base = loan_set(&broker, &borrower);
        sign(&mut base, &broker, false).unwrap();

        let mut first_copy = base.clone();
        sign_loan_set_by_counterparty(&mut first_copy, &borrower, true).unwrap();
        let duplicate = first_copy.clone();

        let mut combined = base;
        let err = combine_loan_set_counterparty_signers(&mut combined, &[first_copy, duplicate])
            .unwrap_err();
        assert!(
            err.to_string().contains("signed more than once"),
            "unexpected error: {}",
            err
        );
    }
}
