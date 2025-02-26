use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Response format for the submit method, which applies a transaction and sends it to
/// the network to be confirmed and included in future ledgers.
///
/// See Submit:
/// `<https://xrpl.org/submit.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Submit<'a> {
    /// Text result code indicating the preliminary result of the transaction,
    /// for example tesSUCCESS
    pub engine_result: Cow<'a, str>,
    /// Numeric version of the result code. Not recommended.
    pub engine_result_code: i32,
    /// Human-readable explanation of the transaction's preliminary result
    pub engine_result_message: Cow<'a, str>,
    /// The complete transaction in hex string format
    pub tx_blob: Cow<'a, str>,
    /// The complete transaction in JSON format
    pub tx_json: Value,
    /// (Omitted in sign-and-submit mode) The value true indicates that the
    /// transaction was applied, queued, broadcast, or kept for later. The value
    /// false indicates that none of those happened, so the transaction cannot
    /// possibly succeed as long as you do not submit it again and have not
    /// already submitted it another time.
    pub accepted: Option<bool>,
    /// (Omitted in sign-and-submit mode) The next Sequence Number available for
    /// the sending account after all pending and queued transactions.
    pub account_sequence_available: Option<u32>,
    /// (Omitted in sign-and-submit mode) The next Sequence Number for the sending
    /// account after all transactions that have been provisionally applied, but
    /// not transactions in the queue.
    pub account_sequence_next: Option<u32>,
    /// (Omitted in sign-and-submit mode) The value true indicates that this
    /// transaction was applied to the open ledger. In this case, the transaction
    /// is likely, but not guaranteed, to be validated in the next ledger version.
    pub applied: Option<bool>,
    /// (Omitted in sign-and-submit mode) The value true indicates this transaction
    /// was broadcast to peer servers in the peer-to-peer XRP Ledger network. The
    /// value false indicates the transaction was not broadcast to any other servers.
    pub broadcast: Option<bool>,
    /// (Omitted in sign-and-submit mode) The value true indicates that the
    /// transaction was kept to be retried later.
    pub kept: Option<bool>,
    /// (Omitted in sign-and-submit mode) The value true indicates the transaction
    /// was put in the Transaction Queue, which means it is likely to be included
    /// in a future ledger version.
    pub queued: Option<bool>,
    /// (Omitted in sign-and-submit mode) The current open ledger cost before
    /// processing this transaction. Transactions with a lower cost are likely
    /// to be queued.
    pub open_ledger_cost: Option<Cow<'a, str>>,
    /// (Omitted in sign-and-submit mode) The ledger index of the newest validated
    /// ledger at the time of submission. This provides a lower bound on the ledger
    /// versions that the transaction can appear in as a result of this request.
    pub validated_ledger_index: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_deserialization() {
        let json = r#"{
            "accepted": true,
            "account_sequence_available": 362,
            "account_sequence_next": 362,
            "applied": true,
            "broadcast": true,
            "engine_result": "tesSUCCESS",
            "engine_result_code": 0,
            "engine_result_message": "The transaction was applied. Only final in a validated ledger.",
            "status": "success",
            "kept": true,
            "open_ledger_cost": "10",
            "queued": false,
            "tx_blob": "1200002280000000240000016961D4838D7EA4C6800000000000000000000000000055534400000000004B4E9C06F24296074F7BC48F92A97916C6DC5EA9684000000000002710732103AB40A0490F9B7ED8DF29D246BF2D6269820A0EE7742ACDD457BEA7C7D0931EDB74473045022100A7CCD11455E47547FF617D5BFC15D120D9053DFD0536B044F10CA3631CD609E502203B61DEE4AC027C5743A1B56AF568D1E2B8E79BB9E9E14744AC87F38375C3C2F181144B4E9C06F24296074F7BC48F92A97916C6DC5EA983143E9D4A2B8AA0780F682D136F7A56D6724EF53754",
            "tx_json": {
                "Account": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
                "DeliverMax": {
                    "currency": "USD",
                    "issuer": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
                    "value": "1"
                },
                "Destination": "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX",
                "Fee": "10000",
                "Flags": 2147483648,
                "Sequence": 361,
                "SigningPubKey": "03AB40A0490F9B7ED8DF29D246BF2D6269820A0EE7742ACDD457BEA7C7D0931EDB",
                "TransactionType": "Payment",
                "TxnSignature": "3045022100A7CCD11455E47547FF617D5BFC15D120D9053DFD0536B044F10CA3631CD609E502203B61DEE4AC027C5743A1B56AF568D1E2B8E79BB9E9E14744AC87F38375C3C2F1",
                "hash": "5B31A7518DC304D5327B4887CD1F7DC2C38D5F684170097020C7C9758B973847"
            },
            "validated_ledger_index": 21184416
        }"#;

        let submit: Submit = serde_json::from_str(json).unwrap();

        // Test required fields
        assert_eq!(submit.engine_result, "tesSUCCESS");
        assert_eq!(submit.engine_result_code, 0);
        assert_eq!(
            submit.engine_result_message,
            "The transaction was applied. Only final in a validated ledger."
        );

        // Test optional fields
        assert_eq!(submit.accepted, Some(true));
        assert_eq!(submit.account_sequence_available, Some(362));
        assert_eq!(submit.account_sequence_next, Some(362));
        assert_eq!(submit.applied, Some(true));
        assert_eq!(submit.broadcast, Some(true));
        assert_eq!(submit.kept, Some(true));
        assert_eq!(submit.open_ledger_cost, Some("10".into()));
        assert_eq!(submit.queued, Some(false));
        assert_eq!(submit.validated_ledger_index, Some(21184416));

        // Test tx_json contents
        let tx_json = submit.tx_json.as_object().unwrap();
        assert_eq!(tx_json["Account"], "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn");
        assert_eq!(tx_json["TransactionType"], "Payment");
        assert_eq!(tx_json["Fee"], "10000");
        assert_eq!(tx_json["Sequence"], 361);

        // Test DeliverMax object in tx_json
        let deliver_max = tx_json["DeliverMax"].as_object().unwrap();
        assert_eq!(deliver_max["currency"], "USD");
        assert_eq!(deliver_max["issuer"], "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn");
        assert_eq!(deliver_max["value"], "1");

        // Test serialization roundtrip
        let serialized = serde_json::to_string(&submit).unwrap();
        let deserialized: Submit = serde_json::from_str(&serialized).unwrap();
        assert_eq!(submit, deserialized);
    }
}
