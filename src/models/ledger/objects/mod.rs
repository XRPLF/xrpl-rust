pub mod account_root;
pub mod amendments;
pub mod amm;
pub mod bridge;
pub mod check;
pub mod credential;
pub mod deposit_preauth;
pub mod did;
pub mod directory_node;
pub mod escrow;
pub mod fee_settings;
pub mod ledger_hashes;
pub mod mptoken;
pub mod mptoken_issuance;
pub mod negative_unl;
pub mod nftoken_offer;
pub mod nftoken_page;
pub mod offer;
pub mod oracle;
pub mod pay_channel;
pub mod permissioned_domain;
pub mod ripple_state;
pub mod signer_list;
pub mod ticket;
pub mod vault;
pub mod xchain_owned_claim_id;
pub mod xchain_owned_create_account_claim_id;

use account_root::AccountRoot;
use amendments::Amendments;
use amm::AMM;
use bridge::Bridge;
use check::Check;
use credential::Credential;
use deposit_preauth::DepositPreauth;
use derive_new::new;
use did::DID;
use directory_node::DirectoryNode;
use escrow::Escrow;
use fee_settings::FeeSettings;
use ledger_hashes::LedgerHashes;
use mptoken::MPToken;
use mptoken_issuance::MPTokenIssuance;
use negative_unl::NegativeUNL;
use nftoken_offer::NFTokenOffer;
use nftoken_page::NFTokenPage;
use offer::Offer;
use oracle::Oracle;
use pay_channel::PayChannel;
use permissioned_domain::PermissionedDomain;
use ripple_state::RippleState;
use signer_list::SignerList;
use strum::IntoEnumIterator;

use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use strum_macros::Display;
use ticket::Ticket;
use vault::Vault;
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
    DID = 0x0049,
    Credential = 0x0081,
    DepositPreauth = 0x0070,
    DirectoryNode = 0x0064,
    Escrow = 0x0075,
    FeeSettings = 0x0073,
    LedgerHashes = 0x0068,
    MPToken = 0x007F,
    MPTokenIssuance = 0x007E,
    NegativeUNL = 0x004E,
    NFTokenOffer = 0x0037,
    NFTokenPage = 0x0050,
    Offer = 0x006F,
    Oracle = 0x0080,
    PayChannel = 0x0078,
    PermissionedDomain = 0x0082,
    RippleState = 0x0072,
    SignerList = 0x0053,
    Ticket = 0x0054,
    Vault = 0x0084,
    XChainOwnedClaimID = 0x0071,
    XChainOwnedCreateAccountClaimID = 0x0074,
}

/// A ledger object, as returned inline by `ledger` (full/accounts) and `ledger_entry`.
///
/// The XRPL wire format for these is a flat JSON object carrying its own
/// `LedgerEntryType` discriminator field (e.g. `{"LedgerEntryType": "AccountRoot",
/// "Account": ..., ...}`), not serde's default externally-tagged representation
/// (`{"AccountRoot": {...}}`). `Serialize`/`Deserialize` are implemented by hand
/// below to match that wire format: deserialization reads `LedgerEntryType` to pick
/// the variant, and serialization flattens straight through to the inner type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerEntry<'a> {
    AccountRoot(AccountRoot<'a>),
    Amendments(Amendments<'a>),
    AMM(AMM<'a>),
    Bridge(Bridge<'a>),
    Check(Check<'a>),
    DID(DID<'a>),
    Credential(Credential<'a>),
    DepositPreauth(DepositPreauth<'a>),
    DirectoryNode(DirectoryNode<'a>),
    Escrow(Escrow<'a>),
    FeeSettings(FeeSettings<'a>),
    LedgerHashes(LedgerHashes<'a>),
    MPToken(MPToken<'a>),
    MPTokenIssuance(MPTokenIssuance<'a>),
    NegativeUNL(NegativeUNL<'a>),
    NFTokenOffer(NFTokenOffer<'a>),
    NFTokenPage(NFTokenPage<'a>),
    Offer(Offer<'a>),
    Oracle(Oracle<'a>),
    PayChannel(PayChannel<'a>),
    PermissionedDomain(PermissionedDomain<'a>),
    RippleState(RippleState<'a>),
    SignerList(SignerList<'a>),
    Ticket(Ticket<'a>),
    Vault(Vault<'a>),
    XChainOwnedClaimID(XChainOwnedClaimID<'a>),
    XChainOwnedCreateAccountClaimID(XChainOwnedCreateAccountClaimID<'a>),
}

const LEDGER_ENTRY_TYPE_VARIANTS: &[&str] = &[
    "AccountRoot",
    "Amendments",
    "AMM",
    "Bridge",
    "Check",
    "DID",
    "Credential",
    "DepositPreauth",
    "DirectoryNode",
    "Escrow",
    "FeeSettings",
    "LedgerHashes",
    "MPToken",
    "MPTokenIssuance",
    "NegativeUNL",
    "NFTokenOffer",
    "NFTokenPage",
    "Offer",
    "Oracle",
    "PayChannel",
    "PermissionedDomain",
    "RippleState",
    "SignerList",
    "Ticket",
    "Vault",
    "XChainOwnedClaimID",
    "XChainOwnedCreateAccountClaimID",
];

impl<'a> Serialize for LedgerEntry<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            LedgerEntry::AccountRoot(inner) => inner.serialize(serializer),
            LedgerEntry::Amendments(inner) => inner.serialize(serializer),
            LedgerEntry::AMM(inner) => inner.serialize(serializer),
            LedgerEntry::Bridge(inner) => inner.serialize(serializer),
            LedgerEntry::Check(inner) => inner.serialize(serializer),
            LedgerEntry::DID(inner) => inner.serialize(serializer),
            LedgerEntry::Credential(inner) => inner.serialize(serializer),
            LedgerEntry::DepositPreauth(inner) => inner.serialize(serializer),
            LedgerEntry::DirectoryNode(inner) => inner.serialize(serializer),
            LedgerEntry::Escrow(inner) => inner.serialize(serializer),
            LedgerEntry::FeeSettings(inner) => inner.serialize(serializer),
            LedgerEntry::LedgerHashes(inner) => inner.serialize(serializer),
            LedgerEntry::MPToken(inner) => inner.serialize(serializer),
            LedgerEntry::MPTokenIssuance(inner) => inner.serialize(serializer),
            LedgerEntry::NegativeUNL(inner) => inner.serialize(serializer),
            LedgerEntry::NFTokenOffer(inner) => inner.serialize(serializer),
            LedgerEntry::NFTokenPage(inner) => inner.serialize(serializer),
            LedgerEntry::Offer(inner) => inner.serialize(serializer),
            LedgerEntry::Oracle(inner) => inner.serialize(serializer),
            LedgerEntry::PayChannel(inner) => inner.serialize(serializer),
            LedgerEntry::PermissionedDomain(inner) => inner.serialize(serializer),
            LedgerEntry::RippleState(inner) => inner.serialize(serializer),
            LedgerEntry::SignerList(inner) => inner.serialize(serializer),
            LedgerEntry::Ticket(inner) => inner.serialize(serializer),
            LedgerEntry::Vault(inner) => inner.serialize(serializer),
            LedgerEntry::XChainOwnedClaimID(inner) => inner.serialize(serializer),
            LedgerEntry::XChainOwnedCreateAccountClaimID(inner) => inner.serialize(serializer),
        }
    }
}

impl<'de, 'a> Deserialize<'de> for LedgerEntry<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let ledger_entry_type = value
            .get("LedgerEntryType")
            .and_then(Value::as_str)
            .ok_or_else(|| serde::de::Error::missing_field("LedgerEntryType"))?
            .to_owned();

        let result: serde_json::Result<Self> = match ledger_entry_type.as_str() {
            "AccountRoot" => serde_json::from_value(value).map(LedgerEntry::AccountRoot),
            "Amendments" => serde_json::from_value(value).map(LedgerEntry::Amendments),
            "AMM" => serde_json::from_value(value).map(LedgerEntry::AMM),
            "Bridge" => serde_json::from_value(value).map(LedgerEntry::Bridge),
            "Check" => serde_json::from_value(value).map(LedgerEntry::Check),
            "DID" => serde_json::from_value(value).map(LedgerEntry::DID),
            "Credential" => serde_json::from_value(value).map(LedgerEntry::Credential),
            "DepositPreauth" => serde_json::from_value(value).map(LedgerEntry::DepositPreauth),
            "DirectoryNode" => serde_json::from_value(value).map(LedgerEntry::DirectoryNode),
            "Escrow" => serde_json::from_value(value).map(LedgerEntry::Escrow),
            "FeeSettings" => serde_json::from_value(value).map(LedgerEntry::FeeSettings),
            "LedgerHashes" => serde_json::from_value(value).map(LedgerEntry::LedgerHashes),
            "MPToken" => serde_json::from_value(value).map(LedgerEntry::MPToken),
            "MPTokenIssuance" => serde_json::from_value(value).map(LedgerEntry::MPTokenIssuance),
            "NegativeUNL" => serde_json::from_value(value).map(LedgerEntry::NegativeUNL),
            "NFTokenOffer" => serde_json::from_value(value).map(LedgerEntry::NFTokenOffer),
            "NFTokenPage" => serde_json::from_value(value).map(LedgerEntry::NFTokenPage),
            "Offer" => serde_json::from_value(value).map(LedgerEntry::Offer),
            "Oracle" => serde_json::from_value(value).map(LedgerEntry::Oracle),
            "PayChannel" => serde_json::from_value(value).map(LedgerEntry::PayChannel),
            "PermissionedDomain" => {
                serde_json::from_value(value).map(LedgerEntry::PermissionedDomain)
            }
            "RippleState" => serde_json::from_value(value).map(LedgerEntry::RippleState),
            "SignerList" => serde_json::from_value(value).map(LedgerEntry::SignerList),
            "Ticket" => serde_json::from_value(value).map(LedgerEntry::Ticket),
            "Vault" => serde_json::from_value(value).map(LedgerEntry::Vault),
            "XChainOwnedClaimID" => {
                serde_json::from_value(value).map(LedgerEntry::XChainOwnedClaimID)
            }
            "XChainOwnedCreateAccountClaimID" => {
                serde_json::from_value(value).map(LedgerEntry::XChainOwnedCreateAccountClaimID)
            }
            other => Err(<serde_json::Error as serde::de::Error>::unknown_variant(
                other,
                LEDGER_ENTRY_TYPE_VARIANTS,
            )),
        };

        result.map_err(serde::de::Error::custom)
    }
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
    use alloc::string::ToString;

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
        // Serialize/Deserialize are hand-written to match the flat XRPL wire
        // format (tagged via an inline `LedgerEntryType` field), not serde's
        // default externally-tagged `{"Bridge": {...}}`.
        assert!(!serialized.starts_with("{\"Bridge\":"));
        let deserialized: LedgerEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn test_ledger_entry_enum_deserializes_flat_wire_format() {
        // Real rippled `ledger_entry`/`ledger` output: a flat object with an
        // inline `LedgerEntryType` discriminator, no externally-tagged wrapper.
        // Covers a non-AccountRoot entry, since that's the shape that used to
        // fail before `node` was widened past a hardcoded AccountRoot struct.
        let json = r#"{
            "Flags": 0,
            "Indexes": ["AAB0000000000000000000000000000000000000000000000000000000000"],
            "IndexNext": 1,
            "IndexPrevious": 0,
            "LedgerEntryType": "DirectoryNode",
            "Owner": "rN7n3473SaZBCG4dFL83w7p1W9cgPLAPkS",
            "RootIndex": "A832B09498B80B1B1BB0E2B31B41B8A3A4B57B8C1C23DAF43A76C6B1B3F7CD60",
            "index": "A832B09498B80B1B1BB0E2B31B41B8A3A4B57B8C1C23DAF43A76C6B1B3F7CD60"
        }"#;

        let entry: LedgerEntry = serde_json::from_str(json).unwrap();
        match entry {
            LedgerEntry::DirectoryNode(directory_node) => {
                assert_eq!(
                    directory_node.owner.as_deref(),
                    Some("rN7n3473SaZBCG4dFL83w7p1W9cgPLAPkS")
                );
                assert_eq!(directory_node.index_next, Some(1));
            }
            other => panic!("expected DirectoryNode, got {other:?}"),
        }
    }

    #[test]
    fn test_ledger_entry_enum_rejects_unknown_ledger_entry_type() {
        let json = r#"{"LedgerEntryType": "NotARealType", "index": "AABB"}"#;
        let err = serde_json::from_str::<LedgerEntry>(json).unwrap_err();
        assert!(err.to_string().contains("NotARealType"));
    }

    #[test]
    fn test_ledger_entry_enum_missing_ledger_entry_type() {
        let json = r#"{"index": "AABB"}"#;
        let err = serde_json::from_str::<LedgerEntry>(json).unwrap_err();
        assert!(err.to_string().contains("LedgerEntryType"));
    }
}
