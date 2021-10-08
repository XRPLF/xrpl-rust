#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc as std;
#[cfg(feature = "std")]
extern crate std;

pub mod constants;
pub mod core;
pub mod macros;
pub mod utils;
