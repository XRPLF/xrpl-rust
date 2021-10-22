#![no_std]
#![allow(dead_code)] // Remove eventually

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

pub mod constants;
#[cfg(feature = "core")]
pub mod core;
pub mod macros;
pub mod utils;
pub mod wallet;
