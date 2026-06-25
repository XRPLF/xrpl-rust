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
use super::mptoken_issuance_set::validate_holder_address;
use super::{CommonTransactionBuilder, Memo, Signer};

/// Claws back issued currency amount or MPT issued by the sender.
///
/// For IssuedCurrencyAmount: `amount.issuer` must be the token holder's address
/// and `Holder` must be absent.
/// For MPTAmount: `Holder` must be present and must not equal `Account`.
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
    /// The amount to claw back. Must be IssuedCurrencyAmount or MPTAmount (not XRP).
    /// For ICA: `amount.issuer` must be the holder's address.
    /// For MPT: supply the `holder` field instead.
    pub amount: Amount<'a>,
    /// (MPT only) The account to claw back from. Required when `amount` is
    /// MPTAmount; must not equal the transaction `account`.
    pub holder: Option<Cow<'a, str>>,
}

pub trait ClawbackError {
    fn _get_amount_error(&self) -> crate::models::XRPLModelResult<()>;
    fn _get_holder_error(&self) -> crate::models::XRPLModelResult<()>;
}

impl<'a> ClawbackError for Clawback<'a> {
    fn _get_amount_error(&self) -> crate::models::XRPLModelResult<()> {
        if self.amount.is_xrp() {
            return Err(XRPLClawbackException::AmountMustNotBeXRP.into());
        }
        self.amount.get_errors()
    }

    fn _get_holder_error(&self) -> crate::models::XRPLModelResult<()> {
        match &self.amount {
            Amount::IssuedCurrencyAmount(ica) => {
                if self.common_fields.account == ica.issuer {
                    return Err(XRPLClawbackException::IssuerMustNotEqualAccount.into());
                }
                if self.holder.is_some() {
                    return Err(XRPLClawbackException::HolderMustNotBePresentForIOU.into());
                }
                Ok(())
            }
            Amount::MPTAmount(_) => match &self.holder {
                None => Err(XRPLClawbackException::HolderRequiredForMPT.into()),
                Some(holder) if holder.as_ref() == self.common_fields.account.as_ref() => {
                    Err(XRPLClawbackException::HolderMustNotEqualAccount.into())
                }
                Some(holder) => validate_holder_address(holder.as_ref()),
            },
            Amount::XRPAmount(_) => Ok(()),
        }
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
    use crate::utils::testing::test_constants::*;

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

        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let actual: serde_json::Value = serde_json::from_str(&serialized_string).unwrap();
        let expected: serde_json::Value = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(actual, expected);

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
        let actual: serde_json::Value = serde_json::from_str(&serialized_string).unwrap();
        let expected: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(actual, expected);

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

    #[test]
    fn test_new_constructor() {
        let clawback = Clawback::new(
            "rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S".into(),
            None,
            Some("12".into()),
            Some(7108682),
            None,
            Some(123),
            None,
            Some(12345),
            None,
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "FOO".into(),
                "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                "314.159".into(),
            )),
            None,
        );

        assert_eq!(
            clawback.common_fields.account,
            "rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S"
        );
        assert_eq!(
            clawback.common_fields.transaction_type,
            TransactionType::Clawback
        );
        assert_eq!(clawback.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(clawback.common_fields.sequence, Some(123));
        assert_eq!(clawback.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(clawback.common_fields.source_tag, Some(12345));
        assert!(clawback.holder.is_none());
        assert!(clawback.validate().is_ok());
    }

    #[test]
    fn test_with_holder_builder() {
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
        .with_holder("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into());

        assert_eq!(
            clawback.holder.as_deref(),
            Some("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW")
        );
    }

    #[test]
    fn test_transaction_trait_getters() {
        let mut clawback = Clawback {
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

        assert_eq!(
            Transaction::get_transaction_type(&clawback),
            &TransactionType::Clawback
        );
        assert_eq!(
            Transaction::get_common_fields(&clawback).account,
            "rp6abvbTbjoce8ZDJkT6snvxTZSYMBCC9S"
        );

        let common_mut = Transaction::get_mut_common_fields(&mut clawback);
        common_mut.sequence = Some(42);
        assert_eq!(clawback.common_fields.sequence, Some(42));
    }

    #[test]
    fn test_clawback_ica_valid_holder_differs_from_account() {
        let account = ACCOUNT_HOLDER;
        let holder = ACCOUNT_HOLDER_2;
        let amount =
            Amount::IssuedCurrencyAmount(crate::models::amount::IssuedCurrencyAmount::new(
                "USD".into(),
                holder.into(),
                "100".into(),
            ));

        let clawback = Clawback {
            common_fields: CommonFields {
                account: account.into(),
                transaction_type: TransactionType::Clawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            amount,
            holder: None,
        };

        assert!(clawback.get_errors().is_ok());
    }

    #[test]
    fn test_clawback_ica_rejects_self_clawback() {
        let account = ACCOUNT_HOLDER;
        let amount =
            Amount::IssuedCurrencyAmount(crate::models::amount::IssuedCurrencyAmount::new(
                "USD".into(),
                account.into(),
                "100".into(),
            ));

        let clawback = Clawback {
            common_fields: CommonFields {
                account: account.into(),
                transaction_type: TransactionType::Clawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            amount,
            holder: None,
        };

        assert!(clawback.get_errors().is_err());
    }

    #[test]
    fn test_clawback_mpt_valid() {
        let account = ACCOUNT_HOLDER;
        let holder = ACCOUNT_HOLDER_2;
        let amount = Amount::MPTAmount(crate::models::amount::MPTAmount::new(
            "100".into(),
            "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
        ));
        let clawback = Clawback {
            common_fields: CommonFields {
                account: account.into(),
                transaction_type: TransactionType::Clawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            amount,
            holder: Some(holder.into()),
        };
        assert!(clawback.get_errors().is_ok());
    }

    #[test]
    fn test_clawback_mpt_missing_holder() {
        let account = ACCOUNT_HOLDER;
        let amount = Amount::MPTAmount(crate::models::amount::MPTAmount::new(
            "100".into(),
            "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
        ));
        let clawback = Clawback {
            common_fields: CommonFields {
                account: account.into(),
                transaction_type: TransactionType::Clawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            amount,
            holder: None,
        };
        assert!(clawback.get_errors().is_err());
    }

    #[test]
    fn test_clawback_mpt_holder_equals_account() {
        let account = ACCOUNT_HOLDER;
        let amount = Amount::MPTAmount(crate::models::amount::MPTAmount::new(
            "100".into(),
            "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
        ));
        let clawback = Clawback {
            common_fields: CommonFields {
                account: account.into(),
                transaction_type: TransactionType::Clawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            amount,
            holder: Some(account.into()),
        };
        assert!(clawback.get_errors().is_err());
    }

    #[test]
    fn test_clawback_mpt_invalid_holder_address() {
        let account = ACCOUNT_HOLDER;
        let amount = Amount::MPTAmount(crate::models::amount::MPTAmount::new(
            "100".into(),
            "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
        ));
        let clawback = Clawback {
            common_fields: CommonFields {
                account: account.into(),
                transaction_type: TransactionType::Clawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            amount,
            holder: Some("not-a-valid-xrpl-address".into()),
        };
        assert!(clawback.get_errors().is_err());
    }

    #[test]
    fn test_clawback_mpt_rejects_malformed_issuance_id() {
        // mpt_issuance_id must be 48 hex chars (192-bit Hash192). This is only
        // 8 chars — MPTAmount::get_errors() must reject it via _get_amount_error().
        let account = ACCOUNT_HOLDER_2;
        let holder = ACCOUNT_HOLDER;
        let clawback = Clawback {
            common_fields: CommonFields {
                account: account.into(),
                transaction_type: TransactionType::Clawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            amount: Amount::MPTAmount(crate::models::amount::MPTAmount::new(
                "100".into(),
                "DEADBEEF".into(), // too short — not a valid Hash192
            )),
            holder: Some(holder.into()),
        };
        assert!(
            clawback.get_errors().is_err(),
            "expected get_errors() to fail for malformed mpt_issuance_id"
        );
    }
}
