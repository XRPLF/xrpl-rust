//! Methods for working with XRPL wallets.

use crate::constants::CryptoAlgorithm;
use crate::core::addresscodec::classic_address_to_xaddress;
use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::core::keypairs::derive_classic_address;
use crate::core::keypairs::derive_keypair;
use crate::core::keypairs::exceptions::XRPLKeypairsException;
use crate::core::keypairs::generate_seed;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use zeroize::Zeroize;

/// The cryptographic keys needed to control an
/// XRP Ledger account.
///
/// See Cryptographic Keys:
/// `<https://xrpl.org/cryptographic-keys.html>`
#[derive(Debug)]
pub struct Wallet {
    /// The seed from which the public and private keys
    /// are derived.
    pub seed: String,
    /// The public key that is used to identify this wallet's
    /// signatures, as a hexadecimal string.
    pub public_key: String,
    /// The private key that is used to create signatures, as
    /// a hexadecimal string. MUST be kept secret!
    ///
    /// TODO Use seckey
    pub private_key: String,
    /// The address that publicly identifies this wallet, as
    /// a base58 string.
    pub classic_address: String,
    /// The next available sequence number to use for
    /// transactions from this wallet. Must be updated by the
    /// user. Increments on the ledger with every successful
    /// transaction submission, and stays the same with every
    /// failed transaction submission.
    pub sequence: u64,
}

// Zeroize the memory where sensitive data is stored.
impl Drop for Wallet {
    fn drop(&mut self) {
        self.seed.zeroize();
        self.public_key.zeroize();
        self.private_key.zeroize();
        self.classic_address.zeroize();
        self.sequence.zeroize();
    }
}

impl Wallet {
    /// Generate a new Wallet.
    pub fn new(seed: &str, sequence: u64) -> Result<Self, XRPLKeypairsException> {
        let (public_key, private_key) = derive_keypair(seed, false)?;
        let classic_address = derive_classic_address(&public_key)?;

        Ok(Wallet {
            seed: seed.into(),
            public_key,
            private_key,
            classic_address,
            sequence,
        })
    }

    /// Generates a new seed and Wallet.
    pub fn create(
        crypto_algorithm: Option<CryptoAlgorithm>,
    ) -> Result<Self, XRPLKeypairsException> {
        Self::new(&generate_seed(None, crypto_algorithm)?, 0)
    }

    /// Returns the X-Address of the Wallet's account.
    pub fn get_xaddress(
        &self,
        tag: Option<u64>,
        is_test_network: bool,
    ) -> Result<String, XRPLAddressCodecException> {
        classic_address_to_xaddress(&self.classic_address, tag, is_test_network)
    }
}

impl ToString for Wallet {
    /// Returns a string representation of a Wallet.
    fn to_string(&self) -> String {
        let string_list = [
            format!("public_key: {}", self.public_key),
            format!("private_key: {}", "-HIDDEN-"),
            format!("classic_address: {}", self.classic_address),
        ];

        string_list.join("-")
    }
}
