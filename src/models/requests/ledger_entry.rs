use crate::Err;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::requests::XRPLLedgerEntryException;
use crate::models::{requests::RequestMethod, Model};

/// Required fields for requesting a DepositPreauth if not
/// querying by object ID.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DepositPreauth<'a> {
    pub owner: &'a str,
    pub authorized: &'a str,
}

/// Required fields for requesting a DirectoryNode if not
/// querying by object ID.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Directory<'a> {
    pub owner: &'a str,
    pub dir_root: &'a str,
    pub sub_index: Option<u8>,
}

/// Required fields for requesting a Escrow if not querying
/// by object ID.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Escrow<'a> {
    pub owner: &'a str,
    pub seq: u64,
}

/// Required fields for requesting a Escrow if not querying
/// by object ID.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Offer<'a> {
    pub account: &'a str,
    pub seq: u64,
}

/// Required fields for requesting a Ticket, if not
/// querying by object ID.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Ticket<'a> {
    pub owner: &'a str,
    pub ticket_sequence: u64,
}

/// Required fields for requesting a RippleState.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct RippleState<'a> {
    pub account: &'a str,
    pub currency: &'a str,
}

/// The ledger_entry method returns a single ledger object
/// from the XRP Ledger in its raw format. See ledger formats
/// for information on the different types of objects you can
/// retrieve.
///
/// See Ledger Formats:
/// `<https://xrpl.org/ledger-data-formats.html#ledger-data-formats>`
///
/// See Ledger Entry:
/// `<https://xrpl.org/ledger_entry.html#ledger_entry>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LedgerEntry<'a> {
    /// The unique request id.
    pub id: Option<&'a str>,
    pub index: Option<&'a str>,
    pub account_root: Option<&'a str>,
    pub check: Option<&'a str>,
    pub payment_channel: Option<&'a str>,
    pub deposit_preauth: Option<DepositPreauth<'a>>,
    pub directory: Option<Directory<'a>>,
    pub escrow: Option<Escrow<'a>>,
    pub offer: Option<Offer<'a>>,
    pub ripple_state: Option<RippleState<'a>>,
    pub ticket: Option<Ticket<'a>>,
    /// If true, return the requested ledger object's contents as a
    /// hex string in the XRP Ledger's binary format. Otherwise, return
    /// data in JSON format. The default is false.
    pub binary: Option<bool>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<&'a str>,
    /// The ledger index of the ledger to use, or a shortcut string
    /// (e.g. "validated" or "closed" or "current") to choose a ledger
    /// automatically.
    pub ledger_index: Option<&'a str>,
    /// The request method.
    #[serde(default = "RequestMethod::ledger_entry")]
    pub command: RequestMethod,
}

impl<'a> Default for LedgerEntry<'a> {
    fn default() -> Self {
        LedgerEntry {
            id: None,
            index: None,
            account_root: None,
            check: None,
            payment_channel: None,
            deposit_preauth: None,
            directory: None,
            escrow: None,
            offer: None,
            ripple_state: None,
            ticket: None,
            binary: None,
            ledger_hash: None,
            ledger_index: None,
            command: RequestMethod::LedgerEntry,
        }
    }
}

impl<'a: 'static> Model for LedgerEntry<'a> {
    fn get_errors(&self) -> Result<()> {
        match self._get_field_error() {
            Err(error) => Err!(error),
            Ok(_no_error) => Ok(()),
        }
    }
}

impl<'a> LedgerEntryError for LedgerEntry<'a> {
    fn _get_field_error(&self) -> Result<(), XRPLLedgerEntryException> {
        let mut signing_methods: u32 = 0;
        for method in [self.index, self.account_root, self.check] {
            if method.is_some() {
                signing_methods += 1
            }
        }
        if self.directory.is_some() {
            signing_methods += 1
        }
        if self.offer.is_some() {
            signing_methods += 1
        }
        if self.ripple_state.is_some() {
            signing_methods += 1
        }
        if self.escrow.is_some() {
            signing_methods += 1
        }
        if self.payment_channel.is_some() {
            signing_methods += 1
        }
        if self.deposit_preauth.is_some() {
            signing_methods += 1
        }
        if self.ticket.is_some() {
            signing_methods += 1
        }
        if signing_methods != 1 {
            Err(XRPLLedgerEntryException::DefineExactlyOneOf {
                field1: "index",
                field2: "account_root",
                field3: "check",
                field4: "directory",
                field5: "offer",
                field6: "ripple_state",
                field7: "escrow",
                field8: "payment_channel",
                field9: "deposit_preauth",
                field10: "ticket",
                resource: "",
            })
        } else {
            Ok(())
        }
    }
}

impl<'a> LedgerEntry<'a> {
    pub fn new(
        id: Option<&'a str>,
        index: Option<&'a str>,
        account_root: Option<&'a str>,
        check: Option<&'a str>,
        payment_channel: Option<&'a str>,
        deposit_preauth: Option<DepositPreauth<'a>>,
        directory: Option<Directory<'a>>,
        escrow: Option<Escrow<'a>>,
        offer: Option<Offer<'a>>,
        ripple_state: Option<RippleState<'a>>,
        ticket: Option<Ticket<'a>>,
        binary: Option<bool>,
        ledger_hash: Option<&'a str>,
        ledger_index: Option<&'a str>,
    ) -> Self {
        Self {
            id,
            index,
            account_root,
            check,
            payment_channel,
            deposit_preauth,
            directory,
            escrow,
            offer,
            ripple_state,
            ticket,
            binary,
            ledger_hash,
            ledger_index,
            command: RequestMethod::LedgerData,
        }
    }
}

pub trait LedgerEntryError {
    fn _get_field_error(&self) -> Result<(), XRPLLedgerEntryException>;
}

#[cfg(test)]
mod test_ledger_entry_errors {
    use super::Offer;
    use crate::models::requests::XRPLLedgerEntryException;
    use crate::models::Model;
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_fields_error() {
        let ledger_entry = LedgerEntry {
            command: RequestMethod::LedgerEntry,
            id: None,
            index: None,
            account_root: Some("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"),
            check: None,
            payment_channel: None,
            deposit_preauth: None,
            directory: None,
            escrow: None,
            offer: Some(Offer {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
                seq: 359,
            }),
            ripple_state: None,
            ticket: None,
            binary: None,
            ledger_hash: None,
            ledger_index: None,
        };
        let _expected = XRPLLedgerEntryException::DefineExactlyOneOf {
            field1: "index",
            field2: "account_root",
            field3: "check",
            field4: "directory",
            field5: "offer",
            field6: "ripple_state",
            field7: "escrow",
            field8: "payment_channel",
            field9: "deposit_preauth",
            field10: "ticket",
            resource: "",
        };
        assert_eq!(
            ledger_entry.validate().unwrap_err().to_string().as_str(),
            "Define one of: `index`, `account_root`, `check`, `directory`, `offer`, `ripple_state`, `escrow`, `payment_channel`, `deposit_preauth`, `ticket`. Define exactly one of them. For more information see: "
        );
    }
}
