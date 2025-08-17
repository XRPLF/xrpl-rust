use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{FlagCollection, NoFlags};
use crate::models::{
    Model, ValidateCurrencies,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

use super::{CommonFields, CommonTransactionBuilder};

/// An AccountDelete transaction deletes an account and any objects it
/// owns in the XRP Ledger, if possible, sending the account's remaining
/// XRP to a specified destination account.
///
/// See AccountDelete:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/accountdelete>`
#[skip_serializing_none]
#[derive(
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    xrpl_rust_macros::ValidateCurrencies,
)]
#[serde(rename_all = "PascalCase")]
pub struct AccountDelete<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The address of an account to receive any leftover XRP after
    /// deleting the sending account. Must be a funded account in
    /// the ledger, and must not be the sending account.
    pub destination: Cow<'a, str>,
    /// Arbitrary destination tag that identifies a hosted
    /// recipient or other information for the recipient
    /// of the deleted account's leftover XRP.
    pub destination_tag: Option<u32>,
}

impl<'a> Model for AccountDelete<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for AccountDelete<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for AccountDelete<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> AccountDelete<'a> {
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
        destination: Cow<'a, str>,
        destination_tag: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::AccountDelete,
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
            destination,
            destination_tag,
        }
    }

    /// Set destination tag
    pub fn with_destination_tag(mut self, tag: u32) -> Self {
        self.destination_tag = Some(tag);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let default_txn = AccountDelete {
            common_fields: CommonFields {
                account: "rWYkbWkCeg8dP6rXALnjgZSjjLyih5NXm".into(),
                transaction_type: TransactionType::AccountDelete,
                fee: Some("2000000".into()),
                sequence: Some(2470665),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            destination: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".into(),
            destination_tag: Some(13),
        };

        let default_json_str = r#"{"Account":"rWYkbWkCeg8dP6rXALnjgZSjjLyih5NXm","TransactionType":"AccountDelete","Fee":"2000000","Flags":0,"Sequence":2470665,"SigningPubKey":"","Destination":"rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe","DestinationTag":13}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: AccountDelete = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let account_delete = AccountDelete {
            common_fields: CommonFields {
                account: "rWYkbWkCeg8dP6rXALnjgZSjjLyih5NXm".into(),
                transaction_type: TransactionType::AccountDelete,
                ..Default::default()
            },
            destination: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".into(),
            ..Default::default()
        }
        .with_destination_tag(13)
        .with_fee("2000000".into())
        .with_sequence(2470665)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_memo(Memo {
            memo_data: Some("deleting account".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(
            account_delete.destination,
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
        );
        assert_eq!(account_delete.destination_tag, Some(13));
        assert_eq!(
            account_delete.common_fields.fee.as_ref().unwrap().0,
            "2000000"
        );
        assert_eq!(account_delete.common_fields.sequence, Some(2470665));
        assert_eq!(
            account_delete.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(account_delete.common_fields.source_tag, Some(12345));
        assert_eq!(
            account_delete.common_fields.memos.as_ref().unwrap().len(),
            1
        );
    }

    #[test]
    fn test_default() {
        let account_delete = AccountDelete {
            common_fields: CommonFields {
                account: "rWYkbWkCeg8dP6rXALnjgZSjjLyih5NXm".into(),
                transaction_type: TransactionType::AccountDelete,
                ..Default::default()
            },
            destination: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".into(),
            ..Default::default()
        };

        assert_eq!(
            account_delete.common_fields.account,
            "rWYkbWkCeg8dP6rXALnjgZSjjLyih5NXm"
        );
        assert_eq!(
            account_delete.common_fields.transaction_type,
            TransactionType::AccountDelete
        );
        assert_eq!(
            account_delete.destination,
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
        );
        assert!(account_delete.destination_tag.is_none());
        assert!(account_delete.common_fields.fee.is_none());
        assert!(account_delete.common_fields.sequence.is_none());
    }

    #[test]
    fn test_minimal_delete() {
        let minimal_delete = AccountDelete {
            common_fields: CommonFields {
                account: "rAccountToDelete123".into(),
                transaction_type: TransactionType::AccountDelete,
                ..Default::default()
            },
            destination: "rDestinationAccount456".into(),
            ..Default::default()
        }
        .with_fee("2000000".into()) // 2 XRP minimum fee for AccountDelete
        .with_sequence(100);

        assert_eq!(minimal_delete.destination, "rDestinationAccount456");
        assert!(minimal_delete.destination_tag.is_none());
        assert_eq!(
            minimal_delete.common_fields.fee.as_ref().unwrap().0,
            "2000000"
        );
        assert_eq!(minimal_delete.common_fields.sequence, Some(100));
    }

    #[test]
    fn test_with_destination_tag() {
        let tagged_delete = AccountDelete {
            common_fields: CommonFields {
                account: "rDeleteAccount789".into(),
                transaction_type: TransactionType::AccountDelete,
                ..Default::default()
            },
            destination: "rExchange123".into(), // Exchange account
            ..Default::default()
        }
        .with_destination_tag(987654321) // Exchange customer ID
        .with_fee("2000000".into())
        .with_sequence(200)
        .with_memo(Memo {
            memo_data: Some("closing account".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(tagged_delete.destination, "rExchange123");
        assert_eq!(tagged_delete.destination_tag, Some(987654321));
        assert_eq!(tagged_delete.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_delete = AccountDelete {
            common_fields: CommonFields {
                account: "rTicketUser111".into(),
                transaction_type: TransactionType::AccountDelete,
                ..Default::default()
            },
            destination: "rDestination222".into(),
            ..Default::default()
        }
        .with_ticket_sequence(54321)
        .with_fee("2000000".into());

        assert_eq!(ticket_delete.common_fields.ticket_sequence, Some(54321));
        assert_eq!(ticket_delete.destination, "rDestination222");
        // When using tickets, sequence should be None or 0
        assert!(ticket_delete.common_fields.sequence.is_none());
    }

    #[test]
    fn test_multiple_memos() {
        let multi_memo_delete = AccountDelete {
            common_fields: CommonFields {
                account: "rMultiMemoAccount333".into(),
                transaction_type: TransactionType::AccountDelete,
                ..Default::default()
            },
            destination: "rFinalDestination444".into(),
            ..Default::default()
        }
        .with_memo(Memo {
            memo_data: Some("reason 1".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_memo(Memo {
            memo_data: Some("reason 2".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_destination_tag(555)
        .with_fee("2000000".into())
        .with_sequence(300);

        assert_eq!(
            multi_memo_delete
                .common_fields
                .memos
                .as_ref()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(multi_memo_delete.destination_tag, Some(555));
        assert_eq!(multi_memo_delete.common_fields.sequence, Some(300));
    }
}
