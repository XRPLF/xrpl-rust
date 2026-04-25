//! Strongly-typed byte-array wrappers used across the safe API.
//!
//! Public-information types ([`Pubkey`], [`Ciphertext`], [`Commitment`],
//! [`ContextHash`], [`AccountId`], [`IssuanceId`], the per-tx `*Proof`
//! types) are `Copy` newtypes you can construct, compare, and serialize
//! freely.
//!
//! Secret types ([`Privkey`], [`BlindingFactor`]) implement [`Zeroize`] and
//! [`ZeroizeOnDrop`], have **redacted** [`Debug`] output, and are not `Copy`
//! — their bytes are wiped from memory when they go out of scope.

use core::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

// ─────────────────────────────────────────────────────────────────────────
//  Macros to reduce boilerplate
// ─────────────────────────────────────────────────────────────────────────

/// Public newtype: `Copy + Eq + Debug-as-hex-prefix`.
macro_rules! public_bytes {
    ($name:ident, $size:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(pub [u8; $size]);

        impl $name {
            #[inline]
            pub const fn new(bytes: [u8; $size]) -> Self { Self(bytes) }
            #[inline]
            pub const fn as_bytes(&self) -> &[u8; $size] { &self.0 }
            #[inline]
            pub const fn into_bytes(self) -> [u8; $size] { self.0 }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), hex_short(&self.0))
            }
        }
    };
}

/// Secret newtype: `Zeroize + ZeroizeOnDrop`, no `Copy`, redacted `Debug`.
macro_rules! secret_bytes {
    ($name:ident, $size:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Zeroize, ZeroizeOnDrop)]
        pub struct $name(pub(crate) [u8; $size]);

        impl $name {
            #[inline]
            pub fn new(bytes: [u8; $size]) -> Self { Self(bytes) }
            #[inline]
            pub fn as_bytes(&self) -> &[u8; $size] { &self.0 }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "(<redacted>)"))
            }
        }
    };
}

// ─────────────────────────────────────────────────────────────────────────
//  Identity / addressing
// ─────────────────────────────────────────────────────────────────────────

public_bytes!(AccountId,   20, "20-byte XRPL AccountID.");
public_bytes!(IssuanceId,  24, "24-byte XRPL `MPTokenIssuanceID`.");

// ─────────────────────────────────────────────────────────────────────────
//  Key material
// ─────────────────────────────────────────────────────────────────────────

secret_bytes!(Privkey,        32, "32-byte ElGamal/secp256k1 secret key. Zeroized on drop.");
public_bytes!(Pubkey,         33, "33-byte compressed secp256k1 public key.");
secret_bytes!(BlindingFactor, 32, "32-byte ElGamal randomness / Pedersen blinding factor. Zeroized on drop.");

// ─────────────────────────────────────────────────────────────────────────
//  Cryptographic objects
// ─────────────────────────────────────────────────────────────────────────

public_bytes!(Ciphertext,  66, "66-byte EC-ElGamal ciphertext (two compressed points: R || S).");
public_bytes!(Commitment,  33, "33-byte Pedersen commitment.");
public_bytes!(ContextHash, 32, "32-byte SHA-256 transcript hash binding a proof to its transaction.");

// ─────────────────────────────────────────────────────────────────────────
//  Proof blobs (sized exactly per XLS-0096 §5.4)
// ─────────────────────────────────────────────────────────────────────────

public_bytes!(ConvertProof,      64,  "64-byte Schnorr Proof of Knowledge (ConfidentialMPTConvert).");
public_bytes!(SendProof,         946, "946-byte ConfidentialMPTSend proof: 192 B compact sigma + 754 B aggregated Bulletproof.");
public_bytes!(ConvertBackProof,  816, "816-byte ConfidentialMPTConvertBack proof: 128 B compact sigma + 688 B Bulletproof.");
public_bytes!(ClawbackProof,     64,  "64-byte ConfidentialMPTClawback compact sigma proof.");

// ─────────────────────────────────────────────────────────────────────────
//  Helpers
// ─────────────────────────────────────────────────────────────────────────

/// Hex-encode the first 4 and last 4 bytes for a Debug summary.
fn hex_short(bytes: &[u8]) -> String {
    if bytes.len() <= 8 {
        bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
    } else {
        let head: String = bytes[..4].iter().map(|b| format!("{:02x}", b)).collect();
        let tail: String = bytes[bytes.len()-4..].iter().map(|b| format!("{:02x}", b)).collect();
        format!("0x{head}…{tail}")
    }
}
