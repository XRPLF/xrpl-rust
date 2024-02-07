use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// The manifest method reports the current "manifest"
/// information for a given validator public key. The
/// "manifest" is the public portion of that validator's
/// configured token.
///
/// See Manifest:
/// `<https://xrpl.org/manifest.html#manifest>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Manifest<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// The base58-encoded public key of the validator
    /// to look up. This can be the master public key or
    /// ephemeral public key.
    pub public_key: Cow<'a, str>,
}

impl<'a> Model for Manifest<'a> {}

impl<'a> Request for Manifest<'a> {
    fn get_command(&self) -> RequestMethod {
        self.common_fields.command.clone()
    }
}

impl<'a> Manifest<'a> {
    pub fn new(id: Option<Cow<'a, str>>, public_key: Cow<'a, str>) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::Manifest,
                id,
            },
            public_key,
        }
    }
}
