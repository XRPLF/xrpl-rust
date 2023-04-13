use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// The submit method applies a transaction and sends it to
/// the network to be confirmed and included in future ledgers.
///
/// This command has two modes:
/// * Submit-only mode takes a signed, serialized transaction
///   as a binary blob, and submits it to the network as-is.
///   Since signed transaction objects are immutable, no part
///   of the transaction can be modified or automatically
///   filled in after submission.
/// * Sign-and-submit mode takes a JSON-formatted Transaction
///   object, completes and signs the transaction in the same
///   manner as the sign method, and then submits the signed
///   transaction. We recommend only using this mode for
///   testing and development.
///
/// To send a transaction as robustly as possible, you should
/// construct and sign it in advance, persist it somewhere that
/// you can access even after a power outage, then submit it as
/// a tx_blob. After submission, monitor the network with the
/// tx method command to see if the transaction was successfully
/// applied; if a restart or other problem occurs, you can
/// safely re-submit the tx_blob transaction: it won't be
/// applied twice since it has the same sequence number as the
/// old transaction.
///
/// See Submit:
/// `<https://xrpl.org/submit.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Submit<'a> {
    /// Hex representation of the signed transaction to submit.
    /// This can also be a multi-signed transaction.
    pub tx_blob: &'a str,
    /// The unique request id.
    pub id: Option<&'a str>,
    /// If true, and the transaction fails locally, do not retry
    /// or relay the transaction to other servers
    pub fail_hard: Option<bool>,
    /// The request method.
    #[serde(default = "RequestMethod::submit")]
    pub command: RequestMethod,
}

impl<'a> Default for Submit<'a> {
    fn default() -> Self {
        Submit {
            tx_blob: "",
            id: None,
            fail_hard: None,
            command: RequestMethod::Submit,
        }
    }
}

impl<'a> Model for Submit<'a> {}

impl<'a> Submit<'a> {
    fn new(tx_blob: &'a str, id: Option<&'a str>, fail_hard: Option<bool>) -> Self {
        Self {
            tx_blob,
            id,
            fail_hard,
            command: RequestMethod::Submit,
        }
    }
}
