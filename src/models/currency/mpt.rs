use crate::models::{Model, XRPLModelException, XRPLModelResult};
use alloc::borrow::Cow;
use alloc::string::ToString;
use serde::{Deserialize, Serialize};

pub const MPT_ISSUANCE_ID_HEX_LEN: usize = 48;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
pub struct MPTCurrency<'a> {
    pub mpt_issuance_id: Cow<'a, str>,
}

impl Model for MPTCurrency<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        validate_mpt_issuance_id(&self.mpt_issuance_id)
    }
}

impl<'a> MPTCurrency<'a> {
    pub fn new(mpt_issuance_id: Cow<'a, str>) -> Self {
        Self { mpt_issuance_id }
    }
}

pub(crate) fn validate_mpt_issuance_id(value: &str) -> XRPLModelResult<()> {
    if value.len() != MPT_ISSUANCE_ID_HEX_LEN {
        return Err(XRPLModelException::InvalidValueFormat {
            field: "mpt_issuance_id".to_string(),
            format: "48 hex characters (192-bit MPT issuance ID)".to_string(),
            found: value.to_string(),
        });
    }
    if !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(XRPLModelException::InvalidValueFormat {
            field: "mpt_issuance_id".to_string(),
            format: "ASCII hexadecimal".to_string(),
            found: value.to_string(),
        });
    }
    Ok(())
}
