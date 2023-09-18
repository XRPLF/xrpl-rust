use crate::models::ledger::LedgerEntryType;
use crate::models::Model;
use alloc::{borrow::Cow, string::String};
use alloc::vec::Vec;
use derive_new::new;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::serde_with_tag;
use serde_with::skip_serializing_none;

serde_with_tag! {
    /// `<https://xrpl.org/amendments-object.html#amendments-fields>`
    #[derive(Debug, PartialEq, Eq, Clone, new, Default)]
    pub struct Majority {
        /// The Amendment ID of the pending amendment.
        pub amendment: String,
        /// The `close_time` field of the ledger version where this amendment most recently gained a
        /// majority.
        pub close_time: u32,
    }
}

/// The `Amendments` object type contains a list of `Amendments` that are currently active.
/// Each ledger version contains at most one Amendments`` object.
///
/// `<https://xrpl.org/amendments-object.html#amendments>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Amendments<'a> {
    /// The value `0x0066`, mapped to the string `Amendments`, indicates that this object describes
    /// the status of `amendments` to the XRP Ledger.
    pub ledger_entry_type: LedgerEntryType,
    /// A bit-map of boolean flags enabled for this object. Currently, the protocol defines no flags
    /// for `Amendments` objects. The value is always 0.
    pub flags: u32,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// Array of 256-bit amendment IDs for all currently enabled amendments. If omitted, there are
    /// no enabled amendments.
    pub amendments: Option<Vec<Cow<'a, str>>>,
    /// Array of objects describing the status of amendments that have majority support but are not
    /// yet enabled. If omitted, there are no pending amendments with majority support.
    pub majorities: Option<Vec<Majority>>,
}

impl<'a> Model for Amendments<'a> {}

impl<'a> Default for Amendments<'a> {
    fn default() -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::Amendments,
            flags: Default::default(),
            index: Default::default(),
            amendments: Default::default(),
            majorities: Default::default(),
        }
    }
}

impl<'a> Amendments<'a> {
    pub fn new(
        index: Cow<'a, str>,
        amendments: Option<Vec<Cow<'a, str>>>,
        majorities: Option<Vec<Majority>>,
    ) -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::Amendments,
            flags: 0,
            index,
            amendments,
            majorities,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use crate::models::ledger::{Amendments, Majority};
    use alloc::string::ToString;
    use alloc::borrow::Cow;
    use alloc::vec;

    #[test]
    fn test_serialize() {
        let amendments = Amendments::new(
            Cow::from("7DB0788C020F02780A673DC74757F23823FA3014C1866E72CC4CD8B226CD6EF4"),
            Some(vec![
                Cow::from("42426C4D4F1009EE67080A9B7965B44656D7714D104A72F9B4369F97ABF044EE"),
                Cow::from("4C97EBA926031A7CF7D7B36FDE3ED66DDA5421192D63DE53FFB46E43B9DC8373"),
                Cow::from("6781F8368C4771B83E8B821D88F580202BCB4228075297B19E4FDC5233F1EFDC"),
                Cow::from("740352F2412A9909880C23A559FCECEDA3BE2126FED62FC7660D628A06927F11"),
            ]),
            Some(vec![Majority {
                amendment: "1562511F573A19AE9BD103B5D6B9E01B3B46805AEC5D3C4805C902B514399146".to_string(),
                close_time: 535589001,
            }]),
        );
        let amendments_json = serde_json::to_string(&amendments).unwrap();
        let actual = amendments_json.as_str();
        let expected = r#"{"LedgerEntryType":"Amendments","Flags":0,"index":"7DB0788C020F02780A673DC74757F23823FA3014C1866E72CC4CD8B226CD6EF4","Amendments":["42426C4D4F1009EE67080A9B7965B44656D7714D104A72F9B4369F97ABF044EE","4C97EBA926031A7CF7D7B36FDE3ED66DDA5421192D63DE53FFB46E43B9DC8373","6781F8368C4771B83E8B821D88F580202BCB4228075297B19E4FDC5233F1EFDC","740352F2412A9909880C23A559FCECEDA3BE2126FED62FC7660D628A06927F11"],"Majorities":[{"Majority":{"Amendment":"1562511F573A19AE9BD103B5D6B9E01B3B46805AEC5D3C4805C902B514399146","CloseTime":535589001}}]}"#;

        assert_eq!(expected, actual);
    }

    // TODO: test_deserialize
}
