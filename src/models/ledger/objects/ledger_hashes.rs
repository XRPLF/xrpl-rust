use crate::models::ledger::LedgerEntryType;
use crate::models::Model;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;

/// The `LedgerHashes` object type contains a history of prior ledgers that led up to this
/// ledger version, in the form of their hashes. Objects of this ledger type are modified
/// automatically when closing a ledger. The `LedgerHashes` objects exist to make it possible
/// to look up a previous ledger's hash with only the current ledger version and at most one
/// lookup of a previous ledger version.
///
/// There are two kinds of LedgerHashes object. Both types have the same fields.
/// Each ledger version contains:
/// - Exactly one "recent history" LedgerHashes object
/// - A number of "previous history" `LedgerHashes` objects based on the current ledger index.
/// Specifically, the XRP Ledger adds a new "previous history" object every 65536 ledger versions.
///
/// `<https://xrpl.org/ledgerhashes.html#ledgerhashes>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct LedgerHashes<'a> {
    /// The value `0x0068`, mapped to the string `LedgerHashes`, indicates that this object is a
    /// list of ledger hashes.
    pub ledger_entry_type: LedgerEntryType,
    pub flags: u32,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// **DEPRECATED** Do not use.
    pub first_ledger_sequence: u32,
    /// An array of up to 256 ledger hashes. The contents depend on which sub-type of `LedgerHashes`
    /// object this is.
    pub hashes: Vec<Cow<'a, str>>,
    /// The Ledger Index of the last entry in this object's `Hashes` array.
    pub last_ledger_sequence: u32,
}

impl<'a> Default for LedgerHashes<'a> {
    fn default() -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::LedgerHashes,
            flags: Default::default(),
            index: Default::default(),
            first_ledger_sequence: Default::default(),
            hashes: Default::default(),
            last_ledger_sequence: Default::default(),
        }
    }
}

impl<'a> Model for LedgerHashes<'a> {}

impl<'a> LedgerHashes<'a> {
    pub fn new(
        index: Cow<'a, str>,
        first_ledger_sequence: u32,
        hashes: Vec<Cow<'a, str>>,
        last_ledger_sequence: u32,
    ) -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::LedgerHashes,
            flags: 0,
            index,
            first_ledger_sequence,
            hashes,
            last_ledger_sequence,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_serialize() {
        let ledger_hashes = LedgerHashes::new(
            Cow::from("B4979A36CDC7F3D3D5C31A4EAE2AC7D7209DDA877588B9AFC66799692AB0D66B"),
            2,
            vec![
                Cow::from("D638208ADBD04CBB10DE7B645D3AB4BA31489379411A3A347151702B6401AA78"),
                Cow::from("254D690864E418DDD9BCAC93F41B1F53B1AE693FC5FE667CE40205C322D1BE3B"),
                Cow::from("A2B31D28905E2DEF926362822BC412B12ABF6942B73B72A32D46ED2ABB7ACCFA"),
                Cow::from("AB4014846DF818A4B43D6B1686D0DE0644FE711577C5AB6F0B2A21CCEE280140"),
                Cow::from("3383784E82A8BA45F4DD5EF4EE90A1B2D3B4571317DBAC37B859836ADDE644C1"),
            ],
            33872029,
        );
        let ledger_hashes_json = serde_json::to_string(&ledger_hashes).unwrap();
        let actual = ledger_hashes_json.as_str();
        let expected = r#"{"LedgerEntryType":"LedgerHashes","Flags":0,"index":"B4979A36CDC7F3D3D5C31A4EAE2AC7D7209DDA877588B9AFC66799692AB0D66B","FirstLedgerSequence":2,"Hashes":["D638208ADBD04CBB10DE7B645D3AB4BA31489379411A3A347151702B6401AA78","254D690864E418DDD9BCAC93F41B1F53B1AE693FC5FE667CE40205C322D1BE3B","A2B31D28905E2DEF926362822BC412B12ABF6942B73B72A32D46ED2ABB7ACCFA","AB4014846DF818A4B43D6B1686D0DE0644FE711577C5AB6F0B2A21CCEE280140","3383784E82A8BA45F4DD5EF4EE90A1B2D3B4571317DBAC37B859836ADDE644C1"],"LastLedgerSequence":33872029}"#;

        assert_eq!(expected, actual);
    }

    // TODO: test_deserialize
}
