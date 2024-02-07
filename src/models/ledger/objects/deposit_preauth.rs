use crate::models::FlagCollection;
use crate::models::Model;
use crate::models::{ledger::LedgerEntryType, NoFlags};
use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

/// A `DepositPreauth` object tracks a preauthorization from one account to another.
/// `DepositPreauth` transactions create these objects.
///
/// `<https://xrpl.org/depositpreauth-object.html#depositpreauth>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DepositPreauth<'a> {
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the DepositPreauth model.
    //
    // See DepositPreauth fields:
    // `<https://xrpl.org/depositpreauth-object.html#depositpreauth-fields>`
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

impl<'a> Model for DepositPreauth<'a> {}

impl<'a> LedgerObject<NoFlags> for DepositPreauth<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> DepositPreauth<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        authorize: Cow<'a, str>,
        owner_node: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::DepositPreauth,
                index,
                ledger_index,
            },
            account,
            authorize,
            owner_node,
            previous_txn_id,
            previous_txn_lgr_seq,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let deposit_preauth = DepositPreauth::new(
            Some(Cow::from(
                "4A255038CC3ADCC1A9C91509279B59908251728D0DAADB248FFE297D0F7E068C",
            )),
            None,
            Cow::from("rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8"),
            Cow::from("rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de"),
            Cow::from("0000000000000000"),
            Cow::from("3E8964D5A86B3CD6B9ECB33310D4E073D64C865A5B866200AD2B7E29F8326702"),
            7,
        );
        let serialized = serde_json::to_string(&deposit_preauth).unwrap();

        let deserialized: DepositPreauth = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deposit_preauth, deserialized);
    }
}
