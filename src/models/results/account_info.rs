use core::convert::TryFrom;

use alloc::{borrow::Cow, string::ToString, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::models::{
    ledger::objects::{signer_list::SignerList, AccountRoot},
    XRPLModelException, XRPLModelResult,
};

use super::{exceptions::XRPLResultException, XRPLResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AccountFlags {
    pub default_ripple: bool,
    pub deposit_auth: bool,
    pub disable_master_key: bool,
    pub disallow_incoming_check: bool,
    #[serde(rename = "disallowIncomingNFTokenOffer")]
    pub disallow_incoming_nftoken_offer: bool,
    pub disallow_incoming_pay_chan: bool,
    pub disallow_incoming_trustline: bool,
    #[serde(rename = "disallowIncomingXRP")]
    pub disallow_incoming_xrp: bool,
    pub global_freeze: bool,
    pub no_freeze: bool,
    pub password_spent: bool,
    pub require_authorization: bool,
    pub require_destination_tag: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueData<'a> {
    pub txn_count: u32,
    pub auth_change_queued: Option<bool>,
    pub lowest_sequence: Option<u32>,
    pub highest_sequence: Option<u32>,
    pub max_spend_drops_total: Option<Cow<'a, str>>,
    pub transactions: Option<Vec<QueueDataTransaction<'a>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueDataTransaction<'a> {
    pub auth_change: bool,
    pub fee: Cow<'a, str>,
    pub fee_level: Cow<'a, str>,
    pub max_spend_drops: Cow<'a, str>,
    pub seq: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInfo<'a> {
    pub account_data: AccountRoot<'a>,
    pub account_flags: AccountFlags,
    pub validated: bool,
    pub signer_lists: Option<Vec<SignerList<'a>>>,
    pub ledger_current_index: Option<u32>,
    pub ledger_index: Option<u32>,
    pub queue_data: Option<QueueData<'a>>,
}

impl<'a> TryFrom<XRPLResult<'a>> for AccountInfo<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::AccountInfo(account_info) => Ok(account_info),
            res => Err(XRPLResultException::UnexpectedResultType(
                "AccountInfo".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}
