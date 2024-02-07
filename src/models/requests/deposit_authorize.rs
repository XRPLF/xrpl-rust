use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// The deposit_authorized command indicates whether one account
/// is authorized to send payments directly to another.
///
/// See Deposit Authorization:
/// `<https://xrpl.org/depositauth.html#deposit-authorization>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DepositAuthorized<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// The recipient of a possible payment.
    pub destination_account: Cow<'a, str>,
    /// The sender of a possible payment.
    pub source_account: Cow<'a, str>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger to use, or a shortcut
    /// string to choose a ledger automatically.
    pub ledger_index: Option<Cow<'a, str>>,
}

impl<'a> Model for DepositAuthorized<'a> {}

impl<'a> Request for DepositAuthorized<'a> {
    fn get_command(&self) -> RequestMethod {
        self.common_fields.command.clone()
    }
}

impl<'a> DepositAuthorized<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        destination_account: Cow<'a, str>,
        source_account: Cow<'a, str>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::DepositAuthorized,
                id,
            },
            source_account,
            destination_account,
            ledger_hash,
            ledger_index,
        }
    }
}
