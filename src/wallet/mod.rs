//! Methods for working with XRPL wallets.

pub mod exceptions;
#[cfg(feature = "helpers")]
pub mod faucet_generation;

use crate::constants::CryptoAlgorithm;
use crate::core::addresscodec::classic_address_to_xaddress;
use crate::core::keypairs::derive_classic_address;
use crate::core::keypairs::derive_keypair;
use crate::core::keypairs::generate_seed;
use alloc::string::String;
use core::fmt::{Debug, Display};
use exceptions::XRPLWalletResult;
use zeroize::Zeroize;

/// The cryptographic keys needed to control an
/// XRP Ledger account.
///
/// See Cryptographic Keys:
/// `<https://xrpl.org/cryptographic-keys.html>`
///
/// Both `Debug` and `Display` redact the `seed` and `private_key` fields so that
/// `dbg!(wallet)`, `format!("{wallet:?}")`, and `tracing::debug!(?wallet)` cannot leak
/// secrets into logs or terminal scrollback (issue #287).
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
    pub fn new(seed: &str, sequence: u64) -> XRPLWalletResult<Self> {
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
    pub fn create(crypto_algorithm: Option<CryptoAlgorithm>) -> XRPLWalletResult<Self> {
        Self::new(&generate_seed(None, crypto_algorithm)?, 0)
    }

    /// Returns the X-Address of the Wallet's account.
    pub fn get_xaddress(
        &self,
        tag: Option<u64>,
        is_test_network: bool,
    ) -> XRPLWalletResult<String> {
        Ok(classic_address_to_xaddress(
            &self.classic_address,
            tag,
            is_test_network,
        )?)
    }
}

impl Display for Wallet {
    /// Returns a string representation of a Wallet.
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "Wallet {{ public_key: {}, private_key: -HIDDEN-, classic_address: {} }}",
            self.public_key, self.classic_address
        )
    }
}

// `Debug` delegates to `Display` so `{:?}`, `{:#?}`, `dbg!(...)`, and
// `tracing::debug!(?wallet)` all redact the `seed` and `private_key` fields the same
// way the human-readable formatter does. The default derive would print the raw
// secrets — see issue #287 (CLI faucet path printed `{:#?}` on the generated wallet).
impl Debug for Wallet {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    const TEST_SEED: &str = "sEdSkooMk31MeTjbHVE7vLvgCpEMAdB";

    #[test]
    fn debug_does_not_leak_private_key_or_seed() {
        // Regression for #287. With the default `#[derive(Debug)]`, this output would
        // include the raw `seed` and `private_key`. The custom impl mirrors `Display`,
        // which redacts both.
        let wallet = Wallet::new(TEST_SEED, 0).unwrap();
        let dbg_output = format!("{wallet:?}");
        assert!(
            !dbg_output.contains(wallet.private_key.as_str()),
            "Debug output must not contain the raw private_key. Got: {dbg_output}",
        );
        assert!(
            !dbg_output.contains(wallet.seed.as_str()),
            "Debug output must not contain the raw seed. Got: {dbg_output}",
        );
        // Sanity: redacted marker must be present and public fields must still appear.
        assert!(dbg_output.contains("-HIDDEN-"));
        assert!(dbg_output.contains(wallet.public_key.as_str()));
        assert!(dbg_output.contains(wallet.classic_address.as_str()));
    }

    #[test]
    fn pretty_debug_also_redacts() {
        // The CLI faucet path uses `{:#?}` (alternate Debug). It must redact too.
        let wallet = Wallet::new(TEST_SEED, 0).unwrap();
        let dbg_output = format!("{wallet:#?}");
        assert!(!dbg_output.contains(wallet.private_key.as_str()));
        assert!(!dbg_output.contains(wallet.seed.as_str()));
    }

    #[test]
    fn display_redacts_private_key_and_seed() {
        // Pin the pre-existing Display behaviour so a future change to it can't silently
        // reintroduce the leak via the `Debug -> Display` delegation above.
        let wallet = Wallet::new(TEST_SEED, 0).unwrap();
        let display_output = format!("{wallet}");
        assert!(!display_output.contains(wallet.private_key.as_str()));
        assert!(!display_output.contains(wallet.seed.as_str()));
        assert!(display_output.contains("-HIDDEN-"));
    }
}
