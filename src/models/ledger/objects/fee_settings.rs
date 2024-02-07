use crate::models::FlagCollection;
use crate::models::Model;
use crate::models::{ledger::LedgerEntryType, NoFlags};
use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

/// The `FeeSettings` object type contains the current base transaction cost and reserve amounts
/// as determined by fee voting. Each ledger version contains at most one `FeeSettings` object.
///
/// `<https://xrpl.org/feesettings.html#feesettings>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct FeeSettings<'a> {
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the FeeSettings model.
    //
    // See FeeSettings fields:
    // `<https://xrpl.org/feesettings.html#feesettings-fields>`
    /// The transaction cost of the "reference transaction" in drops of XRP as hexadecimal.
    pub base_fee: Cow<'a, str>,
    /// The BaseFee translated into "fee units".
    pub reference_fee_units: u32,
    /// The base reserve for an account in the XRP Ledger, as drops of XRP.
    pub reserve_base: u32,
    /// The incremental owner reserve for owning objects, as drops of XRP.
    pub reserve_increment: u32,
}

impl<'a> Model for FeeSettings<'a> {}

impl<'a> LedgerObject<NoFlags> for FeeSettings<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> FeeSettings<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        base_fee: Cow<'a, str>,
        reference_fee_units: u32,
        reserve_base: u32,
        reserve_increment: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::FeeSettings,
                index,
                ledger_index,
            },
            base_fee,
            reference_fee_units,
            reserve_base,
            reserve_increment,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let fee_settings = FeeSettings::new(
            Some(Cow::from(
                "4BC50C9B0D8515D3EAAE1E74B29A95804346C491EE1A95BF25E4AAB854A6A651",
            )),
            None,
            Cow::from("000000000000000A"),
            10,
            20000000,
            5000000,
        );
        let serialized = serde_json::to_string(&fee_settings).unwrap();

        let deserialized: FeeSettings = serde_json::from_str(&serialized).unwrap();

        assert_eq!(fee_settings, deserialized);
    }
}
