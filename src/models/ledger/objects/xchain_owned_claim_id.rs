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
