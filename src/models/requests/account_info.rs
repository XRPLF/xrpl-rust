use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// This request retrieves information about an account, its
/// activity, and its XRP balance. All information retrieved
/// is relative to a particular version of the ledger.
///
/// See Account Info:
/// `<https://xrpl.org/account_info.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AccountInfo<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// A unique identifier for the account, most commonly the
    /// account's Address.
    pub account: Cow<'a, str>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger to use, or a shortcut
    /// string to choose a ledger automatically.
    pub ledger_index: Option<Cow<'a, str>>,
    /// If true, then the account field only accepts a public
    /// key or XRP Ledger address. Otherwise, account can be
    /// a secret or passphrase (not recommended).
    /// The default is false.
    pub strict: Option<bool>,
    /// If true, and the FeeEscalation amendment is enabled,
    /// also returns stats about queued transactions associated
    /// with this account. Can only be used when querying for the
    /// data from the current open ledger. New in: rippled 0.33.0
    /// Not available from servers in Reporting Mode.
    pub queue: Option<bool>,
    /// If true, and the MultiSign amendment is enabled, also
    /// returns any SignerList objects associated with this account.
    pub signer_lists: Option<bool>,
}

impl<'a> Model for AccountInfo<'a> {}

impl<'a> Request<'a> for AccountInfo<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> AccountInfo<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        strict: Option<bool>,
        queue: Option<bool>,
        signer_lists: Option<bool>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::AccountInfo,
                id,
            },
            account,
            ledger_hash,
            ledger_index,
            strict,
            queue,
            signer_lists,
        }
    }
}
