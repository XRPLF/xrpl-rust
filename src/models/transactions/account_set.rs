use alloc::borrow::Cow;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::amount::XRPAmount;
use crate::models::transactions::{exceptions::XRPLAccountSetException, CommonFields};
use crate::models::{XRPLModelException, XRPLModelResult};
use crate::{
    constants::{
        DISABLE_TICK_SIZE, MAX_DOMAIN_LENGTH, MAX_TICK_SIZE, MAX_TRANSFER_RATE, MIN_TICK_SIZE,
        MIN_TRANSFER_RATE, SPECIAL_CASE_TRANFER_RATE,
    },
    models::{
        transactions::{Memo, Signer, Transaction, TransactionType},
        Model,
    },
};

use super::{CommonTransactionBuilder, FlagCollection};

/// Transactions of the AccountSet type support additional values
/// in the Flags field. This enum represents those options.
///
/// See AccountSet flags:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/accountset>`
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter, Copy,
)]
#[repr(u32)]
pub enum AccountSetFlag {
    /// Require a destination tag to send transactions to this account.
    AsfRequireDest = 1,
    /// Require authorization for users to hold balances issued by
    /// this address. Can only be enabled if the address has no
    /// trust lines connected to it.
    AsfRequireAuth = 2,
    /// XRP should not be sent to this account.
    /// (Enforced by client applications, not by rippled)
    AsfDisallowXRP = 3,
    /// Disallow use of the master key pair. Can only be enabled if the
    /// account has configured another way to sign transactions, such as
    /// a Regular Key or a Signer List.
    AsfDisableMaster = 4,
    /// Track the ID of this account's most recent transaction
    /// Required for AccountTxnID
    AsfAccountTxnID = 5,
    /// Permanently give up the ability to freeze individual
    /// trust lines or disable Global Freeze. This flag can never
    /// be disabled after being enabled.
    AsfNoFreeze = 6,
    /// Freeze all assets issued by this account.
    AsfGlobalFreeze = 7,
    /// Enable rippling on this account's trust lines by default.
    AsfDefaultRipple = 8,
    /// Enable Deposit Authorization on this account.
    /// (Added by the DepositAuth amendment.)
    AsfDepositAuth = 9,
    /// Enable to allow another account to mint non-fungible tokens (NFTokens)
    /// on this account's behalf. Specify the authorized account in the
    /// NFTokenMinter field of the AccountRoot object.
    AsfAuthorizedNFTokenMinter = 10,
    /// Disallow incoming Checks from other accounts.
    AsfDisallowIncomingCheck = 11,
    /// Disallow incoming Payment Channels from other accounts.
    AsfDisallowIncomingPayChan = 12,
    /// Disallow incoming trust lines from other accounts.
    AsfDisallowIncomingTrustline = 13,
    /// Disallow incoming NFToken offers from other accounts.
    AsfDisallowIncomingNFTokenOffer = 14,
    /// Allow other accounts to mint NFTokens with this account set as the issuer.
    AsfAllowTrustLineClawback = 15,
}

/// An AccountSet transaction modifies the properties of an
/// account in the XRP Ledger.
///
/// See AccountSet:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/accountset>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct AccountSet<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, AccountSetFlag>,
    // The custom fields for the AccountSet model.
    //
    // See AccountSet fields:
    // `<https://xrpl.org/docs/references/protocol/transactions/types/accountset>`
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

impl<'a> Model for AccountSet<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_tick_size_error()?;
        self._get_transfer_rate_error()?;
        self._get_domain_error()?;
        self._get_clear_flag_error()?;
        self._get_nftoken_minter_error()?;

        Ok(())
    }
}

impl<'a> Transaction<'a, AccountSetFlag> for AccountSet<'a> {
    fn has_flag(&self, flag: &AccountSetFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, AccountSetFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, AccountSetFlag> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, AccountSetFlag> for AccountSet<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, AccountSetFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> AccountSetError for AccountSet<'a> {
    fn _get_tick_size_error(&self) -> Result<(), XRPLModelException> {
        if let Some(tick_size) = self.tick_size {
            if tick_size > MAX_TICK_SIZE {
                Err(XRPLModelException::ValueTooHigh {
                    field: "tick_size".into(),
                    max: MAX_TICK_SIZE,
                    found: tick_size,
                })
            } else if tick_size < MIN_TICK_SIZE && tick_size != DISABLE_TICK_SIZE {
                Err(XRPLModelException::ValueTooLow {
                    field: "tick_size".into(),
                    min: MIN_TICK_SIZE,
                    found: tick_size,
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_transfer_rate_error(&self) -> Result<(), XRPLModelException> {
        if let Some(transfer_rate) = self.transfer_rate {
            if transfer_rate > MAX_TRANSFER_RATE {
                Err(XRPLModelException::ValueTooHigh {
                    field: "transfer_rate".into(),
                    max: MAX_TRANSFER_RATE,
                    found: transfer_rate,
                })
            } else if transfer_rate < MIN_TRANSFER_RATE
                && transfer_rate != SPECIAL_CASE_TRANFER_RATE
            {
                Err(XRPLModelException::ValueTooLow {
                    field: "transfer_rate".into(),
                    min: MIN_TRANSFER_RATE,
                    found: transfer_rate,
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_domain_error(&self) -> Result<(), XRPLModelException> {
        if let Some(domain) = &self.domain {
            if domain.to_lowercase().as_str() != domain {
                Err(XRPLModelException::InvalidValueFormat {
                    field: "domain".into(),
                    found: domain.to_string(),
                    format: "lowercase".into(),
                })
            } else if domain.len() > MAX_DOMAIN_LENGTH {
                Err(XRPLModelException::ValueTooLong {
                    field: "domain".into(),
                    max: MAX_DOMAIN_LENGTH,
                    found: domain.len(),
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_clear_flag_error(&self) -> Result<(), XRPLModelException> {
        if self.clear_flag.is_some() && self.set_flag.is_some() && self.clear_flag == self.set_flag
        {
            Err(XRPLAccountSetException::SetAndUnsetSameFlag {
                found: self.clear_flag.unwrap(),
            }
            .into())
        } else {
            Ok(())
        }
    }

    fn _get_nftoken_minter_error(&self) -> Result<(), XRPLModelException> {
        if let Some(_nftoken_minter) = &self.nftoken_minter {
            if self.set_flag.is_none() {
                if let Some(clear_flag) = &self.clear_flag {
                    match clear_flag {
                        AccountSetFlag::AsfAuthorizedNFTokenMinter => {
                            Err(XRPLAccountSetException::SetFieldWhenUnsetRequiredFlag {
                                field: "nftoken_minter".into(),
                                flag: AccountSetFlag::AsfAuthorizedNFTokenMinter,
                            }
                            .into())
                        }
                        _ => Ok(()),
                    }
                } else {
                    Err(XRPLAccountSetException::FieldRequiresFlag {
                        field: "set_flag".into(),
                        flag: AccountSetFlag::AsfAuthorizedNFTokenMinter,
                    }
                    .into())
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
                    }
                    .into())
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
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<AccountSetFlag>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
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
            common_fields: CommonFields::new(
                account,
                TransactionType::AccountSet,
                account_txn_id,
                fee,
                flags,
                last_ledger_sequence,
                memos,
                None,
                sequence,
                signers,
                None,
                source_tag,
                ticket_sequence,
                None,
            ),
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

    /// Set clear flag
    pub fn with_clear_flag(mut self, flag: AccountSetFlag) -> Self {
        self.clear_flag = Some(flag);
        self
    }

    /// Set domain
    pub fn with_domain(mut self, domain: Cow<'a, str>) -> Self {
        self.domain = Some(domain);
        self
    }

    /// Set email hash
    pub fn with_email_hash(mut self, email_hash: Cow<'a, str>) -> Self {
        self.email_hash = Some(email_hash);
        self
    }

    /// Set message key
    pub fn with_message_key(mut self, message_key: Cow<'a, str>) -> Self {
        self.message_key = Some(message_key);
        self
    }

    /// Set NFToken minter
    pub fn with_nftoken_minter(mut self, nftoken_minter: Cow<'a, str>) -> Self {
        self.nftoken_minter = Some(nftoken_minter);
        self
    }

    /// Set flag to enable
    pub fn with_set_flag(mut self, flag: AccountSetFlag) -> Self {
        self.set_flag = Some(flag);
        self
    }

    /// Set transfer rate
    pub fn with_transfer_rate(mut self, transfer_rate: u32) -> Self {
        self.transfer_rate = Some(transfer_rate);
        self
    }

    /// Set tick size
    pub fn with_tick_size(mut self, tick_size: u32) -> Self {
        self.tick_size = Some(tick_size);
        self
    }
}

impl FromStr for AccountSetFlag {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // See https://xrpl.org/docs/references/protocol/transactions/types/accountset
        match s {
            "asfRequireDest" => Ok(AccountSetFlag::AsfRequireDest),
            "asfRequireAuth" => Ok(AccountSetFlag::AsfRequireAuth),
            "asfDisallowXRP" => Ok(AccountSetFlag::AsfDisallowXRP),
            "asfDisableMaster" => Ok(AccountSetFlag::AsfDisableMaster),
            "asfAccountTxnID" => Ok(AccountSetFlag::AsfAccountTxnID),
            "asfNoFreeze" => Ok(AccountSetFlag::AsfNoFreeze),
            "asfGlobalFreeze" => Ok(AccountSetFlag::AsfGlobalFreeze),
            "asfDefaultRipple" => Ok(AccountSetFlag::AsfDefaultRipple),
            "asfDepositAuth" => Ok(AccountSetFlag::AsfDepositAuth),
            "asfAuthorizedNFTokenMinter" => Ok(AccountSetFlag::AsfAuthorizedNFTokenMinter),
            "asfDisallowIncomingCheck" => Ok(AccountSetFlag::AsfDisallowIncomingCheck),
            "asfDisallowIncomingPayChan" => Ok(AccountSetFlag::AsfDisallowIncomingPayChan),
            "asfDisallowIncomingTrustline" => Ok(AccountSetFlag::AsfDisallowIncomingTrustline),
            "asfDisallowIncomingNFTokenOffer" => {
                Ok(AccountSetFlag::AsfDisallowIncomingNFTokenOffer)
            }
            "asfAllowTrustLineClawback" => Ok(AccountSetFlag::AsfAllowTrustLineClawback),
            _ => Err(()),
        }
    }
}

pub trait AccountSetError {
    fn _get_tick_size_error(&self) -> Result<(), XRPLModelException>;
    fn _get_transfer_rate_error(&self) -> Result<(), XRPLModelException>;
    fn _get_domain_error(&self) -> Result<(), XRPLModelException>;
    fn _get_clear_flag_error(&self) -> Result<(), XRPLModelException>;
    fn _get_nftoken_minter_error(&self) -> Result<(), XRPLModelException>;
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;
    use crate::models::Model;

    #[test]
    fn test_tick_size_error() {
        let mut account_set = AccountSet {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            tick_size: Some(2), // Too low
            ..Default::default()
        };

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"tick_size\"` is defined below its minimum (min 3, found 2)"
        );

        account_set.tick_size = Some(16); // Too high

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"tick_size\"` is defined above its maximum (max 15, found 16)"
        );
    }

    #[test]
    fn test_transfer_rate_error() {
        let mut account_set = AccountSet {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            transfer_rate: Some(999999999), // Too low
            ..Default::default()
        };

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"transfer_rate\"` is defined below its minimum (min 1000000000, found 999999999)"
        );

        account_set.transfer_rate = Some(2000000001); // Too high

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"transfer_rate\"` is defined above its maximum (max 2000000000, found 2000000001)"
        );
    }

    #[test]
    fn test_domain_error() {
        let mut account_set = AccountSet {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            domain: Some("https://Example.com/".into()), // Not lowercase
            ..Default::default()
        };

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"domain\"` does not have the correct format (expected \"lowercase\", found \"https://Example.com/\")"
        );

        account_set.domain = Some("https://example.com/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into()); // Too long

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"domain\"` exceeds its maximum length of characters (max 256, found 270)"
        );
    }

    #[test]
    fn test_flag_error() {
        let account_set = AccountSet {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            clear_flag: Some(AccountSetFlag::AsfDisallowXRP),
            set_flag: Some(AccountSetFlag::AsfDisallowXRP),
            ..Default::default()
        };

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "A flag cannot be set and unset at the same time (found AsfDisallowXRP)"
        );
    }

    #[test]
    fn test_asf_authorized_nftoken_minter_error() {
        let mut account_set = AccountSet {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            nftoken_minter: Some("rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into()),
            ..Default::default()
        };

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "For the field `\"set_flag\"` to be defined it is required to set the flag `AsfAuthorizedNFTokenMinter`"
        );

        account_set.nftoken_minter = None;
        account_set.set_flag = Some(AccountSetFlag::AsfAuthorizedNFTokenMinter);

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "For the flag `AsfAuthorizedNFTokenMinter` to be set it is required to define the field `\"nftoken_minter\"`"
        );

        account_set.set_flag = None;
        account_set.nftoken_minter = Some("rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into());
        account_set.clear_flag = Some(AccountSetFlag::AsfAuthorizedNFTokenMinter);

        assert_eq!(
            account_set.validate().unwrap_err().to_string().as_str(),
            "The field `\"nftoken_minter\"` cannot be defined if its required flag `AsfAuthorizedNFTokenMinter` is being unset"
        );
    }

    #[test]
    fn test_serde() {
        let default_txn = AccountSet {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::AccountSet,
                fee: Some("12".into()),
                sequence: Some(5),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            domain: Some("6578616D706C652E636F6D".into()),
            message_key: Some(
                "03AB40A0490F9B7ED8DF29D246BF2D6269820A0EE7742ACDD457BEA7C7D0931EDB".into(),
            ),
            set_flag: Some(AccountSetFlag::AsfAccountTxnID),
            ..Default::default()
        };

        let default_json_str = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"AccountSet","Fee":"12","Flags":0,"Sequence":5,"SigningPubKey":"","Domain":"6578616D706C652E636F6D","MessageKey":"03AB40A0490F9B7ED8DF29D246BF2D6269820A0EE7742ACDD457BEA7C7D0931EDB","SetFlag":5}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: AccountSet = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let account_set = AccountSet {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_domain("6578616D706C652E636F6D".into())
        .with_message_key(
            "03AB40A0490F9B7ED8DF29D246BF2D6269820A0EE7742ACDD457BEA7C7D0931EDB".into(),
        )
        .with_set_flag(AccountSetFlag::AsfAccountTxnID)
        .with_fee("12".into())
        .with_sequence(5)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_memo(Memo {
            memo_data: Some("setting up account".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(
            account_set.domain.as_ref().unwrap(),
            "6578616D706C652E636F6D"
        );
        assert_eq!(
            account_set.message_key.as_ref().unwrap(),
            "03AB40A0490F9B7ED8DF29D246BF2D6269820A0EE7742ACDD457BEA7C7D0931EDB"
        );
        assert_eq!(account_set.set_flag, Some(AccountSetFlag::AsfAccountTxnID));
        assert_eq!(account_set.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(account_set.common_fields.sequence, Some(5));
        assert_eq!(
            account_set.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(account_set.common_fields.source_tag, Some(12345));
        assert_eq!(account_set.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_default() {
        let account_set = AccountSet {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            account_set.common_fields.account,
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"
        );
        assert_eq!(
            account_set.common_fields.transaction_type,
            TransactionType::AccountSet
        );
        assert!(account_set.clear_flag.is_none());
        assert!(account_set.domain.is_none());
        assert!(account_set.email_hash.is_none());
        assert!(account_set.message_key.is_none());
        assert!(account_set.nftoken_minter.is_none());
        assert!(account_set.set_flag.is_none());
        assert!(account_set.transfer_rate.is_none());
        assert!(account_set.tick_size.is_none());
    }

    #[test]
    fn test_enable_deposit_auth() {
        let deposit_auth_set = AccountSet {
            common_fields: CommonFields {
                account: "rDepositAccount123".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_set_flag(AccountSetFlag::AsfDepositAuth)
        .with_fee("12".into())
        .with_sequence(100)
        .with_memo(Memo {
            memo_data: Some("enabling deposit authorization".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(
            deposit_auth_set.set_flag,
            Some(AccountSetFlag::AsfDepositAuth)
        );
        assert_eq!(deposit_auth_set.common_fields.sequence, Some(100));
        assert!(deposit_auth_set.validate().is_ok());
    }

    #[test]
    fn test_set_transfer_rate() {
        let transfer_rate_set = AccountSet {
            common_fields: CommonFields {
                account: "rTokenIssuer456".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_transfer_rate(1020000000) // 2% transfer fee
        .with_fee("12".into())
        .with_sequence(200);

        assert_eq!(transfer_rate_set.transfer_rate, Some(1020000000));
        assert_eq!(transfer_rate_set.common_fields.sequence, Some(200));
        assert!(transfer_rate_set.validate().is_ok());
    }

    #[test]
    fn test_set_tick_size() {
        let tick_size_set = AccountSet {
            common_fields: CommonFields {
                account: "rMarketMaker789".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_tick_size(5) // 5 significant digits
        .with_fee("12".into())
        .with_sequence(300);

        assert_eq!(tick_size_set.tick_size, Some(5));
        assert_eq!(tick_size_set.common_fields.sequence, Some(300));
        assert!(tick_size_set.validate().is_ok());
    }

    #[test]
    fn test_authorize_nftoken_minter() {
        let nftoken_auth_set = AccountSet {
            common_fields: CommonFields {
                account: "rNFTIssuer111".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_set_flag(AccountSetFlag::AsfAuthorizedNFTokenMinter)
        .with_nftoken_minter("rAuthorizedMinter222".into())
        .with_fee("12".into())
        .with_sequence(400);

        assert_eq!(
            nftoken_auth_set.set_flag,
            Some(AccountSetFlag::AsfAuthorizedNFTokenMinter)
        );
        assert_eq!(
            nftoken_auth_set.nftoken_minter.as_ref().unwrap(),
            "rAuthorizedMinter222"
        );
        assert_eq!(nftoken_auth_set.common_fields.sequence, Some(400));
        assert!(nftoken_auth_set.validate().is_ok());
    }

    #[test]
    fn test_disable_master_key() {
        let disable_master = AccountSet {
            common_fields: CommonFields {
                account: "rSecureAccount333".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_set_flag(AccountSetFlag::AsfDisableMaster)
        .with_fee("12".into())
        .with_sequence(500)
        .with_memo(Memo {
            memo_data: Some("disabling master key for security".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(
            disable_master.set_flag,
            Some(AccountSetFlag::AsfDisableMaster)
        );
        assert_eq!(disable_master.common_fields.sequence, Some(500));
        assert!(disable_master.validate().is_ok());
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_account_set = AccountSet {
            common_fields: CommonFields {
                account: "rTicketUser444".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_set_flag(AccountSetFlag::AsfRequireDest)
        .with_ticket_sequence(12345)
        .with_fee("12".into());

        assert_eq!(
            ticket_account_set.common_fields.ticket_sequence,
            Some(12345)
        );
        assert_eq!(
            ticket_account_set.set_flag,
            Some(AccountSetFlag::AsfRequireDest)
        );
        // When using tickets, sequence should be None or 0
        assert!(ticket_account_set.common_fields.sequence.is_none());
    }

    #[test]
    fn test_clear_and_set_different_flags() {
        let multi_flag_set = AccountSet {
            common_fields: CommonFields {
                account: "rMultiFlagAccount555".into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_set_flag(AccountSetFlag::AsfRequireDest)
        .with_clear_flag(AccountSetFlag::AsfDisallowXRP)
        .with_fee("12".into())
        .with_sequence(600);

        assert_eq!(
            multi_flag_set.set_flag,
            Some(AccountSetFlag::AsfRequireDest)
        );
        assert_eq!(
            multi_flag_set.clear_flag,
            Some(AccountSetFlag::AsfDisallowXRP)
        );
        assert_eq!(multi_flag_set.common_fields.sequence, Some(600));
        assert!(multi_flag_set.validate().is_ok());
    }
}
