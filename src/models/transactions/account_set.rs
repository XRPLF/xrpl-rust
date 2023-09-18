use alloc::vec::Vec;
use alloc::borrow::Cow;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::amount::XRPAmount;
use crate::models::transactions::XRPLAccountSetException;
use crate::{
    _serde::txn_flags,
    constants::{
        DISABLE_TICK_SIZE, MAX_DOMAIN_LENGTH, MAX_TICK_SIZE, MAX_TRANSFER_RATE, MIN_TICK_SIZE,
        MIN_TRANSFER_RATE, SPECIAL_CASE_TRANFER_RATE,
    },
    models::{
        model::Model,
        transactions::{Flag, Memo, Signer, Transaction, TransactionType},
    },
    Err,
};

/// Transactions of the AccountSet type support additional values
/// in the Flags field. This enum represents those options.
///
/// See AccountSet flags:
/// `<https://xrpl.org/accountset.html#accountset-flags>`
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum AccountSetFlag {
    /// Track the ID of this account's most recent transaction
    /// Required for AccountTxnID
    AsfAccountTxnID = 5,
    /// Enable to allow another account to mint non-fungible tokens (NFTokens)
    /// on this account's behalf. Specify the authorized account in the
    /// NFTokenMinter field of the AccountRoot object. This is an experimental
    /// field to enable behavior for NFToken support.
    AsfAuthorizedNFTokenMinter = 10,
    /// Enable rippling on this account's trust lines by default.
    AsfDefaultRipple = 8,
    /// Enable Deposit Authorization on this account.
    /// (Added by the DepositAuth amendment.)
    AsfDepositAuth = 9,
    /// Disallow use of the master key pair. Can only be enabled if the
    /// account has configured another way to sign transactions, such as
    /// a Regular Key or a Signer List.
    AsfDisableMaster = 4,
    /// XRP should not be sent to this account.
    /// (Enforced by client applications, not by rippled)
    AsfDisallowXRP = 3,
    /// Freeze all assets issued by this account.
    AsfGlobalFreeze = 7,
    /// Permanently give up the ability to freeze individual
    /// trust lines or disable Global Freeze. This flag can never
    /// be disabled after being enabled.
    AsfNoFreeze = 6,
    /// Require authorization for users to hold balances issued by
    /// this address. Can only be enabled if the address has no
    /// trust lines connected to it.
    AsfRequireAuth = 2,
    /// Require a destination tag to send transactions to this account.
    AsfRequireDest = 1,
}

/// An AccountSet transaction modifies the properties of an
/// account in the XRP Ledger.
///
/// See AccountSet:
/// `<https://xrpl.org/accountset.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AccountSet<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(default = "TransactionType::account_set")]
    pub transaction_type: TransactionType,
    /// The unique address of the account that initiated the transaction.
    pub account: Cow<'a, str>,
    /// Integer amount of XRP, in drops, to be destroyed as a cost
    /// for distributing this transaction to the network. Some
    /// transaction types have different minimum requirements.
    /// See Transaction Cost for details.
    pub fee: Option<XRPAmount<'a>>,
    /// The sequence number of the account sending the transaction.
    /// A transaction is only valid if the Sequence number is exactly
    /// 1 greater than the previous transaction from the same account.
    /// The special case 0 means the transaction is using a Ticket instead.
    pub sequence: Option<u32>,
    /// Highest ledger index this transaction can appear in.
    /// Specifying this field places a strict upper limit on how long
    /// the transaction can wait to be validated or rejected.
    /// See Reliable Transaction Submission for more details.
    pub last_ledger_sequence: Option<u32>,
    /// Hash value identifying another transaction. If provided, this
    /// transaction is only valid if the sending account's
    /// previously-sent transaction matches the provided hash.
    #[serde(rename = "AccountTxnID")]
    pub account_txn_id: Option<Cow<'a, str>>,
    /// Hex representation of the public key that corresponds to the
    /// private key used to sign this transaction. If an empty string,
    /// indicates a multi-signature is present in the Signers field instead.
    pub signing_pub_key: Option<Cow<'a, str>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction
    /// is made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub source_tag: Option<u32>,
    /// The sequence number of the ticket to use in place
    /// of a Sequence number. If this is provided, Sequence must
    /// be 0. Cannot be used with AccountTxnID.
    pub ticket_sequence: Option<u32>,
    /// The signature that verifies this transaction as originating
    /// from the account it says it is from.
    pub txn_signature: Option<Cow<'a, str>>,
    /// Set of bit-flags for this transaction.
    #[serde(default)]
    #[serde(with = "txn_flags")]
    pub flags: Option<Vec<AccountSetFlag>>,
    /// Additional arbitrary information used to identify this transaction.
    pub memos: Option<Vec<Memo>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction is
    /// made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub signers: Option<Vec<Signer<'a>>>,
    // The custom fields for the AccountSet model.
    //
    // See AccountSet fields:
    // `<https://xrpl.org/accountset.html#accountset-fields>`
    /// Unique identifier of a flag to disable for this account.
    pub clear_flag: Option<AccountSetFlag>,
    /// The domain that owns this account, as a string of hex
    /// representing the ASCII for the domain in lowercase.
    /// Cannot be more than 256 bytes in length.
    pub domain: Option<Cow<'a, str>>,
    /// Hash of an email address to be used for generating an
    /// avatar image. Conventionally, clients use Gravatar
    /// to display this image.
    pub email_hash: Option<Cow<'a, str>>,
    /// Public key for sending encrypted messages to this account.
    /// To set the key, it must be exactly 33 bytes, with the
    /// first byte indicating the key type: 0x02 or 0x03 for
    /// secp256k1 keys, 0xED for Ed25519 keys. To remove the
    /// key, use an empty value.
    pub message_key: Option<Cow<'a, str>>,
    /// Sets an alternate account that is allowed to mint NFTokens
    /// on this account's behalf using NFTokenMint's Issuer field.
    /// This field is part of the experimental XLS-20 standard
    /// for non-fungible tokens.
    pub nftoken_minter: Option<Cow<'a, str>>,
    /// Flag to enable for this account.
    pub set_flag: Option<AccountSetFlag>,
    /// The fee to charge when users transfer this account's tokens,
    /// represented as billionths of a unit. Cannot be more than
    /// 2000000000 or less than 1000000000, except for the special
    /// case 0 meaning no fee.
    pub transfer_rate: Option<u32>,
    /// Tick size to use for offers involving a currency issued by
    /// this address. The exchange rates of those offers is rounded
    /// to this many significant digits. Valid values are 3 to 15
    /// inclusive, or 0 to disable.
    pub tick_size: Option<u32>,
}

impl<'a> Default for AccountSet<'a> {
    fn default() -> Self {
        Self {
            transaction_type: TransactionType::AccountSet,
            account: Default::default(),
            fee: Default::default(),
            sequence: Default::default(),
            last_ledger_sequence: Default::default(),
            account_txn_id: Default::default(),
            signing_pub_key: Default::default(),
            source_tag: Default::default(),
            ticket_sequence: Default::default(),
            txn_signature: Default::default(),
            flags: Default::default(),
            memos: Default::default(),
            signers: Default::default(),
            clear_flag: Default::default(),
            domain: Default::default(),
            email_hash: Default::default(),
            message_key: Default::default(),
            nftoken_minter: Default::default(),
            set_flag: Default::default(),
            transfer_rate: Default::default(),
            tick_size: Default::default(),
        }
    }
}

impl<'a: 'static> Model for AccountSet<'a> {
    fn get_errors(&self) -> Result<()> {
        match self._get_tick_size_error() {
            Err(error) => Err!(error),
            Ok(_no_error) => match self._get_transfer_rate_error() {
                Err(error) => Err!(error),
                Ok(_no_error) => match self._get_domain_error() {
                    Err(error) => Err!(error),
                    Ok(_no_error) => match self._get_clear_flag_error() {
                        Err(error) => Err!(error),
                        Ok(_no_error) => match self._get_nftoken_minter_error() {
                            Err(error) => Err!(error),
                            Ok(_no_error) => Ok(()),
                        },
                    },
                },
            },
        }
    }
}

impl<'a> Transaction for AccountSet<'a> {
    fn has_flag(&self, flag: &Flag) -> bool {
        let mut flags = &Vec::new();

        if let Some(flag_set) = self.flags.as_ref() {
            flags = flag_set;
        }

        match flag {
            Flag::AccountSet(account_set_flag) => match account_set_flag {
                AccountSetFlag::AsfAccountTxnID => flags.contains(&AccountSetFlag::AsfAccountTxnID),
                AccountSetFlag::AsfAuthorizedNFTokenMinter => {
                    flags.contains(&AccountSetFlag::AsfAuthorizedNFTokenMinter)
                }
                AccountSetFlag::AsfDefaultRipple => {
                    flags.contains(&AccountSetFlag::AsfDefaultRipple)
                }
                AccountSetFlag::AsfDepositAuth => flags.contains(&AccountSetFlag::AsfDepositAuth),
                AccountSetFlag::AsfDisableMaster => {
                    flags.contains(&AccountSetFlag::AsfDisableMaster)
                }
                AccountSetFlag::AsfDisallowXRP => flags.contains(&AccountSetFlag::AsfDisallowXRP),
                AccountSetFlag::AsfGlobalFreeze => flags.contains(&AccountSetFlag::AsfGlobalFreeze),
                AccountSetFlag::AsfNoFreeze => flags.contains(&AccountSetFlag::AsfNoFreeze),
                AccountSetFlag::AsfRequireAuth => flags.contains(&AccountSetFlag::AsfRequireAuth),
                AccountSetFlag::AsfRequireDest => flags.contains(&AccountSetFlag::AsfRequireDest),
            },
            _ => false,
        }
    }

    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }
}

impl<'a> AccountSetError for AccountSet<'a> {
    fn _get_tick_size_error(&self) -> Result<(), XRPLAccountSetException> {
        if let Some(tick_size) = self.tick_size {
            if tick_size > MAX_TICK_SIZE {
                Err(XRPLAccountSetException::ValueTooHigh {
                    field: "tick_size".into(),
                    max: MAX_TICK_SIZE,
                    found: tick_size,
                    resource: "".into(),
                })
            } else if tick_size < MIN_TICK_SIZE && tick_size != DISABLE_TICK_SIZE {
                Err(XRPLAccountSetException::ValueTooLow {
                    field: "tick_size".into(),
                    min: MIN_TICK_SIZE,
                    found: tick_size,
                    resource: "".into(),
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_transfer_rate_error(&self) -> Result<(), XRPLAccountSetException> {
        if let Some(transfer_rate) = self.transfer_rate {
            if transfer_rate > MAX_TRANSFER_RATE {
                Err(XRPLAccountSetException::ValueTooHigh {
                    field: "transfer_rate".into(),
                    max: MAX_TRANSFER_RATE,
                    found: transfer_rate,
                    resource: "".into(),
                })
            } else if transfer_rate < MIN_TRANSFER_RATE
                && transfer_rate != SPECIAL_CASE_TRANFER_RATE
            {
                Err(XRPLAccountSetException::ValueTooLow {
                    field: "transfer_rate".into(),
                    min: MIN_TRANSFER_RATE,
                    found: transfer_rate,
                    resource: "".into(),
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_domain_error(&self) -> Result<(), XRPLAccountSetException> {
        if let Some(domain) = self.domain.clone() {
            if domain.to_lowercase().as_str() != domain {
                Err(XRPLAccountSetException::InvalidValueFormat {
                    field: "domain".into(),
                    found: domain,
                    format: "lowercase".into(),
                    resource: "".into(),
                })
            } else if domain.len() > MAX_DOMAIN_LENGTH {
                Err(XRPLAccountSetException::ValueTooLong {
                    field: "domain".into(),
                    max: MAX_DOMAIN_LENGTH,
                    found: domain.len(),
                    resource: "".into(),
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_clear_flag_error(&self) -> Result<(), XRPLAccountSetException> {
        if self.clear_flag.is_some() && self.set_flag.is_some() && self.clear_flag == self.set_flag
        {
            Err(XRPLAccountSetException::SetAndUnsetSameFlag {
                found: self.clear_flag.clone().unwrap(),
                resource: "".into(),
            })
        } else {
            Ok(())
        }
    }

    fn _get_nftoken_minter_error(&self) -> Result<(), XRPLAccountSetException> {
        if let Some(_nftoken_minter) = self.nftoken_minter.clone() {
            if self.set_flag.is_none() {
                if let Some(clear_flag) = &self.clear_flag {
                    match clear_flag {
                        AccountSetFlag::AsfAuthorizedNFTokenMinter => {
                            Err(XRPLAccountSetException::SetFieldWhenUnsetRequiredFlag {
                                field: "nftoken_minter".into(),
                                flag: AccountSetFlag::AsfAuthorizedNFTokenMinter,
                                resource: "".into(),
                            })
                        }
                        _ => Ok(()),
                    }
                } else {
                    Err(XRPLAccountSetException::FieldRequiresFlag {
                        field: "set_flag".into(),
                        flag: AccountSetFlag::AsfAuthorizedNFTokenMinter,
                        resource: "".into(),
                    })
                }
            } else {
                Ok(())
            }
        } else if let Some(set_flag) = &self.set_flag {
            match set_flag {
                AccountSetFlag::AsfAuthorizedNFTokenMinter => {
                    Err(XRPLAccountSetException::FlagRequiresField {
                        flag: AccountSetFlag::AsfAuthorizedNFTokenMinter,
                        field: "nftoken_minter".into(),
                        resource: "".into(),
                    })
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
}

impl<'a> AccountSet<'a> {
    pub fn new(
        account: Cow<'a, str>,
        fee: Option<XRPAmount<'a>>,
        sequence: Option<u32>,
        last_ledger_sequence: Option<u32>,
        account_txn_id: Option<Cow<'a, str>>,
        signing_pub_key: Option<Cow<'a, str>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        txn_signature: Option<Cow<'a, str>>,
        flags: Option<Vec<AccountSetFlag>>,
        memos: Option<Vec<Memo>>,
        signers: Option<Vec<Signer<'a>>>,
        clear_flag: Option<AccountSetFlag>,
        domain: Option<Cow<'a, str>>,
        email_hash: Option<Cow<'a, str>>,
        message_key: Option<Cow<'a, str>>,
        set_flag: Option<AccountSetFlag>,
        transfer_rate: Option<u32>,
        tick_size: Option<u32>,
        nftoken_minter: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            transaction_type: TransactionType::AccountSet,
            account,
            fee,
            sequence,
            last_ledger_sequence,
            account_txn_id,
            signing_pub_key,
            source_tag,
            ticket_sequence,
            txn_signature,
            flags,
            memos,
            signers,
            clear_flag,
            domain,
            email_hash,
            message_key,
            nftoken_minter,
            set_flag,
            transfer_rate,
            tick_size,
        }
    }
}

pub trait AccountSetError {
    fn _get_tick_size_error(&self) -> Result<(), XRPLAccountSetException>;
    fn _get_transfer_rate_error(&self) -> Result<(), XRPLAccountSetException>;
    fn _get_domain_error(&self) -> Result<(), XRPLAccountSetException>;
    fn _get_clear_flag_error(&self) -> Result<(), XRPLAccountSetException>;
    fn _get_nftoken_minter_error(&self) -> Result<(), XRPLAccountSetException>;
}

#[cfg(test)]
mod test_account_set_errors {

    use crate::models::Model;
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_tick_size_error() {
        let mut account_set = AccountSet {
            transaction_type: TransactionType::AccountSet,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            account_txn_id: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
            flags: None,
            memos: None,
            signers: None,
            clear_flag: None,
            domain: None,
            email_hash: None,
            message_key: None,
            set_flag: None,
            transfer_rate: None,
            tick_size: None,
            nftoken_minter: None,
        };
        let tick_size_too_low = Some(2);
        account_set.tick_size = tick_size_too_low;

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `tick_size` is defined below its minimum (min 3, found 2). For more information see: "
        );

        let tick_size_too_high = Some(16);
        account_set.tick_size = tick_size_too_high;

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `tick_size` is defined above its maximum (max 15, found 16). For more information see: "
        );
    }

    #[test]
    fn test_transfer_rate_error() {
        let mut account_set = AccountSet {
            transaction_type: TransactionType::AccountSet,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            account_txn_id: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
            flags: None,
            memos: None,
            signers: None,
            clear_flag: None,
            domain: None,
            email_hash: None,
            message_key: None,
            set_flag: None,
            transfer_rate: None,
            tick_size: None,
            nftoken_minter: None,
        };
        let tick_size_too_low = Some(999999999);
        account_set.transfer_rate = tick_size_too_low;

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `transfer_rate` is defined below its minimum (min 1000000000, found 999999999). For more information see: "
        );

        let tick_size_too_high = Some(2000000001);
        account_set.transfer_rate = tick_size_too_high;

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `transfer_rate` is defined above its maximum (max 2000000000, found 2000000001). For more information see: "
        );
    }

    #[test]
    fn test_domain_error() {
        let mut account_set = AccountSet {
            transaction_type: TransactionType::AccountSet,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            account_txn_id: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
            flags: None,
            memos: None,
            signers: None,
            clear_flag: None,
            domain: None,
            email_hash: None,
            message_key: None,
            set_flag: None,
            transfer_rate: None,
            tick_size: None,
            nftoken_minter: None,
        };
        let domain_not_lowercase = Some("https://Example.com/".into());
        account_set.domain = domain_not_lowercase;

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `domain` does not have the correct format (expected lowercase, found https://Example.com/). For more information see: "
        );

        let domain_too_long = Some("https://example.com/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into());
        account_set.domain = domain_too_long;

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `domain` exceeds its maximum length of characters (max 256, found 270). For more information see: "
        );
    }

    #[test]
    fn test_flag_error() {
        let account_set = AccountSet {
            transaction_type: TransactionType::AccountSet,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            account_txn_id: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
            flags: None,
            memos: None,
            signers: None,
            clear_flag: Some(AccountSetFlag::AsfDisallowXRP),
            domain: None,
            email_hash: None,
            message_key: None,
            set_flag: Some(AccountSetFlag::AsfDisallowXRP),
            transfer_rate: None,
            tick_size: None,
            nftoken_minter: None,
        };

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "A flag cannot be set and unset at the same time (found AsfDisallowXRP). For more information see: "
        );
    }

    #[test]
    fn test_asf_authorized_nftoken_minter_error() {
        let mut account_set = AccountSet {
            transaction_type: TransactionType::AccountSet,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            account_txn_id: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
            flags: None,
            memos: None,
            signers: None,
            clear_flag: None,
            domain: None,
            email_hash: None,
            message_key: None,
            set_flag: None,
            transfer_rate: None,
            tick_size: None,
            nftoken_minter: None,
        };
        account_set.nftoken_minter = Some("rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into());

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "For the field `set_flag` to be defined it is required to set the flag `AsfAuthorizedNFTokenMinter`. For more information see: "
        );

        account_set.nftoken_minter = None;
        account_set.set_flag = Some(AccountSetFlag::AsfAuthorizedNFTokenMinter);

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "For the flag `AsfAuthorizedNFTokenMinter` to be set it is required to define the field `nftoken_minter`. For more information see: "
        );

        account_set.set_flag = None;
        account_set.nftoken_minter = Some("rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into());
        account_set.clear_flag = Some(AccountSetFlag::AsfAuthorizedNFTokenMinter);

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The field `nftoken_minter` cannot be defined if its required flag `AsfAuthorizedNFTokenMinter` is being unset. For more information see: "
        );
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    #[test]
    fn test_serialize() {
        let default_txn = AccountSet::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            Some("12".into()),
            Some(5),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("6578616D706C652E636F6D".into()),
            None,
            Some("03AB40A0490F9B7ED8DF29D246BF2D6269820A0EE7742ACDD457BEA7C7D0931EDB".into()),
            Some(AccountSetFlag::AsfAccountTxnID),
            None,
            None,
            None,
        );
        let default_json = r#"{"TransactionType":"AccountSet","Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","Fee":"12","Sequence":5,"Domain":"6578616D706C652E636F6D","MessageKey":"03AB40A0490F9B7ED8DF29D246BF2D6269820A0EE7742ACDD457BEA7C7D0931EDB","SetFlag":5}"#;

        let txn_as_string = serde_json::to_string(&default_txn).unwrap();
        let txn_json = txn_as_string.as_str();

        assert_eq!(txn_json, default_json);
    }

    #[test]
    fn test_deserialize() {
        let default_txn = AccountSet::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            Some("12".into()),
            Some(5),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("6578616D706C652E636F6D".into()),
            None,
            Some("03AB40A0490F9B7ED8DF29D246BF2D6269820A0EE7742ACDD457BEA7C7D0931EDB".into()),
            Some(AccountSetFlag::AsfAccountTxnID),
            None,
            None,
            None,
        );
        let default_json = r#"{"TransactionType":"AccountSet","Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","Fee":"12","Sequence":5,"Domain":"6578616D706C652E636F6D","MessageKey":"03AB40A0490F9B7ED8DF29D246BF2D6269820A0EE7742ACDD457BEA7C7D0931EDB","SetFlag":5}"#;

        let txn_as_obj: AccountSet = serde_json::from_str(default_json).unwrap();

        assert_eq!(txn_as_obj, default_txn);
    }
}
