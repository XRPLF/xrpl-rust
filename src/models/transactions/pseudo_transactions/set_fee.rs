use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    model::Model,
    transactions::{Transaction, TransactionType},
};

/// See SetFee:
/// `<https://xrpl.org/setfee.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SetFee<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(default = "TransactionType::set_fee")]
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
    /// Hex representation of the public key that corresponds to the
    /// private key used to sign this transaction. If an empty string,
    /// indicates a multi-signature is present in the Signers field instead.
    pub signing_pub_key: Option<&'a str>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction
    /// is made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub source_tag: Option<u32>,
    /// The signature that verifies this transaction as originating
    /// from the account it says it is from.
    pub txn_signature: Option<&'a str>,
    /// Set of bit-flags for this transaction.
    pub flags: Option<u32>,
    /// The custom fields for the SetFee model.
    ///
    /// See SetFee fields:
    /// `<https://xrpl.org/setfee.html#setfee-fields>`
    pub base_fee: XRPAmount<'a>,
    pub reference_fee_units: u32,
    pub reserve_base: u32,
    pub reserve_increment: u32,
    pub ledger_sequence: u32,
}

impl<'a> Model for SetFee<'a> {}

impl<'a> Transaction for SetFee<'a> {
    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }
}

impl<'a> SetFee<'a> {
    pub fn new(
        account: &'a str,
        base_fee: XRPAmount<'a>,
        reference_fee_units: u32,
        reserve_base: u32,
        reserve_increment: u32,
        ledger_sequence: u32,
        fee: Option<XRPAmount<'a>>,
        sequence: Option<u32>,
        signing_pub_key: Option<&'a str>,
        source_tag: Option<u32>,
        txn_signature: Option<&'a str>,
    ) -> Self {
        Self {
            transaction_type: TransactionType::SetFee,
            account,
            fee,
            sequence,
            signing_pub_key,
            source_tag,
            txn_signature,
            flags: None,
            base_fee,
            reference_fee_units,
            reserve_base,
            reserve_increment,
            ledger_sequence,
        }
    }
}
