use crate::_serde::txn_flags;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::amount::XRPAmount;
use crate::models::{
    model::Model,
    transactions::{Flag, Transaction, TransactionType},
};

#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum EnableAmendmentFlag {
    /// Support for this amendment increased to at least 80% of trusted
    /// validators starting with this ledger version.
    TfGotMajority = 0x00010000,
    /// Support for this amendment decreased to less than 80% of trusted
    /// validators starting with this ledger version.
    TfLostMajority = 0x00020000,
}

/// See EnableAmendment:
/// `<https://xrpl.org/enableamendment.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct EnableAmendment<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(default = "TransactionType::enable_amendment")]
    transaction_type: TransactionType,
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
    /// Hex representation of the public key that corresponds to the
    /// private key used to sign this transaction. If an empty string,
    /// indicates a multi-signature is present in the Signers field instead.
    pub signing_pub_key: Option<Cow<'a, str>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction
    /// is made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub source_tag: Option<u32>,
    /// The signature that verifies this transaction as originating
    /// from the account it says it is from.
    pub txn_signature: Option<Cow<'a, str>>,
    /// Set of bit-flags for this transaction.
    #[serde(default)]
    #[serde(with = "txn_flags")]
    pub flags: Option<Vec<EnableAmendmentFlag>>,
    /// The custom fields for the EnableAmendment model.
    ///
    /// See EnableAmendment fields:
    /// `<https://xrpl.org/enableamendment.html#enableamendment-fields>`
    pub amendment: Cow<'a, str>,
    pub ledger_sequence: u32,
}

impl<'a> Model for EnableAmendment<'a> {}

impl<'a> Transaction for EnableAmendment<'a> {
    fn has_flag(&self, flag: &Flag) -> bool {
        match flag {
            Flag::EnableAmendment(enable_amendment_flag) => match enable_amendment_flag {
                EnableAmendmentFlag::TfGotMajority => self
                    .flags
                    .as_ref()
                    .unwrap()
                    .contains(&EnableAmendmentFlag::TfGotMajority),
                EnableAmendmentFlag::TfLostMajority => self
                    .flags
                    .as_ref()
                    .unwrap()
                    .contains(&EnableAmendmentFlag::TfLostMajority),
            },
            _ => false,
        }
    }

    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }
}

impl<'a> EnableAmendment<'a> {
    pub fn new(
        account: Cow<'a, str>,
        amendment: Cow<'a, str>,
        ledger_sequence: u32,
        fee: Option<XRPAmount<'a>>,
        sequence: Option<u32>,
        signing_pub_key: Option<Cow<'a, str>>,
        source_tag: Option<u32>,
        txn_signature: Option<Cow<'a, str>>,
        flags: Option<Vec<EnableAmendmentFlag>>,
    ) -> Self {
        Self {
            transaction_type: TransactionType::EnableAmendment,
            account,
            fee,
            sequence,
            signing_pub_key,
            source_tag,
            txn_signature,
            flags,
            amendment,
            ledger_sequence,
        }
    }
}
