use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    Amount, FlagCollection, Model, NoFlags, ValidateCurrencies, XChainBridge, XRPAmount,
};

use super::{CommonFields, Memo, Signer, Transaction, TransactionType};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, xrpl_rust_macros::ValidateCurrencies)]
#[serde(rename_all = "PascalCase")]
pub struct XChainCommit<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    pub amount: Amount<'a>,
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge<'a>,
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: Cow<'a, str>,
    pub other_chain_destination: Option<Cow<'a, str>>,
}

impl Model for XChainCommit<'_> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()?;
        super::reject_mpt_amount("amount", &self.amount)
    }
}

impl<'a> Transaction<'a, NoFlags> for XChainCommit<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn get_transaction_type(&self) -> &super::TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> XChainCommit<'a> {
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
        xchain_bridge: XChainBridge<'a>,
        xchain_claim_id: Cow<'a, str>,
        other_chain_destination: Option<Cow<'a, str>>,
    ) -> XChainCommit<'a> {
        XChainCommit {
            common_fields: CommonFields::new(
                account,
                TransactionType::XChainCommit,
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
            other_chain_destination,
            xchain_bridge,
            xchain_claim_id,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use serde_json::Value;

    use crate::models::transactions::{
        test_fixtures::{GENESIS_ACCOUNT, LOCKING_CHAIN_DOOR_ACCOUNT, XCHAIN_COMMIT_TEST_ACCOUNT},
        xchain_commit::XChainCommit,
    };

    fn example_json() -> Value {
        serde_json::json!({
            "Account": XCHAIN_COMMIT_TEST_ACCOUNT,
            "Flags": 0,
            "TransactionType": "XChainCommit",
            "XChainBridge": {
                "LockingChainDoor": LOCKING_CHAIN_DOOR_ACCOUNT,
                "LockingChainIssue": {
                    "currency": "XRP"
                },
                "IssuingChainDoor": GENESIS_ACCOUNT,
                "IssuingChainIssue": {
                    "currency": "XRP"
                }
            },
            "Amount": "10000",
            "XChainClaimID": "13f"
        })
    }

    #[test]
    fn test_deserialize() {
        let json = example_json();
        let deserialized: Result<XChainCommit<'_>, _> = serde_json::from_value(json);
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_serialize() {
        let expected = example_json();
        let attestation: XChainCommit<'_> = serde_json::from_value(expected.clone()).unwrap();
        let actual = serde_json::to_value(&attestation).unwrap();

        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::currency::XRP;
    use crate::models::transactions::{
        test_fixtures::{GENESIS_ACCOUNT, LOCKING_CHAIN_DOOR_ACCOUNT, XCHAIN_COMMIT_TEST_ACCOUNT},
        Transaction,
    };
    use crate::models::MPTAmount;

    fn xrp_bridge<'a>() -> XChainBridge<'a> {
        XChainBridge {
            locking_chain_door: LOCKING_CHAIN_DOOR_ACCOUNT.into(),
            locking_chain_issue: XRP::new().into(),
            issuing_chain_door: GENESIS_ACCOUNT.into(),
            issuing_chain_issue: XRP::new().into(),
        }
    }

    #[test]
    fn test_constructor_round_trip() {
        let txn = XChainCommit::new(
            XCHAIN_COMMIT_TEST_ACCOUNT.into(),
            None,
            Some(XRPAmount::from("10")),
            None,
            None,
            Some(1),
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("10000")),
            xrp_bridge(),
            "13f".into(),
            Some("rDest11111111111111111111111111111".into()),
        );
        let serialized = serde_json::to_string(&txn).unwrap();
        let deserialized: XChainCommit = serde_json::from_str(&serialized).unwrap();
        let reserialized = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(serialized, reserialized);
        assert!(serialized.contains("\"TransactionType\":\"XChainCommit\""));
        assert!(serialized.contains("\"XChainBridge\""));
        assert!(serialized.contains("\"XChainClaimID\":\"13f\""));
        assert!(serialized.contains("\"OtherChainDestination\""));
        assert_eq!(txn.get_transaction_type(), &TransactionType::XChainCommit);
    }

    #[test]
    fn test_validate_currencies_ok() {
        let txn = XChainCommit::new(
            XCHAIN_COMMIT_TEST_ACCOUNT.into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("10000")),
            xrp_bridge(),
            "1".into(),
            None,
        );
        assert!(txn.get_errors().is_ok());
    }

    #[test]
    fn test_rejects_mpt_amount() {
        let txn = XChainCommit {
            common_fields: CommonFields::new(
                XCHAIN_COMMIT_TEST_ACCOUNT.into(),
                TransactionType::XChainCommit,
                None,
                None,
                Some(FlagCollection::default()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ),
            amount: Amount::MPTAmount(MPTAmount::new(
                "100".into(),
                crate::models::transactions::test_fixtures::MPT_ISSUANCE_ID.into(),
            )),
            xchain_bridge: xrp_bridge(),
            xchain_claim_id: "1".into(),
            other_chain_destination: None,
        };
        assert!(txn.get_errors().is_err());
    }
}
