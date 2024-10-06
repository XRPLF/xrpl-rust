use crate::models::ledger::objects::LedgerEntryType;
use crate::models::FlagCollection;
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

use super::{CommonFields, LedgerObject};

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
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, SignerListFlag>,
    // The custom fields for the SignerList model.
    //
    // See SignerList fields:
    // `<https://xrpl.org/signerlist.html#signerlist-fields>`
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

impl<'a> Model for SignerList<'a> {}

impl<'a> LedgerObject<SignerListFlag> for SignerList<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> SignerList<'a> {
    pub fn new(
        flags: FlagCollection<SignerListFlag>,
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        owner_node: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
        signer_entries: Vec<SignerEntry>,
        signer_list_id: u32,
        signer_quorum: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags,
                ledger_entry_type: LedgerEntryType::SignerList,
                index,
                ledger_index,
            },
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
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_serde() {
        let signer_list = SignerList::new(
            vec![].into(),
            Some(Cow::from(
                "A9C28A28B85CD533217F5C0A0C7767666B093FA58A0F2D80026FCC4CD932DDC7",
            )),
            None,
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
        let serialized = serde_json::to_string(&signer_list).unwrap();

        let deserialized: SignerList = serde_json::from_str(&serialized).unwrap();

        assert_eq!(signer_list, deserialized);
    }
}
