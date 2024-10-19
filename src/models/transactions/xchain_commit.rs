use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{Amount, FlagCollection, Model, NoFlags, XChainBridge, XRPAmount};

use super::{CommonFields, Memo, Signer, Transaction, TransactionType};

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
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

impl Model for XChainCommit<'_> {}

impl<'a> Transaction<'a, NoFlags> for XChainCommit<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn get_transaction_type(&self) -> super::TransactionType {
        TransactionType::XChainCommit
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
        signers: Option<Vec<Signer<'a>>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        amount: Amount<'a>,
        xchain_bridge: XChainBridge<'a>,
        xchain_claim_id: Cow<'a, str>,
        other_chain_destination: Option<Cow<'a, str>>,
    ) -> XChainCommit<'a> {
        XChainCommit {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::XChainCommit,
                account_txn_id,
                fee,
                last_ledger_sequence,
                memos,
                sequence,
                signers,
                source_tag,
                ticket_sequence,
                flags: FlagCollection::default(),
                network_id: None,
                signing_pub_key: None,
                txn_signature: None,
            },
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
