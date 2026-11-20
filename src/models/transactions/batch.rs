//! Batch transaction (XLS-56, BatchV1_1).
//!
//! See: <https://xls.xrpl.org/xls/XLS-0056-batch.html>

use alloc::borrow::Cow;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt::Debug;

use serde::de::Deserializer;
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use sha2::{Digest, Sha512};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::core::binarycodec::{encode, encode_for_signing_batch_v1_1, types::AccountId};
use crate::models::{Model, XRPLModelException, XRPLModelResult};

use super::exceptions::XRPLTransactionException;
use super::{
    CommonFields, CommonTransactionBuilder, Signer, Transaction, TransactionType,
    TRANSACTION_HASH_PREFIX,
};

/// Global flag required on every inner transaction.
pub const TF_INNER_BATCH_TXN: u32 = 0x4000_0000;
pub const MIN_RAW_TRANSACTIONS: usize = 2;
pub const MAX_RAW_TRANSACTIONS: usize = 8;
pub const MAX_BATCH_SIGNERS: usize = 24;

/// Batch modes. Exactly one must be set.
///
/// See Batch flags:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/batch>`
#[derive(
    Debug, Eq, PartialEq, Clone, Copy, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum BatchFlag {
    /// All inner transactions must succeed, or none are applied.
    TfAllOrNothing = 0x00010000,
    /// Only the first inner transaction that succeeds is applied.
    TfOnlyOne = 0x00020000,
    /// Apply inner transactions until the first failure.
    TfUntilFailure = 0x00040000,
    /// Apply every inner transaction regardless of failures.
    TfIndependent = 0x00080000,
}

fn invalid(field: &str, expected: &str, found: impl Into<String>) -> XRPLModelException {
    XRPLModelException::InvalidValue {
        field: field.to_string(),
        expected: expected.to_string(),
        found: found.into(),
    }
}

fn account_id_bytes(address: &str) -> XRPLModelResult<[u8; 20]> {
    let id = AccountId::try_from(address)
        .map_err(|_| invalid("account", "a valid classic address", address))?;
    let bytes: &[u8] = id.as_ref();
    <[u8; 20]>::try_from(bytes).map_err(|_| invalid("account", "a 20-byte AccountID", address))
}

/// Sorts by AccountID bytes (the ledger order), NOT by the base58 string.
fn sort_by_account_id<T>(items: Vec<T>, account: impl Fn(&T) -> &str) -> XRPLModelResult<Vec<T>> {
    let mut keyed = Vec::with_capacity(items.len());
    for item in items {
        keyed.push((account_id_bytes(account(&item))?, item));
    }
    keyed.sort_by_key(|a| a.0);
    Ok(keyed.into_iter().map(|(_, item)| item).collect())
}

/// One entry of `RawTransactions`; serializes as `{"RawTransaction": {...}}`.
///
/// Holds the inner transaction as normalized JSON because inner transactions
/// are heterogeneous and carry `tfInnerBatchTxn`, which no per-type flag enum has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawTransaction(Value);

impl RawTransaction {
    /// Build from any typed transaction. Forces `Fee = "0"`, `SigningPubKey = ""`,
    /// removes `TxnSignature`/`Signers`, sets `tfInnerBatchTxn`. The caller sets
    /// `Sequence` (or `TicketSequence`) on the transaction beforehand.
    pub fn from_transaction<'a, T, F>(transaction: &T) -> XRPLModelResult<Self>
    where
        T: Transaction<'a, F>,
        F: IntoEnumIterator + Serialize + Debug + PartialEq,
    {
        Self::from_json(serde_json::to_value(transaction)?)
    }

    pub fn from_json(mut json: Value) -> XRPLModelResult<Self> {
        let Some(object) = json.as_object_mut() else {
            return Err(invalid("raw_transactions", "a JSON object", "non-object"));
        };
        if object.get("TransactionType").and_then(Value::as_str) == Some("Batch") {
            return Err(invalid("raw_transactions", "no nested Batch", "Batch"));
        }
        let flags = object.get("Flags").and_then(Value::as_u64).unwrap_or(0) as u32;
        object.insert("Flags".to_string(), Value::from(flags | TF_INNER_BATCH_TXN));
        object.insert("Fee".to_string(), Value::from("0"));
        object.insert("SigningPubKey".to_string(), Value::from(""));
        object.remove("TxnSignature");
        object.remove("Signers");
        // Ticketed inner txns must carry an explicit `Sequence: 0` (XLS-56 §2.3.15).
        if object.contains_key("TicketSequence") && !object.contains_key("Sequence") {
            object.insert("Sequence".to_string(), Value::from(0u32));
        }
        Ok(Self(json))
    }

    pub fn as_json(&self) -> &Value {
        &self.0
    }

    pub fn account(&self) -> Option<&str> {
        self.0.get("Account").and_then(Value::as_str)
    }

    pub fn transaction_type(&self) -> XRPLModelResult<TransactionType> {
        let value = self
            .0
            .get("TransactionType")
            .ok_or_else(|| XRPLModelException::MissingField("TransactionType".to_string()))?;
        Ok(serde_json::from_value(value.clone())?)
    }

    /// Transaction ID of the inner transaction (SHA-512Half of `TXN\0 || blob`).
    /// `Transaction::get_hash` can't be used: it refuses unsigned transactions.
    pub fn transaction_id(&self) -> XRPLModelResult<String> {
        let encoded = encode(&self.0).map_err(XRPLTransactionException::XRPLCoreError)?;
        let mut bytes = TRANSACTION_HASH_PREFIX.to_be_bytes().to_vec();
        bytes.extend(hex::decode(encoded)?);
        let hash = Sha512::digest(&bytes);
        Ok(hex::encode_upper(&hash[..32]))
    }

    fn validate(&self, index: usize) -> XRPLModelResult<()> {
        let field = format!("raw_transactions[{index}]");
        let Some(object) = self.0.as_object() else {
            return Err(invalid(&field, "a JSON object", "non-object"));
        };
        if object.get("Account").and_then(Value::as_str).is_none() {
            return Err(XRPLModelException::MissingField(format!("{field}.Account")));
        }
        if self.transaction_type()? == TransactionType::Batch {
            return Err(invalid(&field, "no nested Batch", "Batch")); // temINVALID_INNER_BATCH
        }
        let flags = object.get("Flags").and_then(Value::as_u64).unwrap_or(0);
        if (flags & TF_INNER_BATCH_TXN as u64) == 0 {
            return Err(invalid(&field, "tfInnerBatchTxn set", "flag missing")); // temINVALID_FLAG
        }
        if object.get("Fee").and_then(Value::as_str) != Some("0") {
            return Err(invalid(&field, "Fee \"0\"", "non-zero or missing Fee"));
            // temBAD_FEE
        }
        if object.contains_key("TxnSignature") {
            return Err(invalid(&field, "no TxnSignature", "TxnSignature")); // temBAD_SIGNATURE
        }
        if object.get("SigningPubKey").and_then(Value::as_str) != Some("") {
            return Err(invalid(&field, "empty SigningPubKey", "non-empty")); // temBAD_REGKEY
        }
        if object.contains_key("Signers") {
            return Err(invalid(&field, "no Signers", "Signers")); // temBAD_SIGNER
        }
        let sequence = object.get("Sequence").and_then(Value::as_u64).unwrap_or(0);
        let ticketed = object.contains_key("TicketSequence");
        if ticketed != (sequence == 0) {
            return Err(invalid(
                &field,
                "Sequence XOR TicketSequence (ticketed: Sequence 0)", // temSEQ_AND_TICKET
                format!("Sequence={sequence}, ticketed={ticketed}"),
            ));
        }
        Ok(())
    }
}

fn serialize_tagged<S: Serializer, T: Serialize>(
    serializer: S,
    tag: &'static str,
    value: &T,
) -> Result<S::Ok, S::Error> {
    let mut map = serializer.serialize_map(Some(1))?;
    map.serialize_entry(tag, value)?;
    map.end()
}

impl Serialize for RawTransaction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_tagged(serializer, "RawTransaction", &self.0)
    }
}

impl<'de> Deserialize<'de> for RawTransaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(rename = "RawTransaction")]
            inner: Value,
        }
        Ok(Self(Wrapper::deserialize(deserializer)?.inner))
    }
}

/// One entry of `BatchSigners`; serializes as `{"BatchSigner": {...}}`.
/// Either `signing_pub_key` + `txn_signature`, or `signers`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BatchSigner {
    pub account: String,
    pub signing_pub_key: Option<String>,
    pub txn_signature: Option<String>,
    pub signers: Option<Vec<Signer>>,
}

impl BatchSigner {
    pub fn single(
        account: impl Into<String>,
        signing_pub_key: impl Into<String>,
        txn_signature: impl Into<String>,
    ) -> Self {
        Self {
            account: account.into(),
            signing_pub_key: Some(signing_pub_key.into()),
            txn_signature: Some(txn_signature.into()),
            signers: None,
        }
    }

    /// Multi-signed entry; `signers` are sorted by AccountID.
    pub fn multi(account: impl Into<String>, signers: Vec<Signer>) -> XRPLModelResult<Self> {
        Ok(Self {
            account: account.into(),
            signing_pub_key: None,
            txn_signature: None,
            signers: Some(sort_by_account_id(signers, |s| s.account.as_str())?),
        })
    }

    fn validate(&self, index: usize) -> XRPLModelResult<()> {
        let field = format!("batch_signers[{index}]");
        let single = self.signing_pub_key.is_some() && self.txn_signature.is_some();
        let partial = self.signing_pub_key.is_some() != self.txn_signature.is_some();
        let multi = self.signers.as_ref().is_some_and(|s| !s.is_empty());
        if partial || single == multi {
            return Err(invalid(
                &field,
                "either SigningPubKey + TxnSignature, or Signers", // temBAD_SIGNER
                "invalid combination",
            ));
        }
        Ok(())
    }
}

impl Serialize for BatchSigner {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct Body<'b> {
            account: &'b str,
            #[serde(skip_serializing_if = "Option::is_none")]
            signing_pub_key: Option<&'b str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            txn_signature: Option<&'b str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            signers: Option<&'b Vec<Signer>>,
        }
        serialize_tagged(
            serializer,
            "BatchSigner",
            &Body {
                account: &self.account,
                signing_pub_key: self.signing_pub_key.as_deref(),
                txn_signature: self.txn_signature.as_deref(),
                signers: self.signers.as_ref(),
            },
        )
    }
}

impl<'de> Deserialize<'de> for BatchSigner {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct Body {
            account: String,
            signing_pub_key: Option<String>,
            txn_signature: Option<String>,
            signers: Option<Vec<Signer>>,
        }
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(rename = "BatchSigner")]
            body: Body,
        }
        let Body {
            account,
            signing_pub_key,
            txn_signature,
            signers,
        } = Wrapper::deserialize(deserializer)?.body;
        Ok(Self {
            account,
            signing_pub_key,
            txn_signature,
            signers,
        })
    }
}

/// Wraps up to eight transactions and applies them atomically according to the
/// batch mode flag.
///
/// See Batch:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/batch>`
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Batch<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, BatchFlag>,
    /// The inner transactions (2 to 8).
    pub raw_transactions: Vec<RawTransaction>,
    /// Signatures of every non-outer account that must authorize an inner
    /// transaction, sorted by AccountID.
    pub batch_signers: Option<Vec<BatchSigner>>,
}

impl<'a: 'static> Model for Batch<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_mode_error()?;
        self._get_raw_transactions_error()?;
        self._get_batch_signers_error()
    }
}

impl<'a> Transaction<'a, BatchFlag> for Batch<'a> {
    fn has_flag(&self, flag: &BatchFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, BatchFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, BatchFlag> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, BatchFlag> for Batch<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, BatchFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> Batch<'a> {
    pub fn new(
        account: impl Into<Cow<'a, str>>,
        mode: BatchFlag,
        raw_transactions: Vec<RawTransaction>,
    ) -> Self {
        Self {
            common_fields: CommonFields::from_account(account)
                .with_transaction_type(TransactionType::Batch)
                .with_flag(mode),
            raw_transactions,
            batch_signers: None,
        }
    }

    /// Replace the batch mode.
    pub fn with_mode(mut self, mode: BatchFlag) -> Self {
        self.common_fields.flags = alloc::vec![mode].into();
        self
    }

    pub fn with_raw_transaction(mut self, raw_transaction: RawTransaction) -> Self {
        self.raw_transactions.push(raw_transaction);
        self
    }

    /// Sets `BatchSigners`, sorted by AccountID as the ledger requires.
    pub fn with_batch_signers(mut self, signers: Vec<BatchSigner>) -> XRPLModelResult<Self> {
        self.batch_signers = Some(sort_by_account_id(signers, |s| s.account.as_str())?);
        Ok(self)
    }

    /// Adds one entry and re-sorts.
    pub fn add_batch_signer(mut self, signer: BatchSigner) -> XRPLModelResult<Self> {
        let mut signers = self.batch_signers.take().unwrap_or_default();
        signers.push(signer);
        self.with_batch_signers(signers)
    }

    /// Transaction IDs of the inner transactions, in order. Use these to look up
    /// each inner result: the outer result is `tesSUCCESS` even if inners fail.
    pub fn inner_transaction_ids(&self) -> XRPLModelResult<Vec<String>> {
        self.raw_transactions
            .iter()
            .map(RawTransaction::transaction_id)
            .collect()
    }

    /// `Sequence`, or `TicketSequence` when `Sequence` is 0 (XLS-56 §2.1.3.2).
    pub fn outer_sequence(&self) -> XRPLModelResult<u32> {
        match (
            self.common_fields.sequence,
            self.common_fields.ticket_sequence,
        ) {
            (Some(sequence), _) if sequence != 0 => Ok(sequence),
            (_, Some(ticket)) => Ok(ticket),
            _ => Err(XRPLModelException::MissingField("sequence".to_string())),
        }
    }

    fn mode_flags(&self) -> u32 {
        self.common_fields
            .flags
            .0
            .iter()
            .fold(0u32, |acc, flag| acc | *flag as u32)
    }

    /// Hex payload a BatchSigner signs. `batch_account` is the `BatchSigner.Account`;
    /// `multisigner_account` is the individual signer for multi-signed entries.
    /// Requires the outer `Sequence`/`TicketSequence` and flags to be final.
    pub fn batch_signing_data(
        &self,
        batch_account: &str,
        multisigner_account: Option<&str>,
    ) -> XRPLModelResult<String> {
        let ids = self.inner_transaction_ids()?;
        let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        Ok(encode_for_signing_batch_v1_1(
            &self.common_fields.account,
            self.outer_sequence()?,
            self.mode_flags(),
            &id_refs,
            batch_account,
            multisigner_account,
        )
        .map_err(XRPLTransactionException::XRPLCoreError)?)
    }

    fn _get_mode_error(&self) -> XRPLModelResult<()> {
        let selected = [
            BatchFlag::TfAllOrNothing,
            BatchFlag::TfOnlyOne,
            BatchFlag::TfUntilFailure,
            BatchFlag::TfIndependent,
        ]
        .iter()
        .filter(|mode| self.has_flag(mode))
        .count();
        if selected != 1 {
            return Err(invalid(
                "flags",
                "exactly one batch mode", // temINVALID_FLAG
                format!("{selected} modes set"),
            ));
        }
        Ok(())
    }

    fn _get_raw_transactions_error(&self) -> XRPLModelResult<()> {
        let count = self.raw_transactions.len();
        if !(MIN_RAW_TRANSACTIONS..=MAX_RAW_TRANSACTIONS).contains(&count) {
            return Err(invalid(
                "raw_transactions",
                "between 2 and 8 transactions", // temARRAY_EMPTY / malformed
                format!("{count} transactions"),
            ));
        }
        for (i, raw) in self.raw_transactions.iter().enumerate() {
            raw.validate(i)?;
            if self.raw_transactions[..i].contains(raw) {
                return Err(invalid(
                    &format!("raw_transactions[{i}]"),
                    "no duplicate transactions", // temREDUNDANT
                    "duplicate",
                ));
            }
        }
        Ok(())
    }

    fn _get_batch_signers_error(&self) -> XRPLModelResult<()> {
        let Some(signers) = &self.batch_signers else {
            return Ok(());
        };
        if signers.len() > MAX_BATCH_SIGNERS {
            return Err(invalid(
                "batch_signers",
                "at most 24 entries", // temARRAY_TOO_LARGE
                format!("{} entries", signers.len()),
            ));
        }
        let mut previous: Option<[u8; 20]> = None;
        for (i, signer) in signers.iter().enumerate() {
            signer.validate(i)?;
            if signer.account == self.common_fields.account {
                return Err(invalid(
                    &format!("batch_signers[{i}]"),
                    "no entry for the outer account", // temBAD_SIGNER
                    signer.account.clone(),
                ));
            }
            let id = account_id_bytes(&signer.account)?;
            if previous.is_some_and(|prev| prev >= id) {
                return Err(invalid(
                    &format!("batch_signers[{i}]"),
                    "strictly ascending AccountID order, no duplicates", // temBAD_SIGNER
                    signer.account.clone(),
                ));
            }
            previous = Some(id);
        }
        Ok(())
    }
}

#[cfg(feature = "wallet")]
impl<'a> Batch<'a> {
    /// Single-key BatchSigner entry authorizing `account` (the account whose
    /// inner transactions are being approved; `wallet` holds its master or regular key).
    pub fn sign_as_batch_signer(
        &self,
        account: &str,
        wallet: &crate::wallet::Wallet,
    ) -> crate::asynch::exceptions::XRPLHelperResult<BatchSigner> {
        let payload = hex::decode(self.batch_signing_data(account, None)?)?;
        let signature = crate::core::keypairs::sign(&payload, &wallet.private_key)?;
        Ok(BatchSigner::single(
            account,
            wallet.public_key.clone(),
            signature,
        ))
    }

    /// One signer's contribution to a multi-signed BatchSigner of `batch_account`.
    /// Collect these and wrap with `BatchSigner::multi`.
    pub fn sign_as_batch_multisigner(
        &self,
        batch_account: &str,
        wallet: &crate::wallet::Wallet,
    ) -> crate::asynch::exceptions::XRPLHelperResult<Signer> {
        let payload =
            hex::decode(self.batch_signing_data(batch_account, Some(&wallet.classic_address))?)?;
        let signature = crate::core::keypairs::sign(&payload, &wallet.private_key)?;
        Ok(Signer::new(
            wallet.classic_address.clone(),
            signature,
            wallet.public_key.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::models::amount::Amount;
    use crate::models::transactions::payment::Payment;

    const OUTER: &str = "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb";
    const DEST: &str = "rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK";
    const OTHER_A: &str = "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn";
    const OTHER_B: &str = "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX";

    fn payment(sequence: u32) -> RawTransaction {
        let payment = Payment {
            common_fields: CommonFields {
                account: OUTER.into(),
                transaction_type: TransactionType::Payment,
                ..Default::default()
            },
            amount: Amount::XRPAmount("1000000".into()),
            destination: DEST.into(),
            ..Default::default()
        }
        .with_sequence(sequence);
        RawTransaction::from_transaction(&payment).unwrap()
    }

    fn batch() -> Batch<'static> {
        Batch::new(
            OUTER,
            BatchFlag::TfAllOrNothing,
            vec![payment(4), payment(5)],
        )
        .with_sequence(3)
        .with_fee("40".into())
    }

    #[test]
    fn test_flag_values() {
        assert_eq!(BatchFlag::TfAllOrNothing as u32, 0x00010000);
        assert_eq!(BatchFlag::TfOnlyOne as u32, 0x00020000);
        assert_eq!(BatchFlag::TfUntilFailure as u32, 0x00040000);
        assert_eq!(BatchFlag::TfIndependent as u32, 0x00080000);
    }

    #[test]
    fn test_inner_normalization() {
        let raw = payment(4);
        let json = raw.as_json();
        assert_eq!(json["Fee"], "0");
        assert_eq!(json["SigningPubKey"], "");
        assert_eq!(json["Flags"], 1073741824u32);
        assert!(json.get("TxnSignature").is_none());
    }

    #[test]
    fn test_serde_shape_and_roundtrip() {
        let batch = batch();
        let json = serde_json::to_value(&batch).unwrap();
        assert_eq!(json["TransactionType"], "Batch");
        assert_eq!(json["Flags"], 65536);
        assert_eq!(json["RawTransactions"][0]["RawTransaction"]["Fee"], "0");
        assert!(json.get("BatchSigners").is_none());
        let back: Batch = serde_json::from_value(json).unwrap();
        assert_eq!(back, batch);
    }

    #[test]
    fn test_validation() {
        assert!(batch().validate().is_ok());

        let mut b = batch();
        b.common_fields.flags = vec![].into();
        assert!(b.validate().is_err(), "no mode");
        let b = batch().with_flag_extra();
        assert!(b.validate().is_err(), "two modes");

        let mut b = batch();
        b.raw_transactions.pop();
        assert!(b.validate().is_err(), "fewer than 2");

        let mut b = batch();
        b.raw_transactions = (4..13).map(payment).collect();
        assert!(b.validate().is_err(), "more than 8");

        let mut b = batch();
        b.raw_transactions[1] = b.raw_transactions[0].clone();
        assert!(b.validate().is_err(), "duplicate");

        let mut b = batch();
        b.raw_transactions[0].0["Fee"] = Value::from("10");
        assert!(b.validate().is_err(), "inner fee");

        let mut b = batch();
        b.raw_transactions[0].0["TxnSignature"] = Value::from("AA");
        assert!(b.validate().is_err(), "inner signature");

        let mut b = batch();
        b.raw_transactions[0].0["Sequence"] = Value::from(0u32);
        assert!(b.validate().is_err(), "sequence 0 without ticket");
    }

    impl Batch<'static> {
        fn with_flag_extra(mut self) -> Self {
            self.common_fields.flags.0.push(BatchFlag::TfOnlyOne);
            self
        }
    }

    #[test]
    fn test_batch_signer_order() {
        let a = BatchSigner::single(OTHER_A, "02AA", "BB");
        let b = BatchSigner::single(OTHER_B, "02AA", "BB");
        let sorted = batch()
            .with_batch_signers(vec![a.clone(), b.clone()])
            .unwrap();
        assert!(sorted.validate().is_ok());

        let mut reversed = sorted.clone();
        reversed.batch_signers.as_mut().unwrap().reverse();
        assert!(reversed.validate().is_err());

        let mut dup = sorted.clone();
        dup.batch_signers = Some(vec![a.clone(), a]);
        assert!(dup.validate().is_err());

        let mut outer = batch();
        outer.batch_signers = Some(vec![BatchSigner::single(OUTER, "02AA", "BB")]);
        assert!(outer.validate().is_err());
    }

    #[test]
    fn test_signing_payload_layout() {
        let batch = batch();
        let single = batch.batch_signing_data(OTHER_A, None).unwrap();
        assert!(single.starts_with("42434800")); // BCH\0
        assert_eq!(single.len() / 2, 4 + 20 + 4 + 4 + 4 + 2 * 32 + 20);
        let multi = batch.batch_signing_data(OTHER_A, Some(OTHER_B)).unwrap();
        assert_eq!(multi.len() / 2, single.len() / 2 + 20);
        // Outer sequence is bound into the payload.
        assert_ne!(
            single,
            batch
                .clone()
                .with_sequence(9)
                .batch_signing_data(OTHER_A, None)
                .unwrap()
        );
    }

    /// Fails if the vendored definitions.json predates Batch.
    #[test]
    fn test_definitions_present() {
        use crate::core::binarycodec::definitions::{
            get_field_instance, get_transaction_type_code,
        };
        assert_eq!(get_transaction_type_code("Batch"), Some(&71));
        for field in [
            "RawTransactions",
            "RawTransaction",
            "BatchSigners",
            "BatchSigner",
        ] {
            assert!(get_field_instance(field).is_some(), "missing field {field}");
        }
    }

    #[test]
    fn test_encode_decode() {
        let hex = crate::core::binarycodec::encode(&batch()).unwrap();
        let decoded = crate::core::binarycodec::decode(&hex).unwrap();
        assert_eq!(decoded["TransactionType"], "Batch");
        assert_eq!(decoded["RawTransactions"].as_array().unwrap().len(), 2);
        assert_eq!(
            decoded["RawTransactions"][0]["RawTransaction"]["TransactionType"],
            "Payment"
        );
    }
}
