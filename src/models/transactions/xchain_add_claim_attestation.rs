use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{Amount, FlagCollection, Model, NoFlags, ValidateCurrencies, XChainBridge};

use super::{CommonFields, Transaction, TransactionType};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, xrpl_rust_macros::ValidateCurrencies)]
#[serde(rename_all = "PascalCase")]
pub struct XChainAddClaimAttestation<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    pub amount: Amount<'a>,
    pub attestation_reward_account: Cow<'a, str>,
    pub attestation_signer_account: Cow<'a, str>,
    pub other_chain_source: Cow<'a, str>,
    pub public_key: Cow<'a, str>,
    pub signature: Cow<'a, str>,
    pub was_locking_chain_send: u8,
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge<'a>,
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: Cow<'a, str>,
    pub destination: Option<Cow<'a, str>>,
}

impl Model for XChainAddClaimAttestation<'_> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for XChainAddClaimAttestation<'a> {
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

impl<'a> XChainAddClaimAttestation<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<crate::models::XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<super::Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<super::Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        amount: Amount<'a>,
        attestation_reward_account: Cow<'a, str>,
        attestation_signer_account: Cow<'a, str>,
        other_chain_source: Cow<'a, str>,
        public_key: Cow<'a, str>,
        signature: Cow<'a, str>,
        was_locking_chain_send: u8,
        xchain_bridge: XChainBridge<'a>,
        xchain_claim_id: Cow<'a, str>,
        destination: Option<Cow<'a, str>>,
    ) -> XChainAddClaimAttestation<'a> {
        XChainAddClaimAttestation {
            common_fields: CommonFields::new(
                account,
                TransactionType::XChainAddClaimAttestation,
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
            attestation_reward_account,
            attestation_signer_account,
            other_chain_source,
            public_key,
            signature,
            was_locking_chain_send,
            xchain_bridge,
            xchain_claim_id,
            destination,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::amount::XRPAmount;
    use crate::models::currency::XRP;

    fn xrp_bridge<'a>() -> XChainBridge<'a> {
        XChainBridge {
            locking_chain_door: "rMAXACCrp3Y8PpswXcg3bKggHX76V3F8M4".into(),
            locking_chain_issue: XRP::new().into(),
            issuing_chain_door: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
            issuing_chain_issue: XRP::new().into(),
        }
    }

    #[test]
    fn test_serde_round_trip() {
        let txn = XChainAddClaimAttestation::new(
            "rPV4mZjsXfH2HvUSPLNmqz1J8d3Lpv7tpe".into(),
            None,
            Some(XRPAmount::from("10")),
            None,
            None,
            Some(1),
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("10000")),
            "rPV4mZjsXfH2HvUSPLNmqz1J8d3Lpv7tpe".into(),
            "rPV4mZjsXfH2HvUSPLNmqz1J8d3Lpv7tpe".into(),
            "rSrc111111111111111111111111111111".into(),
            "ED1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF".into(),
            "30440220ABCDEF".into(),
            1,
            xrp_bridge(),
            "13f".into(),
            Some("rDest11111111111111111111111111111".into()),
        );
        let serialized = serde_json::to_string(&txn).unwrap();
        let deserialized: XChainAddClaimAttestation = serde_json::from_str(&serialized).unwrap();
        let reserialized = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(serialized, reserialized);
        assert!(serialized.contains("\"TransactionType\":\"XChainAddClaimAttestation\""));
        assert!(serialized.contains("\"XChainBridge\""));
        assert!(serialized.contains("\"XChainClaimID\":\"13f\""));
        assert!(serialized.contains("\"WasLockingChainSend\":1"));
    }

    #[test]
    fn test_get_transaction_type() {
        let txn = XChainAddClaimAttestation::new(
            "rPV4mZjsXfH2HvUSPLNmqz1J8d3Lpv7tpe".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("10000")),
            "rPV4mZjsXfH2HvUSPLNmqz1J8d3Lpv7tpe".into(),
            "rPV4mZjsXfH2HvUSPLNmqz1J8d3Lpv7tpe".into(),
            "rSrc111111111111111111111111111111".into(),
            "ED00".into(),
            "30".into(),
            0,
            xrp_bridge(),
            "1".into(),
            None,
        );
        assert_eq!(
            txn.get_transaction_type(),
            &TransactionType::XChainAddClaimAttestation
        );
    }
}
