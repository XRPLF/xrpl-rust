use alloc::{borrow::Cow, format, vec::Vec};
use bigdecimal::{BigDecimal, Signed};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::{
        vault_common::{validate_hash256, validate_vault_id},
        CommonTransactionBuilder, Memo, Signer,
    },
    FlagCollection, Model, NoFlags, ValidateCurrencies, XRPAmount, XRPLModelException,
    XRPLModelResult,
};

use super::{CommonFields, Transaction, TransactionType};

const MAX_DATA_LENGTH: usize = 512;

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
pub struct LoanBrokerSet<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    #[serde(rename = "VaultID")]
    /// The Vault ID that the Lending Protocol will use to access liquidity.
    pub vault_id: Cow<'a, str>,
    /// The Loan Broker ID that the transaction is modifying.
    #[serde(rename = "LoanBrokerID")]
    pub loan_broker_id: Option<Cow<'a, str>>,
    /// Arbitrary metadata in hex format. The field is limited to 256 bytes.
    pub data: Option<Cow<'a, str>>,
    /// The 1/10th basis point fee charged by the lending protocol owner.
    /// Valid values range from 0 to 10000 (inclusive), representing 0% to 10%.
    pub management_fee_rate: Option<u16>,
    /// The maximum amount the protocol can owe the vault.
    /// The default value of 0 means there is no limit to the debt. Must be a positive value.
    pub debt_maximum: Option<Cow<'a, str>>,
    /// The 1/10th basis point DebtTotal that the first-loss capital must cover.
    /// Valid values range from 0 to 100000 (inclusive), representing 0% to 100%.
    pub cover_rate_minimum: Option<u32>,
    /// The 1/10th basis point of minimum required first-loss capital that is moved to an asset vault to cover a loan default.
    /// Valid values range from 0 to 100000 (inclusive), representing 0% to 100%.
    pub cover_rate_liquidation: Option<u32>,
}

impl Model for LoanBrokerSet<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.validate_currencies()?;

        validate_vault_id(&self.vault_id)?;

        if let Some(loan_broker_id) = &self.loan_broker_id {
            validate_hash256("loan_broker_id", loan_broker_id)?;

            if self.management_fee_rate.is_some() {
                return Err(XRPLModelException::InvalidValue {
                    field: "loan_broker_id".into(),
                    expected: "only values for Flags, Data, or DebtMaximum can be modified when loan_broker_id is set".into(),
                    found: "management_fee_rate".into(),
                });
            }
            if self.cover_rate_minimum.is_some() {
                return Err(XRPLModelException::InvalidValue {
                    field: "loan_broker_id".into(),
                    expected: "only values for Flags, Data, or DebtMaximum can be modified when loan_broker_id is set".into(),
                    found: "cover_rate_minimum".into(),
                });
            }
            if self.cover_rate_liquidation.is_some() {
                return Err(XRPLModelException::InvalidValue {
                    field: "loan_broker_id".into(),
                    expected: "only values for Flags, Data, or DebtMaximum can be modified when loan_broker_id is set".into(),
                    found: "cover_rate_liquidation".into(),
                });
            }
        }

        if self
            .data
            .as_ref()
            .is_some_and(|s| s.len() > MAX_DATA_LENGTH)
        {
            return Err(XRPLModelException::ValueTooLong {
                field: "data".into(),
                max: 512,
                found: self.data.as_ref().unwrap().len(),
            });
        }

        if self.data.as_ref().is_some_and(|s| s.is_empty()) {
            return Err(XRPLModelException::ValueTooShort {
                field: "data".into(),
                min: 1,
                found: 0,
            });
        }

        if let Some(Err(e)) = self.data.as_ref().map(|s| hex::decode(s.as_ref())) {
            return Err(XRPLModelException::FromHexError(e));
        }

        if self.management_fee_rate.is_some_and(|v| v > 10_000) {
            return Err(XRPLModelException::ValueTooHigh {
                field: "management_fee_rate".into(),
                max: 10_000,
                found: self.management_fee_rate.unwrap() as u32,
            });
        }

        if self.cover_rate_minimum.is_some_and(|v| v > 100_000) {
            return Err(XRPLModelException::ValueTooHigh {
                field: "cover_rate_minimum".into(),
                max: 100_000,
                found: self.cover_rate_minimum.unwrap(),
            });
        }

        if self.cover_rate_liquidation.is_some_and(|v| v > 100_000) {
            return Err(XRPLModelException::ValueTooHigh {
                field: "cover_rate_liquidation".into(),
                max: 100_000,
                found: self.cover_rate_liquidation.unwrap(),
            });
        }

        if let Some(s) = &self.debt_maximum {
            let decimal = s
                .parse::<BigDecimal>()
                .map_err(XRPLModelException::BigDecimalError)?;

            if decimal.is_negative() {
                return Err(XRPLModelException::InvalidValue {
                    field: "debt_maximum".into(),
                    expected: "debt_maximum should be at least zero(0)".into(),
                    found: format!("{}", decimal),
                });
            }
        }

        match (self.cover_rate_liquidation, self.cover_rate_minimum) {
            (Some(crl), Some(crm)) if (crl == 0) != (crm == 0) => {
                return Err(XRPLModelException::InvalidValue {
                    field: "cover_rate_liquidation and cover_rate_minimum".into(),
                    expected: "Both should be either None, Zero or Non-Zero".into(),
                    found: format!(
                        "cover_rate_liquidation: {}, cover_rate_minimum: {}",
                        crl, crm
                    ),
                });
            }
            (Some(_), None) | (None, Some(_)) => {
                return Err(XRPLModelException::InvalidValue {
                    field: "cover_rate_liquidation and cover_rate_minimum".into(),
                    expected: "Both should be either None, Zero or Non-Zero".into(),
                    found: format!(
                        "cover_rate_liquidation: {:?}, cover_rate_minimum: {:?}",
                        self.cover_rate_liquidation, self.cover_rate_minimum
                    ),
                });
            }
            _ => {}
        }

        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for LoanBrokerSet<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for LoanBrokerSet<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> LoanBrokerSet<'a> {
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
        data: Option<Cow<'a, str>>,
        vault_id: Cow<'a, str>,
        loan_broker_id: Option<Cow<'a, str>>,
        management_fee_rate: Option<u16>,
        debt_maximum: Option<Cow<'a, str>>,
        cover_rate_minimum: Option<u32>,
        cover_rate_liquidation: Option<u32>,
    ) -> LoanBrokerSet<'a> {
        LoanBrokerSet {
            common_fields: CommonFields::new(
                account,
                TransactionType::LoanBrokerSet,
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
            vault_id,
            loan_broker_id,
            data,
            management_fee_rate,
            debt_maximum,
            cover_rate_minimum,
            cover_rate_liquidation,
        }
    }

    /// Set the data field.
    pub fn with_data(mut self, data: Cow<'a, str>) -> Self {
        self.data = Some(data);
        self
    }

    /// Set the LoanBroker ID field.
    pub fn with_loan_broker_id(mut self, loan_broker_id: Cow<'a, str>) -> Self {
        self.loan_broker_id = Some(loan_broker_id);
        self
    }

    /// Set the ManagementFeeRate field.
    pub fn with_management_fee_rate(mut self, rate: u16) -> Self {
        self.management_fee_rate = Some(rate);
        self
    }

    /// Set the DebtMaximum field.
    pub fn with_debt_maximum(mut self, debt_maximum: Cow<'a, str>) -> Self {
        self.debt_maximum = Some(debt_maximum);
        self
    }
    /// Set the CoverRateMinimum field.
    pub fn with_cover_rate_minimum(mut self, cover_rate_minimum: u32) -> Self {
        self.cover_rate_minimum = Some(cover_rate_minimum);
        self
    }

    /// Set the CoverRateLiquidation field.
    pub fn with_cover_rate_liquidation(mut self, cover_rate_liquidation: u32) -> Self {
        self.cover_rate_liquidation = Some(cover_rate_liquidation);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "r9LqNeG6qHxLoanBrokerSetter5weJ9mZg";
    const LOAN_BROKER_ID: &str = "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD";
    const VAULT_ID: &str = "77D6234D074E505024D39C04C3F262997B773719AB29ACFA83119E4210328776";

    #[test]
    fn test_invalid_data_too_long() {
        let tx = LoanBrokerSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            loan_broker_id: None,
            data: Some("48656C6C6F".repeat(67).into()),
            management_fee_rate: None,
            debt_maximum: None,
            cover_rate_liquidation: None,
            cover_rate_minimum: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooLong { .. })
        ));
    }

    #[test]
    fn test_invalid_data_empty() {
        let tx = LoanBrokerSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            loan_broker_id: None,
            data: Some("".into()),
            management_fee_rate: None,
            debt_maximum: None,
            cover_rate_liquidation: None,
            cover_rate_minimum: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooShort { .. })
        ));
    }

    #[test]
    fn test_invalid_data_non_hex_string() {
        let tx = LoanBrokerSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            loan_broker_id: None,
            data: Some("Z".into()),
            management_fee_rate: None,
            debt_maximum: None,
            cover_rate_liquidation: None,
            cover_rate_minimum: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::FromHexError(..))
        ));
    }

    #[test]
    fn test_invalid_management_fee_too_high() {
        let tx = LoanBrokerSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            loan_broker_id: None,
            data: None,
            management_fee_rate: Some(10_001),
            debt_maximum: None,
            cover_rate_liquidation: None,
            cover_rate_minimum: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooHigh { .. })
        ));
    }

    #[test]
    fn test_invalid_cover_rate_minimum_too_high() {
        let tx = LoanBrokerSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            loan_broker_id: None,
            data: None,
            management_fee_rate: None,
            debt_maximum: None,
            cover_rate_liquidation: None,
            cover_rate_minimum: Some(100_001),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooHigh { .. })
        ));
    }

    #[test]
    fn test_in_cover_rate_liquidation_too_high() {
        let tx = LoanBrokerSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            loan_broker_id: None,
            data: None,
            management_fee_rate: None,
            debt_maximum: None,
            cover_rate_liquidation: Some(100_001),
            cover_rate_minimum: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooHigh { .. })
        ));
    }

    #[test]
    fn test_invalid_debt_maximum_too_low() {
        let tx = LoanBrokerSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            loan_broker_id: None,
            data: None,
            management_fee_rate: None,
            debt_maximum: Some("-1".into()),
            cover_rate_liquidation: None,
            cover_rate_minimum: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_debt_maximum_empty() {
        let tx = LoanBrokerSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            loan_broker_id: None,
            data: None,
            management_fee_rate: None,
            debt_maximum: Some("".into()),
            cover_rate_liquidation: None,
            cover_rate_minimum: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::BigDecimalError(..))
        ));
    }

    #[test]
    fn test_cover_rate_minimum_cover_rate_liquidation_mismatch() {
        let tx = LoanBrokerSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            loan_broker_id: None,
            data: None,
            management_fee_rate: None,
            debt_maximum: None,
            cover_rate_liquidation: Some(0),
            cover_rate_minimum: Some(1),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));

        // Swapping values
        let mut updated = tx.with_cover_rate_liquidation(1).with_cover_rate_minimum(0);

        assert!(updated.get_errors().is_err());
        assert!(matches!(
            updated.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));

        // cover_rate_minimum: Some(500) + cover_rate_liquidation: None
        updated.cover_rate_liquidation = None;
        updated.cover_rate_minimum = Some(500);

        assert!(updated.get_errors().is_err());
        assert!(matches!(
            updated.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_serde() {
        let tx = LoanBrokerSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            loan_broker_id: None,
            data: None,
            management_fee_rate: Some(10),
            debt_maximum: Some("10000".into()),
            cover_rate_liquidation: Some(0),
            cover_rate_minimum: Some(0),
        };

        let default_json_str = r#"{"Account":"r9LqNeG6qHxLoanBrokerSetter5weJ9mZg","TransactionType":"LoanBrokerSet","Flags":0,"SigningPubKey":"","VaultID":"77D6234D074E505024D39C04C3F262997B773719AB29ACFA83119E4210328776","ManagementFeeRate":10,"DebtMaximum":"10000","CoverRateMinimum":0,"CoverRateLiquidation":0}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_tx = serde_json::to_value(serde_json::to_string(&tx).unwrap()).unwrap();

        assert_eq!(serialized_tx, default_json_value);

        let deserilized_tx: LoanBrokerSet = serde_json::from_str(default_json_str).unwrap();

        assert_eq!(tx, deserilized_tx);
    }

    #[test]
    fn test_invalid_loan_broker_id() {
        let tx = LoanBrokerSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            loan_broker_id: Some("E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF123456".into()),
            data: None,
            management_fee_rate: Some(10),
            debt_maximum: Some("10000".into()),
            cover_rate_liquidation: Some(0),
            cover_rate_minimum: Some(0),
        };

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    #[test]
    fn test_invalid_vault_id() {
        let tx = LoanBrokerSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: "77D6234D074E505024D39C04C3F262997B773719AB29ACFA83119E42103".into(),
            loan_broker_id: None,
            data: None,
            management_fee_rate: None,
            debt_maximum: None,
            cover_rate_liquidation: None,
            cover_rate_minimum: None,
        };

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_broker_id_management_fee_set() {
        let tx = LoanBrokerSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            loan_broker_id: Some(LOAN_BROKER_ID.into()),
            data: None,
            management_fee_rate: Some(10000),
            debt_maximum: None,
            cover_rate_liquidation: None,
            cover_rate_minimum: None,
        };

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }
}
