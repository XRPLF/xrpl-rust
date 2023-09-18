use crate::Err;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::transactions::XRPLEscrowFinishException;
use crate::models::{
    amount::XRPAmount,
    model::Model,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

/// Finishes an Escrow and delivers XRP from a held payment to the recipient.
///
/// See EscrowFinish:
/// `<https://xrpl.org/escrowfinish.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct EscrowFinish<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(default = "TransactionType::escrow_finish")]
    pub transaction_type: TransactionType,
    /// The unique address of the account that initiated the transaction.
    pub account: Cow<'a, str>,
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
    pub account_txn_id: Option<Cow<'a, str>>,
    /// Hex representation of the public key that corresponds to the
    /// private key used to sign this transaction. If an empty string,
    /// indicates a multi-signature is present in the Signers field instead.
    pub signing_pub_key: Option<Cow<'a, str>>,
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
    pub txn_signature: Option<Cow<'a, str>>,
    /// Set of bit-flags for this transaction.
    pub flags: Option<u32>,
    /// Additional arbitrary information used to identify this transaction.
    pub memos: Option<Vec<Memo>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction is
    /// made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub signers: Option<Vec<Signer<'a>>>,
    /// The custom fields for the EscrowFinish model.
    ///
    /// See EscrowFinish fields:
    /// `<https://xrpl.org/escrowfinish.html#escrowfinish-fields>`
    pub owner: Cow<'a, str>,
    pub offer_sequence: u32,
    pub condition: Option<Cow<'a, str>>,
    pub fulfillment: Option<Cow<'a, str>>,
}

impl<'a> Default for EscrowFinish<'a> {
    fn default() -> Self {
        Self {
            transaction_type: TransactionType::EscrowFinish,
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
            owner: Default::default(),
            offer_sequence: Default::default(),
            condition: Default::default(),
            fulfillment: Default::default(),
        }
    }
}

impl<'a: 'static> Model for EscrowFinish<'a> {
    fn get_errors(&self) -> Result<()> {
        match self._get_condition_and_fulfillment_error() {
            Ok(_) => Ok(()),
            Err(error) => Err!(error),
        }
    }
}

impl<'a> Transaction for EscrowFinish<'a> {
    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }
}

impl<'a> EscrowFinishError for EscrowFinish<'a> {
    fn _get_condition_and_fulfillment_error(&self) -> Result<(), XRPLEscrowFinishException> {
        if (self.condition.is_some() && self.fulfillment.is_none())
            || (self.condition.is_none() && self.condition.is_some())
        {
            Err(XRPLEscrowFinishException::FieldRequiresField {
                field1: "condition".into(),
                field2: "fulfillment".into(),
                resource: "".into(),
            })
        } else {
            Ok(())
        }
    }
}

impl<'a> EscrowFinish<'a> {
    pub fn new(
        account: Cow<'a, str>,
        owner: Cow<'a, str>,
        offer_sequence: u32,
        fee: Option<XRPAmount<'a>>,
        sequence: Option<u32>,
        last_ledger_sequence: Option<u32>,
        account_txn_id: Option<Cow<'a, str>>,
        signing_pub_key: Option<Cow<'a, str>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        txn_signature: Option<Cow<'a, str>>,
        memos: Option<Vec<Memo>>,
        signers: Option<Vec<Signer<'a>>>,
        condition: Option<Cow<'a, str>>,
        fulfillment: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            transaction_type: TransactionType::EscrowFinish,
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
            owner,
            offer_sequence,
            condition,
            fulfillment,
        }
    }
}

pub trait EscrowFinishError {
    fn _get_condition_and_fulfillment_error(&self) -> Result<(), XRPLEscrowFinishException>;
}

#[cfg(test)]
mod test_escrow_finish_errors {

    use crate::models::Model;
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_condition_and_fulfillment_error() {
        let escrow_finish = EscrowFinish {
            transaction_type: TransactionType::EscrowCancel,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
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
            owner: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            offer_sequence: 10,
            condition: Some(
                "A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100"
                    .into(),
            ),
            fulfillment: None,
        };

        assert_eq!(
            escrow_finish.validate().unwrap_err().to_string().as_str(),
            "For the field `condition` to be defined it is required to also define the field `fulfillment`. For more information see: "
        );
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    #[test]
    fn test_serialize() {
        let default_txn = EscrowFinish::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            7,
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
            Some(
                "A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100"
                    .into(),
            ),
            Some("A0028000".into()),
        );
        let default_json = r#"{"TransactionType":"EscrowFinish","Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","Owner":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","OfferSequence":7,"Condition":"A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100","Fulfillment":"A0028000"}"#;

        let txn_as_string = serde_json::to_string(&default_txn).unwrap();
        let txn_json = txn_as_string.as_str();

        assert_eq!(txn_json, default_json);
    }

    #[test]
    fn test_deserialize() {
        let default_txn = EscrowFinish::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            7,
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
            Some(
                "A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100"
                    .into(),
            ),
            Some("A0028000".into()),
        );
        let default_json = r#"{"TransactionType":"EscrowFinish","Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","Owner":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","OfferSequence":7,"Condition":"A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100","Fulfillment":"A0028000"}"#;

        let txn_as_obj: EscrowFinish = serde_json::from_str(default_json).unwrap();

        assert_eq!(txn_as_obj, default_txn);
    }
}
