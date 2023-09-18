use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

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
    /// The base58-encoded public key of the validator
    /// to look up. This can be the master public key or
    /// ephemeral public key.
    pub public_key: Cow<'a, str>,
    /// The unique request id.
    pub id: Option<Cow<'a, str>>,
    /// The request method.
    #[serde(default = "RequestMethod::manifest")]
    pub command: RequestMethod,
}

impl<'a> Default for Manifest<'a> {
    fn default() -> Self {
        Manifest {
            public_key: "".into(),
            id: None,
            command: RequestMethod::Manifest,
        }
    }
}

impl<'a> Model for Manifest<'a> {}

impl<'a> Manifest<'a> {
    pub fn new(public_key: Cow<'a, str>, id: Option<Cow<'a, str>>) -> Self {
        Self {
            public_key,
            id,
            command: RequestMethod::Manifest,
        }
    }
}
