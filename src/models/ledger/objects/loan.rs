use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    ledger::objects::{CommonFields, LedgerEntryType, LedgerObject},
    FlagCollection, Model,
};

#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum LoanFlag {
    /// Indicates that the Loan is defaulted
    LsfLoanDefault = 0x00010000,
    /// Indicates that the Loan is impaired
    LsfLoanImpaired = 0x00020000,
    /// Indicates that the Loan supports overpayments
    LsfLoanOverpayment = 0x00040000,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Loan<'a> {
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, LoanFlag>,
    #[serde(rename = "PreviousTxnID")]
    /// The ID of the transaction that last
    /// modified this object.
    pub previous_txn_id: Cow<'a, str>,
    /// The ledger sequence containing the
    /// transaction that last modified this object.
    pub previous_txn_lgr_seq: u32,
    /// The sequence number of the Loan.
    pub loan_sequence: u32,
    /// Identifies the page where this item is
    /// referenced in the Borrower owner's directory.
    pub owner_node: u64,
    /// Identifies the page where this item
    /// is referenced in the LoanBrokers owner directory.
    pub loan_broker_node: Cow<'a, str>,
    /// The ID of the LoanBroker associated
    /// with this Loan Instance.
    #[serde(rename = "LoanBrokerID")]
    pub loan_broker_id: Cow<'a, str>,
    /// The address of the account that is the borrower.
    pub borrower: Cow<'a, str>,
    /// A nominal funds amount paid to the
    /// LoanBroker.Owner when the Loan is created.
    pub loan_origination_fee: Cow<'a, str>,
    /// A nominal funds amount paid to the
    /// LoanBroker.Owner with every Loan payment.
    pub loan_service_fee: Cow<'a, str>,
    /// A nominal funds amount paid to the
    /// LoanBroker.Owner when a payment is late.
    pub late_payment_fee: Cow<'a, str>,
    /// A nominal funds amount paid to the
    /// LoanBroker.Owner when a full payment is made.
    pub close_payment_fee: Cow<'a, str>,
    /// A fee charged on overpayments in 1/10th
    /// basis points. Valid values are between 0 and 100000 inclusive. (0 - 100%)
    pub overpayment_fee: u32,
    /// Annualized interest rate of the Loan in 1/10th basis points.
    pub interest_rate: Option<u32>,
    /// A premium is added to the interest rate for
    /// late payments in 1/10th basis points.
    /// Valid values are between 0 and 100000 inclusive. (0 - 100%)
    pub late_interest_rate: Option<u32>,
    /// An interest rate charged for repaying
    /// the Loan early in 1/10th basis points.
    /// Valid values are between 0 and 100000 inclusive. (0 - 100%)
    pub close_interest_rate: Option<u32>,
    /// An interest rate charged on overpayments
    /// in 1/10th basis points. Valid values are between
    /// 0 and 100000 inclusive. (0 - 100%)
    pub overpayment_interest_rate: Option<u32>,
    /// The timestamp of when the Loan started
    /// Ripple Epoch.(https://xrpl.org/docs/references/protocol/data-types/basic-data-types/#specifying-time)
    pub start_date: u32,
    /// Number of seconds between Loan payments.
    pub payment_interval: u32,
    /// The number of seconds after the Loan's Payment Due Date that the Loan can be Defaulted.
    pub grace_period: u32,
    /// The timestamp of when the previous payment was made
    /// in Ripple Epoch. (https://xrpl.org/docs/references/protocol/data-types/basic-data-types/#specifying-time)
    pub previous_payment_due_date: u32,
    /// The timestamp of when the next payment is due
    /// in Ripple Epoch. (https://xrpl.org/docs/references/protocol/data-types/basic-data-types/#specifying-time)
    pub next_payment_due_date: Option<u32>,
    /// The number of payments remaining on the Loan.
    pub payment_remaining: Option<u32>,
    /// The total outstanding value of the Loan, including all
    /// fees and interest.
    pub total_value_outstanding: Option<Cow<'a, str>>,
    /// The principal amount that the Borrower still owes.
    pub principal_outstanding: Option<Cow<'a, str>>,
    /// The remaining Management Fee owed to the LoanBroker.
    pub management_fee_outstanding: Option<Cow<'a, str>>,
    /// The calculated periodic payment amount for each payment interval.
    pub periodic_payment: Cow<'a, str>,
    /// The scale factor that ensures all computed amounts are
    /// rounded to the same number of decimal places.
    /// It is determined based on the total loan value at creation time.
    pub loan_scale: Option<i32>,
}

impl<'a> Model for Loan<'a> {}

impl<'a> LedgerObject<LoanFlag> for Loan<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> Loan<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Cow<'a, str>,
        flags: FlagCollection<LoanFlag>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
        loan_sequence: u32,
        owner_node: u64,
        loan_broker_node: Cow<'a, str>,
        loan_broker_id: Cow<'a, str>,
        borrower: Cow<'a, str>,
        loan_origination_fee: Cow<'a, str>,
        loan_service_fee: Cow<'a, str>,
        late_payment_fee: Cow<'a, str>,
        close_payment_fee: Cow<'a, str>,
        overpayment_fee: u32,
        interest_rate: Option<u32>,
        late_interest_rate: Option<u32>,
        close_interest_rate: Option<u32>,
        overpayment_interest_rate: Option<u32>,
        start_date: u32,
        payment_interval: u32,
        grace_period: u32,
        previous_payment_due_date: u32,
        next_payment_due_date: Option<u32>,
        payment_remaining: Option<u32>,
        total_value_outstanding: Option<Cow<'a, str>>,
        principal_outstanding: Option<Cow<'a, str>>,
        management_fee_outstanding: Option<Cow<'a, str>>,
        periodic_payment: Cow<'a, str>,
        loan_scale: Option<i32>,
    ) -> Self {
        Loan {
            common_fields: CommonFields {
                flags,
                ledger_entry_type: LedgerEntryType::Loan,
                index,
                ledger_index: Some(ledger_index),
            },
            previous_txn_id,
            previous_txn_lgr_seq,
            loan_sequence,
            owner_node,
            loan_broker_node,
            loan_broker_id,
            borrower,
            loan_origination_fee,
            loan_service_fee,
            late_payment_fee,
            close_payment_fee,
            overpayment_fee,
            interest_rate,
            late_interest_rate,
            close_interest_rate,
            overpayment_interest_rate,
            start_date,
            payment_interval,
            grace_period,
            previous_payment_due_date,
            next_payment_due_date,
            payment_remaining,
            total_value_outstanding,
            principal_outstanding,
            management_fee_outstanding,
            periodic_payment,
            loan_scale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{borrow::Cow, vec};

    #[test]
    fn test_serde() {
        let loan = Loan::new(
            None,
            Cow::from("ledger_index"),
            FlagCollection::new(vec![LoanFlag::LsfLoanDefault]),
            "108D5CE7EEAF504B2894B8C674E6D68499076441C483728".into(),
            47636435,
            7446366,
            6363252,
            "45372352".into(),
            "FA65C9FE1538FD7E398FFFE9D1908DFA4576D8".into(),
            "r75E1D753E5B91627516F6D7097".into(),
            "1".into(),
            "1".into(),
            "2".into(),
            "1".into(),
            1,
            Some(10),
            Some(12),
            Some(10),
            Some(8),
            177474757,
            86400u32,
            500,
            1777749474,
            Some(175747473),
            Some(453636),
            Some("100074".into()),
            Some("100000".into()),
            Some("1000".into()),
            "500".into(),
            Some(5),
        );

        let serialized = serde_json::to_string(&loan).unwrap();
        let deserialized: Loan = serde_json::from_str(&serialized).unwrap();

        assert_eq!(loan, deserialized);
    }

    #[test]
    fn test_deserialization() {
        let default_loan_str = r#"{"LedgerEntryType":"Loan","LedgerIndex":"E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD","Flags":0,"PreviousTxnID":"9A8765B4321CDE987654321CDE987654321CDE987654321CDE987654321CDE98","PreviousTxnLgrSeq":12345678,"LoanSequence":1,"OwnerNode":2,"LoanBrokerNode":"1","LoanBrokerID":"ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890","Borrower":"rEXAMPLE9AbCdEfGhIjKlMnOpQrStUvWxYz","LoanOriginationFee":"100","LoanServiceFee":"10","LatePaymentFee":"5","ClosePaymentFee":"20","OverpaymentFee":5,"InterestRate":500,"LateInterestRate":1000,"CloseInterestRate":200,"OverpaymentInterestRate":5,"StartDate":1234567890,"PaymentInterval":2592000,"GracePeriod":604800,"PreviousPaymentDueDate":1234587890,"NextPaymentDueDate":1234597890,"PaymentRemaining":12,"PrincipalOutstanding":"10000","TotalValueOutstanding":"12000","ManagementFeeOutstanding":"2000","PeriodicPayment":"1000"}"#;

        let object: Loan = serde_json::from_str(default_loan_str).expect("Failed to deserialize");
        let serialized = serde_json::to_string(&object).unwrap();

        let actual: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        let expected: serde_json::Value = serde_json::from_str(default_loan_str).unwrap();

        assert_eq!(actual, expected);
    }
}
