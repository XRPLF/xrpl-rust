use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::CommonFields;
use crate::models::{
    amount::Amount,
    transactions::{Transaction, TransactionType},
    Model, ValidateCurrencies,
};
use crate::models::{FlagCollection, NoFlags};

use super::exceptions::XRPLClawbackException;
use super::{CommonTransactionBuilder, Memo, Signer};

/// Claw back tokens from a token holder.
///
/// The issuer can only claw back issued tokens if the issuer has set
/// the `asfAllowTrustLineClawback` flag on their account using an
/// AccountSet transaction.
///
/// See Clawback:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/clawback>`
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
pub struct Clawback<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The amount of currency to claw back. For fungible tokens, the `issuer`
    /// field of the Amount object is the token holder's address (the account
    /// from which tokens are being clawed back).
    pub amount: Amount<'a>,
    /// For MPT (Multi-Purpose Token) clawback only. The address of the token
    /// holder from which tokens should be clawed back. Must not be present
    /// for standard issued-currency clawback.
    pub holder: Option<Cow<'a, str>>,
}

pub trait ClawbackError {
    fn _get_amount_error(&self) -> crate::models::XRPLModelResult<()>;
    fn _get_holder_error(&self) -> crate::models::XRPLModelResult<()>;
}

impl<'a> ClawbackError for Clawback<'a> {
    /// Validate that the Amount is not XRP.
    fn _get_amount_error(&self) -> crate::models::XRPLModelResult<()> {
        if self.amount.is_xrp() {
            Err(XRPLClawbackException::AmountMustNotBeXRP.into())
        } else {
            Ok(())
        }
    }

    /// Validate that the Holder field is not present for standard IOU clawback.
    fn _get_holder_error(&self) -> crate::models::XRPLModelResult<()> {
        if let Amount::IssuedCurrencyAmount(_) = &self.amount {
            if self.holder.is_some() {
                return Err(XRPLClawbackException::HolderMustNotBePresentForIOU.into());
            }
        }
        Ok(())
    }
}

impl<'a> Model for Clawback<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self._get_amount_error()?;
        self._get_holder_error()?;
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for Clawback<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for Clawback<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> Clawback<'a> {
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
        amount: Amount<'a>,
        holder: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::Clawback,
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
            amount,
            holder,
        }
    }

    pub fn with_holder(mut self, holder: Cow<'a, str>) -> Self {
        self.holder = Some(holder);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::amount::IssuedCurrencyAmount;

    #[test]
    fn test_serde() {
        let default_txn = Clawback {
            common_fields: CommonFields {
                account: "rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S".into(),
                transaction_type: TransactionType::Clawback,
                fee: Some("12".into()),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "FOO".into(),
                "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                "314.159".into(),
            )),
            holder: None,
        };

        let default_json_str = r#"{"Account":"rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S","TransactionType":"Clawback","Fee":"12","Flags":0,"SigningPubKey":"","Amount":{"currency":"FOO","issuer":"rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW","value":"314.159"}}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: Clawback = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_serde_with_holder() {
        let txn = Clawback {
            common_fields: CommonFields {
                account: "rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S".into(),
                transaction_type: TransactionType::Clawback,
                fee: Some("12".into()),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "FOO".into(),
                "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                "314.159".into(),
            )),
            holder: Some("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into()),
        };

        let json_str = r#"{"Account":"rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S","TransactionType":"Clawback","Fee":"12","Flags":0,"SigningPubKey":"","Amount":{"currency":"FOO","issuer":"rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW","value":"314.159"},"Holder":"rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"}"#;

        let serialized_string = serde_json::to_string(&txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        let expected_value = serde_json::to_value(json_str).unwrap();
        assert_eq!(serialized_value, expected_value);

        let deserialized: Clawback = serde_json::from_str(json_str).unwrap();
        assert_eq!(txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let clawback = Clawback {
            common_fields: CommonFields {
                account: "rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S".into(),
                transaction_type: TransactionType::Clawback,
                ..Default::default()
            },
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "FOO".into(),
                "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                "314.159".into(),
            )),
            ..Default::default()
        }
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345);

        assert_eq!(
            clawback.common_fields.account,
            "rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S"
        );
        assert_eq!(clawback.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(clawback.common_fields.sequence, Some(123));
        assert_eq!(clawback.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(clawback.common_fields.source_tag, Some(12345));
        assert_eq!(
            clawback.common_fields.transaction_type,
            TransactionType::Clawback
        );
        assert!(clawback.holder.is_none());
    }

    #[test]
    fn test_validation_xrp_amount_rejected() {
        use crate::models::amount::XRPAmount;
        use crate::models::transactions::exceptions::{
            XRPLClawbackException, XRPLTransactionException,
        };
        use crate::models::XRPLModelException;

        let clawback = Clawback {
            common_fields: CommonFields {
                account: "rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S".into(),
                transaction_type: TransactionType::Clawback,
                ..Default::default()
            },
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            ..Default::default()
        };

        let err = clawback.validate().unwrap_err();
        assert!(
            matches!(
                err,
                XRPLModelException::XRPLTransactionError(
                    XRPLTransactionException::XRPLClawbackError(
                        XRPLClawbackException::AmountMustNotBeXRP
                    )
                )
            ),
            "Expected AmountMustNotBeXRP, got: {:?}",
            err
        );
    }

    #[test]
    fn test_validation_holder_present_for_iou_rejected() {
        use crate::models::transactions::exceptions::{
            XRPLClawbackException, XRPLTransactionException,
        };
        use crate::models::XRPLModelException;

        let clawback = Clawback {
            common_fields: CommonFields {
                account: "rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S".into(),
                transaction_type: TransactionType::Clawback,
                ..Default::default()
            },
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "FOO".into(),
                "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                "314.159".into(),
            )),
            holder: Some("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into()),
        };

        let err = clawback.validate().unwrap_err();
        assert!(
            matches!(
                err,
                XRPLModelException::XRPLTransactionError(
                    XRPLTransactionException::XRPLClawbackError(
                        XRPLClawbackException::HolderMustNotBePresentForIOU
                    )
                )
            ),
            "Expected HolderMustNotBePresentForIOU, got: {:?}",
            err
        );
    }

    #[test]
    fn test_validation_valid_iou_clawback() {
        let clawback = Clawback {
            common_fields: CommonFields {
                account: "rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S".into(),
                transaction_type: TransactionType::Clawback,
                ..Default::default()
            },
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "FOO".into(),
                "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                "314.159".into(),
            )),
            holder: None,
        };

        assert!(
            clawback.validate().is_ok(),
            "Valid IOU clawback should pass validation"
        );
    }

    #[test]
    fn test_default() {
        let clawback = Clawback {
            common_fields: CommonFields {
                account: "rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S".into(),
                transaction_type: TransactionType::Clawback,
                ..Default::default()
            },
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                "100".into(),
            )),
            ..Default::default()
        };

        assert_eq!(
            clawback.common_fields.account,
            "rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S"
        );
        assert_eq!(
            clawback.common_fields.transaction_type,
            TransactionType::Clawback
        );
        assert!(clawback.holder.is_none());
        assert!(clawback.common_fields.fee.is_none());
        assert!(clawback.common_fields.sequence.is_none());
    }
}
