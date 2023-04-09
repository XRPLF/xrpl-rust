use crate::models::ledger::LedgerEntryType;
use crate::models::Model;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use derive_new::new;
use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;

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
    /// The value `0x0050`, mapped to the string `NFTokenPage`, indicates that this is a page
    /// containing `NFToken` objects.
    pub ledger_entry_type: LedgerEntryType,
    /// A bit-map of boolean flags. No flags are defined for the NegativeUNL object type, so this
    /// value is always 0.
    pub flags: u32,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// The collection of NFToken objects contained in this `NFTokenPage` object.
    /// This specification places an upper bound of 32 `NFToken` objects per page.
    /// Objects are sorted from low to high with the `NFTokenID` used as the sorting parameter.
    #[serde(rename = "NFTokens")]
    pub nftokens: Vec<NFToken<'a>>,
    /// The locator of the next page, if any. Details about this field and how it should be
    /// used are outlined below.
    pub next_page_min: Option<Cow<'a, str>>,
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

impl<'a> Default for NFTokenPage<'a> {
    fn default() -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::NFTokenPage,
            flags: Default::default(),
            index: Default::default(),
            nftokens: Default::default(),
            next_page_min: Default::default(),
            previous_page_min: Default::default(),
            previous_txn_id: Default::default(),
            previous_txn_lgr_seq: Default::default(),
        }
    }
}

impl<'a> Model for NFTokenPage<'a> {}

impl<'a> NFTokenPage<'a> {
    pub fn new(
        index: Cow<'a, str>,
        nftokens: Vec<NFToken<'a>>,
        next_page_min: Option<Cow<'a, str>>,
        previous_page_min: Option<Cow<'a, str>>,
        previous_txn_id: Option<Cow<'a, str>>,
        previous_txn_lgr_seq: Option<u32>,
    ) -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::NFTokenPage,
            flags: 0,
            index,
            nftokens,
            next_page_min,
            previous_page_min,
            previous_txn_id,
            previous_txn_lgr_seq,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_serialize() {
        let nftoken_page = NFTokenPage::new(
            Cow::from("ForTest"),
            vec![NFToken::new(
                Cow::from("000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65"),
                Cow::from("697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469")
            )],
            Some(Cow::from("598EDFD7CF73460FB8C695d6a9397E9073781BA3B78198904F659AAA252A")),
            Some(Cow::from("598EDFD7CF73460FB8C695d6a9397E907378C8A841F7204C793DCBEF5406")),
            Some(Cow::from("95C8761B22894E328646F7A70035E9DFBECC90EDD83E43B7B973F626D21A0822")),
            Some(42891441),
        );
        let nftoken_page_json = serde_json::to_string(&nftoken_page).unwrap();
        let actual = nftoken_page_json.as_str();
        let expected = r#"{"LedgerEntryType":"NFTokenPage","Flags":0,"index":"ForTest","NFTokens":[{"NFTokenID":"000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65","URI":"697066733A2F2F62616679626569676479727A74357366703775646D37687537367568377932366E6634646675796C71616266336F636C67747179353566627A6469"}],"NextPageMin":"598EDFD7CF73460FB8C695d6a9397E9073781BA3B78198904F659AAA252A","PreviousPageMin":"598EDFD7CF73460FB8C695d6a9397E907378C8A841F7204C793DCBEF5406","PreviousTxnID":"95C8761B22894E328646F7A70035E9DFBECC90EDD83E43B7B973F626D21A0822","PreviousTxnLgrSeq":42891441}"#;

        assert_eq!(expected, actual);
    }

    // TODO: test_deserialize
}
