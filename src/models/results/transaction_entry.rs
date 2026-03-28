use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::models::transactions::metadata::TransactionMetadata;

/// Response format for the transaction_entry method.
///
/// See Transaction Entry:
/// `<https://xrpl.org/transaction_entry.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TransactionEntry<'a> {
    /// The ledger index of the ledger version the transaction was found in;
    /// this is the same as the one from the request.
    pub ledger_index: u32,
    /// The identifying hash of the ledger version the transaction was found in;
    /// this is the same as the one from the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The transaction metadata, which shows the exact results of the
    /// transaction in detail.
    pub meta: TransactionMetadata<'a>,
    /// JSON representation of the Transaction object.
    pub tx_json: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_transaction_entry_deserialization() {
        let json = json!({
            "ledger_hash": "793E56131D8D4ABFB27FA383BFC44F2978B046E023FF46C588D7E0C874C2472A",
            "ledger_index": 56865245,
            "meta": {
                "AffectedNodes": [
                    {
                        "ModifiedNode": {
                            "FinalFields": {
                                "ExchangeRate": "4F04C66806CF7400",
                                "Flags": 0,
                                "RootIndex": "02BAAC1E67C1CE0E96F0FA2E8061020536CEDD043FEB0FF54F04C66806CF7400",
                                "TakerGetsCurrency": "0000000000000000000000000000000000000000",
                                "TakerGetsIssuer": "0000000000000000000000000000000000000000",
                                "TakerPaysCurrency": "000000000000000000000000434E590000000000",
                                "TakerPaysIssuer": "CED6E99370D5C00EF4EBF72567DA99F5661BFB3A"
                            },
                            "LedgerEntryType": "DirectoryNode",
                            "LedgerIndex": "02BAAC1E67C1CE0E96F0FA2E8061020536CEDD043FEB0FF54F04C66806CF7400"
                        }
                    }
                ],
                "TransactionIndex": 0,
                "TransactionResult": "tesSUCCESS"
            },
            "tx_json": {
                "Account": "rhhh49pFH96roGyuC4E5P4CHaNjS1k8gzM",
                "Fee": "12",
                "Flags": 0,
                "LastLedgerSequence": 56865248,
                "OfferSequence": 5037708,
                "Sequence": 5037710,
                "SigningPubKey": "03B51A3EDF70E4098DA7FB053A01C5A6A0A163A30ED1445F14F87C7C3295FCB3BE",
                "TakerGets": "15000000000",
                "TakerPays": {
                    "currency": "CNY",
                    "issuer": "rKiCet8SdvWxPXnAgYarFUXMh1zCPz432Y",
                    "value": "20160.75"
                },
                "TransactionType": "OfferCreate",
                "TxnSignature": "3045022100A5023A0E64923616FCDB6D664F569644C7C9D1895772F986CD6B981B515B02A00220530C973E9A8395BC6FE2484948D2751F6B030FC7FB8575D1BFB406368AD554D9",
                "hash": "C53ECF838647FA5A4C780377025FEC7999AB4182590510CA461444B207AB74A9"
            },
            "validated": true
        });

        let entry: TransactionEntry = serde_json::from_value(json).unwrap();

        // Test required fields
        assert_eq!(entry.ledger_index, 56865245);
        assert_eq!(
            entry.ledger_hash,
            Some(Cow::from(
                "793E56131D8D4ABFB27FA383BFC44F2978B046E023FF46C588D7E0C874C2472A"
            ))
        );

        // Test metadata
        assert_eq!(entry.meta.transaction_result, "tesSUCCESS");
        assert_eq!(entry.meta.transaction_index, 0);
        assert_eq!(entry.meta.affected_nodes.len(), 1);

        // Test tx_json contents
        let tx_json = entry.tx_json.as_object().unwrap();
        assert_eq!(tx_json["Account"], "rhhh49pFH96roGyuC4E5P4CHaNjS1k8gzM");
        assert_eq!(tx_json["TransactionType"], "OfferCreate");
        assert_eq!(tx_json["Fee"], "12");
        assert_eq!(tx_json["Sequence"], 5037710);

        // Test TakerPays object in tx_json
        let taker_pays = tx_json["TakerPays"].as_object().unwrap();
        assert_eq!(taker_pays["currency"], "CNY");
        assert_eq!(taker_pays["issuer"], "rKiCet8SdvWxPXnAgYarFUXMh1zCPz432Y");
        assert_eq!(taker_pays["value"], "20160.75");

        // Test serialization roundtrip
        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: TransactionEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entry, deserialized);
    }
}
