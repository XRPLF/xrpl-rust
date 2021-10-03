#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

pub mod constants;
pub mod core;
pub mod macros;
pub mod utils;
