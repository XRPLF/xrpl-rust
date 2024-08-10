use alloc::borrow::Cow;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use anyhow::Result;
use derive_new::new;
use serde::{ser::SerializeMap, Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::transactions::XRPLSignerListSetException;
use crate::models::NoFlags;
use crate::models::{
    amount::XRPAmount,
    model::Model,
    transactions::{Memo, Signer, Transaction, TransactionType},
};
use crate::{serde_with_tag, Err};

use super::CommonFields;

serde_with_tag! {
    #[derive(Debug, PartialEq, Eq, Default, Clone, new)]
    #[skip_serializing_none]
    pub struct SignerEntry {
        pub account: String,
        pub signer_weight: u16,
    }
}

/// The SignerList object type represents a list of parties that,
/// as a group, are authorized to sign a transaction in place of an
/// individual account. You can create, replace, or remove a signer
/// list using a SignerListSet transaction.
///
/// See TicketCreate:
/// `<https://xrpl.org/signerlistset.html>`
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
#[skip_serializing_none]
pub struct SignerListSet<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the TicketCreate model.
    //
    // See TicketCreate fields:
    // `<https://xrpl.org/signerlistset.html#signerlistset-fields>`
    /// A target number for the signer weights. A multi-signature from this list
    /// is valid only if the sum weights of the signatures provided is greater
    /// than or equal to this value. To delete a signer list, use the value 0.
    pub signer_quorum: u32,
    /// A target number for the signer weights. A multi-signature from this list is
    /// valid only if the sum weights of the signatures provided is greater than
    /// or equal to this value. To delete a signer list, use the value 0.
    pub signer_entries: Option<Vec<SignerEntry>>,
}

impl<'a> Model for SignerListSet<'a> {
    fn get_errors(&self) -> Result<()> {
        match self._get_signer_entries_error() {
            Err(error) => Err!(error),
            Ok(_no_error) => match self._get_signer_quorum_error() {
                Err(error) => Err!(error),
                Ok(_no_error) => Ok(()),
            },
        }
    }
}

impl<'a> Transaction<'a, NoFlags> for SignerListSet<'a> {
    fn get_transaction_type(&self) -> TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> SignerListSetError for SignerListSet<'a> {
    fn _get_signer_entries_error(&self) -> Result<(), XRPLSignerListSetException> {
        if let Some(signer_entries) = &self.signer_entries {
            if self.signer_quorum == 0 {
                Err(XRPLSignerListSetException::ValueCausesValueDeletion {
                    field1: "signer_entries".into(),
                    field2: "signer_quorum".into(),
                    resource: "".into(),
                })
            } else if signer_entries.is_empty() {
                Err(XRPLSignerListSetException::CollectionTooFewItems {
                    field: "signer_entries".into(),
                    min: 1_usize,
                    found: signer_entries.len(),
                    resource: "".into(),
                })
            } else if signer_entries.len() > 8 {
                Err(XRPLSignerListSetException::CollectionTooManyItems {
                    field: "signer_entries".into(),
                    max: 8_usize,
                    found: signer_entries.len(),
                    resource: "".into(),
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_signer_quorum_error(&self) -> Result<(), XRPLSignerListSetException> {
        let mut accounts = Vec::new();
        let mut signer_weight_sum: u32 = 0;
        if self.signer_entries.is_some() {
            for signer_entry in self.signer_entries.as_ref().unwrap() {
                accounts.push(signer_entry.account.clone());
                let weight: u32 = signer_entry.signer_weight.into();
                signer_weight_sum += weight;
            }
        }
        accounts.sort_unstable();
        let mut check_account = Vec::new();
        for account in accounts.clone() {
            if check_account.contains(&account) {
                return Err(XRPLSignerListSetException::CollectionItemDuplicate {
                    field: "signer_entries".into(),
                    found: account.into(),
                    resource: "".into(),
                });
            } else {
                check_account.push(account);
            }
        }
        if let Some(_signer_entries) = &self.signer_entries {
            if accounts.contains(&self.common_fields.account.to_string()) {
                Err(XRPLSignerListSetException::CollectionInvalidItem {
                    field: "signer_entries".into(),
                    found: self.common_fields.account.clone(),
                    resource: "".into(),
                })
            } else if self.signer_quorum > signer_weight_sum {
                Err(
                    XRPLSignerListSetException::SignerQuorumExceedsSignerWeight {
                        max: signer_weight_sum,
                        found: self.signer_quorum,
                        resource: "".into(),
                    },
                )
            } else {
                Ok(())
            }
        } else if self.signer_quorum != 0 {
            Err(XRPLSignerListSetException::InvalidValueForValueDeletion {
                field: "signer_quorum".into(),
                expected: 0,
                found: self.signer_quorum,
                resource: "".into(),
            })
        } else {
            Ok(())
        }
    }
}

impl<'a> SignerListSet<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer<'a>>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        signer_quorum: u32,
        signer_entries: Option<Vec<SignerEntry>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::SignerListSet,
                account_txn_id,
                fee,
                flags: None,
                last_ledger_sequence,
                memos,
                sequence,
                signers,
                source_tag,
                ticket_sequence,
                network_id: None,
                signing_pub_key: None,
                txn_signature: None,
            },
            signer_quorum,
            signer_entries,
        }
    }
}

pub trait SignerListSetError {
    fn _get_signer_entries_error(&self) -> Result<(), XRPLSignerListSetException>;
    fn _get_signer_quorum_error(&self) -> Result<(), XRPLSignerListSetException>;
}

#[cfg(test)]
mod test_signer_list_set_error {
    use alloc::string::ToString;
    use alloc::vec;

    use crate::models::Model;

    use super::*;

    #[test]
    fn test_signer_list_deleted_error() {
        let mut signer_list_set = SignerListSet::new(
            "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            Some(vec![SignerEntry {
                account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".to_string(),
                signer_weight: 2,
            }]),
        );

        assert_eq!(
            signer_list_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `signer_entries` can not be defined with the field `signer_quorum` because it would cause the deletion of `signer_entries`. For more information see: "
        );

        signer_list_set.signer_quorum = 3;
        signer_list_set.signer_entries = None;

        assert_eq!(
            signer_list_set.validate().unwrap_err().to_string().as_str(),
            "The field `signer_quorum` has the wrong value to be deleted (expected 0, found 3). For more information see: "
        );
    }

    #[test]
    fn test_signer_entries_error() {
        let mut signer_list_set = SignerListSet::new(
            "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            3,
            Some(vec![]),
        );

        assert_eq!(
            signer_list_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `signer_entries` has too few items in it (min 1, found 0). For more information see: "
        );

        signer_list_set.signer_entries = Some(vec![
            SignerEntry {
                account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".to_string(),
                signer_weight: 1,
            },
            SignerEntry {
                account: "rUpy3eEg8rqjqfUoLeBnZkscbKbFsKXC3v".to_string(),
                signer_weight: 1,
            },
            SignerEntry {
                account: "rUpy3eEg8rqjqfUoLeBnZkscbKbFsKXC3v".to_string(),
                signer_weight: 2,
            },
            SignerEntry {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".to_string(),
                signer_weight: 2,
            },
            SignerEntry {
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".to_string(),
                signer_weight: 1,
            },
            SignerEntry {
                account: "rXTZ5g8X7mrAYEe7iFeM9fiS4ccueyurG".to_string(),
                signer_weight: 1,
            },
            SignerEntry {
                account: "rPbMHxs7vy5t6e19tYfqG7XJ6Fog8EPZLk".to_string(),
                signer_weight: 2,
            },
            SignerEntry {
                account: "r3rhWeE31Jt5sWmi4QiGLMZnY3ENgqw96W".to_string(),
                signer_weight: 3,
            },
            SignerEntry {
                account: "rchGBxcD1A1C2tdxF6papQYZ8kjRKMYcL".to_string(),
                signer_weight: 2,
            },
        ]);

        assert_eq!(
            signer_list_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `signer_entries` has too many items in it (max 8, found 9). For more information see: "
        );

        signer_list_set.signer_entries = Some(vec![
            SignerEntry {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".to_string(),
                signer_weight: 1,
            },
            SignerEntry {
                account: "rUpy3eEg8rqjqfUoLeBnZkscbKbFsKXC3v".to_string(),
                signer_weight: 2,
            },
            SignerEntry {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".to_string(),
                signer_weight: 2,
            },
        ]);

        assert_eq!(
            signer_list_set.validate().unwrap_err().to_string().as_str(),
            "The field `signer_entries` contains an invalid value (found rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb). For more information see: "
        );

        signer_list_set.signer_entries = Some(vec![SignerEntry {
            account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".to_string(),
            signer_weight: 3,
        }]);
        signer_list_set.signer_quorum = 10;

        assert_eq!(
            signer_list_set.validate().unwrap_err().to_string().as_str(),
            "The field `signer_quorum` must be below or equal to the sum of `signer_weight` in `signer_entries`. For more information see: "
        );

        signer_list_set.signer_entries = Some(vec![
            SignerEntry {
                account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".to_string(),
                signer_weight: 3,
            },
            SignerEntry {
                account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".to_string(),
                signer_weight: 2,
            },
        ]);
        signer_list_set.signer_quorum = 2;

        assert_eq!(
            signer_list_set.validate().unwrap_err().to_string().as_str(),
            "The value of the field `signer_entries` has a duplicate in it (found rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW). For more information see: "
        );
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = SignerListSet::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            None,
            Some("12".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            3,
            Some(vec![
                SignerEntry::new("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".to_string(), 2),
                SignerEntry::new("rUpy3eEg8rqjqfUoLeBnZkscbKbFsKXC3v".to_string(), 1),
                SignerEntry::new("raKEEVSGnKSD9Zyvxu4z6Pqpm4ABH8FS6n".to_string(), 1),
            ]),
        );
        let default_json_str = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"SignerListSet","Fee":"12","SignerQuorum":3,"SignerEntries":[{"SignerEntry":{"Account":"rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW","SignerWeight":2}},{"SignerEntry":{"Account":"rUpy3eEg8rqjqfUoLeBnZkscbKbFsKXC3v","SignerWeight":1}},{"SignerEntry":{"Account":"raKEEVSGnKSD9Zyvxu4z6Pqpm4ABH8FS6n","SignerWeight":1}}]}"#;
        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: SignerListSet = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }
}
