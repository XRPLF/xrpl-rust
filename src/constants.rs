//! Collection of public constants for XRPL.

use serde::{Deserialize, Serialize};
use strum_macros::Display;
use strum_macros::EnumIter;

/// Regular expression for determining ISO currency codes.
pub const ISO_CURRENCY_REGEX: &str = r"^[A-Z0-9]{3}$";
/// Regular expression for determining hex currency codes.
pub const HEX_CURRENCY_REGEX: &str = r"^[A-F0-9]{40}$";

/// Length of an account id.
pub const ACCOUNT_ID_LENGTH: usize = 20;

pub const MAX_TICK_SIZE: u32 = 15;
pub const MIN_TICK_SIZE: u32 = 3;
pub const DISABLE_TICK_SIZE: u32 = 0;

pub const MAX_TRANSFER_RATE: u32 = 2000000000;
pub const MIN_TRANSFER_RATE: u32 = 1000000000;
pub const SPECIAL_CASE_TRANFER_RATE: u32 = 0;

pub const MAX_TRANSFER_FEE: u32 = 50000;
pub const MAX_URI_LENGTH: usize = 512;

pub const MAX_DOMAIN_LENGTH: usize = 256;

/// Represents the supported cryptography algorithms.
#[derive(Debug, PartialEq, Eq, Clone, EnumIter, Display, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum CryptoAlgorithm {
    ED25519,
    SECP256K1,
}
