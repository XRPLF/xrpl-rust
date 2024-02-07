use crate::models::FlagCollection;
use crate::models::Model;
use crate::models::{ledger::LedgerEntryType, NoFlags};
use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

/// The `Ticket` object type represents a `Ticket`, which tracks an account sequence number that
/// has been set aside for future use. You can create new tickets with a `TicketCreate` transaction.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Ticket<'a> {
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the Ticket model.
    //
    // See Ticket fields:
    // `<https://xrpl.org/ticket.html#ticket-fields>`
    /// The account that owns this Ticket.
    pub account: Cow<'a, str>,
    /// A hint indicating which page of the owner directory links to this object, in case the
    /// directory consists of multiple pages.
    pub owner_node: Cow<'a, str>,
    /// The identifying hash of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most recently
    /// modified this object.
    pub previous_txn_lgr_seq: u32,
    /// The Sequence Number this Ticket sets aside.
    pub ticket_sequence: u32,
}

impl<'a> Model for Ticket<'a> {}

impl<'a> LedgerObject<NoFlags> for Ticket<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> Ticket<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        owner_node: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
        ticket_sequence: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::Ticket,
                index,
                ledger_index,
            },
            account,
            owner_node,
            previous_txn_id,
            previous_txn_lgr_seq,
            ticket_sequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let ticket = Ticket::new(
            Some(Cow::from("ForTest")),
            None,
            Cow::from("rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de"),
            Cow::from("0000000000000000"),
            Cow::from("F19AD4577212D3BEACA0F75FE1BA1644F2E854D46E8D62E9C95D18E9708CBFB1"),
            4,
            3,
        );
        let serialized = serde_json::to_string(&ticket).unwrap();

        let deserialized: Ticket = serde_json::from_str(&serialized).unwrap();

        assert_eq!(ticket, deserialized);
    }
}
