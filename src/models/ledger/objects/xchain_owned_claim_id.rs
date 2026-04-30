use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{Amount, Model, NoFlags, XChainBridge};

use super::{CommonFields, LedgerEntryType, LedgerObject, XChainClaimProofSig};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct XChainOwnedClaimID<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    pub account: Cow<'a, str>,
    pub other_chain_source: Cow<'a, str>,
    pub signature_reward: Amount<'a>,
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge<'a>,
    #[serde(rename = "XChainClaimAttestations")]
    pub xchain_claim_attestations: Vec<XChainClaimProofSig<'a>>,
    pub xchain_claim_id: Cow<'a, str>,
}

impl Model for XChainOwnedClaimID<'_> {}

impl LedgerObject<NoFlags> for XChainOwnedClaimID<'_> {
    fn get_ledger_entry_type(&self) -> super::LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> XChainOwnedClaimID<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        other_chain_source: Cow<'a, str>,
        signature_reward: Amount<'a>,
        xchain_bridge: XChainBridge<'a>,
        xchain_claim_attestations: Vec<XChainClaimProofSig<'a>>,
        xchain_claim_id: Cow<'a, str>,
    ) -> XChainOwnedClaimID<'a> {
        XChainOwnedClaimID {
            common_fields: CommonFields {
                flags: Default::default(),
                ledger_entry_type: LedgerEntryType::XChainOwnedClaimID,
                index,
                ledger_index,
            },
            account,
            other_chain_source,
            signature_reward,
            xchain_bridge,
            xchain_claim_attestations,
            xchain_claim_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::amount::XRPAmount;
    use crate::models::currency::XRP;
    use alloc::vec;

    #[test]
    fn test_xchain_owned_claim_id_serde_round_trip() {
        let attestation = XChainClaimProofSig {
            amount: Amount::XRPAmount(XRPAmount::from("10000")),
            attestation_reward_account: "rPV4mZjsXfH2HvUSPLNmqz1J8d3Lpv7tpe".into(),
            attestation_signer_account: "rPV4mZjsXfH2HvUSPLNmqz1J8d3Lpv7tpe".into(),
            destination: "rDest11111111111111111111111111111".into(),
            public_key: "ED1234567890ABCDEF".into(),
            was_locking_chain_send: 1,
        };
        let entry = XChainOwnedClaimID::new(
            Some("AABBCC".into()),
            None,
            "rPV4mZjsXfH2HvUSPLNmqz1J8d3Lpv7tpe".into(),
            "rSrc111111111111111111111111111111".into(),
            Amount::XRPAmount(XRPAmount::from("100")),
            XChainBridge {
                locking_chain_door: "rMAXACCrp3Y8PpswXcg3bKggHX76V3F8M4".into(),
                locking_chain_issue: XRP::new().into(),
                issuing_chain_door: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                issuing_chain_issue: XRP::new().into(),
            },
            vec![attestation],
            "13f".into(),
        );
        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: XChainOwnedClaimID = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entry, deserialized);
        assert!(serialized.contains("\"LedgerEntryType\":\"XChainOwnedClaimID\""));
        assert!(serialized.contains("\"XChainClaimAttestations\""));
        assert_eq!(
            entry.get_ledger_entry_type(),
            LedgerEntryType::XChainOwnedClaimID
        );
    }
}
