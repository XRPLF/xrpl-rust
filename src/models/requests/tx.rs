use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// The tx method retrieves information on a single transaction.
///
/// See Tx:
/// `<https://xrpl.org/tx.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Tx<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// If true, return transaction data and metadata as binary
    /// serialized to hexadecimal strings. If false, return
    /// transaction data and metadata as JSON. The default is false.
    pub binary: Option<bool>,
    /// Use this with min_ledger to specify a range of up to 1000
    /// ledger indexes, ending with this ledger (inclusive). If the
    /// server cannot find the transaction, it confirms whether it
    /// was able to search all the ledgers in the requested range.
    pub max_ledger: Option<u32>,
    /// Use this with max_ledger to specify a range of up to 1000
    /// ledger indexes, starting with this ledger (inclusive). If
    /// the server cannot find the transaction, it confirms whether
    /// it was able to search all the ledgers in this range.
    pub min_ledger: Option<u32>,
}

impl<'a> Model for Tx<'a> {}

impl<'a> Request<'a> for Tx<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> Tx<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        binary: Option<bool>,
        max_ledger: Option<u32>,
        min_ledger: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::Tx,
                id,
            },
            binary,
            min_ledger,
            max_ledger,
        }
    }
}
