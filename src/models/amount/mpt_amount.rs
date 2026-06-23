use crate::models::{
    currency::validate_mpt_issuance_id, Model, XRPLModelException, XRPLModelResult,
};
use alloc::borrow::Cow;
use bigdecimal::BigDecimal;
use core::convert::TryInto;
use core::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct MPTAmount<'a> {
    pub mpt_issuance_id: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

impl Model for MPTAmount<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        validate_mpt_issuance_id(&self.mpt_issuance_id)?;
        let v = self.value.parse::<i64>()?;
        if v < 0 {
            return Err(XRPLModelException::InvalidValue {
                field: "value".to_string(),
                expected: "a non-negative MPT amount".to_string(),
                found: self.value.as_ref().to_string(),
            });
        }
        Ok(())
    }
}

impl<'a> MPTAmount<'a> {
    pub fn new(mpt_issuance_id: Cow<'a, str>, value: Cow<'a, str>) -> Self {
        Self {
            mpt_issuance_id,
            value,
        }
    }
}

impl<'a> TryInto<BigDecimal> for MPTAmount<'a> {
    type Error = XRPLModelException;

    fn try_into(self) -> XRPLModelResult<BigDecimal, Self::Error> {
        Ok(BigDecimal::from_str(&self.value)?)
    }
}
