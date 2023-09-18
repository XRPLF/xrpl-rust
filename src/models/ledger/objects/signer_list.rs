use crate::_serde::lgr_obj_flags;
use crate::models::ledger::LedgerEntryType;
use crate::models::Model;
use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use derive_new::new;
use serde::{ser::SerializeMap, Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::serde_with_tag;
use serde_with::skip_serializing_none;

#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum SignerListFlag {
    /// If this flag is enabled, this SignerList counts as one item for purposes of the owner reserve.
    LsfOneOwnerCount = 0x00010000,
}

serde_with_tag! {
    /// Each member of the SignerEntries field is an object that describes that signer in the list.
    ///
    /// `<https://xrpl.org/signerlist.html#signer-entry-object>`
    #[derive(Debug, PartialEq, Eq, Clone, new, Default)]
    pub struct SignerEntry {
        /// An XRP Ledger address whose signature contributes to the multi-signature.
        pub account: String,
        /// The weight of a signature from this signer.
        pub signer_weight: u16,
        /// Arbitrary hexadecimal data. This can be used to identify the signer or for
        /// other, related purposes.
        pub wallet_locator: Option<String>,
    }
}

/// The SignerList object type represents a list of parties that, as a group, are authorized
/// to sign a transaction in place of an individual account.
///
/// `<https://xrpl.org/signerlist.html#signerlist>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SignerList<'a> {
    /// The value 0x0053, mapped to the string SignerList, indicates that this object is a
    /// SignerList object.
    ledger_entry_type: LedgerEntryType,
    /// A bit-map of Boolean flags enabled for this signer list.
    #[serde(with = "lgr_obj_flags")]
    flags: Vec<SignerListFlag>,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// A hint indicating which page of the owner directory links to this object, in case
    /// the directory consists of multiple pages.
    pub owner_node: Cow<'a, str>,
    /// The identifying hash of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most recently
    /// modified this object.
    pub previous_txn_lgr_seq: u32,
    /// An array of Signer Entry objects representing the parties who are part of this
    /// signer list.
    pub signer_entries: Vec<SignerEntry>,
    /// An ID for this signer list. Currently always set to 0.
    #[serde(rename = "SignerListID")]
    pub signer_list_id: u32,
    /// A target number for signer weights. To produce a valid signature for the owner of
    /// this SignerList, the signers must provide valid signatures whose weights sum to this
    /// value or more.
    pub signer_quorum: u32,
}

impl<'a> Default for SignerList<'a> {
    fn default() -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::SignerList,
            flags: Default::default(),
            index: Default::default(),
            owner_node: Default::default(),
            previous_txn_id: Default::default(),
            previous_txn_lgr_seq: Default::default(),
            signer_entries: Default::default(),
            signer_list_id: Default::default(),
            signer_quorum: Default::default(),
        }
    }
}

impl<'a> Model for SignerList<'a> {}

impl<'a> SignerList<'a> {
    pub fn new(
        flags: Vec<SignerListFlag>,
        index: Cow<'a, str>,
        owner_node: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
        signer_entries: Vec<SignerEntry>,
        signer_list_id: u32,
        signer_quorum: u32,
    ) -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::SignerList,
            flags,
            index,
            owner_node,
            previous_txn_id,
            previous_txn_lgr_seq,
            signer_entries,
            signer_list_id,
            signer_quorum,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_serialize() {
        let signer_list = SignerList::new(
            vec![],
            Cow::from("A9C28A28B85CD533217F5C0A0C7767666B093FA58A0F2D80026FCC4CD932DDC7"),
            Cow::from("0000000000000000"),
            Cow::from("5904C0DC72C58A83AEFED2FFC5386356AA83FCA6A88C89D00646E51E687CDBE4"),
            16061435,
            vec![
                SignerEntry::new("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".to_string(), 2, None),
                SignerEntry::new("raKEEVSGnKSD9Zyvxu4z6Pqpm4ABH8FS6n".to_string(), 1, None),
                SignerEntry::new("rUpy3eEg8rqjqfUoLeBnZkscbKbFsKXC3v".to_string(), 1, None),
            ],
            0,
            3,
        );
        let signer_list_json = serde_json::to_string(&signer_list).unwrap();
        let actual = signer_list_json.as_str();
        let expected = r#"{"LedgerEntryType":"SignerList","Flags":0,"index":"A9C28A28B85CD533217F5C0A0C7767666B093FA58A0F2D80026FCC4CD932DDC7","OwnerNode":"0000000000000000","PreviousTxnID":"5904C0DC72C58A83AEFED2FFC5386356AA83FCA6A88C89D00646E51E687CDBE4","PreviousTxnLgrSeq":16061435,"SignerEntries":[{"SignerEntry":{"Account":"rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW","SignerWeight":2,"WalletLocator":null}},{"SignerEntry":{"Account":"raKEEVSGnKSD9Zyvxu4z6Pqpm4ABH8FS6n","SignerWeight":1,"WalletLocator":null}},{"SignerEntry":{"Account":"rUpy3eEg8rqjqfUoLeBnZkscbKbFsKXC3v","SignerWeight":1,"WalletLocator":null}}],"SignerListID":0,"SignerQuorum":3}"#;

        assert_eq!(expected, actual);
    }

    // TODO: test_deserialize
}
