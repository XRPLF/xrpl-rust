use alloc::borrow::Cow;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    transactions::{CommonTransactionBuilder, Memo, Signer},
    FlagCollection, Model, ValidateCurrencies, XRPAmount, XRPLModelException, XRPLModelResult,
};

use super::{CommonFields, Transaction, TransactionType};

#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter, Copy,
)]
#[repr(u32)]
pub enum LoanSetFlag {
    /// Indicates that the loan supports overpayments.
    TfLoanOverpayment = 0x00010000,
}

/// Creates a new Loan ledger entry, representing a loan
/// agreement between a Loan Broker and Borrower.
/// The LoanSet transaction is a mutual agreement between
/// the Loan Broker and Borrower, and must be signed
/// by both parties. The following multi-signature flow
/// can be initiated by either party:
/// 1. The borrower or loan broker creates the transaction
///     with the preagreed terms of the loan. They sign the
///     transaction and set the SigningPubKey, TxnSignature,
///     Signers, Account, Fee, Sequence, and Counterparty fields.
/// 2. The counterparty verifies the loan terms and
///     signature before signing and submitting the transaction.
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
pub struct LoanSet<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, LoanSetFlag>,
    /// The Loan Broker ID associated with the loan.
    #[serde(rename = "LoanBrokerID")]
    pub loan_broker_id: Cow<'a, str>,
    /// Arbitrary metadata in hex format. The field is limited to 256 bytes.
    pub data: Option<Cow<'a, str>>,
    /// The address of the counterparty of the Loan.
    pub counterparty: Option<Cow<'a, str>>,
    /// The signature of the counterparty over the transaction.
    pub counterparty_signature: CounterpartySignature<'a>,
    /// A nominal funds amount paid to the LoanBroker.Owner when the Loan is created.
    pub loan_origination_fee: Option<Cow<'a, str>>,
    /// A nominal amount paid to the LoanBroker.Owner with every Loan payment.
    pub loan_service_fee: Option<Cow<'a, str>>,
    /// A nominal funds amount paid to the LoanBroker.Owner when a payment is late.
    pub late_payment_fee: Option<Cow<'a, str>>,
    /// A nominal funds amount paid to the LoanBroker.Owner when an early full repayment is made.
    pub close_payment_fee: Option<Cow<'a, str>>,
    /// A fee charged on overpayments in 1/10th basis points.
    /// Valid values are between 0 and 100000 inclusive. (0 - 100%)
    pub overpayment_fee: Option<u32>,
    /// Annualized interest rate of the Loan in in 1/10th basis points.
    /// Valid values are between 0 and 100000 inclusive. (0 - 100%)
    pub interest_rate: Option<u32>,
    /// A premium added to the interest rate for late payments in in 1/10th basis points.
    /// alid values are between 0 and 100000 inclusive. (0 - 100%)
    pub late_interest_rate: Option<u32>,
    /// A Fee Rate charged for repaying the Loan early in 1/10th basis points.
    /// Valid values are between 0 and 100000 inclusive. (0 - 100%)
    pub close_interest_rate: Option<u32>,
    /// An interest rate charged on overpayments in 1/10th basis points.
    /// Valid values are between 0 and 100000 inclusive. (0 - 100%)
    pub overpayment_interest_rate: Option<u32>,
    /// The principal amount requested by the Borrower.
    pub principal_requested: Cow<'a, str>,
    /// The total number of payments to be made against the Loan.
    pub payment_total: Option<u32>,
    /// Number of seconds between Loan payments.
    pub payment_interval: Option<u32>,
    /// The number of seconds after the Loan's Payment Due Date can be Defaulted.
    pub grace_period: Option<u32>,
}

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
pub struct CounterpartySignature<'a> {
    pub signing_pub_key: Option<Cow<'a, str>>,
    pub txn_signature: Option<Cow<'a, str>>,
    pub signers: Option<Vec<Signer>>,
}

impl Model for LoanSet<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.validate_currencies()?;

        if self
            .data
            .as_ref()
            .map_or(false, |s: &Cow<'_, str>| s.len() > 256)
        {
            return Err(XRPLModelException::ValueTooLong {
                field: "data".into(),
                max: 256,
                found: self.data.as_ref().unwrap().len(),
            });
        }

        if self
            .data
            .as_ref()
            .map_or(false, |s: &Cow<'_, str>| s.is_empty())
        {
            return Err(XRPLModelException::ValueTooShort {
                field: "data".into(),
                min: 1,
                found: 0,
            });
        }

        if let Some(Err(e)) = self.data.as_ref().map(|s| hex::decode(s.as_ref())) {
            return Err(XRPLModelException::FromHexError(e));
        }

        if let Some(lsf) = &self.loan_service_fee {
            let lsf_decimal = lsf
                .parse::<BigDecimal>()
                .map_err(|e| XRPLModelException::BigDecimalError(e))?;

            if lsf_decimal < 0 {
                return Err(XRPLModelException::InvalidValue {
                    field: "loan_service_fee".into(),
                    expected: "At least zero(0)".into(),
                    found: format!("{}", lsf_decimal),
                });
            }
        }

        if let Some(lpf) = &self.late_payment_fee {
            let lpf_decimal = lpf
                .parse::<BigDecimal>()
                .map_err(|e| XRPLModelException::BigDecimalError(e))?;

            if lpf_decimal < 0 {
                return Err(XRPLModelException::InvalidValue {
                    field: "late_payment_fee".into(),
                    expected: "At least zero(0)".into(),
                    found: format!("{}", lpf_decimal),
                });
            }
        }

        if let Some(cpf) = &self.close_payment_fee {
            let cpf_decimal = cpf
                .parse::<BigDecimal>()
                .map_err(|e| XRPLModelException::BigDecimalError(e))?;

            if cpf_decimal < 0 {
                return Err(XRPLModelException::InvalidValue {
                    field: "close_payment_fee".into(),
                    expected: "At least zero(0)".into(),
                    found: format!("{}", cpf_decimal),
                });
            }
        }

        if self.overpayment_fee.map_or(false, |v| v > 100_000) {
            return Err(XRPLModelException::ValueTooHigh {
                field: "overpayment_fee".into(),
                max: 100_000,
                found: self.overpayment_fee.unwrap() as u32,
            });
        }

        if self.interest_rate.map_or(false, |v| v > 100_000) {
            return Err(XRPLModelException::ValueTooHigh {
                field: "interest_rate".into(),
                max: 100_000,
                found: self.interest_rate.unwrap() as u32,
            });
        }

        if self.late_interest_rate.map_or(false, |v| v > 100_000) {
            return Err(XRPLModelException::ValueTooHigh {
                field: "late_interest_rate".into(),
                max: 100_000,
                found: self.late_interest_rate.unwrap() as u32,
            });
        }

        if self.close_interest_rate.map_or(false, |v| v > 100_000) {
            return Err(XRPLModelException::ValueTooHigh {
                field: "close_interest_rate".into(),
                max: 100_000,
                found: self.close_interest_rate.unwrap() as u32,
            });
        }

        if self
            .overpayment_interest_rate
            .map_or(false, |v| v > 100_000)
        {
            return Err(XRPLModelException::ValueTooHigh {
                field: "overpayment_interest_rate".into(),
                max: 100_000,
                found: self.overpayment_interest_rate.unwrap() as u32,
            });
        }

        let pr_decimal = &self
            .principal_requested
            .parse::<BigDecimal>()
            .map_err(|e| XRPLModelException::BigDecimalError(e))?;

        if pr_decimal < 1 {
            return Err(XRPLModelException::InvalidValue {
                field: "principal_requested".into(),
                expected: "At least one(1)".into(),
                found: format!("{}", pr_decimal),
            });
        }

        if let Some(lof) = &self.loan_origination_fee {
            let lof_decimal = lof
                .parse::<BigDecimal>()
                .map_err(|e| XRPLModelException::BigDecimalError(e))?;

            if lof_decimal < 0 {
                return Err(XRPLModelException::InvalidValue {
                    field: "loan_origination_fee".into(),
                    expected: "At least zero(0)".into(),
                    found: format!("{}", lof_decimal),
                });
            }

            if lof_decimal > *pr_decimal {
                return Err(XRPLModelException::InvalidValue {
                    field: "loan_origination_fee and principal_requested".into(),
                    expected: "loan_origination_fee should be less than principal_requested".into(),
                    found: format!(
                        "loan_origination_fee: {}, principal_requested: {}",
                        lof_decimal, pr_decimal
                    ),
                });
            }
        }

        if self.payment_total.map_or(false, |v| v == 0) {
            return Err(XRPLModelException::ValueTooLow {
                field: "payment_total".into(),
                min: 1,
                found: 0,
            });
        }

        if self.payment_interval.map_or(false, |v| v < 60) {
            return Err(XRPLModelException::ValueTooLow {
                field: "payment_interval".into(),
                min: 60,
                found: self.payment_interval.unwrap(),
            });
        }

        if self.grace_period.map_or(false, |v| v < 60) {
            return Err(XRPLModelException::ValueTooLow {
                field: "grace_period".into(),
                min: 60,
                found: self.grace_period.unwrap(),
            });
        }

        if let (Some(gr), Some(pi)) = (self.grace_period, self.payment_interval) {
            if gr > pi {
                return Err(XRPLModelException::InvalidValue {
                    field: "grace_period and payment_interval".into(),
                    expected: "grace_period should be less than payment_interval".into(),
                    found: format!("grace_period: {}, payment_interval: {}", gr, pi),
                });
            }
        }

        Ok(())
    }
}

impl<'a> Transaction<'a, LoanSetFlag> for LoanSet<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, LoanSetFlag> {
        &self.common_fields
    }
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, LoanSetFlag> {
        &mut self.common_fields
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> CommonTransactionBuilder<'a, LoanSetFlag> for LoanSet<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, LoanSetFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> LoanSet<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<LoanSetFlag>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        loan_broker_id: Cow<'a, str>,
        data: Option<Cow<'a, str>>,
        counterparty: Option<Cow<'a, str>>,
        counterparty_signature: CounterpartySignature<'a>,
        loan_origination_fee: Option<Cow<'a, str>>,
        loan_service_fee: Option<Cow<'a, str>>,
        late_payment_fee: Option<Cow<'a, str>>,
        close_payment_fee: Option<Cow<'a, str>>,
        overpayment_fee: Option<u32>,
        interest_rate: Option<u32>,
        late_interest_rate: Option<u32>,
        close_interest_rate: Option<u32>,
        overpayment_interest_rate: Option<u32>,
        principal_requested: Cow<'a, str>,
        payment_total: Option<u32>,
        payment_interval: Option<u32>,
        grace_period: Option<u32>,
    ) -> LoanSet<'a> {
        LoanSet {
            common_fields: CommonFields::new(
                account,
                TransactionType::LoanSet,
                account_txn_id,
                fee,
                flags,
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
            data,
            counterparty,
            counterparty_signature,
            loan_origination_fee,
            loan_service_fee,
            late_payment_fee,
            close_payment_fee,
            overpayment_fee,
            interest_rate,
            late_interest_rate,
            close_interest_rate,
            overpayment_interest_rate,
            principal_requested,
            payment_total,
            payment_interval,
            grace_period,
        }
    }

    /// Set Data field
    pub fn with_data(mut self, data: Cow<'a, str>) -> Self {
        self.data = Some(data);
        self
    }

    /// Set the Counterparty field
    pub fn with_counterparty(mut self, counterparty: Cow<'a, str>) -> Self {
        self.counterparty = Some(counterparty);
        self
    }

    /// Set LateOriginationFee
    pub fn with_late_origination_fee(mut self, loan_origination_fee: Cow<'a, str>) -> Self {
        self.loan_origination_fee = Some(loan_origination_fee);
        self
    }

    /// Set LoanServiceFee
    pub fn with_loan_service_fee(mut self, loan_service_fee: Cow<'a, str>) -> Self {
        self.loan_service_fee = Some(loan_service_fee);
        self
    }

    /// Set LatePaymentFee
    pub fn with_late_payment_fee(mut self, late_payment_fee: Cow<'a, str>) -> Self {
        self.late_payment_fee = Some(late_payment_fee);
        self
    }

    /// Set ClosePaymentFee
    pub fn with_close_payment_fee(mut self, close_payment_fee: Cow<'a, str>) -> Self {
        self.close_payment_fee = Some(close_payment_fee);
        self
    }

    /// Set OverpaymentFee
    pub fn with_overpayment_fee(mut self, overpayment_fee: u32) -> Self {
        self.overpayment_fee = Some(overpayment_fee);
        self
    }

    /// Set InterestRate
    pub fn with_interest_rate(mut self, interest_rate: u32) -> Self {
        self.interest_rate = Some(interest_rate);
        self
    }

    /// Set LateInterestRate
    pub fn with_late_interest_rate(mut self, late_interest_rate: u32) -> Self {
        self.late_interest_rate = Some(late_interest_rate);
        self
    }

    /// Set CloseInterestRate
    pub fn with_close_interest_rate(mut self, close_interest_rate: u32) -> Self {
        self.close_interest_rate = Some(close_interest_rate);
        self
    }

    /// Set OverpaymentInterestRate
    pub fn with_overpayment_interest_rate(mut self, overpayment_interest_rate: u32) -> Self {
        self.overpayment_interest_rate = Some(overpayment_interest_rate);
        self
    }

    /// Set PaymentTotal
    pub fn with_payment_total(mut self, payment_total: u32) -> Self {
        self.payment_total = Some(payment_total);
        self
    }

    /// Set PaymentInterval
    pub fn with_payment_interval(mut self, payment_interval: u32) -> Self {
        self.payment_interval = Some(payment_interval);
        self
    }

    /// Set GracePeriod
    pub fn with_grace_period(mut self, grace_period: u32) -> Self {
        self.grace_period = Some(grace_period);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "r9LqNeG6qHxLoanSetter5weJ9mZg";
    const VAULT_ID: &str = "rDB303FC1C7611B22C09E773B51044F6BE";
    const LOAN_BROKER_ID: &str = "rDB303FC1C76LOANBROKER09E773B51044F6BE";

    #[test]
    fn test_serde() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        let default_json_str = r#"{"Account":"r9LqNeG6qHxLoanSetter5weJ9mZg","TransactionType":"LoanSet","Flags":0,"SigningPubKey":"","LoanBrokerID":"rDB303FC1C76LOANBROKER09E773B51044F6BE","CounterpartySignature":{},"PrincipalRequested":"1000"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_tx = serde_json::to_value(&serde_json::to_string(&tx).unwrap()).unwrap();

        assert_eq!(serialized_tx, default_json_value);

        let deseriliazed_tx: LoanSet = serde_json::from_str(default_json_str).unwrap();

        assert_eq!(tx, deseriliazed_tx);
    }

    #[test]
    fn test_invalid_data_too_long() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: Some("A".repeat(257).into()),
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooLong { .. })
        ));
    }

    #[test]
    fn test_invalid_data_empty() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: Some("".into()),
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooShort { .. })
        ));
    }

    #[test]
    fn test_invalid_data_non_hex_string() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: Some("Z".into()),
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::FromHexError(..))
        ));
    }

    #[test]
    fn test_invalid_interest_rate_too_high() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: Some(100_001),
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooHigh { .. })
        ));
    }

    #[test]
    fn test_invalid_late_interest_rate_too_high() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: Some(100_001),
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooHigh { .. })
        ));
    }

    #[test]
    fn test_invalid_close_interest_rate_too_high() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: Some(100_001),
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooHigh { .. })
        ));
    }

    #[test]
    fn test_invalid_overpayment_interest_rate_too_high() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: Some(100_001),
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooHigh { .. })
        ));
    }

    #[test]
    fn test_invalid_overpayment_fee_too_high() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: Some(100_001),
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooHigh { .. })
        ));
    }

    #[test]
    fn test_invalid_payment_interval_shorter_than_grace_period() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: Some(61),
            grace_period: Some(62),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_payment_interval_too_short() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: Some(59),
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooLow { .. })
        ));
    }

    #[test]
    fn test_invalid_grace_period_too_short() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: Some(59),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::ValueTooLow { .. })
        ));
    }

    #[test]
    fn test_invalid_principal_request() {
        let mut tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "0".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));

        // Testing negative
        tx.principal_requested = "-1".into();
        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_origination_fee() {
        let mut tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: Some("".into()),
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::BigDecimalError(..))
        ));

        // Testing negative
        tx.loan_origination_fee = Some("-1".into());
        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_service_fee() {
        let mut tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: Some("".into()),
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::BigDecimalError(..))
        ));

        // Testing negative
        tx.loan_service_fee = Some("-1".into());
        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_late_payment_fee() {
        let mut tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: Some("".into()),
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::BigDecimalError(..))
        ));

        // Testing negative
        tx.late_payment_fee = Some("-1".into());
        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_close_payment_fee() {
        let mut tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: Some("".into()),
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "1000".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::BigDecimalError(..))
        ));

        // Testing negative
        tx.close_payment_fee = Some("-1".into());
        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_principal_requested_shorter_than_loan_origination_fee() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: Some("11".into()),
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: "10".into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_valid_loan_set() {
        let tx = LoanSet {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            data: None,
            counterparty: None,
            counterparty_signature: CounterpartySignature {
                signing_pub_key: None,
                txn_signature: None,
                signers: None,
            },
            loan_origination_fee: Some("11".into()),
            loan_service_fee: Some("11".into()),
            late_payment_fee: Some("11".into()),
            close_payment_fee: Some("11".into()),
            overpayment_fee: Some(1000),
            interest_rate: Some(1000),
            late_interest_rate: Some(1000),
            close_interest_rate: Some(1000),
            overpayment_interest_rate: Some(1000),
            principal_requested: "1000".into(),
            payment_total: Some(12),
            payment_interval: Some(61),
            grace_period: Some(60),
        };

        assert!(tx.get_errors().is_ok());
    }
}
