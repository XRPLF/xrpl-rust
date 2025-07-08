pub mod account_delete;
pub mod account_set;
pub mod amm_bid;
pub mod amm_create;
pub mod amm_delete;
pub mod amm_deposit;
pub mod amm_vote;
pub mod amm_withdraw;
pub mod check_cancel;
pub mod check_cash;
pub mod check_create;
pub mod deposit_preauth;
pub mod escrow_cancel;
pub mod escrow_create;
pub mod escrow_finish;
pub mod exceptions;
pub mod metadata;
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

use super::{FlagCollection, XRPLModelResult};
use crate::core::binarycodec::encode;
use crate::models::amount::XRPAmount;
use crate::{_serde::txn_flags, serde_with_tag};
use alloc::borrow::Cow;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Debug;
use core::str::FromStr;
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
    AMMBid,
    AMMCreate,
    AMMDelete,
    AMMDeposit,
    AMMVote,
    AMMWithdraw,
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
    pub signers: Option<Vec<Signer>>,
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
        signers: Option<Vec<Signer>>,
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
            signing_pub_key: Some(signing_pub_key.unwrap_or("".into())),
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

    fn get_transaction_type(&self) -> &TransactionType {
        &self.transaction_type
    }

    fn get_common_fields(&self) -> &CommonFields<'_, T> {
        self
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, T> {
        self
    }
}

impl<'a, T> Default for CommonFields<'a, T>
where
    T: IntoEnumIterator + Serialize + core::fmt::Debug,
{
    fn default() -> Self {
        Self {
            account: "".into(),
            transaction_type: TransactionType::Payment, // Temporary default, will be overridden
            account_txn_id: None,
            fee: None,
            flags: FlagCollection::default(),
            last_ledger_sequence: None,
            memos: None,
            network_id: None,
            sequence: None,
            signers: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
        }
    }
}

impl<'a, T> FromStr for CommonFields<'a, T>
where
    T: IntoEnumIterator + Serialize + core::fmt::Debug,
{
    type Err = core::convert::Infallible;

    fn from_str(account: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            account: Cow::Owned(account.to_string()),
            ..Default::default()
        })
    }
}

impl<'a, T> From<String> for CommonFields<'a, T>
where
    T: IntoEnumIterator + Serialize + core::fmt::Debug,
{
    fn from(account: String) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }
}

impl<'a, T> From<Cow<'a, str>> for CommonFields<'a, T>
where
    T: IntoEnumIterator + Serialize + core::fmt::Debug,
{
    fn from(account: Cow<'a, str>) -> Self {
        Self {
            account,
            ..Default::default()
        }
    }
}

impl<'a, T> CommonFields<'a, T>
where
    T: IntoEnumIterator + Serialize + core::fmt::Debug,
{
    pub fn with_transaction_type(mut self, transaction_type: TransactionType) -> Self {
        self.transaction_type = transaction_type;
        self
    }

    pub fn with_fee(mut self, fee: XRPAmount<'a>) -> Self {
        self.fee = Some(fee);
        self
    }

    pub fn with_sequence(mut self, sequence: u32) -> Self {
        self.sequence = Some(sequence);
        self
    }

    pub fn with_last_ledger_sequence(mut self, last_ledger_sequence: u32) -> Self {
        self.last_ledger_sequence = Some(last_ledger_sequence);
        self
    }

    pub fn with_source_tag(mut self, source_tag: u32) -> Self {
        self.source_tag = Some(source_tag);
        self
    }

    pub fn with_memo(mut self, memo: Memo) -> Self {
        match self.memos {
            Some(ref mut memos) => memos.push(memo),
            None => self.memos = Some(alloc::vec![memo]),
        }
        self
    }

    pub fn with_memos(mut self, memos: Vec<Memo>) -> Self {
        self.memos = Some(memos);
        self
    }

    pub fn with_network_id(mut self, network_id: u32) -> Self {
        self.network_id = Some(network_id);
        self
    }

    pub fn with_ticket_sequence(mut self, ticket_sequence: u32) -> Self {
        self.ticket_sequence = Some(ticket_sequence);
        self
    }

    pub fn with_account_txn_id(mut self, account_txn_id: Cow<'a, str>) -> Self {
        self.account_txn_id = Some(account_txn_id);
        self
    }

    pub fn with_signers(mut self, signers: Vec<Signer>) -> Self {
        self.signers = Some(signers);
        self
    }

    pub fn with_signing_pub_key(mut self, signing_pub_key: Cow<'a, str>) -> Self {
        self.signing_pub_key = Some(signing_pub_key);
        self
    }

    pub fn with_txn_signature(mut self, txn_signature: Cow<'a, str>) -> Self {
        self.txn_signature = Some(txn_signature);
        self
    }

    /// Create CommonFields from an account string (takes ownership)
    pub fn from_account(account: impl Into<Cow<'a, str>>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
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

serde_with_tag! {
    /// Represents one entry in a list of AuthAccounts used in AMMBid transaction.
    #[derive(Debug, Clone, PartialEq, Eq, new)]
    pub struct AuthAccount {
        pub account: String,
    }
}

serde_with_tag! {
/// One Signer in a multi-signature. A multi-signed transaction
/// can have an array of up to 8 Signers, each contributing a
/// signature, in the Signers field.
///
/// See Signers Field:
/// `<https://xrpl.org/transaction-common-fields.html#signers-field>`
#[derive(Debug, PartialEq, Eq, Default, Clone, new)]
pub struct Signer {
    pub account: String,
    pub txn_signature: String,
    pub signing_pub_key: String,
}
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

    fn get_transaction_type(&self) -> &TransactionType;

    fn get_common_fields(&self) -> &CommonFields<'_, T>;

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, T>;

    fn get_field_value(&self, field: &str) -> XRPLModelResult<Option<String>> {
        let value = serde_json::to_value(self)?;

        Ok(value.get(field).map(|v| v.to_string()))
    }

    fn is_signed(&self) -> bool {
        self.get_common_fields().txn_signature.is_some()
            && self.get_common_fields().signing_pub_key.is_some()
    }

    /// Hashes the Transaction object as the ledger does. Only valid for signed
    /// Transaction objects.
    fn get_hash(&self) -> XRPLModelResult<Cow<str>>
    where
        Self: Serialize + DeserializeOwned + Debug + Clone,
    {
        if self.get_common_fields().txn_signature.is_none()
            && self.get_common_fields().signers.is_none()
        {
            return Err(XRPLTransactionException::TxMustBeSigned.into());
        }
        let prefix = format!("{:X}", TRANSACTION_HASH_PREFIX);
        let tx_hex = encode(self).map_err(XRPLTransactionException::XRPLCoreError)?;
        let tx_hex = prefix + &tx_hex;
        let tx_bytes = hex::decode(&tx_hex)?;
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
    feature = "models",
    feature = "helpers",
    feature = "wallet"
))]
#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use super::*;
    use crate::models::transactions::payment::PaymentFlag;
    use crate::models::transactions::{account_set::AccountSet, payment::Payment};
    use crate::models::{Amount, XRPAmount};
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

    #[test]
    fn test_common_fields_default() {
        let common_fields: CommonFields<payment::PaymentFlag> = Default::default();

        assert_eq!(common_fields.account, "");
        assert_eq!(common_fields.transaction_type, TransactionType::Payment);
        assert!(common_fields.fee.is_none());
        assert!(common_fields.sequence.is_none());
        assert!(common_fields.memos.is_none());
    }

    #[test]
    fn test_common_fields_from_str() {
        let account = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH";
        let common_fields: CommonFields<payment::PaymentFlag> = account.parse().unwrap();

        assert_eq!(common_fields.account, account);
        assert_eq!(common_fields.transaction_type, TransactionType::Payment);
        assert!(common_fields.fee.is_none());
    }

    #[test]
    fn test_common_fields_from_string() {
        let account = String::from("rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH");
        let common_fields: CommonFields<payment::PaymentFlag> = CommonFields::from(account.clone());

        assert_eq!(common_fields.account, account);
        assert_eq!(common_fields.transaction_type, TransactionType::Payment);
    }

    #[test]
    fn test_common_fields_from_cow() {
        let account: Cow<str> = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH".into();
        let common_fields: CommonFields<payment::PaymentFlag> = CommonFields::from(account.clone());

        assert_eq!(common_fields.account, account);
    }

    #[test]
    fn test_common_fields_builder_methods() {
        let common_fields = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH"
            .parse::<CommonFields<payment::PaymentFlag>>()
            .unwrap()
            .with_transaction_type(TransactionType::Payment)
            .with_fee("12".into())
            .with_sequence(100)
            .with_last_ledger_sequence(596447)
            .with_source_tag(42)
            .with_network_id(1025);

        assert_eq!(common_fields.account, "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH");
        assert_eq!(common_fields.transaction_type, TransactionType::Payment);
        assert_eq!(common_fields.fee, Some("12".into()));
        assert_eq!(common_fields.sequence, Some(100));
        assert_eq!(common_fields.last_ledger_sequence, Some(596447));
        assert_eq!(common_fields.source_tag, Some(42));
        assert_eq!(common_fields.network_id, Some(1025));
    }

    #[test]
    fn test_memo_builder() {
        let memo = Memo {
            memo_data: Some("Test memo".to_string()),
            memo_format: Some("text/plain".to_string()),
            memo_type: Some("test".to_string()),
        };

        let common_fields = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH"
            .parse::<CommonFields<payment::PaymentFlag>>()
            .unwrap()
            .with_memo(memo.clone());

        assert_eq!(common_fields.memos, Some(alloc::vec![memo]));
    }

    #[test]
    fn test_multiple_memos_builder() {
        let memo1 = Memo {
            memo_data: Some("First memo".to_string()),
            memo_format: Some("text/plain".to_string()),
            memo_type: Some("info".to_string()),
        };

        let memo2 = Memo {
            memo_data: Some("Second memo".to_string()),
            memo_format: Some("text/plain".to_string()),
            memo_type: Some("note".to_string()),
        };

        let common_fields = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH"
            .parse::<CommonFields<payment::PaymentFlag>>()
            .unwrap()
            .with_memo(memo1.clone())
            .with_memo(memo2.clone());

        assert_eq!(common_fields.memos, Some(alloc::vec![memo1, memo2]));
    }

    #[test]
    fn test_memos_builder_replace() {
        let memo1 = Memo {
            memo_data: Some("First memo".to_string()),
            memo_format: None,
            memo_type: None,
        };

        let memo2 = Memo {
            memo_data: Some("Second memo".to_string()),
            memo_format: None,
            memo_type: None,
        };

        let common_fields = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH"
            .parse::<CommonFields<payment::PaymentFlag>>()
            .unwrap()
            .with_memo(memo1)
            .with_memos(alloc::vec![memo2.clone()]);

        assert_eq!(common_fields.memos, Some(alloc::vec![memo2]));
    }

    #[test]
    fn test_from_account_helper() {
        let common_fields: CommonFields<payment::PaymentFlag> =
            CommonFields::from_account("rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH");

        assert_eq!(common_fields.account, "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH");
        assert_eq!(common_fields.transaction_type, TransactionType::Payment);
    }

    #[test]
    fn test_signers_builder() {
        let signer = Signer {
            account: "rSignerAccount123".to_string(),
            txn_signature: "signature123".to_string(),
            signing_pub_key: "pubkey123".to_string(),
        };

        let common_fields = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH"
            .parse::<CommonFields<payment::PaymentFlag>>()
            .unwrap()
            .with_signers(alloc::vec![signer.clone()]);

        assert_eq!(common_fields.signers, Some(alloc::vec![signer]));
    }

    #[test]
    fn test_signature_fields_builder() {
        let common_fields = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH"
            .parse::<CommonFields<payment::PaymentFlag>>()
            .unwrap()
            .with_signing_pub_key("ED12345...".into())
            .with_txn_signature("A1B2C3...".into());

        assert_eq!(common_fields.signing_pub_key, Some("ED12345...".into()));
        assert_eq!(common_fields.txn_signature, Some("A1B2C3...".into()));
    }

    #[test]
    fn test_account_txn_id_builder() {
        let txn_id = "F1E2D3C4B5A69788";
        let common_fields = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH"
            .parse::<CommonFields<payment::PaymentFlag>>()
            .unwrap()
            .with_account_txn_id(txn_id.into());

        assert_eq!(common_fields.account_txn_id, Some(txn_id.into()));
    }

    #[test]
    fn test_ticket_sequence_builder() {
        let common_fields = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH"
            .parse::<CommonFields<payment::PaymentFlag>>()
            .unwrap()
            .with_ticket_sequence(50);

        assert_eq!(common_fields.ticket_sequence, Some(50));
    }

    #[test]
    fn test_payment_with_builder_pattern() {
        let payment = Payment {
            common_fields: "rSender123"
                .parse::<CommonFields<payment::PaymentFlag>>()
                .unwrap()
                .with_transaction_type(TransactionType::Payment)
                .with_fee("12".into())
                .with_sequence(100),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: "rReceiver456".into(),
            ..Default::default()
        };

        assert_eq!(payment.common_fields.account, "rSender123");
        assert_eq!(
            payment.common_fields.transaction_type,
            TransactionType::Payment
        );
        assert_eq!(payment.common_fields.fee, Some("12".into()));
        assert_eq!(payment.common_fields.sequence, Some(100));
        assert_eq!(payment.destination, "rReceiver456");
    }

    #[test]
    fn test_account_set_with_builder_pattern() {
        let account_set = AccountSet {
            common_fields: CommonFields::from_account("rAccount123")
                .with_transaction_type(TransactionType::AccountSet)
                .with_fee("12".into())
                .with_sequence(50),
            domain: Some("6578616d706c652e636f6d".into()), // "example.com"
            ..Default::default()
        };

        assert_eq!(account_set.common_fields.account, "rAccount123");
        assert_eq!(
            account_set.common_fields.transaction_type,
            TransactionType::AccountSet
        );
        assert_eq!(account_set.common_fields.fee, Some("12".into()));
        assert_eq!(account_set.common_fields.sequence, Some(50));
        assert_eq!(account_set.domain, Some("6578616d706c652e636f6d".into()));
    }

    #[test]
    fn test_complex_payment_with_memos() {
        let memo = Memo {
            memo_data: Some("Payment for services".to_string()),
            memo_format: Some("text/plain".to_string()),
            memo_type: Some("payment".to_string()),
        };

        let payment = Payment {
            common_fields: "rSender123"
                .parse::<CommonFields<payment::PaymentFlag>>()
                .unwrap()
                .with_transaction_type(TransactionType::Payment)
                .with_fee("12".into())
                .with_sequence(100)
                .with_last_ledger_sequence(596447)
                .with_source_tag(12345)
                .with_memo(memo.clone()),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: "rReceiver456".into(),
            destination_tag: Some(67890),
            ..Default::default()
        };

        assert_eq!(payment.common_fields.memos, Some(alloc::vec![memo]));
        assert_eq!(payment.common_fields.source_tag, Some(12345));
        assert_eq!(payment.destination_tag, Some(67890));
    }

    #[test]
    fn test_builder_pattern_fluency() {
        // Test that the builder pattern is truly fluent and readable
        let payment = Payment {
            common_fields: "rSender123"
                .parse::<CommonFields<payment::PaymentFlag>>()
                .unwrap()
                .with_transaction_type(TransactionType::Payment)
                .with_fee("12".into())
                .with_sequence(100)
                .with_last_ledger_sequence(596447)
                .with_source_tag(12345)
                .with_network_id(1025)
                .with_memo(Memo {
                    memo_data: Some("Test payment".to_string()),
                    memo_format: Some("text/plain".to_string()),
                    memo_type: Some("test".to_string()),
                }),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: "rReceiver456".into(),
            destination_tag: Some(67890),
            ..Default::default()
        };

        // Verify all fields are set correctly
        assert_eq!(payment.common_fields.account, "rSender123");
        assert_eq!(payment.common_fields.fee, Some("12".into()));
        assert_eq!(payment.common_fields.sequence, Some(100));
        assert_eq!(payment.common_fields.last_ledger_sequence, Some(596447));
        assert_eq!(payment.common_fields.source_tag, Some(12345));
        assert_eq!(payment.common_fields.network_id, Some(1025));
        assert_eq!(payment.destination_tag, Some(67890));
        assert!(payment.common_fields.memos.is_some());
    }

    #[test]
    fn test_different_from_methods() {
        // Test all different ways to create CommonFields
        let account_str = "rAccount123";
        let account_string = String::from("rAccount123");
        let account_cow: Cow<str> = "rAccount123".into();

        let cf1: CommonFields<payment::PaymentFlag> = account_str.parse().unwrap();
        let cf2: CommonFields<payment::PaymentFlag> = CommonFields::from(account_string);
        let cf3: CommonFields<payment::PaymentFlag> = CommonFields::from(account_cow);
        let cf4: CommonFields<payment::PaymentFlag> = CommonFields::from_account("rAccount123");

        assert_eq!(cf1.account, "rAccount123");
        assert_eq!(cf2.account, "rAccount123");
        assert_eq!(cf3.account, "rAccount123");
        assert_eq!(cf4.account, "rAccount123");
    }

    #[test]
    fn test_fromstr_never_fails() {
        // Test edge cases for FromStr
        let empty: Result<CommonFields<PaymentFlag>, _> = "".parse();
        assert!(empty.is_ok());
        assert_eq!(empty.unwrap().account, "");

        let whitespace: Result<CommonFields<PaymentFlag>, _> = "   ".parse();
        assert!(whitespace.is_ok());
        assert_eq!(whitespace.unwrap().account, "   ");

        let long_string: Result<CommonFields<PaymentFlag>, _> = "r".repeat(1000).parse();
        assert!(long_string.is_ok());
        assert_eq!(long_string.unwrap().account.len(), 1000);
    }

    #[test]
    fn test_builder_pattern_overwrites() {
        // Test that builder methods properly overwrite values
        let common_fields = "rAccount123"
            .parse::<CommonFields<payment::PaymentFlag>>()
            .unwrap()
            .with_fee("10".into())
            .with_fee("20".into()) // Should overwrite the first fee
            .with_sequence(100)
            .with_sequence(200); // Should overwrite the first sequence

        assert_eq!(common_fields.fee, Some("20".into()));
        assert_eq!(common_fields.sequence, Some(200));
    }
}
