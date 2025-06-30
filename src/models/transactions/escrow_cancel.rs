use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::CommonFields;
use crate::models::{
    transactions::{Transaction, TransactionType},
    Model,
};
use crate::models::{FlagCollection, NoFlags};

use super::{CommonTransactionBuilder, Memo, Signer};

/// Cancels an Escrow and returns escrowed XRP to the sender.
///
/// See EscrowCancel:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/escrowcancel>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct EscrowCancel<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// Address of the source account that funded the escrow payment.
    pub owner: Cow<'a, str>,
    /// Transaction sequence (or Ticket number) of EscrowCreate transaction that created the escrow to cancel.
    pub offer_sequence: u32,
}

impl<'a> Model for EscrowCancel<'a> {}

impl<'a> Transaction<'a, NoFlags> for EscrowCancel<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for EscrowCancel<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> EscrowCancel<'a> {
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
        owner: Cow<'a, str>,
        offer_sequence: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::EscrowCancel,
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
            owner,
            offer_sequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = EscrowCancel {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::EscrowCancel,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            owner: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            offer_sequence: 7,
        };

        let default_json_str = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"EscrowCancel","Flags":0,"SigningPubKey":"","Owner":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","OfferSequence":7}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: EscrowCancel = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let escrow_cancel = EscrowCancel {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::EscrowCancel,
                ..Default::default()
            },
            owner: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            offer_sequence: 7,
        }
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345);

        assert_eq!(escrow_cancel.owner, "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn");
        assert_eq!(escrow_cancel.offer_sequence, 7);
        assert_eq!(escrow_cancel.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(escrow_cancel.common_fields.sequence, Some(123));
        assert_eq!(
            escrow_cancel.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(escrow_cancel.common_fields.source_tag, Some(12345));
    }

    #[test]
    fn test_default() {
        let escrow_cancel = EscrowCancel {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::EscrowCancel,
                ..Default::default()
            },
            owner: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            offer_sequence: 7,
        };

        assert_eq!(
            escrow_cancel.common_fields.account,
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"
        );
        assert_eq!(
            escrow_cancel.common_fields.transaction_type,
            TransactionType::EscrowCancel
        );
        assert_eq!(escrow_cancel.owner, "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn");
        assert_eq!(escrow_cancel.offer_sequence, 7);
    }
}
