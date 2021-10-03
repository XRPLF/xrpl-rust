//! This module contains commonly-used constants.

use bs58::Alphabet;

/// The dictionary used for XRPL base58 encodings
/// Sourced from the [`bs58`] crate.
///
/// [`bs58`]: bs58::Alphabet
pub const XRPL_ALPHABET: Alphabet = *bs58::Alphabet::RIPPLE;
