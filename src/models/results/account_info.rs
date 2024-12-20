use core::convert::TryFrom;

use alloc::{borrow::Cow, string::ToString, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::models::{
    ledger::objects::{account_root::AccountRoot, signer_list::SignerList},
    XRPLModelException, XRPLModelResult,
};

use super::{exceptions::XRPLResultException, XRPLResponse, XRPLResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum AccountInfoMap<'a> {
    Default(AccountInfo<'a>),
    V1(AccountInfoV1<'a>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInfoBase<'a> {
    /// The AccountRoot ledger object with this account's information, as stored in the ledger.
    pub account_data: AccountRoot<'a>,
    /// The account's flag statuses (see below), based on the Flags field of the account.
    pub account_flags: Option<AccountFlags>,
    /// (Omitted if `ledger_index` is provided instead) The ledger index of the current in-progress ledger,
    /// which was used when retrieving this information.
    pub ledger_current_index: Option<u32>,
    /// (Omitted if `ledger_current_index` is provided instead) The ledger index of the ledger version used
    ///  when retrieving this information. The information does not contain any changes from ledger
    /// versions newer than this one.
    pub ledger_index: Option<u32>,
    /// (Omitted unless queue specified as true and querying the current open ledger.) Information about queued
    /// transactions sent by this account. This information describes the state of the local rippled server, which
    /// may be different from other servers in the peer-to-peer XRP Ledger network. Some fields may be omitted
    /// because the values are calculated "lazily" by the queuing mechanism.
    pub queue_data: Option<QueueData<'a>>,
    /// True if this data is from a validated ledger version; if omitted or set to false, this data is not final.
    pub validated: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInfo<'a> {
    #[serde(flatten)]
    pub base: AccountInfoBase<'a>,
    /// If requested, array of SignerList ledger objects associated with this account for Multi-Signing.
    /// Since an account can own at most one SignerList, this array must have exactly one
    /// member if it is present.
    pub signer_lists: Option<Vec<SignerList<'a>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInfoV1<'a> {
    #[serde(flatten)]
    pub base: AccountInfoBase<'a>,
    /// If requested, array of SignerList ledger objects associated with this account for Multi-Signing.
    /// Since an account can own at most one SignerList, this array must have exactly one
    /// member if it is present.
    pub signer_lists: Option<Vec<SignerList<'a>>>,
}

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
    /// Number of queued transactions from this address.
    pub txn_count: u32,
    /// Whether a transaction in the queue changes this address's ways of authorizing transactions.
    /// If true, this address can queue no further transactions until that transaction has been
    /// executed or dropped from the queue.
    pub auth_change_queued: Option<bool>,
    /// The lowest Sequence Number among transactions queued by this address.
    pub lowest_sequence: Option<u32>,
    /// The highest Sequence Number among transactions queued by this address.
    pub highest_sequence: Option<u32>,
    /// Integer amount of drops of XRP that could be debited from this address if every transaction
    /// in the queue consumes the maximum amount of XRP possible.
    pub max_spend_drops_total: Option<Cow<'a, str>>,
    ///  Information about each queued transaction from this address.
    pub transactions: Option<Vec<QueueDataTransaction<'a>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueDataTransaction<'a> {
    /// Whether this transaction changes this address's ways of authorizing transactions.
    pub auth_change: bool,
    /// The Transaction Cost of this transaction, in drops of XRP.
    pub fee: Cow<'a, str>,
    /// The transaction cost of this transaction, relative to the minimum cost for this type of transaction, in fee levels.
    pub fee_level: Cow<'a, str>,
    /// The maximum amount of XRP, in drops, this transaction could send or destroy.
    pub max_spend_drops: Cow<'a, str>,
    /// The Sequence Number of this transaction.
    pub seq: u32,
}

impl<'a> AccountInfoMap<'a> {
    pub fn get_account_root(&self) -> &AccountRoot<'a> {
        match self {
            AccountInfoMap::Default(account_info) => &account_info.base.account_data,
            AccountInfoMap::V1(account_info) => &account_info.base.account_data,
        }
    }
}

impl<'a> TryFrom<XRPLResult<'a>> for AccountInfoMap<'a> {
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

impl<'a> TryFrom<XRPLResponse<'a>> for AccountInfoMap<'a> {
    type Error = XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => AccountInfoMap::try_from(result),
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    const RESPONSE: &str = r#"{
            "account_data": {
                "Account": "rG1QQv2nh2gr7RCZ1P8YYcBUKCCN633jCn",
                "Balance": "999999999960",
                "Flags": 8388608,
                "LedgerEntryType": "AccountRoot",
                "OwnerCount": 0,
                "PreviousTxnID": "4294BEBE5B569A18C0A2702387C9B1E7146DC3A5850C1E87204951C6FDAA4C42",
                "PreviousTxnLgrSeq": 3,
                "Sequence": 6,
                "index": "92FA6A9FC8EA6018D5D16532D7795C91BFB0831355BDFDA177E86C8BF997985F"
            },
            "ledger_current_index": 4,
            "queue_data": {
                "auth_change_queued": true,
                "highest_sequence": 10,
                "lowest_sequence": 6,
                "max_spend_drops_total": "500",
                "transactions": [
                    {
                        "auth_change": false,
                        "fee": "100",
                        "fee_level": "2560",
                        "max_spend_drops": "100",
                        "seq": 6
                    },
                    {
                        "LastLedgerSequence": 10,
                        "auth_change": true,
                        "fee": "100",
                        "fee_level": "2560",
                        "max_spend_drops": "100",
                        "seq": 10
                    }
                ],
                "txn_count": 5
            },
            "validated": false
    }
    "#;

    #[test]
    fn test_deserialize_account_info<'a>() -> XRPLModelResult<()> {
        let _: AccountInfoMap = serde_json::from_str(RESPONSE)?;

        Ok(())
    }
}
