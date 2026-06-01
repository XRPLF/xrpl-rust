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
        self.validate_currencies()
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

    use crate::models::transactions::xchain_commit::XChainCommit;

    const EXAMPLE_JSON: &str = r#"{
        "Account": "rMTi57fNy2UkUb4RcdoUeJm7gjxVQvxzUo",
        "Flags": 0,
        "TransactionType": "XChainCommit",
        "XChainBridge": {
            "LockingChainDoor": "rMAXACCrp3Y8PpswXcg3bKggHX76V3F8M4",
            "LockingChainIssue": {
                "currency": "XRP"
            },
            "IssuingChainDoor": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "IssuingChainIssue": {
                "currency": "XRP"
            }
        },
        "Amount": "10000",
        "XChainClaimID": "13f"
    }"#;

    #[test]
    fn test_deserialize() {
        let json = EXAMPLE_JSON;
        let deserialized: Result<XChainCommit<'_>, _> = serde_json::from_str(json);
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_serialize() {
        let attestation: XChainCommit<'_> = serde_json::from_str(EXAMPLE_JSON).unwrap();
        let actual = serde_json::to_value(&attestation).unwrap();
        let expected: Value = serde_json::from_str(EXAMPLE_JSON).unwrap();

        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::currency::XRP;
    use crate::models::transactions::Transaction;

    fn xrp_bridge<'a>() -> XChainBridge<'a> {
        XChainBridge {
            locking_chain_door: "rMAXACCrp3Y8PpswXcg3bKggHX76V3F8M4".into(),
            locking_chain_issue: XRP::new().into(),
            issuing_chain_door: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
            issuing_chain_issue: XRP::new().into(),
        }
    }

    #[test]
    fn test_constructor_round_trip() {
        let txn = XChainCommit::new(
            "rMTi57fNy2UkUb4RcdoUeJm7gjxVQvxzUo".into(),
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
            "rMTi57fNy2UkUb4RcdoUeJm7gjxVQvxzUo".into(),
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
}
