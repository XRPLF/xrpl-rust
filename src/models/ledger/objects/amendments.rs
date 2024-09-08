use crate::models::ledger::objects::LedgerEntryType;

use crate::models::FlagCollection;
use crate::models::{Model, NoFlags};
use alloc::vec::Vec;
use alloc::{borrow::Cow, string::String};
use derive_new::new;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::serde_with_tag;
use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

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
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the Amendments model.
    //
    // See Amendments fields:
    // `<https://xrpl.org/amendments-object.html#amendments-fields>`
    /// Array of 256-bit amendment IDs for all currently enabled amendments. If omitted, there are
    /// no enabled amendments.
    pub amendments: Option<Vec<Cow<'a, str>>>,
    /// Array of objects describing the status of amendments that have majority support but are not
    /// yet enabled. If omitted, there are no pending amendments with majority support.
    pub majorities: Option<Vec<Majority>>,
}

impl<'a> Model for Amendments<'a> {}

impl<'a> LedgerObject<NoFlags> for Amendments<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> Amendments<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        amendments: Option<Vec<Cow<'a, str>>>,
        majorities: Option<Vec<Majority>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::Amendments,
                index,
                ledger_index,
            },
            amendments,
            majorities,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::ledger::objects::{Amendments, Majority};
    use alloc::borrow::Cow;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_serde() {
        let amendments = Amendments::new(
            Some(Cow::from(
                "7DB0788C020F02780A673DC74757F23823FA3014C1866E72CC4CD8B226CD6EF4",
            )),
            None,
            Some(vec![
                Cow::from("42426C4D4F1009EE67080A9B7965B44656D7714D104A72F9B4369F97ABF044EE"),
                Cow::from("4C97EBA926031A7CF7D7B36FDE3ED66DDA5421192D63DE53FFB46E43B9DC8373"),
                Cow::from("6781F8368C4771B83E8B821D88F580202BCB4228075297B19E4FDC5233F1EFDC"),
                Cow::from("740352F2412A9909880C23A559FCECEDA3BE2126FED62FC7660D628A06927F11"),
            ]),
            Some(vec![Majority {
                amendment: "1562511F573A19AE9BD103B5D6B9E01B3B46805AEC5D3C4805C902B514399146"
                    .to_string(),
                close_time: 535589001,
            }]),
        );
        let serialized = serde_json::to_string(&amendments).unwrap();

        let deserialized: Amendments = serde_json::from_str(&serialized).unwrap();

        assert_eq!(amendments, deserialized);
    }

    // TODO: test_deserialize
}
