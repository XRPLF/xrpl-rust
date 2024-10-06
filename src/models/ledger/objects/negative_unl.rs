use crate::models::FlagCollection;
use crate::models::Model;
use crate::models::{ledger::objects::LedgerEntryType, NoFlags};
use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use derive_new::new;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::serde_with_tag;
use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

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
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the NegativeUNL model.
    //
    // See NegativeUNL fields:
    // `<https://xrpl.org/negativeunl.html#negativeunl-fields>`
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

impl<'a> Model for NegativeUNL<'a> {}

impl<'a> LedgerObject<NoFlags> for NegativeUNL<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> NegativeUNL<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        disabled_validators: Option<Vec<DisabledValidator>>,
        validator_to_disable: Option<Cow<'a, str>>,
        validator_to_re_enable: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::NegativeUNL,
                index,
                ledger_index,
            },
            disabled_validators,
            validator_to_disable,
            validator_to_re_enable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_serde() {
        let negative_unl = NegativeUNL::new(
            Some(Cow::from(
                "2E8A59AA9D3B5B186B0B9E0F62E6C02587CA74A4D778938E957B6357D364B244",
            )),
            None,
            Some(vec![DisabledValidator::new(
                1609728,
                "ED6629D456285AE3613B285F65BBFF168D695BA3921F309949AFCD2CA7AFEC16FE".to_string(),
            )]),
            None,
            None,
        );
        let serialized = serde_json::to_string(&negative_unl).unwrap();

        let deserialized: NegativeUNL = serde_json::from_str(&serialized).unwrap();

        assert_eq!(negative_unl, deserialized);
    }
}
