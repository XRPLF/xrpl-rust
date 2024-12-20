use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{Model, NoFlags, XChainBridge, XRPAmount};

use super::{CommonFields, LedgerEntryType, LedgerObject};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Bridge<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    pub account: Cow<'a, str>,
    pub signature_reward: XRPAmount<'a>,
    #[serde(rename = "XChainAccountClaimCount")]
    pub xchain_account_claim_count: u64,
    #[serde(rename = "XChainAccountCreateCount")]
    pub xchain_account_create_count: u64,
    pub xchain_bridge: XChainBridge<'a>,
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: Cow<'a, str>,
    pub min_account_create_amount: Option<XRPAmount<'a>>,
}

impl Model for Bridge<'_> {}

impl LedgerObject<NoFlags> for Bridge<'_> {
    fn get_ledger_entry_type(&self) -> super::LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> Bridge<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        signature_reward: XRPAmount<'a>,
        xchain_account_claim_count: u64,
        xchain_account_create_count: u64,
        xchain_bridge: XChainBridge<'a>,
        xchain_claim_id: Cow<'a, str>,
        min_account_create_amount: Option<XRPAmount<'a>>,
    ) -> Bridge<'a> {
        Bridge {
            common_fields: CommonFields {
                flags: Default::default(),
                ledger_entry_type: LedgerEntryType::Bridge,
                index,
                ledger_index,
            },
            account,
            signature_reward,
            xchain_account_claim_count,
            xchain_account_create_count,
            xchain_bridge,
            xchain_claim_id,
            min_account_create_amount,
        }
    }
}
