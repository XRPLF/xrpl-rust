use alloc::borrow::Cow;
use alloc::string::ToString;

use serde::{Deserialize, Serialize};

use crate::models::ledger::objects::account_root::AccountRoot;
use crate::models::ledger::objects::signer_list::SignerList;
use crate::models::{XRPLModelException, XRPLModelResult};

use super::exceptions::XRPLResultException;
use super::{XRPLResponse, XRPLResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum AccountInfoVersionMap<'a> {
    Default(AccountInfo<'a>),
    V1(AccountInfoV1<'a>),
}

/// Account flags status information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccountFlags {
    /// If true, the account allows rippling on its trust lines by default.
    pub default_ripple: bool,
    /// If true, the account is using Deposit Authorization and does not
    /// accept any payments from unknown parties.
    pub deposit_auth: bool,
    /// If true, the account's master key pair is disabled.
    pub disable_master_key: bool,
    /// If true, the account does not allow others to send Checks to it.
    pub disallow_incoming_check: bool,
    /// If true, the account does not allow others to make NFT buy or sell
    /// offers to it.
    #[serde(rename = "disallowIncomingNFTokenOffer")]
    pub disallow_incoming_nftoken_offer: bool,
    /// If true, the account does not allow others to make Payment Channels
    /// to it.
    pub disallow_incoming_pay_chan: bool,
    /// If true, the account does not allow others to make trust lines to it.
    pub disallow_incoming_trustline: bool,
    /// If true, the account does not want to receive XRP from others.
    #[serde(rename = "disallowIncomingXRP")]
    pub disallow_incoming_xrp: bool,
    /// If true, all tokens issued by the account are currently frozen.
    pub global_freeze: bool,
    /// If true, the account has permanently given up the abilities to freeze
    /// individual trust lines or end a global freeze.
    pub no_freeze: bool,
    /// If false, the account can send a special key reset transaction with a
    /// transaction cost of 0.
    pub password_spent: bool,
    /// If true, the account is using Authorized Trust Lines to limit who can
    /// hold the tokens it issues.
    pub require_authorization: bool,
    /// If true, the account requires a destination tag on all payments it
    /// receives.
    pub require_destination_tag: bool,
    /// If true, allows trust line clawback
    pub allow_trust_line_clawback: bool,
}

/// Information about a queued transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct QueuedTransaction<'a> {
    /// Whether this transaction changes this address's ways of authorizing
    /// transactions.
    pub auth_change: bool,
    /// The Transaction Cost of this transaction, in drops of XRP.
    pub fee: Cow<'a, str>,
    /// The transaction cost of this transaction, relative to the minimum
    /// cost for this type
    /// of transaction, in fee levels.
    pub fee_level: Cow<'a, str>,
    /// The maximum amount of XRP, in drops, this transaction could send or
    /// destroy.
    pub max_spend_drops: Cow<'a, str>,
    /// The Sequence Number of this transaction.
    pub seq: u32,
}

/// Queue data for pending transactions
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct QueueData<'a> {
    /// Number of queued transactions from this address.
    pub txn_count: u32,
    /// Whether a transaction in the queue changes this address's ways of
    /// authorizing transactions.
    pub auth_change_queued: Option<bool>,
    /// The lowest Sequence Number among transactions queued by this address.
    pub lowest_sequence: Option<u32>,
    /// The highest Sequence Number among transactions queued by this address.
    pub highest_sequence: Option<u32>,
    /// Integer amount of drops of XRP that could be debited from this address
    /// if every transaction in the queue consumes the maximum amount of XRP
    /// possible.
    pub max_spend_drops_total: Option<Cow<'a, str>>,
    /// Information about each queued transaction from this address.
    pub transactions: Option<Cow<'a, [QueuedTransaction<'a>]>>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInfoBase<'a> {
    /// The AccountRoot ledger object with this account's information, as
    /// stored in the ledger.
    pub account_data: AccountRoot<'a>,
    /// The account's flag statuses (see below), based on the Flags field
    /// of the account.
    pub account_flags: Option<AccountFlags>,
    /// (Omitted if `ledger_index` is provided instead) The ledger index of
    /// the current in-progress ledger, which was used when retrieving this
    /// information.
    pub ledger_current_index: Option<u32>,
    /// (Omitted if `ledger_current_index` is provided instead) The ledger
    /// index of the ledger version used when retrieving this information.
    /// The information does not contain any changes from ledger versions
    /// newer than this one.
    pub ledger_index: Option<u32>,
    /// (Omitted unless queue specified as true and querying the current open
    /// ledger.) Information about queued transactions sent by this account.
    /// This information describes the state of the local rippled server,
    /// which may be different from other servers in the peer-to-peer XRP
    /// Ledger network. Some fields may be omitted because the values are
    /// calculated "lazily" by the queuing mechanism.
    pub queue_data: Option<QueueData<'a>>,
    /// True if this data is from a validated ledger version; if omitted or
    /// set to false, this data is not final.
    pub validated: bool,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInfoV1<'a> {
    #[serde(flatten)]
    pub base: AccountInfoBase<'a>,
    /// If requested, array of SignerList ledger objects associated with this
    /// account for Multi-Signing. Since an account can own at most one
    /// SignerList, this array must have exactly one member if it is present.
    pub signer_lists: Option<Cow<'a, [SignerList<'a>]>>,
}

/// Response from an account_info request, containing information about an
/// account.
///
/// See Account Info:
/// `<https://xrpl.org/account_info.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInfo<'a> {
    #[serde(flatten)]
    pub base: AccountInfoBase<'a>,
    /// If requested, array of SignerList ledger objects associated with this
    /// account for Multi-Signing. Since an account can own at most one
    /// SignerList, this array must have exactly one member if it is present.
    pub signer_lists: Option<Cow<'a, [SignerList<'a>]>>,
}

impl<'a> AccountInfoVersionMap<'a> {
    pub fn get_account_root(&self) -> &AccountRoot<'a> {
        match self {
            AccountInfoVersionMap::Default(account_info) => &account_info.base.account_data,
            AccountInfoVersionMap::V1(account_info) => &account_info.base.account_data,
        }
    }
}

impl<'a> TryFrom<XRPLResult<'a>> for AccountInfoVersionMap<'a> {
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

impl<'a> TryFrom<XRPLResponse<'a>> for AccountInfoVersionMap<'a> {
    type Error = XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => AccountInfoVersionMap::try_from(result),
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_info_deserialization() {
        let json = r#"{
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
            "status": "success",
            "validated": false
        }"#;

        let account_info: AccountInfo = serde_json::from_str(json).unwrap();

        assert_eq!(
            account_info.base.account_data.account,
            "rG1QQv2nh2gr7RCZ1P8YYcBUKCCN633jCn"
        );
        assert_eq!(
            account_info.base.account_data.balance,
            Some("999999999960".into())
        );
        assert_eq!(
            u32::try_from(account_info.base.account_data.common_fields.flags)
                .expect("Failed to convert flags to u32"),
            8388608
        );
        assert_eq!(account_info.base.account_data.owner_count, 0);
        assert_eq!(
            account_info.base.account_data.previous_txn_id.as_ref(),
            "4294BEBE5B569A18C0A2702387C9B1E7146DC3A5850C1E87204951C6FDAA4C42"
        );
        assert_eq!(account_info.base.account_data.previous_txn_lgr_seq, 3);
        assert_eq!(account_info.base.account_data.sequence, 6);
        assert_eq!(account_info.base.ledger_current_index.unwrap(), 4);
        assert_eq!(account_info.base.validated, false);

        let queue_data = account_info.base.queue_data.unwrap();
        assert_eq!(queue_data.auth_change_queued.unwrap(), true);
        assert_eq!(queue_data.highest_sequence.unwrap(), 10);
        assert_eq!(queue_data.lowest_sequence.unwrap(), 6);
        assert_eq!(queue_data.max_spend_drops_total.unwrap(), "500");
        assert_eq!(queue_data.txn_count, 5);

        let transactions = queue_data.transactions.unwrap();
        assert_eq!(transactions.len(), 2);

        let first_tx = &transactions[0];
        assert_eq!(first_tx.auth_change, false);
        assert_eq!(first_tx.fee, "100");
        assert_eq!(first_tx.fee_level, "2560");
        assert_eq!(first_tx.max_spend_drops, "100");
        assert_eq!(first_tx.seq, 6);

        let last_tx = &transactions[1];
        assert_eq!(last_tx.auth_change, true);
        assert_eq!(last_tx.fee, "100");
        assert_eq!(last_tx.fee_level, "2560");
        assert_eq!(last_tx.max_spend_drops, "100");
        assert_eq!(last_tx.seq, 10);
    }
}
