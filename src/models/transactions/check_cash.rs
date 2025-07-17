use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::CommonFields;
use crate::models::{
    FlagCollection, NoFlags, ValidateCurrencies, XRPLModelException, XRPLModelResult,
};
use crate::models::{
    Model,
    amount::Amount,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

use super::CommonTransactionBuilder;

/// Attempt to redeem a Check object in the ledger to receive up to the amount
/// authorized by a corresponding CheckCreate transaction.
///
/// See CheckCash:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/checkcash>`
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
pub struct CheckCash<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The ID of the Check ledger object to cash, as a 64-character hexadecimal string.
    #[serde(rename = "CheckID")]
    pub check_id: Cow<'a, str>,
    /// Redeem the Check for exactly this amount, if possible. The currency must match that of the
    /// SendMax of the corresponding CheckCreate transaction. You must provide either this field or DeliverMin.
    pub amount: Option<Amount<'a>>,
    /// Redeem the Check for at least this amount and for as much as possible. The currency must
    /// match that of the SendMax of the corresponding CheckCreate transaction. You must provide
    /// either this field or Amount.
    pub deliver_min: Option<Amount<'a>>,
}

impl<'a> Model for CheckCash<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_amount_and_deliver_min_error()?;
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for CheckCash<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for CheckCash<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> CheckCash<'a> {
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
        check_id: Cow<'a, str>,
        amount: Option<Amount<'a>>,
        deliver_min: Option<Amount<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::CheckCash,
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
            check_id,
            amount,
            deliver_min,
        }
    }

    pub fn with_amount(mut self, amount: Amount<'a>) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn with_deliver_min(mut self, deliver_min: Amount<'a>) -> Self {
        self.deliver_min = Some(deliver_min);
        self
    }
}

impl<'a> CheckCashError for CheckCash<'a> {
    fn _get_amount_and_deliver_min_error(&self) -> XRPLModelResult<()> {
        if (self.amount.is_none() && self.deliver_min.is_none())
            || (self.amount.is_some() && self.deliver_min.is_some())
        {
            Err(XRPLModelException::InvalidFieldCombination {
                field: "amount",
                other_fields: &["deliver_min"],
            })
        } else {
            Ok(())
        }
    }
}

pub trait CheckCashError {
    fn _get_amount_and_deliver_min_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Model;

    #[test]
    fn test_amount_and_deliver_min_error() {
        let check_cash = CheckCash {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::CheckCash,
                ..Default::default()
            },
            check_id: "".into(),
            amount: None,
            deliver_min: None,
        };

        assert!(check_cash.get_errors().is_err());
    }

    #[test]
    fn test_both_amount_and_deliver_min_error() {
        let check_cash = CheckCash {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::CheckCash,
                ..Default::default()
            },
            check_id: "".into(),
            amount: Some(Amount::XRPAmount("100000000".into())),
            deliver_min: Some(Amount::XRPAmount("50000000".into())),
        };

        assert!(check_cash.get_errors().is_err());
    }

    #[test]
    fn test_valid_with_amount() {
        let check_cash = CheckCash {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::CheckCash,
                ..Default::default()
            },
            check_id: "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334".into(),
            amount: Some(Amount::XRPAmount("100000000".into())),
            deliver_min: None,
        };

        assert!(check_cash.get_errors().is_ok());
    }

    #[test]
    fn test_valid_with_deliver_min() {
        let check_cash = CheckCash {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::CheckCash,
                ..Default::default()
            },
            check_id: "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334".into(),
            amount: None,
            deliver_min: Some(Amount::XRPAmount("50000000".into())),
        };

        assert!(check_cash.get_errors().is_ok());
    }

    #[test]
    fn test_serde() {
        let default_txn = CheckCash {
            common_fields: CommonFields {
                account: "rfkE1aSy9G8Upk4JssnwBxhEv5p4mn2KTy".into(),
                transaction_type: TransactionType::CheckCash,
                fee: Some("12".into()),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            check_id: "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334".into(),
            amount: Some("100000000".into()),
            deliver_min: None,
        };

        let default_json_str = r#"{"Account":"rfkE1aSy9G8Upk4JssnwBxhEv5p4mn2KTy","TransactionType":"CheckCash","Fee":"12","Flags":0,"SigningPubKey":"","CheckID":"838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334","Amount":"100000000"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: CheckCash = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let check_cash = CheckCash {
            common_fields: CommonFields {
                account: "rfkE1aSy9G8Upk4JssnwBxhEv5p4mn2KTy".into(),
                transaction_type: TransactionType::CheckCash,
                ..Default::default()
            },
            check_id: "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334".into(),
            ..Default::default()
        }
        .with_amount(Amount::XRPAmount("100000000".into()))
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345);

        assert_eq!(
            check_cash.check_id,
            "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334"
        );
        assert!(check_cash.amount.is_some());
        assert_eq!(check_cash.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(check_cash.common_fields.sequence, Some(123));
        assert_eq!(check_cash.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(check_cash.common_fields.source_tag, Some(12345));
        assert!(check_cash.get_errors().is_ok());
    }
}
