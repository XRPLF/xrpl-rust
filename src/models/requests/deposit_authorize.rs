use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// The deposit_authorized command indicates whether one account
/// is authorized to send payments directly to another.
///
/// See Deposit Authorization:
/// `<https://xrpl.org/depositauth.html#deposit-authorization>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DepositAuthorized<'a> {
    /// The sender of a possible payment.
    pub source_account: Cow<'a, str>,
    /// The recipient of a possible payment.
    pub destination_account: Cow<'a, str>,
    /// The unique request id.
    pub id: Option<Cow<'a, str>>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger to use, or a shortcut
    /// string to choose a ledger automatically.
    pub ledger_index: Option<Cow<'a, str>>,
    /// The request method.
    #[serde(default = "RequestMethod::deposit_authorization")]
    pub command: RequestMethod,
}

impl<'a> Default for DepositAuthorized<'a> {
    fn default() -> Self {
        DepositAuthorized {
            source_account: "".into(),
            destination_account: "".into(),
            id: None,
            ledger_hash: None,
            ledger_index: None,
            command: RequestMethod::DepositAuthorized,
        }
    }
}

impl<'a> Model for DepositAuthorized<'a> {}

impl<'a> DepositAuthorized<'a> {
    pub fn new(
        source_account: Cow<'a, str>,
        destination_account: Cow<'a, str>,
        id: Option<Cow<'a, str>>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            source_account,
            destination_account,
            id,
            ledger_hash,
            ledger_index,
            command: RequestMethod::DepositAuthorized,
        }
    }
}
