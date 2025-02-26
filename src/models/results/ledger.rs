use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Response from a ledger request, containing information about a specific
/// ledger.
///
/// See Ledger:
/// `<https://xrpl.org/ledger.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ledger<'a> {
    /// The complete ledger header data of this ledger.
    pub ledger: LedgerInner<'a>,
    /// The unique identifying hash of the entire ledger, as hexadecimal.
    pub ledger_hash: Cow<'a, str>,
    /// The Ledger Index of this ledger.
    pub ledger_index: u32,
    /// If true, this is a validated ledger version. If omitted or set to
    /// false, this ledger's data is not final.
    pub validated: Option<bool>,
    /// Array of objects describing queued transactions, in the same order as
    /// the queue.
    pub queue_data: Option<Cow<'a, [QueuedTransaction<'a>]>>,
}

/// The complete ledger header data.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LedgerInner<'a> {
    /// Hash of all account state information in this ledger, as hexadecimal.
    pub account_hash: Cow<'a, str>,
    /// A bit-map of flags relating to the closing of this ledger.
    pub close_flags: u32,
    /// The time this ledger was closed, in seconds since the Ripple Epoch.
    pub close_time: u64,
    /// The time this ledger was closed, in human-readable format.
    /// Always uses UTC.
    pub close_time_human: Option<Cow<'a, str>>,
    /// Ledger close times are rounded to within this many seconds.
    pub close_time_resolution: u32,
    /// Whether or not this ledger has been closed.
    pub closed: bool,
    /// Unique identifying hash of the entire ledger.
    pub ledger_hash: Cow<'a, str>,
    /// The Ledger Index of this ledger.
    pub ledger_index: Cow<'a, str>,
    /// The time at which the previous ledger was closed.
    pub parent_close_time: u64,
    /// The unique identifying hash of the previous ledger, as hexadecimal.
    pub parent_hash: Cow<'a, str>,
    /// Total number of XRP drops in the network, as a quoted integer.
    pub total_coins: Cow<'a, str>,
    /// Hash of the transaction information included in this ledger.
    pub transaction_hash: Cow<'a, str>,
    /// Transactions applied in this ledger version.
    pub transactions: Option<Cow<'a, [Cow<'a, str>]>>,
}

/// Represents a queued transaction in the ledger.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueuedTransaction<'a> {
    /// The Address of the sender for this queued transaction.
    pub account: Cow<'a, str>,
    /// Transaction information, either as hash string or expanded object.
    pub tx: TransactionInfo<'a>,
    /// How many times this transaction can be retried before being dropped.
    pub retries_remaining: u32,
    /// The tentative result from preliminary transaction checking.
    pub preflight_result: Cow<'a, str>,
    /// If this transaction was left in queue after getting a retriable result.
    pub last_result: Option<Cow<'a, str>>,
    /// Whether this transaction changes this address's ways of authorizing
    /// transactions.
    pub auth_change: Option<bool>,
    /// The Transaction Cost of this transaction, in drops of XRP.
    pub fee: Option<Cow<'a, str>>,
    /// The transaction cost relative to the minimum cost, in fee levels.
    pub fee_level: Option<Cow<'a, str>>,
    /// The maximum amount of XRP, in drops, this transaction could send or
    /// destroy.
    pub max_spend_drops: Option<Cow<'a, str>>,
}

/// Transaction information that can be either a hash string or expanded object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum TransactionInfo<'a> {
    Hash(Cow<'a, str>),
    Binary { tx_blob: Cow<'a, str> },
    Json(TransactionObject<'a>),
}

/// Expanded transaction object when requested in JSON format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TransactionObject<'a> {
    /// The identifying hash of the transaction.
    pub hash: Cow<'a, str>,
    // Add other transaction fields as needed
    #[serde(flatten)]
    pub other: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_deserialize() {
        let json = r#"{
            "ledger": {
                "account_hash": "B8B2C0C3F9E75E3AEE31D467B2544AB56244E618890BA58679707D6BFC0AF41D",
                "close_flags": 0,
                "close_time": 752188602,
                "close_time_human": "2023-Nov-01 21:16:42.000000000 UTC",
                "close_time_resolution": 10,
                "closed": true,
                "ledger_hash": "1BEECD5D21592EABDEF98D8E4BC038AD10B5700FF7E98011870DF5D6C2A2F39B",
                "ledger_index": "83626901",
                "parent_close_time": 752188601,
                "parent_hash": "6B32CFC42B32C5FB90019AE17F701D96B499A4C8E148A002E18135A434A19D98",
                "total_coins": "99988256314388830",
                "transaction_hash": "21586C664DC47E12AF34F22EBF1DB55D23F8C98972542BAC0C39B1009CAC84D4"
            },
            "ledger_hash": "1BEECD5D21592EABDEF98D8E4BC038AD10B5700FF7E98011870DF5D6C2A2F39B",
            "ledger_index": 83626901,
            "validated": true
        }"#;

        let result: Ledger = serde_json::from_str(json).unwrap();

        // Test top-level fields
        assert_eq!(
            result.ledger_hash,
            "1BEECD5D21592EABDEF98D8E4BC038AD10B5700FF7E98011870DF5D6C2A2F39B"
        );
        assert_eq!(result.ledger_index, 83626901);
        assert_eq!(result.validated, Some(true));

        // Test inner ledger fields
        let inner = result.ledger;
        assert_eq!(
            inner.account_hash,
            "B8B2C0C3F9E75E3AEE31D467B2544AB56244E618890BA58679707D6BFC0AF41D"
        );
        assert_eq!(inner.close_flags, 0);
        assert_eq!(inner.close_time, 752188602);
        assert_eq!(
            inner.close_time_human,
            Some("2023-Nov-01 21:16:42.000000000 UTC".into())
        );
        assert_eq!(inner.close_time_resolution, 10);
        assert!(inner.closed);
        assert_eq!(
            inner.ledger_hash,
            "1BEECD5D21592EABDEF98D8E4BC038AD10B5700FF7E98011870DF5D6C2A2F39B"
        );
        assert_eq!(inner.ledger_index, "83626901");
        assert_eq!(inner.parent_close_time, 752188601);
        assert_eq!(
            inner.parent_hash,
            "6B32CFC42B32C5FB90019AE17F701D96B499A4C8E148A002E18135A434A19D98"
        );
        assert_eq!(inner.total_coins, "99988256314388830");
        assert_eq!(
            inner.transaction_hash,
            "21586C664DC47E12AF34F22EBF1DB55D23F8C98972542BAC0C39B1009CAC84D4"
        );
    }

    #[test]
    fn test_ledger_serialize() {
        let ledger = Ledger {
            ledger: LedgerInner {
                account_hash: "B8B2C0C3F9E75E3AEE31D467B2544AB56244E618890BA58679707D6BFC0AF41D"
                    .into(),
                close_flags: 0,
                close_time: 752188602,
                close_time_human: Some("2023-Nov-01 21:16:42.000000000 UTC".into()),
                close_time_resolution: 10,
                closed: true,
                ledger_hash: "1BEECD5D21592EABDEF98D8E4BC038AD10B5700FF7E98011870DF5D6C2A2F39B"
                    .into(),
                ledger_index: "83626901".into(),
                parent_close_time: 752188601,
                parent_hash: "6B32CFC42B32C5FB90019AE17F701D96B499A4C8E148A002E18135A434A19D98"
                    .into(),
                total_coins: "99988256314388830".into(),
                transaction_hash:
                    "21586C664DC47E12AF34F22EBF1DB55D23F8C98972542BAC0C39B1009CAC84D4".into(),
                transactions: None,
            },
            ledger_hash: "1BEECD5D21592EABDEF98D8E4BC038AD10B5700FF7E98011870DF5D6C2A2F39B".into(),
            ledger_index: 83626901,
            validated: Some(true),
            queue_data: None,
        };

        let serialized = serde_json::to_string(&ledger).unwrap();
        let deserialized: Ledger = serde_json::from_str(&serialized).unwrap();

        assert_eq!(ledger, deserialized);
    }
}
