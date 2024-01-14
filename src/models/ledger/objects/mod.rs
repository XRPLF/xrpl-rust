pub mod account_root;
pub mod amendments;
pub mod amm;
pub mod check;
pub mod deposit_preauth;
pub mod directory_node;
pub mod escrow;
pub mod fee_settings;
pub mod ledger_hashes;
pub mod negative_unl;
pub mod nftoken_offer;
pub mod nftoken_page;
pub mod offer;
pub mod pay_channel;
pub mod ripple_state;
pub mod signer_list;
pub mod ticket;

pub use account_root::*;
pub use amendments::*;
pub use amm::*;
pub use check::*;
pub use deposit_preauth::*;
pub use directory_node::*;
pub use escrow::*;
pub use fee_settings::*;
pub use ledger_hashes::*;
pub use negative_unl::*;
pub use nftoken_offer::*;
pub use nftoken_page::*;
pub use offer::*;
pub use pay_channel::*;
pub use ripple_state::*;
pub use ticket::*;

use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, PartialEq, Eq)]
pub enum LedgerEntryType {
    AccountRoot = 0x0061,
    Amendments = 0x0066,
    AMM = 0x0079,
    Check = 0x0043,
    DepositPreauth = 0x0070,
    DirectoryNode = 0x0064,
    Escrow = 0x0075,
    FeeSettings = 0x0073,
    LedgerHashes = 0x0068,
    NegativeUNL = 0x004E,
    NFTokenOffer = 0x0037,
    NFTokenPage = 0x0050,
    Offer = 0x006F,
    PayChannel = 0x0078,
    RippleState = 0x0072,
    SignerList = 0x0053,
    Ticket = 0x0054,
}
