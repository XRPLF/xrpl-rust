use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{ledger::objects::LedgerEntryType, FlagCollection, Model, NoFlags};

use super::{CommonFields, LedgerObject};

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct LoanBroker<'a> {
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    #[serde(rename = "PreviousTxnID")]
    /// The ID of the transaction that last modified this object.
    pub previous_txn_id: Cow<'a, str>,
    /// The sequence of the ledger containing the
    /// transaction that last modified this object.
    pub previous_txn_lgr_seq: u32,
    /// The transaction sequence number that
    /// created the LoanBroker.
    pub sequence: u32,
    /// A sequential identifier for Loan objects,
    /// incremented each time a new Loan is
    /// created by this LoanBroker instance.
    pub loan_sequence: u32,
    /// Identifies the page where this item is
    /// referenced in the owner's directory.
    pub owner_node: u64,
    /// Identifies the page where this item is
    /// referenced in the Vault's pseudo-account
    /// owner's directory.
    pub vault_node: u64,
    /// The ID of the Vault object associated with this
    /// Lending Protocol Instance.
    #[serde(rename = "VaultID")]
    pub vault_id: Cow<'a, str>,
    /// The address of the LoanBroker pseudo-account.
    pub account: Cow<'a, str>,
    /// The address of the Loan Broker account.
    pub owner: Cow<'a, str>,
    /// Arbitrary metadata about the LoanBroker.
    /// Limited to 256 bytes.
    pub data: Option<Cow<'a, str>>,
    /// The 1/10th basis point fee charged by the Lending Protocol.
    /// Valid values are between 0 and 10000 inclusive.
    ///  A value of 1 is equivalent to 1/10 bps or 0.001%
    pub management_fee_rate: Option<u16>,
    /// The number of active Loans issued by the LoanBroker.
    pub owner_count: u32,
    /// The total asset amount the protocol owes the
    /// Vault, including interest.
    pub debt_total: Cow<'a, str>,
    /// The maximum amount the protocol can owe the Vault.
    /// The default value of 0 means there is no
    /// limit to the debt.
    pub debt_maximum: Cow<'a, str>,
    /// The total amount of first-loss capital
    /// deposited into the Lending Protocol.
    pub cover_available: Cow<'a, str>,
    /// The 1/10th basis point of the DebtTotal that the
    /// first-loss capital must cover. Valid values are
    /// between 0 and 100000 inclusive. A value of 1
    /// is equivalent to 1/10 bps or 0.001%.
    pub cover_rate_minimum: u32,
    /// The 1/10th basis point of minimum required
    /// first-loss capital that is liquidated to
    /// cover a Loan default. Valid values
    /// are between 0 and 100000 inclusive.
    /// A value of 1 is equivalent to 1/10 bps or 0.001%.
    pub cover_rate_liquidation: u32,
}

impl<'a> Model for LoanBroker<'a> {}

impl<'a> LedgerObject<NoFlags> for LoanBroker<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> LoanBroker<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
        sequence: u32,
        loan_sequence: u32,
        owner_node: u64,
        vault_node: u64,
        vault_id: Cow<'a, str>,
        account: Cow<'a, str>,
        owner: Cow<'a, str>,
        data: Option<Cow<'a, str>>,
        management_fee_rate: Option<u16>,
        owner_count: u32,
        debt_total: Cow<'a, str>,
        debt_maximum: Cow<'a, str>,
        cover_available: Cow<'a, str>,
        cover_rate_minimum: u32,
        cover_rate_liquidation: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::LoanBroker,
                index,
                ledger_index: Some(ledger_index),
            },
            previous_txn_id,
            previous_txn_lgr_seq,
            sequence,
            loan_sequence,
            owner_node,
            vault_node,
            vault_id,
            account,
            owner,
            data,
            management_fee_rate,
            owner_count,
            debt_total,
            debt_maximum,
            cover_available,
            cover_rate_minimum,
            cover_rate_liquidation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::borrow::Cow;

    #[test]
    fn test_serde() {
        let loan_broker = LoanBroker::new(
            None,
            Cow::from("1ESDNBCNSGAFDGCFSGXF563BSGVGV8"),
            Cow::from(""),
            1734636,
            856363,
            638286,
            325452,
            2534267,
            Cow::from(""),
            Cow::from("rVALUE463dghsg26473642Ki436ghdghd"),
            Cow::from("56ERHJFVGRGFCVSG747YVGW"),
            None,
            Some(27),
            245,
            Cow::from("100000"),
            Cow::from("10000"),
            Cow::from("7000"),
            10,
            10,
        );

        let serialized = serde_json::to_string(&loan_broker).unwrap();
        let deserialized: LoanBroker = serde_json::from_str(&serialized).unwrap();

        assert_eq!(loan_broker, deserialized);
    }
}
