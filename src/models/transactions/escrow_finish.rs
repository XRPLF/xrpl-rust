use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{FlagCollection, NoFlags};
use crate::models::{
    Model, ValidateCurrencies, XRPLModelException, XRPLModelResult,
    amount::XRPAmount,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

use super::{CommonFields, CommonTransactionBuilder};

/// Finishes an Escrow and delivers XRP from a held payment to the recipient.
///
/// See EscrowFinish:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/escrowfinish>`
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
pub struct EscrowFinish<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// Address of the source account that funded the held payment.
    pub owner: Cow<'a, str>,
    /// Transaction sequence of EscrowCreate transaction that created the held payment to finish.
    pub offer_sequence: u32,
    /// Hex value matching the previously-supplied PREIMAGE-SHA-256 crypto-condition of the held payment.
    pub condition: Option<Cow<'a, str>>,
    /// Hex value of the PREIMAGE-SHA-256 crypto-condition fulfillment matching the held payment's Condition.
    pub fulfillment: Option<Cow<'a, str>>,
}

impl<'a> Model for EscrowFinish<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_condition_and_fulfillment_error()?;
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for EscrowFinish<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for EscrowFinish<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> EscrowFinish<'a> {
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
        condition: Option<Cow<'a, str>>,
        fulfillment: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::EscrowFinish,
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
            condition,
            fulfillment,
        }
    }

    pub fn with_condition(mut self, condition: Cow<'a, str>) -> Self {
        self.condition = Some(condition);
        self
    }

    pub fn with_fulfillment(mut self, fulfillment: Cow<'a, str>) -> Self {
        self.fulfillment = Some(fulfillment);
        self
    }

    pub fn with_condition_and_fulfillment(
        mut self,
        condition: Cow<'a, str>,
        fulfillment: Cow<'a, str>,
    ) -> Self {
        self.condition = Some(condition);
        self.fulfillment = Some(fulfillment);
        self
    }
}

impl<'a> EscrowFinishError for EscrowFinish<'a> {
    fn _get_condition_and_fulfillment_error(&self) -> XRPLModelResult<()> {
        if (self.condition.is_some() && self.fulfillment.is_none())
            || (self.condition.is_none() && self.fulfillment.is_some())
        {
            Err(XRPLModelException::FieldRequiresField {
                field1: "condition".into(),
                field2: "fulfillment".into(),
            })
        } else {
            Ok(())
        }
    }
}

pub trait EscrowFinishError {
    fn _get_condition_and_fulfillment_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Model;

    #[test]
    fn test_condition_and_fulfillment_error() {
        let escrow_finish = EscrowFinish {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::EscrowFinish,
                ..Default::default()
            },
            owner: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            offer_sequence: 10,
            condition: Some(
                "A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100"
                    .into(),
            ),
            fulfillment: None,
        };

        assert!(escrow_finish.get_errors().is_err());
    }

    #[test]
    fn test_fulfillment_requires_condition() {
        let escrow_finish = EscrowFinish {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::EscrowFinish,
                ..Default::default()
            },
            owner: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            offer_sequence: 10,
            condition: None,
            fulfillment: Some("A0028000".into()),
        };

        assert!(escrow_finish.get_errors().is_err());
    }

    #[test]
    fn test_valid_with_condition_and_fulfillment() {
        let escrow_finish = EscrowFinish {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::EscrowFinish,
                ..Default::default()
            },
            owner: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            offer_sequence: 10,
            condition: Some(
                "A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100"
                    .into(),
            ),
            fulfillment: Some("A0028000".into()),
        };

        assert!(escrow_finish.get_errors().is_ok());
    }

    #[test]
    fn test_valid_without_condition_and_fulfillment() {
        let escrow_finish = EscrowFinish {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::EscrowFinish,
                ..Default::default()
            },
            owner: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            offer_sequence: 10,
            condition: None,
            fulfillment: None,
        };

        assert!(escrow_finish.get_errors().is_ok());
    }

    #[test]
    fn test_serde() {
        let default_txn = EscrowFinish {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::EscrowFinish,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            owner: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            offer_sequence: 7,
            condition: Some(
                "A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100"
                    .into(),
            ),
            fulfillment: Some("A0028000".into()),
        };

        let default_json_str = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"EscrowFinish","Flags":0,"SigningPubKey":"","Owner":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","OfferSequence":7,"Condition":"A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100","Fulfillment":"A0028000"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: EscrowFinish = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let escrow_finish = EscrowFinish {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::EscrowFinish,
                ..Default::default()
            },
            owner: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            offer_sequence: 7,
            ..Default::default()
        }
        .with_condition_and_fulfillment(
            "A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100".into(),
            "A0028000".into(),
        )
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345);

        assert_eq!(escrow_finish.owner, "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn");
        assert_eq!(escrow_finish.offer_sequence, 7);
        assert!(escrow_finish.condition.is_some());
        assert!(escrow_finish.fulfillment.is_some());
        assert_eq!(escrow_finish.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(escrow_finish.common_fields.sequence, Some(123));
        assert_eq!(
            escrow_finish.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(escrow_finish.common_fields.source_tag, Some(12345));
        assert!(escrow_finish.get_errors().is_ok());
    }

    #[test]
    fn test_builder_without_condition() {
        let escrow_finish = EscrowFinish {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::EscrowFinish,
                ..Default::default()
            },
            owner: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            offer_sequence: 7,
            ..Default::default()
        }
        .with_fee("12".into())
        .with_sequence(123);

        assert_eq!(escrow_finish.owner, "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn");
        assert_eq!(escrow_finish.offer_sequence, 7);
        assert!(escrow_finish.condition.is_none());
        assert!(escrow_finish.fulfillment.is_none());
        assert_eq!(escrow_finish.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(escrow_finish.common_fields.sequence, Some(123));
        assert!(escrow_finish.get_errors().is_ok());
    }
}
