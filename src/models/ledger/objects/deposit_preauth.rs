use crate::models::ledger::LedgerEntryType;
use crate::models::Model;
use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;

/// A `DepositPreauth` object tracks a preauthorization from one account to another.
/// `DepositPreauth` transactions create these objects.
///
/// `<https://xrpl.org/depositpreauth-object.html#depositpreauth>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DepositPreauth<'a> {
    /// The value `0x0070`, mapped to the string `DepositPreauth`, indicates that this is a
    /// `DepositPreauth` object.
    pub ledger_entry_type: LedgerEntryType,
    /// A bit-map of boolean flags enabled for this object. Currently, the protocol defines no flags
    /// for `DepositPreauth` objects. The value is always 0.
    pub flags: u32,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// The account that granted the preauthorization.
    pub account: Cow<'a, str>,
    /// The account that received the preauthorization.
    pub authorize: Cow<'a, str>,
    /// A hint indicating which page of the sender's owner directory links to this object, in case
    /// the directory consists of multiple pages.
    pub owner_node: Cow<'a, str>,
    /// The identifying hash of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most recently modified this object.
    pub previous_txn_lgr_seq: u32,
}

impl<'a> Default for DepositPreauth<'a> {
    fn default() -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::DepositPreauth,
            flags: Default::default(),
            index: Default::default(),
            account: Default::default(),
            authorize: Default::default(),
            owner_node: Default::default(),
            previous_txn_id: Default::default(),
            previous_txn_lgr_seq: Default::default(),
        }
    }
}

impl<'a> Model for DepositPreauth<'a> {}

impl<'a> DepositPreauth<'a> {
    pub fn new(
        index: Cow<'a, str>,
        account: Cow<'a, str>,
        authorize: Cow<'a, str>,
        owner_node: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
    ) -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::DepositPreauth,
            flags: 0,
            index,
            account,
            authorize,
            owner_node,
            previous_txn_id,
            previous_txn_lgr_seq,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    #[test]
    fn test_serialize() {
        let deposit_preauth = DepositPreauth::new(
            Cow::from("4A255038CC3ADCC1A9C91509279B59908251728D0DAADB248FFE297D0F7E068C"),
            Cow::from("rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8"),
            Cow::from("rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de"),
            Cow::from("0000000000000000"),
            Cow::from("3E8964D5A86B3CD6B9ECB33310D4E073D64C865A5B866200AD2B7E29F8326702"),
            7,
        );
        let deposit_preauth_json = serde_json::to_string(&deposit_preauth).unwrap();
        let actual = deposit_preauth_json.as_str();
        let expected = r#"{"LedgerEntryType":"DepositPreauth","Flags":0,"index":"4A255038CC3ADCC1A9C91509279B59908251728D0DAADB248FFE297D0F7E068C","Account":"rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8","Authorize":"rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de","OwnerNode":"0000000000000000","PreviousTxnID":"3E8964D5A86B3CD6B9ECB33310D4E073D64C865A5B866200AD2B7E29F8326702","PreviousTxnLgrSeq":7}"#;

        assert_eq!(expected, actual);
    }

    // TODO: test_deserialize
}
