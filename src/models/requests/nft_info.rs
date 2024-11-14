use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::Model;

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Request, RequestMethod};

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct NFTInfo<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    #[serde(flatten)]
    pub ledger_lookup: Option<LookupByLedgerRequest<'a>>,
    pub nft_id: Cow<'a, str>,
}

impl Model for NFTInfo<'_> {}

impl<'a> Request<'a> for NFTInfo<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> NFTInfo<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        nft_id: Cow<'a, str>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<LedgerIndex<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::NFTInfo,
                id,
            },
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
            nft_id,
        }
    }
}
