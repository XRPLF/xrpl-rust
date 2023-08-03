use crate::Err;
use alloc::vec::Vec;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use alloc::string::ToString;

use crate::models::amount::XRPAmount;
use crate::models::transactions::XRPLDepositPreauthException;
use crate::models::{
    model::Model,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

/// A DepositPreauth transaction gives another account pre-approval
/// to deliver payments to the sender of this transaction.
///
/// See DepositPreauth:
/// `<https://xrpl.org/depositpreauth.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DepositPreauth<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(default = "TransactionType::deposit_preauth")]
    pub transaction_type: TransactionType,
    /// The unique address of the account that initiated the transaction.
    pub account: &'a str,
    /// Integer amount of XRP, in drops, to be destroyed as a cost
    /// for distributing this transaction to the network. Some
    /// transaction types have different minimum requirements.
    /// See Transaction Cost for details.
    pub fee: Option<XRPAmount<'a>>,
    /// The sequence number of the account sending the transaction.
    /// A transaction is only valid if the Sequence number is exactly
    /// 1 greater than the previous transaction from the same account.
    /// The special case 0 means the transaction is using a Ticket instead.
    pub sequence: Option<u32>,
    /// Highest ledger index this transaction can appear in.
    /// Specifying this field places a strict upper limit on how long
    /// the transaction can wait to be validated or rejected.
    /// See Reliable Transaction Submission for more details.
    pub last_ledger_sequence: Option<u32>,
    /// Hash value identifying another transaction. If provided, this
    /// transaction is only valid if the sending account's
    /// previously-sent transaction matches the provided hash.
    #[serde(rename = "AccountTxnID")]
    pub account_txn_id: Option<&'a str>,
    /// Hex representation of the public key that corresponds to the
    /// private key used to sign this transaction. If an empty string,
    /// indicates a multi-signature is present in the Signers field instead.
    pub signing_pub_key: Option<&'a str>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction
    /// is made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub source_tag: Option<u32>,
    /// The sequence number of the ticket to use in place
    /// of a Sequence number. If this is provided, Sequence must
    /// be 0. Cannot be used with AccountTxnID.
    pub ticket_sequence: Option<u32>,
    /// The signature that verifies this transaction as originating
    /// from the account it says it is from.
    pub txn_signature: Option<&'a str>,
    /// Set of bit-flags for this transaction.
    pub flags: Option<u32>,
    /// Additional arbitrary information used to identify this transaction.
    pub memos: Option<Vec<Memo<'a>>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction is
    /// made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub signers: Option<Vec<Signer<'a>>>,
    /// The custom fields for the DepositPreauth model.
    ///
    /// See DepositPreauth fields:
    /// `<https://xrpl.org/depositpreauth.html#depositpreauth-fields>`
    pub authorize: Option<&'a str>,
    pub unauthorize: Option<&'a str>,
}

impl<'a> Default for DepositPreauth<'a> {
    fn default() -> Self {
        Self {
            transaction_type: TransactionType::DepositPreauth,
            account: Default::default(),
            fee: Default::default(),
            sequence: Default::default(),
            last_ledger_sequence: Default::default(),
            account_txn_id: Default::default(),
            signing_pub_key: Default::default(),
            source_tag: Default::default(),
            ticket_sequence: Default::default(),
            txn_signature: Default::default(),
            flags: Default::default(),
            memos: Default::default(),
            signers: Default::default(),
            authorize: Default::default(),
            unauthorize: Default::default(),
        }
    }
}

impl<'a: 'static> Model for DepositPreauth<'a> {
    fn get_errors(&self) -> Result<()> {
        match self._get_authorize_and_unauthorize_error() {
            Ok(_no_error) => Ok(()),
            Err(error) => Err!(error),
        }
    }
}

impl<'a> Transaction for DepositPreauth<'a> {
    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }
}

impl<'a> DepositPreauthError for DepositPreauth<'a> {
    fn _get_authorize_and_unauthorize_error(&self) -> Result<(), XRPLDepositPreauthException> {
        if (self.authorize.is_none() && self.unauthorize.is_none())
            || (self.authorize.is_some() && self.unauthorize.is_some())
        {
            Err(XRPLDepositPreauthException::DefineExactlyOneOf {
                field1: "authorize",
                field2: "unauthorize",
                resource: "",
            })
        } else {
            Ok(())
        }
    }
}

impl<'a> DepositPreauth<'a> {
    pub fn new(
        account: &'a str,
        fee: Option<XRPAmount<'a>>,
        sequence: Option<u32>,
        last_ledger_sequence: Option<u32>,
        account_txn_id: Option<&'a str>,
        signing_pub_key: Option<&'a str>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        txn_signature: Option<&'a str>,
        memos: Option<Vec<Memo<'a>>>,
        signers: Option<Vec<Signer<'a>>>,
        authorize: Option<&'a str>,
        unauthorize: Option<&'a str>,
    ) -> Self {
        Self {
            transaction_type: TransactionType::DepositPreauth,
            account,
            fee,
            sequence,
            last_ledger_sequence,
            account_txn_id,
            signing_pub_key,
            source_tag,
            ticket_sequence,
            txn_signature,
            flags: None,
            memos,
            signers,
            authorize,
            unauthorize,
        }
    }
}

pub trait DepositPreauthError {
    fn _get_authorize_and_unauthorize_error(&self) -> Result<(), XRPLDepositPreauthException>;
}

#[cfg(test)]
mod test_deposit_preauth_exception {

    use crate::models::Model;
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_authorize_and_unauthorize_error() {
        let deposit_preauth = DepositPreauth {
            transaction_type: TransactionType::DepositPreauth,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb",
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            account_txn_id: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
            flags: None,
            memos: None,
            signers: None,
            authorize: None,
            unauthorize: None,
        };

        assert_eq!(
            deposit_preauth.validate().unwrap_err().to_string().as_str(),
            "The field `authorize` can not be defined with `unauthorize`. Define exactly one of them. For more information see: "
        );
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    #[test]
    fn test_serialize() {
        let default_txn = DepositPreauth::new(
            "rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8",
            Some("10".into()),
            Some(2),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de"),
            None,
        );
        let default_json = r#"{"TransactionType":"DepositPreauth","Account":"rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8","Fee":"10","Sequence":2,"Authorize":"rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de"}"#;

        let txn_as_string = serde_json::to_string(&default_txn).unwrap();
        let txn_json = txn_as_string.as_str();

        assert_eq!(txn_json, default_json);
    }

    #[test]
    fn test_deserialize() {
        let default_txn = DepositPreauth::new(
            "rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8",
            Some("10".into()),
            Some(2),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de"),
            None,
        );
        let default_json = r#"{"TransactionType":"DepositPreauth","Account":"rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8","Authorize":"rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de","Fee":"10","Sequence":2}"#;

        let txn_as_obj: DepositPreauth = serde_json::from_str(default_json).unwrap();

        assert_eq!(txn_as_obj, default_txn);
    }
}
