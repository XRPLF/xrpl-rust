use crate::models::ledger::LedgerEntryType;
use crate::models::Model;
use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;

/// The `FeeSettings` object type contains the current base transaction cost and reserve amounts
/// as determined by fee voting. Each ledger version contains at most one `FeeSettings` object.
///
/// `<https://xrpl.org/feesettings.html#feesettings>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct FeeSettings<'a> {
    /// The value `0x0073`, mapped to the string `FeeSettings`, indicates that this object contains
    /// the ledger's fee settings.
    pub ledger_entry_type: LedgerEntryType,
    /// A bit-map of boolean flags enabled for this object. Currently, the protocol defines no flags
    /// for `FeeSettings` objects. The value is always `0`.
    pub flags: u32,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// The transaction cost of the "reference transaction" in drops of XRP as hexadecimal.
    pub base_fee: Cow<'a, str>,
    /// The BaseFee translated into "fee units".
    pub reference_fee_units: u32,
    /// The base reserve for an account in the XRP Ledger, as drops of XRP.
    pub reserve_base: u32,
    /// The incremental owner reserve for owning objects, as drops of XRP.
    pub reserve_increment: u32,
}

impl<'a> Default for FeeSettings<'a> {
    fn default() -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::FeeSettings,
            flags: Default::default(),
            index: Default::default(),
            base_fee: Default::default(),
            reference_fee_units: Default::default(),
            reserve_base: Default::default(),
            reserve_increment: Default::default(),
        }
    }
}

impl<'a> Model for FeeSettings<'a> {}

impl<'a> FeeSettings<'a> {
    pub fn new(
        index: Cow<'a, str>,
        base_fee: Cow<'a, str>,
        reference_fee_units: u32,
        reserve_base: u32,
        reserve_increment: u32,
    ) -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::FeeSettings,
            flags: 0,
            index,
            base_fee,
            reference_fee_units,
            reserve_base,
            reserve_increment,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    #[test]
    fn test_serialize() {
        let fee_settings = FeeSettings::new(
            Cow::from("4BC50C9B0D8515D3EAAE1E74B29A95804346C491EE1A95BF25E4AAB854A6A651"),
            Cow::from("000000000000000A"),
            10,
            20000000,
            5000000,
        );
        let fee_settings_json = serde_json::to_string(&fee_settings).unwrap();
        let actual = fee_settings_json.as_str();
        let expected = r#"{"LedgerEntryType":"FeeSettings","Flags":0,"index":"4BC50C9B0D8515D3EAAE1E74B29A95804346C491EE1A95BF25E4AAB854A6A651","BaseFee":"000000000000000A","ReferenceFeeUnits":10,"ReserveBase":20000000,"ReserveIncrement":5000000}"#;

        assert_eq!(expected, actual)
    }

    // TODO: test_deserialize
}
