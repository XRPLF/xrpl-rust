use crate::models::ledger::LedgerEntryType;
use crate::models::{amount::Amount, Model};
use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;

/// The `Escrow` object type represents a held payment of XRP waiting to be executed or canceled.
/// An `EscrowCreate` transaction creates an `Escrow` object in the ledger. A successful `EscrowFinish`
/// or `EscrowCancel` transaction deletes the object. If the `Escrow` object has a crypto-condition,
/// the payment can only succeed if an `EscrowFinish` transaction provides the corresponding
/// fulfillment that satisfies the condition.
/// (The only supported crypto-condition type is PREIMAGE-SHA-256.) If the `Escrow` object has a
/// `FinishAfter` time, the held payment can only execute after that time.
///
/// An `Escrow` object is associated with two addresses:
/// - The owner, who provides the XRP when creating the `Escrow` object. If the held payment is
/// canceled, the XRP returns to the owner.
/// - The destination, where the XRP is paid when the held payment succeeds. The destination can
/// be the same as the owner.
///
/// `<https://xrpl.org/escrow-object.html#escrow>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Escrow<'a> {
    /// The value `0x0075`, mapped to the string `Escrow`, indicates that this object is an
    /// `Escrow` object.
    pub ledger_entry_type: LedgerEntryType,
    /// A bit-map of boolean flags enabled for this object. Currently, the protocol defines no
    /// flags for `Escrow` objects. The value is always `0`.
    pub flags: u32,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// The address of the owner (sender) of this held payment. This is the account that provided
    /// the XRP, and gets it back if the held payment is canceled.
    pub account: Cow<'a, str>,
    /// The amount of XRP, in drops, to be delivered by the held payment.
    pub amount: Amount<'a>,
    /// The destination address where the XRP is paid if the held payment is successful.
    pub destination: Cow<'a, str>,
    /// A hint indicating which page of the owner directory links to this object, in case the
    /// directory consists of multiple pages. Note: The object does not contain a direct link
    /// to the owner directory containing it, since that value can be derived from the Account.
    pub owner_node: Cow<'a, str>,
    #[serde(rename = "PreviousTxnID")]
    /// The identifying hash of the transaction that most recently modified this object.
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most recently modified this object.
    pub previous_txn_lgr_seq: u32,
    /// The held payment can be canceled if and only if this field is present and the time it
    /// specifies has passed. Specifically, this is specified as seconds since the Ripple Epoch
    /// and it "has passed" if it's earlier than the close time of the previous validated ledger.
    pub cancel_after: Option<u32>,
    /// A PREIMAGE-SHA-256 crypto-condition, as hexadecimal. If present, the `EscrowFinish`
    /// transaction must contain a fulfillment that satisfies this condition.
    pub condition: Option<Cow<'a, str>>,
    /// A hint indicating which page of the destination's owner directory links to this object,
    /// in case the directory consists of multiple pages. Omitted on escrows created before
    /// enabling the fix1523 amendment.
    pub destination_node: Option<Cow<'a, str>>,
    /// An arbitrary tag to further specify the destination for this held payment, such as a
    /// hosted recipient at the destination address.
    pub destination_tag: Option<u32>,
    /// The time, in seconds since the Ripple Epoch, after which this held payment can be finished.
    /// Any `EscrowFinish` transaction before this time fails.
    pub finish_after: Option<u32>,
    /// An arbitrary tag to further specify the source for this held payment, such as a hosted
    /// recipient at the owner's address.
    pub source_tag: Option<u32>,
}

impl<'a> Default for Escrow<'a> {
    fn default() -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::Escrow,
            flags: Default::default(),
            index: Default::default(),
            account: Default::default(),
            amount: Default::default(),
            cancel_after: Default::default(),
            condition: Default::default(),
            destination: Default::default(),
            destination_node: Default::default(),
            destination_tag: Default::default(),
            finish_after: Default::default(),
            owner_node: Default::default(),
            previous_txn_id: Default::default(),
            previous_txn_lgr_seq: Default::default(),
            source_tag: Default::default(),
        }
    }
}

impl<'a> Model for Escrow<'a> {}

impl<'a> Escrow<'a> {
    pub fn new(
        index: Cow<'a, str>,
        account: Cow<'a, str>,
        amount: Amount<'a>,
        destination: Cow<'a, str>,
        owner_node: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
        cancel_after: Option<u32>,
        condition: Option<Cow<'a, str>>,
        destination_node: Option<Cow<'a, str>>,
        destination_tag: Option<u32>,
        finish_after: Option<u32>,
        source_tag: Option<u32>,
    ) -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::Escrow,
            flags: 0,
            index,
            account,
            amount,
            destination,
            owner_node,
            previous_txn_id,
            previous_txn_lgr_seq,
            cancel_after,
            condition,
            destination_node,
            destination_tag,
            finish_after,
            source_tag,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use alloc::borrow::Cow;

    #[test]
    fn test_serialize() {
        let escrow = Escrow::new(
            Cow::from("DC5F3851D8A1AB622F957761E5963BC5BD439D5C24AC6AD7AC4523F0640244AC"),
            Cow::from("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"),
            Amount::XRPAmount("10000".into()),
            Cow::from("ra5nK24KXen9AHvsdFTKHSANinZseWnPcX"),
            Cow::from("0000000000000000"),
            Cow::from("C44F2EB84196B9AD820313DBEBA6316A15C9A2D35787579ED172B87A30131DA7"),
            28991004,
            Some(545440232),
            Some(Cow::from(
                "A0258020A82A88B2DF843A54F58772E4A3861866ECDB4157645DD9AE528C1D3AEEDABAB6810120",
            )),
            Some(Cow::from("0000000000000000")),
            Some(23480),
            Some(545354132),
            Some(11747),
        );
        let escrow_json = serde_json::to_string(&escrow).unwrap();
        let actual = escrow_json.as_str();
        let expected = r#"{"LedgerEntryType":"Escrow","Flags":0,"index":"DC5F3851D8A1AB622F957761E5963BC5BD439D5C24AC6AD7AC4523F0640244AC","Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","Amount":"10000","Destination":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX","OwnerNode":"0000000000000000","PreviousTxnID":"C44F2EB84196B9AD820313DBEBA6316A15C9A2D35787579ED172B87A30131DA7","PreviousTxnLgrSeq":28991004,"CancelAfter":545440232,"Condition":"A0258020A82A88B2DF843A54F58772E4A3861866ECDB4157645DD9AE528C1D3AEEDABAB6810120","DestinationNode":"0000000000000000","DestinationTag":23480,"FinishAfter":545354132,"SourceTag":11747}"#;

        assert_eq!(expected, actual);
    }

    // TODO: test_deserialize
}
