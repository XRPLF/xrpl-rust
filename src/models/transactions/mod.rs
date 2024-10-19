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
pub mod xchain_account_create_commit;
pub mod xchain_add_account_create_attestation;
pub mod xchain_add_claim_attestation;
pub mod xchain_claim;
pub mod xchain_commit;
pub mod xchain_create_bridge;
pub mod xchain_create_claim_id;
pub mod xchain_modify_bridge;

use super::FlagCollection;
use crate::core::binarycodec::encode;
use crate::models::amount::XRPAmount;
use crate::Err;
use crate::{_serde::txn_flags, serde_with_tag};
use alloc::borrow::Cow;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::Result;
use core::fmt::Debug;
use derive_new::new;
use exceptions::XRPLTransactionException;
use serde::de::DeserializeOwned;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sha2::{Digest, Sha512};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display};

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
    XChainAccountCreateCommit,
    XChainAddAccountCreateAttestation,
    XChainAddClaimAttestation,
    XChainClaim,
    XChainCommit,
    XChainCreateBridge,
    XChainCreateClaimID,
    XChainModifyBridge,
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
    #[serde(default = "flag_collection_default")]
    pub flags: FlagCollection<F>,
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
            flags: flags.unwrap_or_default(),
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
        self.flags.0.contains(flag)
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

fn flag_collection_default<T>() -> FlagCollection<T>
where
    T: IntoEnumIterator + Serialize + core::fmt::Debug,
{
    FlagCollection::<T>::default()
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

    fn is_signed(&self) -> bool {
        self.get_common_fields().txn_signature.is_some()
            && self.get_common_fields().signing_pub_key.is_some()
    }

    /// Hashes the Transaction object as the ledger does. Only valid for signed
    /// Transaction objects.
    fn get_hash(&self) -> Result<Cow<str>>
    where
        Self: Serialize + DeserializeOwned + Debug + Clone,
    {
        if self.get_common_fields().txn_signature.is_none()
            && self.get_common_fields().signers.is_none()
        {
            return Err!(XRPLTransactionException::TxMustBeSigned);
        }
        let prefix = format!("{:X}", TRANSACTION_HASH_PREFIX);
        let tx_hex = encode(self)?;
        let tx_hex = prefix + &tx_hex;
        let tx_bytes = match hex::decode(&tx_hex) {
            Ok(bytes) => bytes,
            Err(e) => return Err!(e),
        };
        let mut hasher = Sha512::new();
        hasher.update(&tx_bytes);
        let hash = hasher.finalize();
        let hex_string = hex::encode_upper(hash);
        let result = hex_string[..64].to_string();

        Ok(result.into())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, Display, AsRefStr)]
pub enum Flag {
    AccountSet(account_set::AccountSetFlag),
    NFTokenCreateOffer(nftoken_create_offer::NFTokenCreateOfferFlag),
    NFTokenMint(nftoken_mint::NFTokenMintFlag),
    OfferCreate(offer_create::OfferCreateFlag),
    Payment(payment::PaymentFlag),
    PaymentChannelClaim(payment_channel_claim::PaymentChannelClaimFlag),
    TrustSet(trust_set::TrustSetFlag),
    EnableAmendment(pseudo_transactions::enable_amendment::EnableAmendmentFlag),
}

#[cfg(all(
    feature = "std",
    feature = "websocket",
    feature = "transaction-models",
    feature = "transaction-helpers",
    feature = "wallet"
))]
#[cfg(test)]
mod test_tx_common_fields {
    use super::*;
    use account_set::AccountSet;
    
    use offer_create::OfferCreate;

    #[tokio::test]
    async fn test_get_hash() {
        let txn_json = r#"{
            "Account": "rLyttXLh7Ttca9CMUaD3exVoXY2fn2zwj3",
            "Fee": "10",
            "Flags": 0,
            "LastLedgerSequence": 16409087,
            "Sequence": 16409064,
            "SigningPubKey": "ED93BFA583E83331E9DC498DE4558CE4861ACFAB9385EBBC43BC56A0D9845A1DF2",
            "TakerGets": "13100000",
            "TakerPays": {
                "currency": "USD",
                "issuer": "rLyttXLh7Ttca9CMUaD3exVoXY2fn2zwj3",
                "value": "10"
            },
            "TransactionType": "OfferCreate",
            "TxnSignature": "71135999783658A0CB4EBCF02E59ACD94C4D06D5BF909E05E6B97588155482BBA598535AD4728ACA1F90C4DE73FFC741B0A6AB87141BDA8BCC2F2DF9CD8C3703"
        }"#;
        let expected_hash = "66F3D6158CAB6E53405F8C264DB39F07D8D0454433A63DDFB98218ED1BC99B60";
        let txn: OfferCreate = serde_json::from_str(txn_json).unwrap();

        assert_eq!(&txn.get_hash().unwrap(), expected_hash);
    }

    #[test]
    fn test_txn_hash() {
        let tx_json_str = r#"{
            "Account": "rEbY5Tr5B6AjyjuVRhajpnvCWLGkYk5z6",
            "Domain": "6578616d706c652e636f6d",
            "Fee": "10",
            "Flags": 0,
            "LastLedgerSequence": 596447,
            "Sequence": 596427,
            "SigningPubKey": "EDAF73A0E6745EA9C17A2F4EB7043134A055213116CFF6F7888BBFF557B002874F",
            "TransactionType": "AccountSet",
            "TxnSignature": "8666A7E6AF0D6A4B4F19F25D315FA1C31D132FB2E974686C415D5499D43710384FF851C75CCC4E57972DE5C5354289F574B2F604B6AF15E2DADA6BB9F1330A07"
        }"#;
        let expected_hash = "5B765D6C6058CF54F5DBF6230A7F51E23295004FCC043660A77D73AA8537737B";
        let tx: AccountSet = serde_json::from_str(tx_json_str).unwrap();
        assert_eq!(tx.get_hash().unwrap(), expected_hash);
    }
}
