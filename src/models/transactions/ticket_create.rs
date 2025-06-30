use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model,
};
use crate::models::{FlagCollection, NoFlags};

use super::{CommonFields, CommonTransactionBuilder};

/// Sets aside one or more sequence numbers as Tickets.
///
/// See TicketCreate:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/ticketcreate>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct TicketCreate<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// How many Tickets to create. This must be a positive number and cannot cause
    /// the account to own more than 250 Tickets after executing this transaction.
    pub ticket_count: u32,
}

impl<'a> Model for TicketCreate<'a> {}

impl<'a> Transaction<'a, NoFlags> for TicketCreate<'a> {
    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, NoFlags> for TicketCreate<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> TicketCreate<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        ticket_count: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::TicketCreate,
                account_txn_id,
                fee,
                Some(FlagCollection::default()),
                last_ledger_sequence,
                memos,
                None,
                sequence,
                signers,
                None,
                source_tag,
                ticket_sequence,
                None,
            ),
            ticket_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = TicketCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::TicketCreate,
                fee: Some("10".into()),
                sequence: Some(381),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            ticket_count: 10,
        };

        let default_json_str = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"TicketCreate","Fee":"10","Flags":0,"Sequence":381,"SigningPubKey":"","TicketCount":10}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: TicketCreate = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let ticket_create = TicketCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::TicketCreate,
                ..Default::default()
            },
            ticket_count: 10,
        }
        .with_fee("10".into())
        .with_sequence(381)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345);

        assert_eq!(ticket_create.ticket_count, 10);
        assert_eq!(ticket_create.common_fields.fee.as_ref().unwrap().0, "10");
        assert_eq!(ticket_create.common_fields.sequence, Some(381));
        assert_eq!(
            ticket_create.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(ticket_create.common_fields.source_tag, Some(12345));
    }

    #[test]
    fn test_default() {
        let ticket_create = TicketCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::TicketCreate,
                ..Default::default()
            },
            ticket_count: 5,
        };

        assert_eq!(
            ticket_create.common_fields.account,
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"
        );
        assert_eq!(
            ticket_create.common_fields.transaction_type,
            TransactionType::TicketCreate
        );
        assert_eq!(ticket_create.ticket_count, 5);
    }

    #[test]
    fn test_multiple_tickets() {
        let ticket_create = TicketCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::TicketCreate,
                fee: Some("10".into()),
                sequence: Some(381),
                ..Default::default()
            },
            ticket_count: 250, // Maximum allowed
        };

        assert_eq!(ticket_create.ticket_count, 250);
        assert_eq!(ticket_create.common_fields.fee.as_ref().unwrap().0, "10");
        assert_eq!(ticket_create.common_fields.sequence, Some(381));
    }
}
