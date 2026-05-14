use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Request};

/// The transaction_entry method retrieves information on a
/// single transaction from a specific ledger version.
/// (The tx method, by contrast, searches all ledgers for
/// the specified transaction. We recommend using that
/// method instead.)
///
/// See Transaction Entry:
/// `<https://xrpl.org/transaction_entry.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TransactionEntry<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// Unique hash of the transaction you are looking up.
    pub tx_hash: Cow<'a, str>,
    /// The unique identifier of a ledger.
    #[serde(flatten)]
    pub ledger_lookup: Option<LookupByLedgerRequest<'a>>,
}

impl<'a> Model for TransactionEntry<'a> {}

impl<'a> Request<'a> for TransactionEntry<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> TransactionEntry<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        tx_hash: Cow<'a, str>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<LedgerIndex<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::TransactionEntry,
                id,
            },
            tx_hash,
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_round_trip() {
        let req = TransactionEntry::new(
            Some("te-1".into()),
            "C53ECF838647FA5A4C780377025FEC7999AB4182590510CA461444B207AB74A9".into(),
            None,
            Some(LedgerIndex::Int(56865245)),
        );
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: TransactionEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
        assert!(serialized.contains("\"command\":\"transaction_entry\""));
    }
}
