//! Single error type covering every failure mode in the safe wrapper layer.

use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An FFI call returned a non-zero status code. Upstream contract is
    /// "0 on success, -1 on failure" — we surface the raw value for context
    /// rather than coercing to a single "FfiFailure" variant.
    #[error("mpt-crypto FFI returned non-zero status: {0}")]
    NonZeroRc(i32),

    /// A post-condition the FFI promised wasn't met (e.g. the Send proof
    /// utility wrote a different number of bytes than the spec advertises).
    #[error("invariant violated: {0}")]
    Invariant(&'static str),
}

pub type Result<T> = core::result::Result<T, Error>;
