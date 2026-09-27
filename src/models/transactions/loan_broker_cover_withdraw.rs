use alloc::{borrow::Cow, string::ToString, vec::Vec};
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::{
        validate_credential_ids, vault_common::validate_hash256, CommonTransactionBuilder, Memo,
        Signer,
    },
    Amount, FlagCollection, Model, NoFlags, ValidateCurrencies, XRPAmount, XRPLModelException,
};

use super::{CommonFields, Transaction, TransactionType};

/// Withdraws first-loss capital from a LoanBroker ledger entry.
/// Only the owner of the associated LoanBroker entry can
/// initiate this transaction. If you already hold the asset,
/// a self-destination withdrawal succeeds regardless of the
/// issuer's DefaultRipple setting since it is only checked when
/// a new trust line needs to be created.
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
pub struct LoanBrokerCoverWithdraw<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The Loan Broker ID that the transaction is modifying.
    #[serde(rename = "LoanBrokerID")]
    pub loan_broker_id: Cow<'a, str>,
    /// The First-Loss Capital amount to deposit.
    pub amount: Amount<'a>,
    /// An account to receive the assets. It must be able to receive the asset.
    pub destination: Option<Cow<'a, str>>,
    /// Arbitrary tag identifying the reason for the transaction to the destination.
    pub destination_tag: Option<u32>,
    /// The credentials to authorize the withdrawal when the destination is gated by a permissioned domain (XLS-70).
    #[serde(rename = "CredentialIDs")]
    pub credential_ids: Option<Vec<Cow<'a, str>>>,
}

impl Model for LoanBrokerCoverWithdraw<'_> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()?;

        let value = match &self.amount {
            Amount::MPTAmount(amount) => amount.value.as_ref(),
            Amount::IssuedCurrencyAmount(amount) => amount.value.as_ref(),
            Amount::XRPAmount(amount) => amount.0.as_ref(),
        };

        let parsed = bigdecimal::BigDecimal::from_str(value).map_err(|_| {
            XRPLModelException::InvalidValueFormat {
                field: "amount".to_string(),
                format: "a valid decimal number".to_string(),
                found: value.to_string(),
            }
        })?;

        if parsed <= 0 {
            return Err(XRPLModelException::InvalidValue {
                field: "amount".to_string(),
                expected: "a positive amount".to_string(),
                found: value.to_string(),
            });
        }

        validate_hash256("loan_broker_id", &self.loan_broker_id)?;

        validate_credential_ids(&self.credential_ids)?;

        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for LoanBrokerCoverWithdraw<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> CommonTransactionBuilder<'a, NoFlags> for LoanBrokerCoverWithdraw<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> LoanBrokerCoverWithdraw<'a> {
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
        loan_broker_id: Cow<'a, str>,
        amount: Amount<'a>,
        destination: Option<Cow<'a, str>>,
        destination_tag: Option<u32>,
    ) -> LoanBrokerCoverWithdraw<'a> {
        LoanBrokerCoverWithdraw {
            common_fields: CommonFields::new(
                account,
                TransactionType::LoanBrokerCoverWithdraw,
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
            loan_broker_id,
            amount,
            destination,
            destination_tag,
            credential_ids: None,
        }
    }

    /// Set the Destination field.
    pub fn with_destination(mut self, destination: Cow<'a, str>) -> Self {
        self.destination = Some(destination);
        self
    }

    /// Set the DestinationTag field.
    pub fn with_destination_tag(mut self, destination_tag: u32) -> Self {
        self.destination_tag = Some(destination_tag);
        self
    }

    /// Set the credentials authorizing this withdrawal.
    pub fn with_credential_ids(mut self, credential_ids: Vec<Cow<'a, str>>) -> Self {
        self.credential_ids = Some(credential_ids);
        self
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;

    use super::*;

    const ACCOUNT: &str = "r9LqNeG6qHxLoanBrokerCoverWithdraw5weJ9";
    const LOAN_BROKER_ID: &str = "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD";
    const DESTINATION: &str = "rf7HPydP4ihkFkSRHWFq34b4SXRc7GvPCR";

    #[test]
    fn test_new_and_builder_methods() {
        let tx = LoanBrokerCoverWithdraw::new(
            ACCOUNT.into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            LOAN_BROKER_ID.into(),
            Amount::XRPAmount(XRPAmount::from("1000000")),
            None,
            None,
        );

        assert_eq!(tx.destination, None);
        assert_eq!(tx.destination_tag, None);
        assert_eq!(
            tx.get_transaction_type(),
            &TransactionType::LoanBrokerCoverWithdraw
        );

        let tx = tx
            .with_destination(DESTINATION.into())
            .with_destination_tag(32);

        assert_eq!(tx.destination, Some(DESTINATION.into()));
        assert_eq!(tx.destination_tag, Some(32));
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_serde() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: Some(DESTINATION.into()),
            destination_tag: Some(32),
            credential_ids: None,
        };

        let default_json_str = r#"{"Account":"r9LqNeG6qHxLoanBrokerCoverWithdraw5weJ9","TransactionType":"LoanBrokerCoverWithdraw","Flags":0,"SigningPubKey":"","LoanBrokerID":"E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD","Amount":"1000000","Destination":"rf7HPydP4ihkFkSRHWFq34b4SXRc7GvPCR","DestinationTag":32}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_tx = serde_json::to_value(serde_json::to_string(&tx).unwrap()).unwrap();

        assert_eq!(serialized_tx, default_json_value);

        let deserilized_tx: LoanBrokerCoverWithdraw =
            serde_json::from_str(default_json_str).unwrap();

        assert_eq!(tx, deserilized_tx);
    }

    #[test]
    fn test_valid() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: Some(DESTINATION.into()),
            destination_tag: Some(32),
            credential_ids: None,
        };

        assert!(tx.get_errors().is_ok())
    }

    #[test]
    fn test_invalid_amount() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("0")),
            destination: Some(DESTINATION.into()),
            destination_tag: Some(32),
            credential_ids: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_broker_id() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF123456789".into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: Some(DESTINATION.into()),
            destination_tag: Some(32),
            credential_ids: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_broker_id_empty() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: "".into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: Some(DESTINATION.into()),
            destination_tag: Some(32),
            credential_ids: None,
        };

        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_invalid_loan_broker_id_too_long() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            // 66 hex chars instead of 64
            loan_broker_id: format!("{}AB", LOAN_BROKER_ID).into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: Some(DESTINATION.into()),
            destination_tag: Some(32),
            credential_ids: None,
        };

        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_invalid_loan_broker_id_non_hex() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            // Correct length (64), but contains non-hex characters
            loan_broker_id: "Z123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD"
                .into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: Some(DESTINATION.into()),
            destination_tag: Some(32),
            credential_ids: None,
        };

        assert!(tx.get_errors().is_err());
    }

    const MPT_ISSUANCE_ID: &str = "00000012E1A1E1A1E1A1E1A1E1A1E1A1E1A1E1A1E1A1E1A1";

    #[test]
    fn test_new_and_accessors() {
        let mut tx = LoanBrokerCoverWithdraw::new(
            ACCOUNT.into(),
            None,
            Some(XRPAmount::from("12")),
            Some(7108682),
            Some(alloc::vec![Memo {
                memo_data: Some("7769746864726177".into()),
                memo_format: None,
                memo_type: Some("74657874".into()),
            }]),
            Some(8),
            None,
            Some(12345),
            None,
            LOAN_BROKER_ID.into(),
            Amount::XRPAmount(XRPAmount::from("1000000")),
            Some(DESTINATION.into()),
            Some(32),
        );

        assert!(tx.get_errors().is_ok());
        assert_eq!(
            tx.get_transaction_type(),
            &TransactionType::LoanBrokerCoverWithdraw
        );
        assert_eq!(tx.get_common_fields().account, ACCOUNT);
        assert_eq!(tx.get_common_fields().sequence, Some(8));
        assert_eq!(tx.destination, Some(DESTINATION.into()));
        assert_eq!(tx.destination_tag, Some(32));
        assert_eq!(
            Transaction::get_mut_common_fields(&mut tx).source_tag,
            Some(12345)
        );
    }

    #[test]
    fn test_builder_pattern() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            ..Default::default()
        }
        .with_destination(DESTINATION.into())
        .with_destination_tag(32)
        .with_fee("12".into())
        .with_sequence(8)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_ticket_sequence(7)
        .with_memo(Memo {
            memo_data: Some("7769746864726177".into()),
            memo_format: None,
            memo_type: Some("74657874".into()),
        });

        assert_eq!(tx.destination, Some(DESTINATION.into()));
        assert_eq!(tx.destination_tag, Some(32));
        assert_eq!(tx.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(tx.common_fields.sequence, Some(8));
        assert_eq!(tx.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(tx.common_fields.source_tag, Some(12345));
        assert_eq!(tx.common_fields.ticket_sequence, Some(7));
        assert_eq!(tx.common_fields.memos.as_ref().unwrap().len(), 1);
        assert!(tx.get_errors().is_ok());
    }

    /// The amount value is read from whichever `Amount` variant is supplied, so
    /// each variant has to be accepted by `get_errors`.
    #[test]
    fn test_valid_issued_currency_and_mpt_amounts() {
        let base = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: Some(DESTINATION.into()),
            destination_tag: None,
            credential_ids: None,
        };

        let issued = LoanBrokerCoverWithdraw {
            amount: Amount::IssuedCurrencyAmount(crate::models::IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: "rH5gvkKxGHrFAMAACeu9CB3FMu7pQY7Zh4".into(),
                value: "1000".into(),
            }),
            ..base.clone()
        };
        assert!(issued.get_errors().is_ok());

        let mpt = LoanBrokerCoverWithdraw {
            amount: Amount::MPTAmount(crate::models::MPTAmount {
                value: "1000".into(),
                mpt_issuance_id: MPT_ISSUANCE_ID.into(),
            }),
            ..base
        };
        assert!(mpt.get_errors().is_ok());
    }

    /// `IssuedCurrencyAmount` validates its value with `f64::parse`, which accepts
    /// `NaN` and the infinities. `BigDecimal` does not, so the amount parse in
    /// `get_errors` still has to report a format error for those values.
    #[test]
    fn test_unparsable_amount() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::IssuedCurrencyAmount(crate::models::IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: "rH5gvkKxGHrFAMAACeu9CB3FMu7pQY7Zh4".into(),
                value: "NaN".into(),
            }),
            destination: None,
            destination_tag: None,
            credential_ids: None,
        };

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    #[test]
    fn test_invalid_zero_amount() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("0")),
            destination: Some(DESTINATION.into()),
            destination_tag: Some(32),
            credential_ids: None,
        };

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_valid_without_optional_fields() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: None,
            destination_tag: None,
            credential_ids: None,
        };

        assert!(tx.get_errors().is_ok());

        // skip_serializing_none: optional fields must be absent from the JSON
        let json = serde_json::to_string(&tx).unwrap();
        assert!(!json.contains("Destination"));
        assert!(!json.contains("DestinationTag"));
    }

    /// `CredentialIDs` validation (LendingProtocolV1_1).
    ///
    /// Mirrors `test/models/loanBrokerCoverWithdraw.test.ts` in xrpl.js. The
    /// credentials authorize the withdrawal when the destination is gated by a
    /// permissioned domain (XLS-70).
    mod credential_ids {
        use super::*;
        use alloc::vec;

        const CREDENTIAL_ID: &str =
            "0F0B70F4F4C5B27E39D62D4D69E9DF3D0BC0AC29B8FE7CD5AF1AC8C15F1D2E3B";

        fn with_credentials(
            credential_ids: Vec<Cow<'static, str>>,
        ) -> LoanBrokerCoverWithdraw<'static> {
            LoanBrokerCoverWithdraw {
                common_fields: CommonFields {
                    account: ACCOUNT.into(),
                    transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                    signing_pub_key: Some("".into()),
                    ..Default::default()
                },
                loan_broker_id: LOAN_BROKER_ID.into(),
                amount: Amount::XRPAmount(XRPAmount::from("1000000")),
                destination: Some(DESTINATION.into()),
                destination_tag: None,
                credential_ids: Some(credential_ids),
            }
        }

        #[test]
        fn test_valid_credential_ids() {
            assert!(with_credentials(vec![CREDENTIAL_ID.into()])
                .get_errors()
                .is_ok());
        }

        #[test]
        fn test_invalid_duplicate_credential_ids() {
            assert!(matches!(
                with_credentials(vec![CREDENTIAL_ID.into(), CREDENTIAL_ID.into()])
                    .get_errors()
                    .err(),
                Some(XRPLModelException::ValueEqualsValue { .. })
            ));
        }

        #[test]
        fn test_invalid_empty_credential_ids() {
            assert!(matches!(
                with_credentials(vec![]).get_errors().err(),
                Some(XRPLModelException::ValueTooShort { .. })
            ));
        }

        #[test]
        fn test_serde_credential_ids() {
            let tx = with_credentials(vec![CREDENTIAL_ID.into()]);

            let json_str = r#"{"Account":"r9LqNeG6qHxLoanBrokerCoverWithdraw5weJ9","TransactionType":"LoanBrokerCoverWithdraw","Flags":0,"SigningPubKey":"","LoanBrokerID":"E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD","Amount":"1000000","Destination":"rf7HPydP4ihkFkSRHWFq34b4SXRc7GvPCR","CredentialIDs":["0F0B70F4F4C5B27E39D62D4D69E9DF3D0BC0AC29B8FE7CD5AF1AC8C15F1D2E3B"]}"#;

            assert_eq!(
                serde_json::to_value(serde_json::to_string(&tx).unwrap()).unwrap(),
                serde_json::to_value(json_str).unwrap()
            );

            let deserialized: LoanBrokerCoverWithdraw = serde_json::from_str(json_str).unwrap();
            assert_eq!(tx, deserialized);
        }

        #[test]
        fn test_builder_sets_credential_ids() {
            let tx = LoanBrokerCoverWithdraw {
                common_fields: CommonFields {
                    account: ACCOUNT.into(),
                    transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                    ..Default::default()
                },
                loan_broker_id: LOAN_BROKER_ID.into(),
                amount: Amount::XRPAmount(XRPAmount::from("1000000")),
                ..Default::default()
            }
            .with_credential_ids(vec![CREDENTIAL_ID.into()]);

            assert_eq!(tx.credential_ids.as_ref().unwrap().len(), 1);
            assert!(tx.get_errors().is_ok());
        }
    }
}
