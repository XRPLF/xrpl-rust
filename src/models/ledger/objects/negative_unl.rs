use crate::models::ledger::LedgerEntryType;
use crate::models::Model;
use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use derive_new::new;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::serde_with_tag;
use serde_with::skip_serializing_none;

serde_with_tag! {
    /// Each `DisabledValidator` object represents one disabled validator.
    #[derive(Debug, PartialEq, Eq, Clone, new, Default)]
    pub struct DisabledValidator {
        /// The ledger index when the validator was added to the Negative UNL.
        pub first_ledger_sequence: u32,
        /// The master public key of the validator, in hexadecimal.
        pub public_key: String,
    }
}

/// The NegativeUNL object type contains the current status of the Negative UNL, a list of trusted
/// validators currently believed to be offline.
///
/// `<https://xrpl.org/negativeunl.html#negativeunl>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NegativeUNL<'a> {
    /// The value `0x004E`, mapped to the string `NegativeUNL`, indicates that this object is the
    /// Negative UNL.
    pub ledger_entry_type: LedgerEntryType,
    /// A bit-map of boolean flags. No flags are defined for the NegativeUNL object type, so this
    /// value is always 0.
    pub flags: u32,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// A list of `DisabledValidator` objects (see below), each representing a trusted validator
    /// that is currently disabled.
    pub disabled_validators: Option<Vec<DisabledValidator>>,
    /// The public key of a trusted validator that is scheduled to be disabled in the
    /// next flag ledger.
    pub validator_to_disable: Option<Cow<'a, str>>,
    /// The public key of a trusted validator in the Negative UNL that is scheduled to be
    /// re-enabled in the next flag ledger.
    pub validator_to_re_enable: Option<Cow<'a, str>>,
}

impl<'a> Default for NegativeUNL<'a> {
    fn default() -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::NegativeUNL,
            flags: Default::default(),
            index: Default::default(),
            disabled_validators: Default::default(),
            validator_to_disable: Default::default(),
            validator_to_re_enable: Default::default(),
        }
    }
}

impl<'a> Model for NegativeUNL<'a> {}

impl<'a> NegativeUNL<'a> {
    pub fn new(
        index: Cow<'a, str>,
        disabled_validators: Option<Vec<DisabledValidator>>,
        validator_to_disable: Option<Cow<'a, str>>,
        validator_to_re_enable: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::NegativeUNL,
            flags: 0,
            index,
            disabled_validators,
            validator_to_disable,
            validator_to_re_enable,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_serialize() {
        let negative_unl = NegativeUNL::new(
            Cow::from("2E8A59AA9D3B5B186B0B9E0F62E6C02587CA74A4D778938E957B6357D364B244"),
            Some(vec![DisabledValidator::new(
                1609728,
                "ED6629D456285AE3613B285F65BBFF168D695BA3921F309949AFCD2CA7AFEC16FE".to_string(),
            )]),
            None,
            None,
        );
        let negative_unl_json = serde_json::to_string(&negative_unl).unwrap();
        let actual = negative_unl_json.as_str();
        let expected = r#"{"LedgerEntryType":"NegativeUNL","Flags":0,"index":"2E8A59AA9D3B5B186B0B9E0F62E6C02587CA74A4D778938E957B6357D364B244","DisabledValidators":[{"DisabledValidator":{"FirstLedgerSequence":1609728,"PublicKey":"ED6629D456285AE3613B285F65BBFF168D695BA3921F309949AFCD2CA7AFEC16FE"}}]}"#;

        assert_eq!(expected, actual);
    }

    // TODO: test_deserialize
}
