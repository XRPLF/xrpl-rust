//! Batch transaction (XLS-56, BatchV1_1).
//!
//! See: <https://xls.xrpl.org/xls/XLS-0056-batch.html>

use alloc::borrow::Cow;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt::Debug;

use serde::de::{DeserializeOwned, Deserializer};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use sha2::{Digest, Sha512};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::core::binarycodec::{encode, types::AccountId};
use crate::models::transactions::TF_INNER_BATCH_TXN;
use crate::models::{Model, XRPAmount, XRPLModelException, XRPLModelResult};

use super::exceptions::XRPLTransactionException;
use super::{
    CommonFields, CommonTransactionBuilder, Signer, Transaction, TransactionType,
    TRANSACTION_HASH_PREFIX,
};

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

    pub fn from_transactions<'a, T, F>(transaction: &[&T]) -> XRPLModelResult<Vec<Self>>
    where
        T: Transaction<'a, F>,
        F: IntoEnumIterator + Serialize + Debug + PartialEq,
    {
        transaction
            .iter()
            .map(|v| Self::from_json(serde_json::to_value(v)?))
            .collect::<XRPLModelResult<Vec<Self>>>()
    }

    pub fn from_json(mut json: Value) -> XRPLModelResult<Self> {
        let Some(object) = json.as_object_mut() else {
            return Err(invalid("raw_transactions", "a JSON object", "non-object"));
        };

        if object.get("TransactionType").and_then(Value::as_str) == Some("Batch") {
            return Err(invalid("raw_transactions", "no nested Batch", "Batch"));
        }
        let flags = match object.get("Flags") {
            None => 0,
            Some(Value::Number(n)) => n
                .as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| invalid("Flags", "UInt32", n.to_string()))?,
            Some(Value::String(s)) => s
                .parse::<u32>()
                .map_err(|_| invalid("Flags", "UInt32", s.clone()))?,
            Some(v) => return Err(invalid("Flags", "UInt32", v.to_string())),
        };
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

    pub fn get_field<T: DeserializeOwned>(&self, field: &str) -> XRPLModelResult<Option<T>> {
        let value = self
            .0
            .get(field)
            .map(|v| serde_json::from_value::<T>(v.clone()))
            .transpose()?;

        Ok(value)
    }

    pub fn set_field<T: Serialize>(&mut self, field: &str, value: &T) {
        self.0[field] = serde_json::json!(value);
    }

    /// Transaction ID of the inner transaction (SHA-512Half of `TXN\0 || blob`).
    /// `Transaction::get_hash` can't be used: it refuses unsigned transactions.
    pub fn transaction_hash(&self) -> XRPLModelResult<String> {
        let encoded = encode(&self.0).map_err(XRPLTransactionException::XRPLCoreError)?;
        let mut bytes = TRANSACTION_HASH_PREFIX.to_be_bytes().to_vec();
        bytes.extend(hex::decode(encoded)?);
        let hash = Sha512::digest(&bytes);
        Ok(hex::encode_upper(&hash[..32]))
    }

    pub(crate) fn validate(&self, idx: usize) -> XRPLModelResult<()> {
        let field = format!("raw_transactions[{}].{}", idx, self.transaction_type()?);
        let Some(object) = self.0.as_object() else {
            return Err(invalid(&field, "a JSON object", "non-object"));
        };

        if object.get("Account").and_then(Value::as_str).is_none() {
            return Err(XRPLModelException::MissingField(format!("{field}.Account")));
        }

        if self.transaction_type()? == TransactionType::Batch {
            return Err(invalid(&field, "no nested Batch", "Batch")); // temINVALID_INNER_BATCH
        }

        let flags = match object.get("Flags") {
            None => 0,
            Some(value) => json_u32(Some(value))
                .ok_or_else(|| invalid(&field, "Flags as UInt32", value.to_string()))?,
        };

        if flags & TF_INNER_BATCH_TXN == 0 {
            return Err(invalid(&field, "tfInnerBatchTxn set", "flag missing")); // temINVALID_FLAG
        }

        match object.get("Fee") {
            Some(Value::String(s)) if s == "0" => {}
            Some(Value::Number(n)) if n.as_u64() == Some(0) => {}
            None | Some(Value::Null) => {
                return Err(invalid(&field, "Fee \"0\"", "missing Fee")); // temBAD_FEE
            }
            Some(other) => {
                return Err(invalid(&field, "Fee \"0\"", other.to_string())); // temBAD_FEE
            }
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

        fn json_u32(value: Option<&Value>) -> Option<u32> {
            match value? {
                Value::Number(n) => n.as_u64().and_then(|n| u32::try_from(n).ok()),
                Value::String(s) => s.parse().ok(),
                _ => None,
            }
        }

        let sequence = json_u32(object.get("Sequence"));
        let ticket = json_u32(object.get("TicketSequence"));
        let has_ticket_key = object.contains_key("TicketSequence");

        // Either a nonzero Sequence with no ticket key, or Sequence 0 with a ticket.
        let ok = match sequence {
            Some(0) => ticket.is_some(),
            Some(_) => !has_ticket_key,
            None => false,
        };
        if !ok {
            return Err(invalid(
                &field,
                "Sequence XOR TicketSequence (ticketed: Sequence 0)", // temSEQ_AND_TICKET
                format!("Sequence={sequence:?}, TicketSequence={ticket:?}"),
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
        if partial || single == multi || self.signers.as_ref().is_some_and(Vec::is_empty) {
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

impl<'a> Model for Batch<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.get_flag_error()?;
        self.get_raw_transactions_error()?;
        self.get_batch_signers_error()
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

    fn raw_transactions_mut(&mut self) -> Option<&mut Vec<RawTransaction>> {
        Some(&mut self.raw_transactions)
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
        flag: BatchFlag,
        raw_transactions: Vec<RawTransaction>,
        batch_signers: Option<Vec<BatchSigner>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::from_account(account)
                .with_transaction_type(TransactionType::Batch)
                .with_flag(flag),
            raw_transactions,
            batch_signers,
        }
    }

    pub fn with_transaction<'b, T, F>(mut self, transactions: &T) -> XRPLModelResult<Self>
    where
        T: Transaction<'b, F>,
        F: IntoEnumIterator + Serialize + Debug + PartialEq,
    {
        let raw = RawTransaction::from_transaction(transactions)?;

        self.raw_transactions.push(raw);

        Ok(self)
    }

    pub fn with_transactions<'b, T, F>(mut self, transactions: &[&T]) -> XRPLModelResult<Self>
    where
        T: Transaction<'b, F>,
        F: IntoEnumIterator + Serialize + Debug + PartialEq,
    {
        let raws = RawTransaction::from_transactions(transactions)?;

        self.raw_transactions.extend(raws);

        Ok(self)
    }

    /// Replace the batch mode.
    pub fn with_flag(mut self, mode: BatchFlag) -> Self {
        self.common_fields.flags = alloc::vec![mode].into();
        self
    }

    pub fn with_sequence(mut self, sequence: u32) -> Self {
        self.common_fields.sequence = Some(sequence);
        self
    }

    pub fn with_fee(mut self, amount: impl Into<XRPAmount<'static>>) -> Self {
        self.common_fields.fee = Some(amount.into());

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

    /// `Sequence`, or `TicketSequence` when `Sequence` is 0 (XLS-56 §2.1.3.2).
    pub fn outer_sequence(&self) -> XRPLModelResult<u32> {
        match (
            self.common_fields.sequence,
            self.common_fields.ticket_sequence,
        ) {
            (Some(sequence), None) if sequence != 0 => Ok(sequence),
            (Some(0), Some(ticket)) => Ok(ticket),
            (Some(sequence), Some(ticket)) if sequence != 0 => Err(invalid(
                "sequence",
                "Sequence XOR TicketSequence (ticketed: Sequence 0)",
                format!("Sequence={sequence}, TicketSequence={ticket}"),
            )),
            _ => Err(XRPLModelException::MissingField("sequence".to_string())),
        }
    }

    pub(crate) fn mode_flags(&self) -> u32 {
        self.common_fields
            .flags
            .0
            .iter()
            .fold(0u32, |acc, flag| acc | *flag as u32)
    }

    fn get_flag_error(&self) -> XRPLModelResult<()> {
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

    fn get_raw_transactions_error(&self) -> XRPLModelResult<()> {
        let count = self.raw_transactions.len();
        if !(MIN_RAW_TRANSACTIONS..=MAX_RAW_TRANSACTIONS).contains(&count) {
            return Err(invalid(
                "raw_transactions",
                "between 2 and 8 transactions", // temARRAY_EMPTY / malformed
                format!("{count} transactions"),
            ));
        }

        for (idx, raw) in self.raw_transactions.iter().enumerate() {
            raw.validate(idx)?;
        }

        Ok(())
    }

    fn get_batch_signers_error(&self) -> XRPLModelResult<()> {
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

    /// Accounts involved in the inner transactions, de-duplicated, outer account
    /// included if it appears. Per inner transaction: its `Delegate` if present,
    /// else its `Account`; its `Counterparty`; and its `Sponsor` only when a
    /// `SponsorSignature` placeholder selects the co-signed path.
    pub fn involved_accounts(&self) -> Vec<String> {
        let mut accounts: Vec<String> = Vec::new();
        let mut add = |address: &str| {
            if !accounts.iter().any(|a| a == address) {
                accounts.push(address.to_string());
            }
        };

        for raw in &self.raw_transactions {
            let json = raw.as_json();
            if let Some(a) = str_field(json, "Delegate").or_else(|| str_field(json, "Account")) {
                add(a);
            }

            if let Some(c) = str_field(json, "Counterparty") {
                add(c);
            }

            let has_placeholder = json.get("SponsorSignature").is_some_and(|v| !v.is_null());
            if let (Some(s), true) = (str_field(json, "Sponsor"), has_placeholder) {
                add(s);
            }
        }

        accounts
    }

    /// Accounts that must appear in `BatchSigners`: involved accounts minus the
    /// outer account, in AccountID order.
    pub fn required_batch_signer_accounts(&self) -> XRPLModelResult<Vec<String>> {
        let outer: &str = &self.common_fields.account;

        let accounts = self
            .involved_accounts()
            .into_iter()
            .filter(|a| a != outer)
            .collect();

        sort_by_account_id(accounts, |a: &String| a.as_str())
    }

    /// Best-effort, opt-in check that `BatchSigners` covers exactly the accounts
    /// that must authorize the inner transactions.
    ///
    /// Not called by `validate`, the signing helpers, or `combine_batch_signers`:
    /// a batch with no or partial `BatchSigners` is a valid draft, and a single
    /// signer's fragment is intentionally incomplete. Call this yourself after
    /// merging all signatures, right before submitting.
    ///
    /// Checks that every account in
    /// [`required_batch_signer_accounts`](Self::required_batch_signer_accounts)
    /// has an entry, and that no entry exists for any other account. It does not
    /// verify signatures or ledger-side authorization.
    ///
    /// The required set is derived client-side by
    /// [`involved_accounts`](Self::involved_accounts), so it can disagree with the
    /// ledger if the rules change. rippled's preflight is authoritative (an
    /// incomplete batch is rejected with a `tem` code and no fee is charged).
    ///
    /// # Errors
    ///
    /// [`XRPLModelException::InvalidValue`] on `batch_signers`, naming the first
    /// missing or unexpected account.
    pub fn validate_batch_signers(&self) -> XRPLModelResult<()> {
        let required = self.required_batch_signer_accounts()?;
        let declared: Vec<&str> = self
            .batch_signers
            .iter()
            .flatten()
            .map(|s| s.account.as_str())
            .collect();

        if let Some(missing) = required.iter().find(|r| !declared.contains(&r.as_str())) {
            return Err(invalid(
                "batch_signers",
                "a signer for every inner account",
                format!("missing {missing}"),
            ));
        }
        if let Some(extra) = declared.iter().find(|d| !required.iter().any(|r| r == **d)) {
            return Err(invalid(
                "batch_signers",
                "only accounts that authorize an inner transaction",
                (*extra).to_string(),
            ));
        }
        Ok(())
    }
}

pub(crate) fn invalid(field: &str, expected: &str, found: impl Into<String>) -> XRPLModelException {
    XRPLModelException::InvalidValue {
        field: field.to_string(),
        expected: expected.to_string(),
        found: found.into(),
    }
}

pub fn account_id_bytes(address: &str) -> XRPLModelResult<[u8; 20]> {
    let id = AccountId::try_from(address)
        .map_err(|_| invalid("account", "a valid classic address", address))?;

    let bytes: &[u8] = id.as_ref();

    <[u8; 20]>::try_from(bytes).map_err(|_| invalid("account", "a 20-byte AccountID", address))
}

/// Sorts by AccountID bytes (the ledger order), NOT by the base58 string.
pub(crate) fn sort_by_account_id<T>(
    items: Vec<T>,
    account: impl Fn(&T) -> &str,
) -> XRPLModelResult<Vec<T>> {
    let mut keyed = Vec::with_capacity(items.len());
    for item in items {
        keyed.push((account_id_bytes(account(&item))?, item));
    }
    keyed.sort_by_key(|a| a.0);
    Ok(keyed.into_iter().map(|(_, item)| item).collect())
}

fn str_field<'v>(json: &'v Value, key: &str) -> Option<&'v str> {
    json.get(key).and_then(Value::as_str)
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use serde_json::json;

    use super::*;
    use crate::models::amount::Amount;
    use crate::models::transactions::payment::Payment;

    const OUTER: &str = "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb";
    const DEST: &str = "rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK";
    const OTHER_A: &str = "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn";
    const OTHER_B: &str = "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX";

    type Fields = serde_json::Map<String, Value>;

    // -----------------------------------------------------------------------
    // Fixtures and assertion helpers
    // -----------------------------------------------------------------------

    fn payment_from(account: &str, sequence: u32) -> RawTransaction {
        let payment = Payment {
            common_fields: CommonFields {
                account: account.into(),
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

    /// Inner payment from the outer account.
    fn payment(sequence: u32) -> RawTransaction {
        payment_from(OUTER, sequence)
    }

    /// Normalized inner transaction built from raw JSON.
    fn raw(json: Value) -> RawTransaction {
        RawTransaction::from_json(json).unwrap()
    }

    fn signer(account: &str) -> Signer {
        Signer::new(account.to_string(), "BB".to_string(), "02AA".to_string())
    }

    fn single(account: &str) -> BatchSigner {
        BatchSigner::single(account, "02AA", "BB")
    }

    fn batch_of(raw_transactions: Vec<RawTransaction>) -> Batch<'static> {
        Batch::new(OUTER, BatchFlag::TfAllOrNothing, raw_transactions, None)
            .with_sequence(3)
            .with_fee("40")
    }

    /// Two inner payments from the outer account, all-or-nothing, no signers.
    fn batch() -> Batch<'static> {
        batch_of(vec![payment(4), payment(5)])
    }

    /// `InvalidValue` naming exactly `expected_field`, so a test can't pass
    /// because some other rule fired.
    macro_rules! assert_invalid {
        ($result:expr, $field:expr $(,)?) => {
            match $result {
                Err(XRPLModelException::InvalidValue { field, .. }) => {
                    assert_eq!(field, $field)
                }
                other => panic!("expected InvalidValue on `{}`, got {:?}", $field, other),
            }
        };
    }

    macro_rules! assert_missing {
        ($result:expr, $field:expr $(,)?) => {
            match $result {
                Err(XRPLModelException::MissingField(field)) => assert_eq!(field, $field),
                other => panic!("expected MissingField `{}`, got {:?}", $field, other),
            }
        };
    }

    /// Validation result of `batch()` after mutating its second inner transaction.
    fn inner_errors(mutate: impl FnOnce(&mut Fields)) -> XRPLModelResult<()> {
        let mut batch = batch();
        mutate(batch.raw_transactions[1].0.as_object_mut().unwrap());

        batch.get_errors()
    }

    fn assert_inner_rejected(mutate: impl FnOnce(&mut Fields), tx_type: &str) {
        let field = format!("raw_transactions[1].{}", tx_type);
        assert_invalid!(inner_errors(mutate), field);
    }

    fn assert_inner_accepted(mutate: impl FnOnce(&mut Fields)) {
        assert!(inner_errors(mutate).is_ok());
    }

    // RawTransaction: construction

    #[test]
    fn test_from_transaction_normalizes_inner() {
        let raw = payment(4);
        let json = raw.as_json();

        assert_eq!(json["Fee"], "0");
        assert_eq!(json["SigningPubKey"], "");
        assert_eq!(json["Flags"], TF_INNER_BATCH_TXN);
        assert_eq!(json["Sequence"], 4);
        assert!(json.get("TxnSignature").is_none());
        assert!(json.get("Signers").is_none());
    }

    #[test]
    fn test_from_json_rejects_malformed_input() {
        assert!(RawTransaction::from_json(Value::from("nope")).is_err());
        assert!(RawTransaction::from_json(json!({"TransactionType": "Batch"})).is_err());

        let with_flags = |flags: Value| {
            json!({
                "TransactionType": "Payment", "Account": OUTER, "Sequence": 4, "Flags": flags
            })
        };
        assert!(RawTransaction::from_json(with_flags(Value::from("not-a-number"))).is_err());
        assert!(
            RawTransaction::from_json(with_flags(Value::from(u64::from(u32::MAX) + 1))).is_err()
        );
        assert!(RawTransaction::from_json(with_flags(Value::from(-1))).is_err());
        assert!(RawTransaction::from_json(with_flags(Value::Bool(true))).is_err());
        assert!(RawTransaction::from_json(with_flags(Value::Null)).is_err());
    }

    #[test]
    fn test_from_json_preserves_and_forces_fields() {
        let with_flags = |flags: Value| {
            json!({
                "TransactionType": "Payment", "Account": OUTER, "Sequence": 4, "Flags": flags
            })
        };

        // Existing flags (number and decimal string) are kept, with the inner bit added.
        for flags in [Value::from(2u32), Value::from("2")] {
            let raw = RawTransaction::from_json(with_flags(flags)).unwrap();
            assert_eq!(raw.as_json()["Flags"], Value::from(2 | TF_INNER_BATCH_TXN));
        }

        // Missing Flags default to just tfInnerBatchTxn; signature fields are stripped
        // and Fee / SigningPubKey are forced.
        let raw = raw(json!({
            "TransactionType": "Payment", "Account": OUTER, "Sequence": 4,
            "Fee": "12", "SigningPubKey": "02AA", "TxnSignature": "AA", "Signers": []
        }));
        let json = raw.as_json();
        assert_eq!(json["Flags"], Value::from(TF_INNER_BATCH_TXN));
        assert_eq!(json["Fee"], "0");
        assert_eq!(json["SigningPubKey"], "");
        assert!(json.get("TxnSignature").is_none());
        assert!(json.get("Signers").is_none());
    }

    #[test]
    fn test_from_json_ticketed_gets_explicit_sequence_zero() {
        let ticketed = raw(json!({
            "TransactionType": "Payment", "Account": OUTER, "TicketSequence": 9
        }));
        assert_eq!(ticketed.as_json()["Sequence"], Value::from(0u32));

        // An explicit Sequence is left alone.
        let explicit = raw(json!({
            "TransactionType": "Payment", "Account": OUTER, "Sequence": 5, "TicketSequence": 9
        }));

        assert_eq!(explicit.as_json()["Sequence"], Value::from(5u32));
    }

    // RawTransaction: accessors and hashing
    #[test]
    fn test_raw_transaction_accessors() {
        let raw = payment(4);
        assert_eq!(raw.account(), Some(OUTER));
        assert_eq!(raw.transaction_type().unwrap(), TransactionType::Payment);

        let missing = RawTransaction(json!({"Account": OUTER}));
        assert_missing!(missing.transaction_type(), "TransactionType");
        assert_eq!(RawTransaction(Value::Null).account(), None);
    }

    #[test]
    fn test_raw_transaction_get_and_set_field() {
        let mut raw = payment(4);

        assert_eq!(
            raw.get_field::<String>("Destination").unwrap(),
            Some(DEST.to_string())
        );
        assert_eq!(raw.get_field::<String>("Memos").unwrap(), None);
        assert!(raw.get_field::<u32>("Destination").is_err(), "wrong type");

        raw.set_field("DestinationTag", &7u32);
        assert_eq!(raw.get_field::<u32>("DestinationTag").unwrap(), Some(7));
    }

    #[test]
    fn test_transaction_hash() {
        let hash = payment(4).transaction_hash().unwrap();

        assert_eq!(hash.len(), 64);
        assert!(hash
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase()));

        // Deterministic, and sensitive to the content.
        assert_eq!(hash, payment(4).transaction_hash().unwrap());
        assert_ne!(hash, payment(5).transaction_hash().unwrap());

        let mut changed = payment(4);
        changed.set_field("DestinationTag", &1u32);
        assert_ne!(hash, changed.transaction_hash().unwrap());
    }

    #[test]
    fn test_raw_transaction_serde_shape_and_roundtrip() {
        let raw = payment(4);
        let value = serde_json::to_value(&raw).unwrap();
        assert_eq!(value["RawTransaction"], *raw.as_json());

        let back: RawTransaction = serde_json::from_value(value).unwrap();
        assert_eq!(back, raw);
    }

    // RawTransaction: validation
    #[test]
    fn test_raw_transaction_validate_requires_object_and_account() {
        assert_invalid!(
            RawTransaction(serde_json::json!({"Account": OUTER, "TransactionType": "Payment"}))
                .validate(0),
            "raw_transactions[0].Payment",
        );

        let mut raw = payment(4);
        raw.0.as_object_mut().unwrap().remove("Account");
        assert_missing!(raw.validate(0), "raw_transactions[0].Payment.Account");
    }

    #[test]
    fn test_inner_rejects_nested_batch() {
        assert_inner_rejected(
            |o| {
                o.insert("TransactionType".into(), "Batch".into());
            },
            "Batch",
        );
    }

    #[test]
    fn test_inner_flag_rules() {
        assert_inner_rejected(
            |o| {
                o.insert("Flags".into(), 0.into());
            },
            "Payment",
        );
        assert_inner_rejected(
            |o| {
                o.insert("Flags".into(), "not-a-number".into());
            },
            "Payment",
        );
        assert_inner_accepted(|o| {
            o.insert("Flags".into(), TF_INNER_BATCH_TXN.to_string().into());
        });
    }

    #[test]
    fn test_inner_fee_rules() {
        assert_inner_rejected(
            |o| {
                o.remove("Fee");
            },
            "Payment",
        );
        assert_inner_rejected(
            |o| {
                o.insert("Fee".into(), Value::Null);
            },
            "Payment",
        );
        assert_inner_rejected(
            |o| {
                o.insert("Fee".into(), "10".into());
            },
            "Payment",
        );
        assert_inner_accepted(|o| {
            o.insert("Fee".into(), 0.into());
        });
    }

    #[test]
    fn test_inner_signature_rules() {
        assert_inner_rejected(
            |o| {
                o.insert("TxnSignature".into(), "AA".into());
            },
            "Payment",
        );
        assert_inner_rejected(
            |o| {
                o.insert("SigningPubKey".into(), "02AA".into());
            },
            "Payment",
        );
        assert_inner_rejected(
            |o| {
                o.remove("SigningPubKey");
            },
            "Payment",
        );
        assert_inner_rejected(
            |o| {
                o.insert("Signers".into(), Value::Array(vec![]));
            },
            "Payment",
        );
    }

    #[test]
    fn test_inner_sequence_rules() {
        // Sequence is required.
        assert_inner_rejected(
            |o| {
                o.remove("Sequence");
            },
            "Payment",
        );
        // Sequence 0 needs a ticket.
        assert_inner_rejected(
            |o| {
                o.insert("Sequence".into(), 0.into());
            },
            "Payment",
        );
        assert_inner_rejected(
            |o| {
                o.insert("Sequence".into(), 0.into());
                o.insert("TicketSequence".into(), Value::Null);
            },
            "Payment",
        );
        // A nonzero Sequence can't come with a ticket key.
        assert_inner_rejected(
            |o| {
                o.insert("TicketSequence".into(), 9.into());
            },
            "Payment",
        );
        assert_inner_rejected(
            |o| {
                o.insert("TicketSequence".into(), Value::Null);
            },
            "Payment",
        );
        // Ticketed: Sequence 0 + TicketSequence.
        assert_inner_accepted(|o| {
            o.insert("Sequence".into(), 0.into());
            o.insert("TicketSequence".into(), 9.into());
        });
    }

    // -----------------------------------------------------------------------
    // BatchSigner
    // -----------------------------------------------------------------------

    #[test]
    fn test_batch_signer_single_roundtrip() {
        let entry = single(OTHER_A);
        let json = serde_json::to_value(&entry).unwrap();

        assert_eq!(json["BatchSigner"]["Account"], OTHER_A);
        assert!(json["BatchSigner"].get("Signers").is_none());

        let back: BatchSigner = serde_json::from_value(json).unwrap();
        assert_eq!(back, entry);
    }

    #[test]
    fn test_batch_signer_multi_sorts_and_roundtrips() {
        let entry = BatchSigner::multi(OTHER_A, vec![signer(OTHER_B), signer(DEST)]).unwrap();
        let signers = entry.signers.as_ref().unwrap();

        assert!(
            account_id_bytes(&signers[0].account).unwrap()
                < account_id_bytes(&signers[1].account).unwrap()
        );
        assert!(entry.signing_pub_key.is_none() && entry.txn_signature.is_none());
        assert!(entry.validate(0).is_ok());

        let json = serde_json::to_value(&entry).unwrap();
        assert!(json["BatchSigner"]["Signers"].is_array());
        assert!(json["BatchSigner"].get("SigningPubKey").is_none());

        let text = serde_json::to_string(&entry).unwrap();
        let back: BatchSigner = serde_json::from_str(&text).unwrap();
        assert_eq!(back, entry);
    }

    #[test]
    fn test_batch_signer_multi_rejects_bad_address() {
        assert_invalid!(
            BatchSigner::multi(OTHER_A, vec![signer("bogus")]),
            "account",
        );
    }

    #[test]
    fn test_batch_signer_validate() {
        let ok = single(OTHER_A);
        assert!(ok.validate(0).is_ok());

        let mut pub_key_only = ok.clone();
        pub_key_only.txn_signature = None;
        assert_invalid!(pub_key_only.validate(2), "batch_signers[2]");

        let mut sig_only = ok.clone();
        sig_only.signing_pub_key = None;
        assert_invalid!(sig_only.validate(0), "batch_signers[0]");

        let neither = BatchSigner {
            account: OTHER_A.into(),
            ..Default::default()
        };
        assert_invalid!(neither.validate(0), "batch_signers[0]");

        let mut both = ok.clone();
        both.signers = Some(vec![signer(OTHER_B)]);
        assert_invalid!(both.validate(0), "batch_signers[0]");

        let mut empty_multi = ok.clone();
        empty_multi.signing_pub_key = None;
        empty_multi.txn_signature = None;
        empty_multi.signers = Some(vec![]);
        assert_invalid!(empty_multi.validate(0), "batch_signers[0]");

        let mut single_with_empty = ok;
        single_with_empty.signers = Some(vec![]);
        assert_invalid!(single_with_empty.validate(0), "batch_signers[0]");
    }

    // Batch: construction and builders
    #[test]
    fn test_new_sets_type_account_and_mode() {
        let b = batch();

        assert_eq!(b.get_transaction_type(), &TransactionType::Batch);
        assert_eq!(&*b.common_fields.account, OUTER);
        assert!(b.has_flag(&BatchFlag::TfAllOrNothing));
        assert_eq!(b.raw_transactions.len(), 2);
        assert!(b.batch_signers.is_none());
    }

    #[test]
    fn test_sequence_and_fee_builders() {
        let b = batch();

        assert_eq!(b.common_fields.sequence, Some(3));
        assert!(b.common_fields.fee.is_some());
    }

    #[test]
    fn test_with_flag_replaces_mode() {
        let b = batch().with_flag(BatchFlag::TfIndependent);

        assert!(b.has_flag(&BatchFlag::TfIndependent));
        assert!(!b.has_flag(&BatchFlag::TfAllOrNothing));
        assert!(b.get_errors().is_ok());
    }

    #[test]
    fn test_with_raw_transaction_appends() {
        let b = batch().with_raw_transaction(payment(6));

        assert_eq!(b.raw_transactions.len(), 3);
        assert!(b.get_errors().is_ok());
    }

    #[test]
    fn test_with_batch_signers_sorts_by_account_id() {
        let b = batch()
            .with_batch_signers(vec![single(OTHER_B), single(OTHER_A)])
            .unwrap();
        let signers = b.batch_signers.as_ref().unwrap();

        assert!(
            account_id_bytes(&signers[0].account).unwrap()
                < account_id_bytes(&signers[1].account).unwrap()
        );
        assert!(b.get_errors().is_ok());
    }

    #[test]
    fn test_add_batch_signer_keeps_order() {
        let b = batch()
            .add_batch_signer(single(OTHER_A))
            .unwrap()
            .add_batch_signer(single(OTHER_B))
            .unwrap();
        let signers = b.batch_signers.as_ref().unwrap();

        assert_eq!(signers.len(), 2);
        assert!(
            account_id_bytes(&signers[0].account).unwrap()
                < account_id_bytes(&signers[1].account).unwrap()
        );
        assert!(b.get_errors().is_ok());
    }

    #[test]
    fn test_with_batch_signers_rejects_bad_address() {
        assert_invalid!(
            batch().with_batch_signers(vec![single("not-an-address")]),
            "account",
        );
    }

    // -----------------------------------------------------------------------
    // Batch: serialization and binary codec
    // -----------------------------------------------------------------------

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
    fn test_serde_roundtrip_with_batch_signers() {
        let batch = batch().with_batch_signers(vec![single(OTHER_A)]).unwrap();
        let json = serde_json::to_value(&batch).unwrap();

        assert_eq!(json["BatchSigners"][0]["BatchSigner"]["Account"], OTHER_A);

        let back: Batch = serde_json::from_value(json).unwrap();
        assert_eq!(back, batch);
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

    // -----------------------------------------------------------------------
    // Batch: validation (TS: validateBatch)
    // -----------------------------------------------------------------------

    #[test]
    fn test_valid_batch_passes() {
        assert!(batch().get_errors().is_ok());
        assert!(batch().validate().is_ok());
    }

    #[test]
    fn test_mode_flags_value() {
        assert_eq!(batch().mode_flags(), 0x00010000);
        assert_eq!(
            batch().with_flag(BatchFlag::TfIndependent).mode_flags(),
            0x00080000
        );

        let mut two = batch();
        two.common_fields.flags.0.push(BatchFlag::TfOnlyOne);
        assert_eq!(two.mode_flags(), 0x00030000, "modes are OR-ed together");
    }

    #[test]
    fn test_exactly_one_mode_required() {
        for mode in BatchFlag::iter() {
            assert!(batch().with_flag(mode).get_errors().is_ok(), "{mode}");
        }

        let mut none = batch();
        none.common_fields.flags = vec![].into();
        assert_invalid!(none.get_errors(), "flags");

        let mut two = batch();
        two.common_fields.flags.0.push(BatchFlag::TfOnlyOne);
        assert_invalid!(two.get_errors(), "flags");
    }

    #[test]
    fn test_raw_transactions_count_bounds() {
        let with_count = |count: u32| batch_of((4..4 + count).map(payment).collect());

        assert_invalid!(with_count(0).get_errors(), "raw_transactions");
        assert_invalid!(with_count(1).get_errors(), "raw_transactions");
        assert!(with_count(MIN_RAW_TRANSACTIONS as u32).get_errors().is_ok());
        assert!(with_count(MAX_RAW_TRANSACTIONS as u32).get_errors().is_ok());
        assert_invalid!(with_count(9).get_errors(), "raw_transactions");
    }

    #[test]
    fn test_inner_error_reports_its_index() {
        // `assert_inner_rejected` mutates index 1, so the index in the error matters.
        let mut b = batch();
        b.raw_transactions[0]
            .0
            .as_object_mut()
            .unwrap()
            .insert("Fee".into(), "10".into());

        assert_invalid!(b.get_errors(), "raw_transactions[0].Payment");
    }

    #[test]
    fn test_batch_signers_must_be_ascending_and_unique() {
        let sorted = batch()
            .with_batch_signers(vec![single(OTHER_A), single(OTHER_B)])
            .unwrap();
        assert!(sorted.get_errors().is_ok());

        let mut reversed = sorted.clone();
        reversed.batch_signers.as_mut().unwrap().reverse();
        assert_invalid!(reversed.get_errors(), "batch_signers[1]");

        let mut duplicate = sorted;
        duplicate.batch_signers = Some(vec![single(OTHER_A), single(OTHER_A)]);
        assert_invalid!(duplicate.get_errors(), "batch_signers[1]");
    }

    #[test]
    fn test_batch_signers_reject_outer_account() {
        let mut b = batch();
        b.batch_signers = Some(vec![single(OUTER)]);

        assert_invalid!(b.get_errors(), "batch_signers[0]");
    }

    #[test]
    fn test_batch_signers_reject_malformed_entries() {
        let mut partial = batch();
        let mut entry = single(OTHER_A);
        entry.txn_signature = None;
        partial.batch_signers = Some(vec![entry]);
        assert_invalid!(partial.get_errors(), "batch_signers[0]");

        let mut bad_address = batch();
        bad_address.batch_signers = Some(vec![single("not-an-address")]);
        assert_invalid!(bad_address.get_errors(), "account");
    }

    #[test]
    fn test_too_many_batch_signers() {
        let mut b = batch();
        b.batch_signers = Some(vec![single(OTHER_A); MAX_BATCH_SIGNERS + 1]);

        assert_invalid!(b.get_errors(), "batch_signers");
    }

    // -----------------------------------------------------------------------
    // Batch: outer sequence
    // -----------------------------------------------------------------------

    #[test]
    fn test_outer_sequence_variants() {
        assert_eq!(batch().outer_sequence().unwrap(), 3);

        let mut ticketed = batch();
        ticketed.common_fields.sequence = Some(0);
        ticketed.common_fields.ticket_sequence = Some(7);
        assert_eq!(ticketed.outer_sequence().unwrap(), 7);

        let mut both = batch();
        both.common_fields.ticket_sequence = Some(7); // Sequence stays 3
        assert_invalid!(both.outer_sequence(), "sequence");

        let mut unset = batch();
        unset.common_fields.sequence = None;
        assert_missing!(unset.outer_sequence(), "sequence");
        unset.common_fields.ticket_sequence = Some(7);
        assert_missing!(unset.outer_sequence(), "sequence"); // ticket needs explicit Sequence 0

        let mut zero = batch();
        zero.common_fields.sequence = Some(0);
        assert_missing!(zero.outer_sequence(), "sequence"); // Sequence 0 needs a ticket
    }

    // -----------------------------------------------------------------------
    // Batch: involved and required signers
    // -----------------------------------------------------------------------

    #[test]
    fn test_involved_accounts_delegate_counterparty_and_dedup() {
        let b = batch_of(vec![
            raw(json!({"TransactionType": "Payment", "Account": OUTER, "Sequence": 4})),
            // The delegate authorizes, not the account holder.
            raw(json!({
                "TransactionType": "Payment", "Account": OTHER_A, "Delegate": OTHER_B, "Sequence": 1
            })),
            // Same account again is de-duplicated; the counterparty is added.
            raw(json!({
                "TransactionType": "Payment", "Account": OUTER, "Counterparty": DEST, "Sequence": 5
            })),
        ]);

        assert_eq!(b.involved_accounts(), vec![OUTER, OTHER_B, DEST]);
    }

    #[test]
    fn test_involved_accounts_sponsor_needs_signature_placeholder() {
        let sponsored = |sponsor_signature: Value| {
            batch_of(vec![
                payment(4),
                raw(json!({
                    "TransactionType": "Payment", "Account": OTHER_A, "Sequence": 1,
                    "Sponsor": OTHER_B, "SponsorSignature": sponsor_signature
                })),
            ])
        };

        // The empty placeholder selects the co-signed path: the sponsor must sign.
        assert_eq!(
            sponsored(json!({})).involved_accounts(),
            vec![OUTER, OTHER_A, OTHER_B]
        );
        // Pre-funded sponsorship: the sponsor stays out.
        assert_eq!(
            sponsored(Value::Null).involved_accounts(),
            vec![OUTER, OTHER_A]
        );
    }

    #[test]
    fn test_required_signers_exclude_outer_and_are_ordered() {
        assert!(batch().required_batch_signer_accounts().unwrap().is_empty());

        let b = batch_of(vec![
            payment(4),
            payment_from(OTHER_B, 1),
            payment_from(OTHER_A, 1),
        ]);
        let required = b.required_batch_signer_accounts().unwrap();

        assert_eq!(required.len(), 2);
        assert!(!required.iter().any(|a| a == OUTER));

        let ids: Vec<_> = required
            .iter()
            .map(|a| account_id_bytes(a).unwrap())
            .collect();
        assert!(ids.windows(2).all(|w| w[0] < w[1]), "AccountID order");
    }

    #[test]
    fn test_validate_batch_signers() {
        // Nobody but the outer account is involved: nothing to sign.
        assert!(batch().validate_batch_signers().is_ok());

        let b = batch_of(vec![
            payment(4),
            payment_from(OTHER_A, 1),
            payment_from(OTHER_A, 2),
        ]);
        assert_eq!(
            b.required_batch_signer_accounts().unwrap(),
            vec![OTHER_A.to_string()]
        );

        // A draft is a valid Batch, but not submittable until the signer is present.
        assert!(b.get_errors().is_ok());
        assert_invalid!(b.validate_batch_signers(), "batch_signers");

        let signed = b.add_batch_signer(single(OTHER_A)).unwrap();
        assert!(signed.validate_batch_signers().is_ok());

        let extra = signed.add_batch_signer(single(OTHER_B)).unwrap();
        assert_invalid!(extra.validate_batch_signers(), "batch_signers");
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    #[test]
    fn test_account_id_bytes() {
        assert_eq!(account_id_bytes(OUTER).unwrap().len(), 20);
        assert_ne!(
            account_id_bytes(OUTER).unwrap(),
            account_id_bytes(OTHER_A).unwrap()
        );
        assert_invalid!(account_id_bytes("not-an-address"), "account");
    }

    #[test]
    fn test_sort_by_account_id_orders_by_bytes() {
        let accounts: Vec<String> = [OTHER_B, OTHER_A, DEST, OUTER]
            .iter()
            .map(|a| a.to_string())
            .collect();
        let sorted = sort_by_account_id(accounts, |a: &String| a.as_str()).unwrap();

        assert_eq!(sorted.len(), 4);
        let ids: Vec<_> = sorted
            .iter()
            .map(|a| account_id_bytes(a).unwrap())
            .collect();
        assert!(ids.windows(2).all(|w| w[0] < w[1]));

        assert_invalid!(
            sort_by_account_id(vec!["bogus".to_string()], |a: &String| a.as_str()),
            "account",
        );
    }
}
