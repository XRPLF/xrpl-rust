pub mod account_root;
pub mod amendments;
pub mod amm;
pub mod bridge;
pub mod check;
pub mod deposit_preauth;
pub mod directory_node;
pub mod escrow;
pub mod fee_settings;
pub mod ledger_hashes;
pub mod negative_unl;
pub mod nftoken_offer;
pub mod nftoken_page;
pub mod offer;
pub mod pay_channel;
pub mod ripple_state;
pub mod signer_list;
pub mod ticket;
pub mod xchain_owned_claim_id;
pub mod xchain_owned_create_account_claim_id;

use account_root::AccountRoot;
use amendments::Amendments;
use amm::AMM;
use bridge::Bridge;
use check::Check;
use deposit_preauth::DepositPreauth;
use derive_new::new;
use directory_node::DirectoryNode;
use escrow::Escrow;
use fee_settings::FeeSettings;
use ledger_hashes::LedgerHashes;
use negative_unl::NegativeUNL;
use nftoken_offer::NFTokenOffer;
use nftoken_page::NFTokenPage;
use offer::Offer;
use pay_channel::PayChannel;
use ripple_state::RippleState;
use signer_list::SignerList;
use strum::IntoEnumIterator;

use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum_macros::Display;
use ticket::Ticket;
use xchain_owned_claim_id::XChainOwnedClaimID;
use xchain_owned_create_account_claim_id::XChainOwnedCreateAccountClaimID;

use crate::_serde::lgr_obj_flags;
use crate::models::{Amount, FlagCollection};

#[derive(Debug, Clone, Serialize, Deserialize, Display, PartialEq, Eq)]
pub enum LedgerEntryType {
    AccountRoot = 0x0061,
    Amendments = 0x0066,
    AMM = 0x0079,
    Bridge = 0x0069,
    Check = 0x0043,
    DepositPreauth = 0x0070,
    DirectoryNode = 0x0064,
    Escrow = 0x0075,
    FeeSettings = 0x0073,
    LedgerHashes = 0x0068,
    NegativeUNL = 0x004E,
    NFTokenOffer = 0x0037,
    NFTokenPage = 0x0050,
    Offer = 0x006F,
    PayChannel = 0x0078,
    RippleState = 0x0072,
    SignerList = 0x0053,
    Ticket = 0x0054,
    XChainOwnedClaimID = 0x0071,
    XChainOwnedCreateAccountClaimID = 0x0074,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LedgerEntry<'a> {
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
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct XChainClaimProofSig<'a> {
    pub amount: Amount<'a>,
    pub attestation_reward_account: Cow<'a, str>,
    pub attestation_signer_account: Cow<'a, str>,
    pub destination: Cow<'a, str>,
    pub public_key: Cow<'a, str>,
    pub was_locking_chain_send: u8,
}

/// The base fields for all ledger object models.
///
/// See Ledger Object Common Fields:
/// `<https://xrpl.org/ledger-entry-common-fields.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
#[serde(rename_all = "PascalCase")]
pub struct CommonFields<'a, F>
where
    F: IntoEnumIterator + Serialize + core::fmt::Debug,
{
    /// A bit-map of boolean flags enabled for this account.
    #[serde(with = "lgr_obj_flags")]
    pub flags: FlagCollection<F>,
    /// The type of the ledger object.
    pub ledger_entry_type: LedgerEntryType,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Option<Cow<'a, str>>,
    /// The object ID in transaction metadata of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    pub ledger_index: Option<Cow<'a, str>>,
}

impl<'a, T> LedgerObject<T> for CommonFields<'a, T>
where
    T: IntoEnumIterator + Serialize + PartialEq + core::fmt::Debug,
{
    fn has_flag(&self, flag: &T) -> bool {
        self.flags.0.contains(flag)
    }

    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.ledger_entry_type.clone()
    }
}

/// Standard functions for ledger objects.
pub trait LedgerObject<T>
where
    T: IntoEnumIterator + Serialize,
{
    fn has_flag(&self, flag: &T) -> bool {
        let _txn_flag = flag;
        false
    }

    fn get_ledger_entry_type(&self) -> LedgerEntryType;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::amount::XRPAmount;
    use crate::models::currency::XRP;
    use crate::models::NoFlags;

    #[test]
    fn test_common_fields_new() {
        let fields: CommonFields<'_, NoFlags> = CommonFields::new(
            FlagCollection::default(),
            LedgerEntryType::Bridge,
            Some("AABBCC".into()),
            Some("DDEEFF".into()),
        );
        assert_eq!(fields.get_ledger_entry_type(), LedgerEntryType::Bridge);
        // The default impl on `LedgerObject` should return `false` here - no
        // flag set is true. Pull a flag from the enum to satisfy IntoEnumIter.
        // Because NoFlags has no variants, simply check the trait wiring works.
        assert_eq!(fields.flags.0.len(), 0);
    }

    #[test]
    fn test_ledger_entry_type_display() {
        // The Display impl is auto-derived from `strum_macros::Display`.
        assert_eq!(LedgerEntryType::AccountRoot.to_string(), "AccountRoot");
        assert_eq!(LedgerEntryType::Bridge.to_string(), "Bridge");
        assert_eq!(
            LedgerEntryType::XChainOwnedClaimID.to_string(),
            "XChainOwnedClaimID"
        );
    }

    #[test]
    fn test_ledger_entry_enum_serde_round_trip() {
        let bridge = Bridge::new(
            Some("AABBCC".into()),
            Some("DDEEFF".into()),
            "rPV4mZjsXfH2HvUSPLNmqz1J8d3Lpv7tpe".into(),
            XRPAmount::from("100"),
            0,
            0,
            crate::models::XChainBridge {
                locking_chain_door: "rMAXACCrp3Y8PpswXcg3bKggHX76V3F8M4".into(),
                locking_chain_issue: XRP::new().into(),
                issuing_chain_door: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                issuing_chain_issue: XRP::new().into(),
            },
            "1".into(),
            None,
        );
        let entry = LedgerEntry::Bridge(bridge);
        let serialized = serde_json::to_string(&entry).unwrap();
        // The enum is untagged-ish; default behaviour is externally tagged on
        // variant name. Either way, round-trip must work for the same JSON.
        let deserialized: LedgerEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entry, deserialized);
    }
}
