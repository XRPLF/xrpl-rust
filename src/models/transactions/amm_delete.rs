use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::models::{Currency, FlagCollection, Model, NoFlags, XRPAmount};

use super::{CommonFields, Memo, Signer, Transaction, TransactionType};

/// Delete an empty Automated Market Maker (AMM) instance that could not be fully
/// deleted automatically.
///
/// Tip: The AMMWithdraw transaction automatically tries to delete an AMM, along with
/// associated ledger entries such as empty trust lines, if it withdrew all the assets
/// from the AMM's pool. However, if there are too many trust lines to the AMM account
/// to remove in one transaction, it may stop before fully removing the AMM. Similarly,
/// an AMMDelete transaction removes up to a maximum number of trust lines; in extreme
/// cases, it may take several AMMDelete transactions to fully delete the trust lines
/// and the associated AMM. In all cases, the AMM ledger entry and AMM account are
/// deleted by the last such transaction.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AMMDelete<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The definition for one of the assets in the AMM's pool.
    pub asset: Currency<'a>,
    /// The definition for the other asset in the AMM's pool.
    pub asset2: Currency<'a>,
}

impl Model for AMMDelete<'_> {}

impl<'a> Transaction<'a, NoFlags> for AMMDelete<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }

    fn get_transaction_type(&self) -> TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> AMMDelete<'a> {
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
        asset: Currency<'a>,
        asset2: Currency<'a>,
    ) -> AMMDelete<'a> {
        AMMDelete {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::AMMDelete,
                account_txn_id,
                fee,
                flags: FlagCollection::default(),
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
            asset,
            asset2,
        }
    }
}
