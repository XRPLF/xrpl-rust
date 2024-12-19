use core::fmt::Debug;

use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{Amount, FlagCollection, Model, NoFlags, XChainBridge, XRPAmount};

use super::{CommonFields, Memo, Signer, Transaction, TransactionType};

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainAccountCreateCommit<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    pub amount: Amount<'a>,
    pub destination: Cow<'a, str>,
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge<'a>,
    pub signature_reward: Option<Amount<'a>>,
}

impl Model for XChainAccountCreateCommit<'_> {}

impl<'a> Transaction<'a, NoFlags> for XChainAccountCreateCommit<'a> {
    fn get_transaction_type(&self) -> &super::TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }
}

impl<'a> XChainAccountCreateCommit<'a> {
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
        destination: Cow<'a, str>,
        xchain_bridge: XChainBridge<'a>,
        signature_reward: Option<Amount<'a>>,
    ) -> XChainAccountCreateCommit<'a> {
        XChainAccountCreateCommit {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::XChainAccountCreateCommit,
                account_txn_id,
                fee,
                flags: FlagCollection::default(),
                last_ledger_sequence,
                memos,
                sequence,
                signers,
                source_tag,
                ticket_sequence,
                network_id: None,
                signing_pub_key: None,
                txn_signature: None,
            },
            amount,
            destination,
            xchain_bridge,
            signature_reward,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::XChainAccountCreateCommit;

    #[test]
    fn test_deserialize() {
        let json = r#"{
            "Account": "rwEqJ2UaQHe7jihxGqmx6J4xdbGiiyMaGa",
            "Destination": "rD323VyRjgzzhY4bFpo44rmyh2neB5d8Mo",
            "TransactionType": "XChainAccountCreateCommit",
            "Amount": "20000000",
            "SignatureReward": "100",
            "XChainBridge": {
                "LockingChainDoor": "rMAXACCrp3Y8PpswXcg3bKggHX76V3F8M4",
                "LockingChainIssue": {
                    "currency": "XRP"
                },
                "IssuingChainDoor": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
                "IssuingChainIssue": {
                    "currency": "XRP"
                }
            }
        }"#;
        let txn: XChainAccountCreateCommit<'_> = serde_json::from_str(json).unwrap();
        assert_eq!(txn.amount, "20000000".into());
    }
}
