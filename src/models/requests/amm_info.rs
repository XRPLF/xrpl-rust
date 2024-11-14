use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{Currency, Model};

use super::{CommonFields, Request};

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AMMInfo<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    pub amm_account: Option<Cow<'a, str>>,
    pub asset: Option<Currency<'a>>,
    pub asset2: Option<Currency<'a>>,
}

impl Model for AMMInfo<'_> {}

impl<'a> Request<'a> for AMMInfo<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> AMMInfo<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        amm_account: Option<Cow<'a, str>>,
        asset: Option<Currency<'a>>,
        asset2: Option<Currency<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: super::RequestMethod::AMMInfo,
                id,
            },
            amm_account,
            asset,
            asset2,
        }
    }
}
