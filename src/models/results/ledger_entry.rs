use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::ledger::objects::{
    account_root::AccountRoot, amendments::Amendments, amm::AMM, bridge::Bridge, check::Check,
    deposit_preauth::DepositPreauth, directory_node::DirectoryNode, escrow::Escrow,
    fee_settings::FeeSettings, ledger_hashes::LedgerHashes, negative_unl::NegativeUNL,
    nftoken_offer::NFTokenOffer, nftoken_page::NFTokenPage, offer::Offer, pay_channel::PayChannel,
    ripple_state::RippleState, signer_list::SignerList, ticket::Ticket,
    xchain_owned_claim_id::XChainOwnedClaimID,
    xchain_owned_create_account_claim_id::XChainOwnedCreateAccountClaimID,
};

/// A discriminated union representing any ledger object type that can be
/// returned by the `ledger_entry` method. Dispatches on the `LedgerEntryType`
/// field, mirroring the `LedgerEntry` union type in xrpl.js.
///
/// Each variant wraps the corresponding ledger object struct from
/// `crate::models::ledger::objects`. The `Unknown` variant handles any
/// ledger entry types not yet modeled in the library.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum LedgerEntryNode<'a> {
    AccountRoot(AccountRoot<'a>),
    Amendments(Amendments<'a>),
    AMM(AMM<'a>),
    Bridge(Bridge<'a>),
    Check(Check<'a>),
    DepositPreauth(DepositPreauth<'a>),
    DirectoryNode(DirectoryNode<'a>),
    Escrow(Escrow<'a>),
    FeeSettings(FeeSettings<'a>),
    LedgerHashes(LedgerHashes<'a>),
    NegativeUNL(NegativeUNL<'a>),
    NFTokenOffer(NFTokenOffer<'a>),
    NFTokenPage(NFTokenPage<'a>),
    Offer(Offer<'a>),
    PayChannel(PayChannel<'a>),
    RippleState(RippleState<'a>),
    SignerList(SignerList<'a>),
    Ticket(Ticket<'a>),
    XChainOwnedClaimID(XChainOwnedClaimID<'a>),
    XChainOwnedCreateAccountClaimID(XChainOwnedCreateAccountClaimID<'a>),
    /// Fallback for unknown or new ledger entry types not yet modeled.
    Unknown(Value),
}

/// Custom deserializer that reads `LedgerEntryType` to dispatch to the
/// correct variant, avoiding the serde limitation where internally tagged
/// enums strip the tag from flattened content.
impl<'de, 'a> Deserialize<'de> for LedgerEntryNode<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let entry_type = value
            .get("LedgerEntryType")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let result = match entry_type {
            "AccountRoot" => serde_json::from_value(value.clone()).map(Self::AccountRoot),
            "Amendments" => serde_json::from_value(value.clone()).map(Self::Amendments),
            "AMM" => serde_json::from_value(value.clone()).map(Self::AMM),
            "Bridge" => serde_json::from_value(value.clone()).map(Self::Bridge),
            "Check" => serde_json::from_value(value.clone()).map(Self::Check),
            "DepositPreauth" => serde_json::from_value(value.clone()).map(Self::DepositPreauth),
            "DirectoryNode" => serde_json::from_value(value.clone()).map(Self::DirectoryNode),
            "Escrow" => serde_json::from_value(value.clone()).map(Self::Escrow),
            "FeeSettings" => serde_json::from_value(value.clone()).map(Self::FeeSettings),
            "LedgerHashes" => serde_json::from_value(value.clone()).map(Self::LedgerHashes),
            "NegativeUNL" => serde_json::from_value(value.clone()).map(Self::NegativeUNL),
            "NFTokenOffer" => serde_json::from_value(value.clone()).map(Self::NFTokenOffer),
            "NFTokenPage" => serde_json::from_value(value.clone()).map(Self::NFTokenPage),
            "Offer" => serde_json::from_value(value.clone()).map(Self::Offer),
            "PayChannel" => serde_json::from_value(value.clone()).map(Self::PayChannel),
            "RippleState" => serde_json::from_value(value.clone()).map(Self::RippleState),
            "SignerList" => serde_json::from_value(value.clone()).map(Self::SignerList),
            "Ticket" => serde_json::from_value(value.clone()).map(Self::Ticket),
            "XChainOwnedClaimID" => {
                serde_json::from_value(value.clone()).map(Self::XChainOwnedClaimID)
            }
            "XChainOwnedCreateAccountClaimID" => {
                serde_json::from_value(value.clone()).map(Self::XChainOwnedCreateAccountClaimID)
            }
            _ => return Ok(Self::Unknown(value)),
        };

        result.map_err(serde::de::Error::custom)
    }
}

/// Response format for the ledger_entry method, which returns a single ledger
/// object from the XRP Ledger in its raw format.
///
/// The `node` field is a typed enum (`LedgerEntryNode`) that can represent any
/// ledger object type (AccountRoot, DirectoryNode, Offer, RippleState, etc.),
/// mirroring the `LedgerEntry` union type in xrpl.js.
///
/// See Ledger Entry:
/// `<https://xrpl.org/ledger_entry.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LedgerEntry<'a> {
    /// The unique ID of this ledger entry.
    pub index: Cow<'a, str>,
    /// The ledger index of the ledger that was used when retrieving this data.
    pub ledger_index: Option<u32>,
    /// The identifying hash of the ledger version used to retrieve this data
    pub ledger_hash: Option<Cow<'a, str>>,
    /// Object containing the data of this ledger entry, according to the
    /// ledger format. Can be any ledger object type (AccountRoot,
    /// DirectoryNode, Offer, etc.). Omitted if "binary": true specified.
    pub node: Option<LedgerEntryNode<'a>>,
    /// The binary representation of the ledger object, as hexadecimal.
    /// Only present if "binary": true specified.
    pub node_binary: Option<Cow<'a, str>>,
    /// (Clio server only) The ledger index where the ledger entry object was
    /// deleted. Only present if include_deleted parameter is set.
    pub deleted_ledger_index: Option<Cow<'a, str>>,
    /// Whether this data is from a validated ledger version
    pub validated: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_entry_deserialize() {
        let json = r#"{
            "index": "13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8",
            "ledger_hash": "31850E8E48E76D1064651DF39DF4E9542E8C90A9A9B629F4DE339EB3FA74F726",
            "ledger_index": 61966146,
            "node": {
                "Account": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
                "AccountTxnID": "4E0AA11CBDD1760DE95B68DF2ABBE75C9698CEB548BEA9789053FCB3EBD444FB",
                "Balance": "424021949",
                "Domain": "6D64756F31332E636F6D",
                "EmailHash": "98B4375E1D753E5B91627516F6D70977",
                "Flags": 9568256,
                "LedgerEntryType": "AccountRoot",
                "MessageKey": "0000000000000000000000070000000300",
                "OwnerCount": 12,
                "PreviousTxnID": "4E0AA11CBDD1760DE95B68DF2ABBE75C9698CEB548BEA9789053FCB3EBD444FB",
                "PreviousTxnLgrSeq": 61965653,
                "RegularKey": "rD9iJmieYHn8jTtPjwwkW2Wm9sVDvPXLoJ",
                "Sequence": 385,
                "TransferRate": 4294967295,
                "index": "13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8"
            },
            "validated": true
        }"#;

        let result: LedgerEntry = serde_json::from_str(json).unwrap();

        assert_eq!(
            result.index,
            "13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8"
        );
        assert_eq!(result.ledger_index, Some(61966146));
        assert_eq!(
            result.ledger_hash,
            Some("31850E8E48E76D1064651DF39DF4E9542E8C90A9A9B629F4DE339EB3FA74F726".into())
        );
        assert_eq!(result.validated, Some(true));

        let node = result.node.unwrap();
        match node {
            LedgerEntryNode::AccountRoot(account_root) => {
                assert_eq!(account_root.account, "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn");
                assert_eq!(
                    account_root.account_txn_id,
                    Some("4E0AA11CBDD1760DE95B68DF2ABBE75C9698CEB548BEA9789053FCB3EBD444FB".into())
                );
                assert_eq!(account_root.sequence, 385);
            }
            _ => panic!("Expected AccountRoot variant"),
        }
    }
}
