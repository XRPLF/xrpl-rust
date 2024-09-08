use crate::models::FlagCollection;
use crate::models::Model;
use crate::models::{ledger::objects::LedgerEntryType, NoFlags};
use alloc::borrow::Cow;
use alloc::vec::Vec;
use derive_new::new;
use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new, Default)]
#[serde(rename_all = "PascalCase")]
pub struct NFToken<'a> {
    #[serde(rename = "NFTokenID")]
    nftoken_id: Cow<'a, str>,
    #[serde(rename = "URI")]
    uri: Cow<'a, str>,
}

/// The `NFTokenPage` object represents a collection of `NFToken` objects owned by the same account.
///
/// `<https://xrpl.org/nftokenpage.html#nftokenpage>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenPage<'a> {
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the NFTokenPage model.
    //
    // See NFTokenPage fields:
    // `<https://xrpl.org/nftokenpage.html#nftokenpage-fields>`
    /// The locator of the next page, if any. Details about this field and how it should be
    /// used are outlined below.
    pub next_page_min: Option<Cow<'a, str>>,
    /// The collection of NFToken objects contained in this `NFTokenPage` object.
    /// This specification places an upper bound of 32 `NFToken` objects per page.
    /// Objects are sorted from low to high with the `NFTokenID` used as the sorting parameter.
    #[serde(rename = "NFTokens")]
    pub nftokens: Vec<NFToken<'a>>,
    /// The locator of the previous page, if any. Details about this field and how it should
    /// be used are outlined below.
    pub previous_page_min: Option<Cow<'a, str>>,
    /// Identifies the transaction ID of the transaction that most recently modified
    /// this `NFTokenPage` object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Option<Cow<'a, str>>,
    /// The sequence of the ledger that contains the transaction that most recently
    /// modified this `NFTokenPage` object.
    pub previous_txn_lgr_seq: Option<u32>,
}

impl<'a> Model for NFTokenPage<'a> {}

impl<'a> LedgerObject<NoFlags> for NFTokenPage<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> NFTokenPage<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        next_page_min: Option<Cow<'a, str>>,
        nftokens: Vec<NFToken<'a>>,
        previous_page_min: Option<Cow<'a, str>>,
        previous_txn_id: Option<Cow<'a, str>>,
        previous_txn_lgr_seq: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::NFTokenPage,
                index,
                ledger_index,
            },
            nftokens,
            next_page_min,
            previous_page_min,
            previous_txn_id,
            previous_txn_lgr_seq,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_serde() {
        let nftoken_page = NFTokenPage::new(
            Some(Cow::from("ForTest")),
            None,
            Some(Cow::from("598EDFD7CF73460FB8C695d6a9397E9073781BA3B78198904F659AAA252A")),
            vec![NFToken::new(
                Cow::from("000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65"),
                Cow::from("697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469")
            )],
            Some(Cow::from("598EDFD7CF73460FB8C695d6a9397E907378C8A841F7204C793DCBEF5406")),
            Some(Cow::from("95C8761B22894E328646F7A70035E9DFBECC90EDD83E43B7B973F626D21A0822")),
            Some(42891441),
        );
        let serialized = serde_json::to_string(&nftoken_page).unwrap();

        let deserialized: NFTokenPage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(nftoken_page, deserialized);
    }
}
