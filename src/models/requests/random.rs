use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// The random command provides a random number to be used
/// as a source of entropy for random number generation
/// by clients.
///
/// See Random:
/// `<https://xrpl.org/random.html#random>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Random<'a> {
    /// The unique request id.
    pub id: Option<&'a str>,
    /// The request method.
    #[serde(default = "RequestMethod::random")]
    pub command: RequestMethod,
}

impl<'a> Default for Random<'a> {
    fn default() -> Self {
        Random {
            id: None,
            command: RequestMethod::Random,
        }
    }
}

impl<'a> Model for Random<'a> {}

impl<'a> Random<'a> {
    fn new(id: Option<&'a str>) -> Self {
        Self {
            id,
            command: RequestMethod::Random,
        }
    }
}
