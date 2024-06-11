pub mod account_delete;
pub mod account_set;
pub mod check_cancel;
pub mod check_cash;
pub mod check_create;
pub mod deposit_preauth;
pub mod escrow_cancel;
pub mod escrow_create;
pub mod escrow_finish;
pub mod exceptions;
pub mod nftoken_accept_offer;
pub mod nftoken_burn;
pub mod nftoken_cancel_offer;
pub mod nftoken_create_offer;
pub mod nftoken_mint;
pub mod offer_cancel;
pub mod offer_create;
pub mod payment;
pub mod payment_channel_claim;
pub mod payment_channel_create;
pub mod payment_channel_fund;
pub mod pseudo_transactions;
pub mod set_regular_key;
pub mod signer_list_set;
pub mod ticket_create;
pub mod trust_set;

pub use account_delete::*;
pub use account_set::*;
pub use check_cancel::*;
pub use check_cash::*;
pub use check_create::*;
pub use deposit_preauth::*;
pub use escrow_cancel::*;
pub use escrow_create::*;
pub use escrow_finish::*;
pub use exceptions::*;
pub use nftoken_accept_offer::*;
pub use nftoken_burn::*;
pub use nftoken_cancel_offer::*;
pub use nftoken_create_offer::*;
pub use nftoken_mint::*;
pub use offer_cancel::*;
pub use offer_create::*;
pub use payment::*;
pub use payment_channel_claim::*;
pub use payment_channel_create::*;
pub use payment_channel_fund::*;
pub use pseudo_transactions::*;

pub use set_regular_key::*;
pub use signer_list_set::*;
pub use ticket_create::*;
pub use trust_set::*;

use crate::models::amount::XRPAmount;
use crate::{_serde::txn_flags, serde_with_tag};
use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::Result;
use derive_new::new;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display};

use super::FlagCollection;

/// Enum containing the different Transaction types.
#[derive(Debug, Clone, Serialize, Deserialize, Display, PartialEq, Eq)]
pub enum TransactionType {
    AccountDelete,
    AccountSet,
    CheckCancel,
    CheckCash,
    CheckCreate,
    DepositPreauth,
    EscrowCancel,
    EscrowCreate,
    EscrowFinish,
    NFTokenAcceptOffer,
    NFTokenBurn,
    NFTokenCancelOffer,
    NFTokenCreateOffer,
    NFTokenMint,
    OfferCancel,
    OfferCreate,
    Payment,
    PaymentChannelClaim,
    PaymentChannelCreate,
    PaymentChannelFund,
    SetRegularKey,
    SignerListSet,
    TicketCreate,
    TrustSet,

    // Psuedo-Transaction types,
    EnableAmendment,
    SetFee,
    UNLModify,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
#[serde(rename_all = "PascalCase")]
pub struct PreparedTransaction<'a, T> {
    #[serde(flatten)]
    pub transaction: T,
    /// Hex representation of the public key that corresponds to the
    /// private key used to sign this transaction. If an empty string,
    /// indicates a multi-signature is present in the Signers field instead.
    pub signing_pub_key: Cow<'a, str>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
#[serde(rename_all = "PascalCase")]
pub struct SignedTransaction<'a, T> {
    #[serde(flatten)]
    pub prepared_transaction: PreparedTransaction<'a, T>,
    /// The signature that verifies this transaction as originating
    /// from the account it says it is from.
    pub txn_signature: Cow<'a, str>,
}

/// The base fields for all transaction models.
///
/// See Transaction Common Fields:
/// `<https://xrpl.org/transaction-common-fields.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CommonFields<'a, F>
where
    F: IntoEnumIterator + Serialize + core::fmt::Debug,
{
    /// The unique address of the account that initiated the transaction.
    pub account: Cow<'a, str>,
    /// The type of transaction.
    ///
    /// See Transaction Types:
    /// `<https://xrpl.org/transaction-types.html>`
    pub transaction_type: TransactionType,
    /// Hash value identifying another transaction. If provided, this
    /// transaction is only valid if the sending account's
    /// previously-sent transaction matches the provided hash.
    #[serde(rename = "AccountTxnID")]
    pub account_txn_id: Option<Cow<'a, str>>,
    /// Integer amount of XRP, in drops, to be destroyed as a cost
    /// for distributing this transaction to the network. Some
    /// transaction types have different minimum requirements.
    /// See Transaction Cost for details.
    pub fee: Option<XRPAmount<'a>>,
    /// Set of bit-flags for this transaction.
    #[serde(with = "txn_flags")]
    #[serde(default = "optional_flag_collection_default")]
    pub flags: Option<FlagCollection<F>>,
    /// Highest ledger index this transaction can appear in.
    /// Specifying this field places a strict upper limit on how long
    /// the transaction can wait to be validated or rejected.
    /// See Reliable Transaction Submission for more details.
    pub last_ledger_sequence: Option<u32>,
    /// Additional arbitrary information used to identify this transaction.
    pub memos: Option<Vec<Memo>>,
    /// The sequence number of the account sending the transaction.
    /// A transaction is only valid if the Sequence number is exactly
    /// 1 greater than the previous transaction from the same account.
    /// The special case 0 means the transaction is using a Ticket instead.
    pub sequence: Option<u32>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction is
    /// made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub signers: Option<Vec<Signer<'a>>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction
    /// is made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub source_tag: Option<u32>,
    /// The sequence number of the ticket to use in place
    /// of a Sequence number. If this is provided, Sequence must
    /// be 0. Cannot be used with AccountTxnID.
    pub ticket_sequence: Option<u32>,
}

impl<'a, T> CommonFields<'a, T>
where
    T: IntoEnumIterator + Serialize + core::fmt::Debug,
{
    pub fn new(
        account: Cow<'a, str>,
        transaction_type: TransactionType,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<T>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer<'a>>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
    ) -> Self {
        CommonFields {
            account,
            transaction_type,
            account_txn_id,
            fee,
            flags,
            last_ledger_sequence,
            memos,
            sequence,
            signers,
            source_tag,
            ticket_sequence,
        }
    }
}

impl<'a, T> Transaction<T> for CommonFields<'a, T>
where
    T: IntoEnumIterator + Serialize + PartialEq + core::fmt::Debug,
{
    fn has_flag(&self, flag: &T) -> bool {
        match &self.flags {
            Some(flag_collection) => flag_collection.0.contains(flag),
            None => false,
        }
    }

    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }
}

fn optional_flag_collection_default<T>() -> Option<FlagCollection<T>>
where
    T: IntoEnumIterator + Serialize + core::fmt::Debug,
{
    None
}

serde_with_tag! {
/// An arbitrary piece of data attached to a transaction. A
/// transaction can have multiple Memo objects as an array
/// in the Memos field.
///
/// Must contain one or more of `memo_data`, `memo_format`,
/// and `memo_type`.
///
/// See Memos Field:
/// `<https://xrpl.org/transaction-common-fields.html#memos-field>`
// `#[derive(Serialize)]` is defined in the macro
#[derive(Debug, PartialEq, Eq, Default, Clone, new)]
pub struct Memo {
    pub memo_data: Option<String>,
    pub memo_format: Option<String>,
    pub memo_type: Option<String>,
}
}

/// One Signer in a multi-signature. A multi-signed transaction
/// can have an array of up to 8 Signers, each contributing a
/// signature, in the Signers field.
///
/// See Signers Field:
/// `<https://xrpl.org/transaction-common-fields.html#signers-field>`
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default, Clone, new)]
#[serde(rename_all = "PascalCase")]
pub struct Signer<'a> {
    account: Cow<'a, str>,
    txn_signature: Cow<'a, str>,
    signing_pub_key: Cow<'a, str>,
}

/// Standard functions for transactions.
pub trait Transaction<T>
where
    T: IntoEnumIterator + Serialize,
{
    fn has_flag(&self, flag: &T) -> bool {
        let _txn_flag = flag;
        false
    }

    fn get_transaction_type(&self) -> TransactionType;
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, Display, AsRefStr)]
pub enum Flag {
    AccountSet(AccountSetFlag),
    NFTokenCreateOffer(NFTokenCreateOfferFlag),
    NFTokenMint(NFTokenMintFlag),
    OfferCreate(OfferCreateFlag),
    Payment(PaymentFlag),
    PaymentChannelClaim(PaymentChannelClaimFlag),
    TrustSet(TrustSetFlag),
    EnableAmendment(EnableAmendmentFlag),
}
