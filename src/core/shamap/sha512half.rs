//! Streaming SHA-512/256 hasher (first 32 bytes of SHA-512).
//!
//! This is the standard hash function used throughout the XRP Ledger for
//! constructing ShaMap node hashes, transaction hashes, and ledger hashes.

use sha2::{Digest, Sha512};

/// A streaming hasher that produces the first 32 bytes of a SHA-512 digest.
///
/// This matches the "SHA-512Half" algorithm used in the XRP Ledger, where
/// SHA-512 is computed and then truncated to 256 bits.
pub struct Sha512Half {
    inner: Sha512,
}

impl Sha512Half {
    /// Create a new streaming hasher.
    pub fn new() -> Self {
        Sha512Half {
            inner: Sha512::new(),
        }
    }

    /// Feed data into the hasher.
    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    /// Consume the hasher and return the first 32 bytes of the SHA-512 digest.
    pub fn finish(self) -> [u8; 32] {
        let result = self.inner.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result[..32]);
        out
    }
}

impl Default for Sha512Half {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function: compute SHA-512Half of a single byte slice.
pub fn sha512half(data: &[u8]) -> [u8; 32] {
    let mut h = Sha512Half::new();
    h.update(data);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_hash() {
        // SHA-512 of empty string, first 32 bytes
        let expected = sha2::Sha512::digest([]);
        let mut expected_half = [0u8; 32];
        expected_half.copy_from_slice(&expected[..32]);
        assert_eq!(sha512half(&[]), expected_half);
    }

    #[test]
    fn test_streaming_matches_oneshot() {
        let data = b"Hello, XRP Ledger!";
        let oneshot = sha512half(data);

        let mut streaming = Sha512Half::new();
        streaming.update(&data[..7]);
        streaming.update(&data[7..]);
        let streamed = streaming.finish();

        assert_eq!(oneshot, streamed);
    }

    #[test]
    fn test_known_vector() {
        // "test message" SHA-512 first 32 bytes (matches keypairs::utils test)
        let expected: [u8; 32] = [
            149, 11, 42, 126, 255, 167, 143, 81, 166, 53, 21, 236, 69, 224, 62, 206, 190, 80, 239,
            47, 28, 65, 230, 150, 41, 181, 7, 120, 241, 27, 192, 128,
        ];
        assert_eq!(sha512half(b"test message"), expected);
    }

    #[test]
    fn test_default_trait() {
        let h = Sha512Half::default();
        let result = h.finish();
        assert_eq!(result, sha512half(&[]));
    }
}
