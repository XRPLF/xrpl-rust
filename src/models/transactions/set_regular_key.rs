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

/// You can protect your account by assigning a regular key pair to
/// it and using it instead of the master key pair to sign transactions
/// whenever possible. If your regular key pair is compromised, but
/// your master key pair is not, you can use a SetRegularKey transaction
/// to regain control of your account.
///
/// See SetRegularKey:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/setregularkey>`
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
pub struct SetRegularKey<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// A base-58-encoded Address that indicates the regular key pair to be
    /// assigned to the account. If omitted, removes any existing regular key
    /// pair from the account. Must not match the master key pair for the address.
    pub regular_key: Option<Cow<'a, str>>,
}

impl<'a> Model for SetRegularKey<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for SetRegularKey<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for SetRegularKey<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> SetRegularKey<'a> {
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
        regular_key: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::SetRegularKey,
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
            regular_key,
        }
    }

    pub fn with_regular_key(mut self, regular_key: Cow<'a, str>) -> Self {
        self.regular_key = Some(regular_key);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = SetRegularKey {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::SetRegularKey,
                fee: Some("12".into()),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            regular_key: Some("rAR8rR8sUkBoCZFawhkWzY4Y5YoyuznwD".into()),
        };

        let default_json_str = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"SetRegularKey","Fee":"12","Flags":0,"SigningPubKey":"","RegularKey":"rAR8rR8sUkBoCZFawhkWzY4Y5YoyuznwD"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: SetRegularKey = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let set_regular_key = SetRegularKey {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::SetRegularKey,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_regular_key("rAR8rR8sUkBoCZFawhkWzY4Y5YoyuznwD".into())
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345);

        assert_eq!(
            set_regular_key.regular_key.as_ref().unwrap(),
            "rAR8rR8sUkBoCZFawhkWzY4Y5YoyuznwD"
        );
        assert_eq!(set_regular_key.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(set_regular_key.common_fields.sequence, Some(123));
        assert_eq!(
            set_regular_key.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(set_regular_key.common_fields.source_tag, Some(12345));
    }

    #[test]
    fn test_remove_regular_key() {
        let set_regular_key = SetRegularKey {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::SetRegularKey,
                fee: Some("12".into()),
                ..Default::default()
            },
            regular_key: None, // Removes existing regular key
        };

        assert!(set_regular_key.regular_key.is_none());
        assert_eq!(set_regular_key.common_fields.fee.as_ref().unwrap().0, "12");
    }

    #[test]
    fn test_default() {
        let set_regular_key = SetRegularKey {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::SetRegularKey,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            set_regular_key.common_fields.account,
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"
        );
        assert_eq!(
            set_regular_key.common_fields.transaction_type,
            TransactionType::SetRegularKey
        );
        assert!(set_regular_key.regular_key.is_none());
    }
}
