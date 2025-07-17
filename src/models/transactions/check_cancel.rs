use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::CommonFields;
use crate::models::{FlagCollection, NoFlags, ValidateCurrencies};
use crate::models::{
    Model,
    transactions::{Transaction, TransactionType},
};

use super::{CommonTransactionBuilder, Memo, Signer};

/// Cancels an unredeemed Check, removing it from the ledger without
/// sending any money. The source or the destination of the check can
/// cancel a Check at any time using this transaction type. If the Check
/// has expired, any address can cancel it.
///
/// See CheckCancel:
/// `<https://xrpl.org/checkcancel.html>`
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
pub struct CheckCancel<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The ID of the Check ledger object to cancel, as a 64-character hexadecimal string.
    #[serde(rename = "CheckID")]
    pub check_id: Cow<'a, str>,
}

impl<'a> Model for CheckCancel<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for CheckCancel<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for CheckCancel<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> CheckCancel<'a> {
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
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::CheckCancel,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = CheckCancel {
            common_fields: CommonFields {
                account: "rUn84CUYbNjRoTQ6mSW7BVJPSVJNLb1QLo".into(),
                transaction_type: TransactionType::CheckCancel,
                fee: Some("12".into()),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            check_id: "49647F0D748DC3FE26BDACBC57F251AADEFFF391403EC9BF87C97F67E9977FB0".into(),
        };

        let default_json_str = r#"{"Account":"rUn84CUYbNjRoTQ6mSW7BVJPSVJNLb1QLo","TransactionType":"CheckCancel","Fee":"12","Flags":0,"SigningPubKey":"","CheckID":"49647F0D748DC3FE26BDACBC57F251AADEFFF391403EC9BF87C97F67E9977FB0"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: CheckCancel = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let check_cancel = CheckCancel {
            common_fields: CommonFields {
                account: "rUn84CUYbNjRoTQ6mSW7BVJPSVJNLb1QLo".into(),
                transaction_type: TransactionType::CheckCancel,
                ..Default::default()
            },
            check_id: "49647F0D748DC3FE26BDACBC57F251AADEFFF391403EC9BF87C97F67E9977FB0".into(),
        }
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345);

        assert_eq!(
            check_cancel.check_id,
            "49647F0D748DC3FE26BDACBC57F251AADEFFF391403EC9BF87C97F67E9977FB0"
        );
        assert_eq!(check_cancel.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(check_cancel.common_fields.sequence, Some(123));
        assert_eq!(
            check_cancel.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(check_cancel.common_fields.source_tag, Some(12345));
    }

    #[test]
    fn test_default() {
        let check_cancel = CheckCancel {
            common_fields: CommonFields {
                account: "rUn84CUYbNjRoTQ6mSW7BVJPSVJNLb1QLo".into(),
                transaction_type: TransactionType::CheckCancel,
                ..Default::default()
            },
            check_id: "49647F0D748DC3FE26BDACBC57F251AADEFFF391403EC9BF87C97F67E9977FB0".into(),
        };

        assert_eq!(
            check_cancel.common_fields.account,
            "rUn84CUYbNjRoTQ6mSW7BVJPSVJNLb1QLo"
        );
        assert_eq!(
            check_cancel.common_fields.transaction_type,
            TransactionType::CheckCancel
        );
        assert_eq!(
            check_cancel.check_id,
            "49647F0D748DC3FE26BDACBC57F251AADEFFF391403EC9BF87C97F67E9977FB0"
        );
    }
}
