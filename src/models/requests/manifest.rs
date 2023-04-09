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
    pub public_key: &'a str,
    /// The unique request id.
    pub id: Option<&'a str>,
    /// The request method.
    #[serde(default = "RequestMethod::manifest")]
    pub command: RequestMethod,
}

impl<'a> Default for Manifest<'a> {
    fn default() -> Self {
        Manifest {
            public_key: "",
            id: None,
            command: RequestMethod::Manifest,
        }
    }
}

impl<'a> Model for Manifest<'a> {}

impl<'a> Manifest<'a> {
    fn new(public_key: &'a str, id: Option<&'a str>) -> Self {
        Self {
            public_key,
            id,
            command: RequestMethod::Manifest,
        }
    }
}
