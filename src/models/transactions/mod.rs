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

use core::fmt::Debug;

pub use account_delete::*;
pub use account_set::*;
use alloc::format;
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

use serde::de::DeserializeOwned;
pub use set_regular_key::*;
use sha2::{Digest, Sha512};
pub use signer_list_set::*;
pub use ticket_create::*;
pub use trust_set::*;

use crate::core::binarycodec::encode;
use crate::models::amount::XRPAmount;
use crate::Err;
use crate::{_serde::txn_flags, serde_with_tag};
use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::Result;
use derive_new::new;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display};

use super::FlagCollection;

const TRANSACTION_HASH_PREFIX: u32 = 0x54584E00;

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
    /// The network ID of the chain this transaction is intended for.
    /// MUST BE OMITTED for Mainnet and some test networks.
    /// REQUIRED on chains whose network ID is 1025 or higher.
    pub network_id: Option<u32>,
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
        network_id: Option<u32>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer<'a>>>,
        signing_pub_key: Option<Cow<'a, str>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        txn_signature: Option<Cow<'a, str>>,
    ) -> Self {
        CommonFields {
            account,
            transaction_type,
            account_txn_id,
            fee,
            flags,
            last_ledger_sequence,
            memos,
            network_id,
            sequence,
            signers,
            signing_pub_key,
            source_tag,
            ticket_sequence,
            txn_signature,
        }
    }
}

impl<T> CommonFields<'_, T>
where
    T: IntoEnumIterator + Serialize + Debug + PartialEq + Clone,
{
    pub fn is_signed(&self) -> bool {
        if let Some(signers) = &self.signers {
            signers
                .iter()
                .all(|signer| signer.txn_signature.len() > 0 && signer.signing_pub_key.len() > 0)
        } else {
            self.txn_signature.is_some() && self.signing_pub_key.is_some()
        }
    }
}

impl<'a, T> Transaction<'a, T> for CommonFields<'a, T>
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

    fn get_common_fields(&self) -> &CommonFields<'_, T> {
        self
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, T> {
        self
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
    pub account: Cow<'a, str>,
    pub txn_signature: Cow<'a, str>,
    pub signing_pub_key: Cow<'a, str>,
}

/// Standard functions for transactions.
pub trait Transaction<'a, T>
where
    Self: Serialize,
    T: IntoEnumIterator + Serialize + Debug + PartialEq,
{
    fn has_flag(&self, flag: &T) -> bool {
        let _txn_flag = flag;
        false
    }

    fn get_transaction_type(&self) -> TransactionType;

    fn get_common_fields(&self) -> &CommonFields<'_, T>;

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, T>;

    fn get_field_value(&self, field: &str) -> Result<Option<String>> {
        match serde_json::to_value(self) {
            Ok(value) => Ok(value.get(field).map(|v| v.to_string())),
            Err(e) => Err!(e),
        }
    }

    /// Hashes the Transaction object as the ledger does. Only valid for signed
    /// Transaction objects.
    fn get_hash(&self) -> Result<Cow<str>>
    where
        Self: Serialize + DeserializeOwned + Debug + Clone,
    {
        // if !self.is_signed() {
        //     return Err!(XRPLTransactionException::TxMustBeSigned);
        // }
        let prefix = format!("{:X}", TRANSACTION_HASH_PREFIX);
        let encoded_tx = encode(self)?;
        let encoded = prefix + &encoded_tx;
        let encoded_bytes = match hex::decode(&encoded) {
            Ok(bytes) => bytes,
            Err(e) => return Err!(e),
        };
        let mut hasher = Sha512::new();
        hasher.update(&encoded_bytes);
        let hash = hasher.finalize();
        let hex_string = hex::encode_upper(hash);
        let result = hex_string[..64].to_string();

        Ok(result.into())
    }
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

#[cfg(test)]
mod test_tx_common_fields {
    use super::*;
    use crate::{
        asynch::transaction::sign,
        models::{amount::IssuedCurrencyAmount, transactions::OfferCreate},
        wallet::Wallet,
    };

    #[tokio::test]
    async fn test_get_hash() {
        let mut wallet = Wallet::new("sEdT7wHTCLzDG7ueaw4hroSTBvH7Mk5", 0).unwrap();
        let mut txn = OfferCreate::new(
            "rLyttXLh7Ttca9CMUaD3exVoXY2fn2zwj3".into(),
            None,
            Some("10".into()),
            Some(FlagCollection::default()),
            Some(16409087),
            None,
            Some(16409064),
            None,
            None,
            None,
            "13100000".into(),
            IssuedCurrencyAmount::new(
                "USD".into(),
                "rLyttXLh7Ttca9CMUaD3exVoXY2fn2zwj3".into(),
                "10".into(),
            )
            .into(),
            None,
            None,
        );
        sign(&mut txn, &mut wallet, false).unwrap();
        let expected_hash = "39530980D3D6F848E619BF05A57988D42A62075289B99C5728CBDE0D1710284B";

        assert_eq!(&txn.get_hash().unwrap(), expected_hash);
    }
}
